//! Exact-target service batches. Network planning precedes writable DB access.

use super::*;

const SERVICES: [(&str, &str); 2] = [("amateur", "HA"), ("gmrs", "ZA")];

#[derive(Debug, Serialize)]
struct BatchApplyDocument {
    format: &'static str,
    format_version: u8,
    target_source_date: NaiveDate,
    services: Vec<UpdateApplyDocument>,
}

pub(super) async fn execute(
    options: UpdateOptions,
    db_path: &Path,
    download_config: DownloadConfig,
    today: NaiveDate,
) -> Result<()> {
    let target = options
        .through
        .ok_or_else(|| anyhow::anyhow!("--service all requires --through YYYY-MM-DD"))?;
    if options.plan || options.check_only || options.daily_only || options.force {
        bail!("--service all --through conflicts with --plan, --check, --daily-only and --force");
    }
    let structured = matches!(options.format.as_str(), "json" | "json-pretty");
    let mode = if options.minimal {
        ImportMode::Minimal
    } else {
        ImportMode::Full
    };
    let client = FccClient::new(download_config)?;
    let planning_db = open_database_for_planning(db_path)?;
    let plans = plan(&planning_db, &client, today, target).await?;
    drop(planning_db);

    let db = open_database_for_update(db_path, structured)?;
    let result = apply(&db, &client, &mode, plans, target, structured)?;
    if structured {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("\n✓ Amateur and GMRS reached FCC source date {target}.");
    }
    Ok(())
}

async fn plan(
    db: &Database,
    client: &FccClient,
    today: NaiveDate,
    target: NaiveDate,
) -> Result<Vec<planner::PlannedUpdate>> {
    let mut plans = Vec::new();
    for (service, code) in SERVICES {
        let planned = planner::build_update_plan(db, client, service, code, today).await?;
        if planned.route_to(target).is_none() {
            bail!("source date {target} is not exactly reachable for {code}");
        }
        plans.push(planned);
    }
    Ok(plans)
}

fn apply(
    db: &Database,
    client: &FccClient,
    mode: &ImportMode,
    plans: Vec<planner::PlannedUpdate>,
    target: NaiveDate,
    structured: bool,
) -> Result<BatchApplyDocument> {
    if plans.len() != SERVICES.len() {
        bail!("batch requires exactly one Amateur and one GMRS plan");
    }
    // Check every base and route before the first service can commit anything.
    let mut routes = Vec::new();
    for ((_, code), planned) in SERVICES.iter().zip(&plans) {
        if planned.document.service_code != *code {
            bail!("batch plan service order is invalid");
        }
        verify_planned_base(db, planned, code)?;
        routes.push(planned.route_to(target).ok_or_else(|| {
            anyhow::anyhow!("source date {target} is not exactly reachable for {code}")
        })?);
    }

    let applied = Importer::new(db).batch(|batch| -> Result<_> {
        let mut applied = Vec::new();
        for ((_, code), route) in SERVICES.iter().zip(routes) {
            let result = match route {
                planner::ApplyRoute::Noop => ("none", false, 0),
                planner::ApplyRoute::CurrentDailies(dailies) => (
                    "daily",
                    false,
                    apply_dailies_in_batch(batch, code, mode, &dailies, structured)?,
                ),
                planner::ApplyRoute::Weekly(route) => {
                    import_weekly_in_batch(batch, client, code, mode, &route.archive, structured)?;
                    let count =
                        apply_dailies_in_batch(batch, code, mode, &route.dailies, structured)?;
                    ("weekly", true, count)
                }
            };
            applied.push(result);
        }
        Ok(applied)
    })?;

    // Cleanup has succeeded before checking and serializing the final result.
    let mut services = Vec::new();
    for (((_, code), planned), (route, weekly_applied, daily_updates_applied)) in
        SERVICES.iter().zip(plans).zip(applied)
    {
        let source_date = database_source_date(db, code)?;
        if source_date != Some(target) {
            bail!("{code} reached source date {source_date:?}, expected exact target {target}");
        }
        let previous_source_date = planned.document.current.source_date;
        services.push(UpdateApplyDocument {
            format: "uls.update_result",
            format_version: 1,
            service_code: (*code).to_owned(),
            previous_source_date,
            source_date: target,
            target_source_date: target,
            changed: previous_source_date != Some(target),
            route,
            weekly_applied,
            daily_updates_applied,
        });
    }
    Ok(BatchApplyDocument {
        format: "uls.update_batch_result",
        format_version: 1,
        target_source_date: target,
        services,
    })
}

#[cfg(test)]
mod tests {
    use super::super::tests::{
        build_fixture_zip, default_update_options, fresh_db, mount_zip, test_client,
        test_download_config, write_malformed_patch,
    };
    use super::*;
    use tempfile::TempDir;
    use wiremock::MockServer;

