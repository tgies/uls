use std::collections::{BTreeSet, HashSet};

use anyhow::{bail, Result};
use chrono::NaiveDate;
use serde::Serialize;
use uls_db::Database;
use uls_download::FccClient;

use super::{
    build_chain_from_inventory, contiguous_patch_coverage, download_daily_inventory,
    inspect_weekly_archive, DailyArchive, DailyGap, WeeklyArchive,
};

const PLAN_FORMAT: &str = "uls.update_plan";
const PLAN_FORMAT_VERSION: u8 = 1;

#[derive(Debug, Serialize)]
pub(super) struct UpdatePlanDocument {
    pub format: &'static str,
    pub format_version: u8,
    pub service: String,
    pub service_code: String,
    pub current: CurrentCoverage,
    pub reachable_source_dates: Vec<NaiveDate>,
    pub recommended_source_date: Option<NaiveDate>,
    pub observed: ObservedArchives,
    pub current_daily_gap: Option<GapDocument>,
}

#[derive(Debug, Serialize)]
pub(super) struct CurrentCoverage {
    pub weekly_date: Option<NaiveDate>,
    pub source_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize)]
pub(super) struct ObservedArchives {
    pub weekly_date: Option<NaiveDate>,
    pub weekly_status: WeeklyObservationStatus,
    pub daily_status: DailyObservationStatus,
    pub complete: bool,
    pub latest_daily_date: Option<NaiveDate>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum WeeklyObservationStatus {
    Available,
    NotPublished,
    Unavailable,
    InvalidArchive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum DailyObservationStatus {
    Complete,
    Unavailable,
    InvalidArchive,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub(super) struct GapDocument {
    pub missing_date: NaiveDate,
    pub next_available_date: NaiveDate,
}

impl From<DailyGap> for GapDocument {
    fn from(gap: DailyGap) -> Self {
        Self {
            missing_date: gap.missing_date,
            next_available_date: gap.next_available_date,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct WeeklyRoute {
    pub archive: WeeklyArchive,
    pub dailies: Vec<DailyArchive>,
}

#[derive(Debug)]
pub(super) struct PlannedUpdate {
    pub document: UpdatePlanDocument,
    current_source_date: Option<NaiveDate>,
    current_dailies: Vec<DailyArchive>,
    weekly_route: Option<WeeklyRoute>,
}

#[derive(Clone, Debug)]
pub(super) enum ApplyRoute {
    Noop,
    CurrentDailies(Vec<DailyArchive>),
    Weekly(WeeklyRoute),
}

impl PlannedUpdate {
    pub(super) fn route_to(&self, target: NaiveDate) -> Option<ApplyRoute> {
        if self.current_source_date == Some(target) {
            return Some(ApplyRoute::Noop);
        }

        if let Some(position) = self
            .current_dailies
            .iter()
            .position(|archive| archive.date == target)
        {
            return Some(ApplyRoute::CurrentDailies(
                self.current_dailies[..=position].to_vec(),
            ));
        }

        let weekly_route = self.weekly_route.as_ref()?;
        if weekly_route.archive.date == target {
            return Some(ApplyRoute::Weekly(WeeklyRoute {
                archive: weekly_route.archive.clone(),
                dailies: vec![],
            }));
        }
        weekly_route
            .dailies
            .iter()
            .position(|archive| archive.date == target)
            .map(|position| {
                ApplyRoute::Weekly(WeeklyRoute {
                    archive: weekly_route.archive.clone(),
                    dailies: weekly_route.dailies[..=position].to_vec(),
                })
            })
    }
}

pub(super) async fn build_update_plan(
    db: &Database,
    client: &FccClient,
    service: &str,
    service_code: &str,
    today: NaiveDate,
) -> Result<PlannedUpdate> {
    let current_weekly_date = db.get_last_weekly_date(service_code)?;
    let applied: HashSet<_> = db
        .get_applied_patches(service_code)?
        .into_iter()
        .map(|patch| patch.patch_date)
        .collect();

    if current_weekly_date.is_none() && !applied.is_empty() {
        bail!(
            "{} has daily patch metadata but no weekly anchor",
            service_code
        );
    }

    let current_source_date = current_weekly_date
        .map(|weekly_date| {
            contiguous_patch_coverage(weekly_date, &applied).map_err(|gap| {
                anyhow::anyhow!(
                    "{} patch metadata is not contiguous: missing {} before {}",
                    service_code,
                    gap.missing_date,
                    gap.next_available_date
                )
            })
        })
        .transpose()?;

    let daily_observation = download_daily_inventory(client, service_code, today, None).await;
    let (inventory, daily_status) = match daily_observation {
        Ok(inventory) => (inventory, DailyObservationStatus::Complete),
        Err(error) => {
            tracing::warn!(
                service_code,
                error = %error,
                "daily archives could not be included in the update plan"
            );
            (Vec::new(), classify_daily_error(&error))
        }
    };
    let latest_daily_date = inventory.iter().map(|archive| archive.date).max();

    let current_chain =
        current_source_date.map(|source_date| build_chain_from_inventory(source_date, &inventory));
    let current_dailies = current_chain
        .as_ref()
        .map(|chain| chain.contiguous.clone())
        .unwrap_or_default();

    let weekly_observation = inspect_weekly_archive(client, service_code, today).await;
    let (observed_weekly, weekly_status) = match &weekly_observation {
        Ok(archive) => (Some(archive.date), WeeklyObservationStatus::Available),
        Err(error) => {
            tracing::warn!(
                service_code,
                error = %error,
                "weekly archive could not be included in the update plan"
            );
            (None, classify_weekly_error(error))
        }
    };

    if current_source_date.is_none() && weekly_observation.is_err() {
        return Err(weekly_observation
            .expect_err("checked above")
            .context("cannot plan an initial database without a valid weekly archive"));
    }

    let weekly_route = weekly_observation.ok().and_then(|archive| {
        let newer_snapshot = current_weekly_date
            .map(|date| archive.date > date)
            .unwrap_or(true);
        let non_regressing = current_source_date
            .map(|date| archive.date >= date)
            .unwrap_or(true);
        if !newer_snapshot || !non_regressing {
            return None;
        }

        let chain = build_chain_from_inventory(archive.date, &inventory);
        Some(WeeklyRoute {
            archive,
            dailies: chain.contiguous,
        })
    });

    let mut reachable = BTreeSet::new();
    if let Some(date) = current_source_date {
        reachable.insert(date);
    }
    reachable.extend(current_dailies.iter().map(|archive| archive.date));
    if let Some(route) = &weekly_route {
        reachable.insert(route.archive.date);
        reachable.extend(route.dailies.iter().map(|archive| archive.date));
    }
    let reachable_source_dates: Vec<_> = reachable.into_iter().collect();
    let recommended_source_date = reachable_source_dates.last().copied();

    let current_daily_gap = current_chain
        .as_ref()
        .and_then(|chain| chain.gap)
        .map(Into::into);

    Ok(PlannedUpdate {
        document: UpdatePlanDocument {
            format: PLAN_FORMAT,
            format_version: PLAN_FORMAT_VERSION,
            service: service.to_owned(),
            service_code: service_code.to_owned(),
            current: CurrentCoverage {
                weekly_date: current_weekly_date,
                source_date: current_source_date,
            },
            reachable_source_dates,
            recommended_source_date,
            observed: ObservedArchives {
                weekly_date: observed_weekly,
                weekly_status,
                daily_status,
                complete: matches!(
                    weekly_status,
                    WeeklyObservationStatus::Available | WeeklyObservationStatus::NotPublished
                ) && daily_status == DailyObservationStatus::Complete,
                latest_daily_date,
            },
            current_daily_gap,
        },
        current_source_date,
        current_dailies,
        weekly_route,
    })
}

pub(super) fn classify_daily_error(error: &anyhow::Error) -> DailyObservationStatus {
    match error.downcast_ref::<uls_download::DownloadError>() {
        Some(uls_download::DownloadError::Zip(_)) | None => DailyObservationStatus::InvalidArchive,
        Some(_) => DailyObservationStatus::Unavailable,
    }
}

pub(super) fn classify_weekly_error(error: &anyhow::Error) -> WeeklyObservationStatus {
    match error.downcast_ref::<uls_download::DownloadError>() {
        Some(uls_download::DownloadError::NotFound { .. }) => WeeklyObservationStatus::NotPublished,
        Some(uls_download::DownloadError::Zip(_)) => WeeklyObservationStatus::InvalidArchive,
        Some(_) => WeeklyObservationStatus::Unavailable,
        None => WeeklyObservationStatus::InvalidArchive,
    }
}