    fn date(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 7, day).unwrap()
    }

    async fn weeklies(server: &MockServer, timestamp: &str) {
        mount_zip(
            server,
            "/complete/l_amat.zip",
            build_fixture_zip("l_amat", timestamp),
        )
        .await;
        mount_zip(
            server,
            "/complete/l_gmrs.zip",
            build_fixture_zip("l_gmrs", timestamp),
        )
        .await;
    }

    fn schema_version(db: &Database) -> i64 {
        db.conn()
            .unwrap()
            .pragma_query_value(None, "schema_version", |row| row.get(0))
            .unwrap()
    }

    fn assert_ready(db: &Database, expected_indexes: i64) {
        let conn = db.conn().unwrap();
        assert_eq!(
            conn.query_row::<i64, _, _>(
                "SELECT count(*) FROM sqlite_schema WHERE type='index'",
                [],
                |row| row.get(0)
            )
            .unwrap(),
            expected_indexes
        );
        assert_eq!(
            conn.pragma_query_value::<String, _>(None, "journal_mode", |row| row.get(0))
                .unwrap(),
            "wal"
        );
        assert_eq!(
            conn.pragma_query_value::<i64, _>(None, "synchronous", |row| row.get(0))
                .unwrap(),
            1
        );
        assert_eq!(
            conn.query_row::<String, _, _>("PRAGMA quick_check", [], |row| row.get(0))
                .unwrap(),
            "ok"
        );
    }

    fn index_count(db: &Database) -> i64 {
        db.conn()
            .unwrap()
            .query_row(
                "SELECT count(*) FROM sqlite_schema WHERE type='index'",
                [],
                |row| row.get(0),
            )
            .unwrap()
    }

    #[tokio::test]
    async fn combined_entrypoint_commits_both_services_in_each_output_mode() {
        for (minimal, format) in [(true, "json"), (false, "table")] {
            let server = MockServer::start().await;
            let dir = TempDir::new().unwrap();
            weeklies(&server, "Fri Jul 17 08:00:00 EDT 2026").await;
            let path = dir.path().join("new.db");
            let mut options = default_update_options("ALL");
            options.through = Some(date(17));
            options.minimal = minimal;
            options.format = format.into();
            execute_with_context(
                options,
                &path,
                test_download_config(&server, &dir.path().join("cache")),
                date(23),
            )
            .await
            .unwrap();
            let db = Database::with_config(DatabaseConfig::with_path(&path)).unwrap();
            for code in ["HA", "ZA"] {
                assert_eq!(database_source_date(&db, code).unwrap(), Some(date(17)));
                assert!(db.count_by_service(&[code]).unwrap() > 0);
            }
            assert_eq!(
                db.get_last_updated().unwrap().as_deref(),
                Some("Fri Jul 17 08:00:00 EDT 2026")
            );
        }
    }

    #[tokio::test]
    async fn single_gmrs_entrypoint_remains_supported() {
        let server = MockServer::start().await;
        let dir = TempDir::new().unwrap();
        mount_zip(
            &server,
            "/complete/l_gmrs.zip",
            build_fixture_zip("l_gmrs", "Fri Jul 17 08:00:00 EDT 2026"),
        )
        .await;
        let path = dir.path().join("gmrs.db");
        let mut options = default_update_options("gmrs");
        options.through = Some(date(17));
        execute_with_context(
            options,
            &path,
            test_download_config(&server, &dir.path().join("cache")),
            date(23),
        )
        .await
        .unwrap();
        let db = Database::with_config(DatabaseConfig::with_path(&path)).unwrap();
        assert_eq!(database_source_date(&db, "ZA").unwrap(), Some(date(17)));
        assert_eq!(database_source_date(&db, "HA").unwrap(), None);
        assert!(db.count_by_service(&["ZA"]).unwrap() > 0);
        assert_eq!(db.count_by_service(&["HA"]).unwrap(), 0);
    }

    #[tokio::test]
    async fn unsupported_combined_options_fail_before_network_or_database_access() {
        let server = MockServer::start().await;
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("absent.db");
        for flag in ["missing-target", "plan", "check", "daily-only", "force"] {
            let mut options = default_update_options("all");
            options.through = Some(date(17));
            match flag {
                "missing-target" => options.through = None,
                "plan" => options.plan = true,
                "check" => options.check_only = true,
                "daily-only" => options.daily_only = true,
                "force" => options.force = true,
                _ => unreachable!(),
            }
            let error = execute_with_context(
                options,
                &path,
                test_download_config(&server, &dir.path().join("cache")),
                date(23),
            )
            .await
            .unwrap_err();
            let expected = if flag == "missing-target" {
                "requires --through"
            } else {
                "conflicts with"
            };
            assert!(error.to_string().contains(expected), "{flag}: {error}");
            assert!(!path.exists());
        }
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn archive_timestamp_drift_is_rejected_before_importing_records() {
        for timestamp in ["Sat Jul 18 08:00:00 EDT 2026", ""] {
            let server = MockServer::start().await;
            let dir = TempDir::new().unwrap();
            let db = fresh_db(dir.path());
            let client = test_client(&server, &dir.path().join("cache"));
            weeklies(&server, "Fri Jul 17 08:00:00 EDT 2026").await;
            let plans = plan(&db, &client, date(23), date(17)).await.unwrap();
            let planner::ApplyRoute::Weekly(route) = plans[0].route_to(date(17)).unwrap() else {
                panic!("weekly expected")
            };
            std::fs::write(&route.archive.path, build_fixture_zip("l_amat", timestamp)).unwrap();
            let error =
                apply(&db, &client, &ImportMode::Minimal, plans, date(17), true).unwrap_err();
            assert!(error.to_string().contains("FCC creation"), "{error:#}");
            assert_eq!(db.get_stats().unwrap().total_licenses, 0);
            assert_eq!(database_source_date(&db, "HA").unwrap(), None);
        }
    }

    #[tokio::test]
    async fn later_unreachable_service_does_not_create_database() {
        let server = MockServer::start().await;
        let dir = TempDir::new().unwrap();
        mount_zip(
            &server,
            "/complete/l_amat.zip",
            build_fixture_zip("l_amat", "Fri Jul 17 08:00:00 EDT 2026"),
        )
        .await;
        let path = dir.path().join("absent.db");
        let mut options = default_update_options("all");
        options.through = Some(date(17));
        let error = execute(
            options,
            &path,
            test_download_config(&server, &dir.path().join("cache")),
            date(23),
        )
        .await
        .unwrap_err();
        assert!(
            error.to_string().contains("not exactly reachable for ZA"),
            "{error:#}"
        );
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn both_bases_are_checked_before_first_archive_is_applied() {
        let server = MockServer::start().await;
        let dir = TempDir::new().unwrap();
        let db = fresh_db(dir.path());
        let client = test_client(&server, &dir.path().join("cache"));
        weeklies(&server, "Fri Jul 17 08:00:00 EDT 2026").await;
        let plans = plan(&db, &client, date(23), date(17)).await.unwrap();
        db.set_last_weekly_date("ZA", date(12)).unwrap();
        let version = schema_version(&db);
        let error = apply(&db, &client, &ImportMode::Minimal, plans, date(17), true).unwrap_err();
        assert!(error
            .to_string()
            .contains("ZA database changed while planning"));
        assert_eq!(db.get_stats().unwrap().total_licenses, 0);
        assert_eq!(db.get_last_weekly_date("HA").unwrap(), None);
        assert_eq!(schema_version(&db), version);
    }

    #[tokio::test]
    async fn later_weekly_failure_preserves_first_service_and_retry_uses_noop() {
        let server = MockServer::start().await;
        let dir = TempDir::new().unwrap();
        let db = fresh_db(dir.path());
        let client = test_client(&server, &dir.path().join("cache"));
        weeklies(&server, "Fri Jul 17 08:00:00 EDT 2026").await;
        let indexes = index_count(&db);
        let plans = plan(&db, &client, date(23), date(17)).await.unwrap();
        let planner::ApplyRoute::Weekly(route) = plans[1].route_to(date(17)).unwrap() else {
            panic!("weekly expected")
        };
        write_malformed_patch(&route.archive.path);
        let error = apply(&db, &client, &ImportMode::Minimal, plans, date(17), true).unwrap_err();
        assert!(error.to_string().contains("parser error"), "{error:#}");
        assert_eq!(database_source_date(&db, "HA").unwrap(), Some(date(17)));
        assert_eq!(database_source_date(&db, "ZA").unwrap(), None);
        assert_eq!(db.count_by_service(&["ZA"]).unwrap(), 0);
        assert_ready(&db, indexes);
        std::fs::write(
            &route.archive.path,
            build_fixture_zip("l_gmrs", "Fri Jul 17 08:00:00 EDT 2026"),
        )
        .unwrap();
        let plans = plan(&db, &client, date(23), date(17)).await.unwrap();
        let result = apply(&db, &client, &ImportMode::Minimal, plans, date(17), true).unwrap();
        let json = serde_json::to_value(result).unwrap();
        assert_eq!(json["format"], "uls.update_batch_result");
        assert_eq!(json["format_version"], 1);
        assert_eq!(json["services"].as_array().unwrap().len(), 2);
        assert_eq!(json["services"][0]["service_code"], "HA");
        assert_eq!(json["services"][0]["changed"], false);
        assert_eq!(json["services"][0]["route"], "none");
        assert_eq!(json["services"][1]["service_code"], "ZA");
        assert_eq!(json["services"][1]["route"], "weekly");
        assert_ready(&db, indexes);
    }

    #[tokio::test]
    async fn weekly_daily_batches_stop_at_exact_prefix_and_noop_keeps_schema() {
        let server = MockServer::start().await;
        let dir = TempDir::new().unwrap();
        let db = fresh_db(dir.path());
        let client = test_client(&server, &dir.path().join("cache"));
        weeklies(&server, "Sun Jul 19 08:00:00 EDT 2026").await;
        for (prefix, fixture) in [("am", "l_amat"), ("gm", "l_gmrs")] {
            mount_zip(
                &server,
                &format!("/daily/l_{prefix}_sun.zip"),
                build_fixture_zip(fixture, "Mon Jul 20 08:00:00 EDT 2026"),
            )
            .await;
            mount_zip(
                &server,
                &format!("/daily/l_{prefix}_tue.zip"),
                build_fixture_zip(fixture, "Wed Jul 22 08:00:00 EDT 2026"),
            )
            .await;
        }
        let plans = plan(&db, &client, date(23), date(20)).await.unwrap();
        let indexes = index_count(&db);
        let result = apply(&db, &client, &ImportMode::Minimal, plans, date(20), true).unwrap();
        for service in &result.services {
            assert!(service.weekly_applied);
            assert_eq!(service.daily_updates_applied, 1);
            assert_eq!(
                database_source_date(&db, &service.service_code).unwrap(),
                Some(date(20))
            );
            assert_eq!(
                db.get_applied_patches(&service.service_code).unwrap().len(),
                1
            );
        }
        assert_ready(&db, indexes);
        let version = schema_version(&db);
        let plans = plan(&db, &client, date(23), date(20)).await.unwrap();
        let result = apply(&db, &client, &ImportMode::Minimal, plans, date(20), true).unwrap();
        assert!(result
            .services
            .iter()
            .all(|service| !service.changed && service.route == "none"));
        assert_eq!(schema_version(&db), version);
        assert!(plan(&db, &client, date(23), date(22)).await.is_err());
        assert_eq!(schema_version(&db), version);
    }

    #[tokio::test]
    async fn daily_and_weekly_routes_preserve_each_service_history() {
        let server = MockServer::start().await;
        let dir = TempDir::new().unwrap();
        let db = fresh_db(dir.path());
        let client = test_client(&server, &dir.path().join("cache"));
        db.set_last_weekly_date("HA", date(16)).unwrap();
        mount_zip(
            &server,
            "/daily/l_am_thu.zip",
            build_fixture_zip("l_amat", "Fri Jul 17 08:00:00 EDT 2026"),
        )
        .await;
        mount_zip(
            &server,
            "/complete/l_gmrs.zip",
            build_fixture_zip("l_gmrs", "Fri Jul 17 08:00:00 EDT 2026"),
        )
        .await;
        let plans = plan(&db, &client, date(23), date(17)).await.unwrap();
        let result = apply(&db, &client, &ImportMode::Minimal, plans, date(17), true).unwrap();
        assert_eq!(result.services[0].route, "daily");
        assert_eq!(result.services[1].route, "weekly");
        assert_eq!(db.get_last_weekly_date("HA").unwrap(), Some(date(16)));
        assert_eq!(db.get_applied_patches("HA").unwrap().len(), 1);
        assert_eq!(db.get_last_weekly_date("ZA").unwrap(), Some(date(17)));
    }

    #[tokio::test]
    async fn final_target_failure_cannot_report_batch_success() {
        let server = MockServer::start().await;
        let dir = TempDir::new().unwrap();
        let db = fresh_db(dir.path());
        let client = test_client(&server, &dir.path().join("cache"));
        weeklies(&server, "Fri Jul 17 08:00:00 EDT 2026").await;
        let plans = plan(&db, &client, date(23), date(17)).await.unwrap();
        db.conn().unwrap().execute_batch("CREATE TRIGGER change_target AFTER INSERT ON metadata WHEN NEW.key='last_weekly_date_ZA' BEGIN UPDATE metadata SET value='2026-07-16' WHERE key=NEW.key; END;").unwrap();
        let indexes = index_count(&db);
        let error = apply(&db, &client, &ImportMode::Minimal, plans, date(17), true).unwrap_err();
        assert!(error.to_string().contains("expected exact target"));
        assert_ready(&db, indexes);
    }
}
