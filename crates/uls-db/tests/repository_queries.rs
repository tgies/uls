//! Integration tests for Database query methods: freshness/patch tracking,
//! stats aggregation, and FRN lookups against an in-memory database.

use chrono::NaiveDate;

use uls_core::records::{EntityRecord, HeaderRecord, UlsRecord};
use uls_db::{Database, DatabaseConfig};

/// Create an in-memory database for query tests.
fn create_test_db() -> Database {
    let db = Database::with_config(DatabaseConfig::in_memory()).unwrap();
    db.initialize().unwrap();
    db
}

/// Build a header record with explicit identifiers and status.
fn header(usi: i64, callsign: &str, status: char, service: &str) -> HeaderRecord {
    let mut hd = HeaderRecord::from_fields(&["HD", &usi.to_string()]);
    hd.unique_system_identifier = usi;
    hd.call_sign = Some(callsign.to_string());
    hd.license_status = Some(status);
    hd.radio_service_code = Some(service.to_string());
    hd
}

/// Build an entity record carrying the given FRN for a license.
fn entity_with_frn(usi: i64, callsign: &str, frn: &str) -> EntityRecord {
    EntityRecord::from_fields(&[
        "EN",
        &usi.to_string(),
        "",
        "",
        callsign,
        "L",
        "L00100001",
        "DOE, JOHN A",
        "JOHN",
        "A",
        "DOE",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        "000",
        frn,
        "I",
        "",
        "",
        "",
        "",
        "",
        "",
    ])
}

// =============================================================================
// Stats aggregation
// =============================================================================

#[test]
fn test_stats_counts_each_status() {
    let db = create_test_db();
    db.insert_record(&UlsRecord::Header(header(1, "W1A", 'A', "HA")))
        .unwrap();
    db.insert_record(&UlsRecord::Header(header(2, "W2A", 'A', "HA")))
        .unwrap();
    db.insert_record(&UlsRecord::Header(header(3, "W3E", 'E', "HA")))
        .unwrap();
    db.insert_record(&UlsRecord::Header(header(4, "W4C", 'C', "HA")))
        .unwrap();

    let stats = db.get_stats().unwrap();
    assert_eq!(stats.total_licenses, 4);
    assert_eq!(stats.active_licenses, 2);
    assert_eq!(stats.expired_licenses, 1);
    assert_eq!(stats.cancelled_licenses, 1);
    assert_eq!(stats.schema_version, uls_db::schema::SCHEMA_VERSION);
}

#[test]
fn test_stats_last_updated_reflects_metadata() {
    let db = create_test_db();

    let stats = db.get_stats().unwrap();
    assert!(stats.last_updated.is_none());

    db.set_last_updated("2026-01-13T08:00:00Z").unwrap();
    let stats = db.get_stats().unwrap();
    assert_eq!(stats.last_updated, Some("2026-01-13T08:00:00Z".to_string()));
}

#[test]
fn test_stats_empty_database() {
    let db = create_test_db();
    let stats = db.get_stats().unwrap();
    assert_eq!(stats.total_licenses, 0);
    assert_eq!(stats.active_licenses, 0);
    assert_eq!(stats.expired_licenses, 0);
    assert_eq!(stats.cancelled_licenses, 0);
}

// =============================================================================
// FRN lookups
// =============================================================================

#[test]
fn test_get_licenses_by_frn_returns_all_matches() {
    let db = create_test_db();

    // Two licenses share one FRN, a third has a different FRN.
    db.insert_record(&UlsRecord::Header(header(10, "AAA", 'A', "HA")))
        .unwrap();
    db.insert_record(&UlsRecord::Entity(entity_with_frn(10, "AAA", "0009999999")))
        .unwrap();

    db.insert_record(&UlsRecord::Header(header(11, "BBB", 'A', "ZA")))
        .unwrap();
    db.insert_record(&UlsRecord::Entity(entity_with_frn(11, "BBB", "0009999999")))
        .unwrap();

    db.insert_record(&UlsRecord::Header(header(12, "CCC", 'A', "HA")))
        .unwrap();
    db.insert_record(&UlsRecord::Entity(entity_with_frn(12, "CCC", "0001111111")))
        .unwrap();

    let licenses = db.get_licenses_by_frn("0009999999").unwrap();
    assert_eq!(licenses.len(), 2);
    let callsigns: Vec<&str> = licenses.iter().map(|l| l.call_sign.as_str()).collect();
    assert!(callsigns.contains(&"AAA"));
    assert!(callsigns.contains(&"BBB"));
    for license in &licenses {
        assert_eq!(license.frn.as_deref(), Some("0009999999"));
    }
}

#[test]
fn test_get_licenses_by_frn_trims_input() {
    let db = create_test_db();
    db.insert_record(&UlsRecord::Header(header(20, "DDD", 'A', "HA")))
        .unwrap();
    db.insert_record(&UlsRecord::Entity(entity_with_frn(20, "DDD", "0002222222")))
        .unwrap();

    let licenses = db.get_licenses_by_frn("  0002222222  ").unwrap();
    assert_eq!(licenses.len(), 1);
    assert_eq!(licenses[0].call_sign, "DDD");
}

#[test]
fn test_get_licenses_by_frn_empty_when_no_entity() {
    let db = create_test_db();
    // Header with no entity row: the INNER JOIN on entities yields nothing.
    db.insert_record(&UlsRecord::Header(header(30, "EEE", 'A', "HA")))
        .unwrap();

    let licenses = db.get_licenses_by_frn("0001234567").unwrap();
    assert!(licenses.is_empty());
}

// =============================================================================
// count_by_service edge cases
// =============================================================================

#[test]
fn test_count_by_service_multiple_codes() {
    let db = create_test_db();
    db.insert_record(&UlsRecord::Header(header(40, "HAM1", 'A', "HA")))
        .unwrap();
    db.insert_record(&UlsRecord::Header(header(41, "HAM2", 'A', "HV")))
        .unwrap();
    db.insert_record(&UlsRecord::Header(header(42, "GM1", 'A', "ZA")))
        .unwrap();

    assert_eq!(db.count_by_service(&["HA", "HV"]).unwrap(), 2);
    assert_eq!(db.count_by_service(&["ZA"]).unwrap(), 1);
    assert_eq!(db.count_by_service(&["HA", "HV", "ZA"]).unwrap(), 3);
}

#[test]
fn test_count_by_service_unknown_code_returns_zero() {
    let db = create_test_db();
    db.insert_record(&UlsRecord::Header(header(50, "X1", 'A', "HA")))
        .unwrap();

    // "QQ" is not a recognized RadioService, so it filters out to no codes.
    assert_eq!(db.count_by_service(&["QQ"]).unwrap(), 0);
}

// =============================================================================
// Last-updated / last-weekly metadata
// =============================================================================

#[test]
fn test_last_updated_roundtrip() {
    let db = create_test_db();
    assert!(db.get_last_updated().unwrap().is_none());

    db.set_last_updated("2026-01-13 08:00:00 UTC").unwrap();
    assert_eq!(
        db.get_last_updated().unwrap(),
        Some("2026-01-13 08:00:00 UTC".to_string())
    );
}

#[test]
fn test_last_weekly_date_roundtrip_per_service() {
    let db = create_test_db();
    assert!(db.get_last_weekly_date("HA").unwrap().is_none());

    let date = NaiveDate::from_ymd_opt(2026, 1, 4).unwrap();
    db.set_last_weekly_date("HA", date).unwrap();

    assert_eq!(db.get_last_weekly_date("HA").unwrap(), Some(date));
    // A different service has its own slot.
    assert!(db.get_last_weekly_date("ZA").unwrap().is_none());
}

// =============================================================================
// Applied patch tracking
// =============================================================================

#[test]
fn test_applied_patches_record_and_order() {
    let db = create_test_db();

    // Insert out of date order to verify the query sorts by patch_date.
    db.record_applied_patch(
        "HA",
        NaiveDate::from_ymd_opt(2026, 1, 8).unwrap(),
        "thu",
        Some("etag-thu"),
        Some(120),
    )
    .unwrap();
    db.record_applied_patch(
        "HA",
        NaiveDate::from_ymd_opt(2026, 1, 6).unwrap(),
        "tue",
        None,
        Some(80),
    )
    .unwrap();

    let patches = db.get_applied_patches("HA").unwrap();
    assert_eq!(patches.len(), 2);
    assert_eq!(
        patches[0].patch_date,
        NaiveDate::from_ymd_opt(2026, 1, 6).unwrap()
    );
    assert_eq!(
        patches[1].patch_date,
        NaiveDate::from_ymd_opt(2026, 1, 8).unwrap()
    );

    assert_eq!(patches[0].weekday, "tue");
    assert_eq!(patches[0].etag, None);
    assert_eq!(patches[0].record_count, Some(80));

    assert_eq!(patches[1].weekday, "thu");
    assert_eq!(patches[1].etag, Some("etag-thu".to_string()));
    assert_eq!(patches[1].record_count, Some(120));
}

#[test]
fn test_applied_patch_replace_same_date() {
    let db = create_test_db();
    let date = NaiveDate::from_ymd_opt(2026, 1, 9).unwrap();

    db.record_applied_patch("HA", date, "fri", Some("old"), Some(10))
        .unwrap();
    db.record_applied_patch("HA", date, "fri", Some("new"), Some(99))
        .unwrap();

    let patches = db.get_applied_patches("HA").unwrap();
    assert_eq!(patches.len(), 1);
    assert_eq!(patches[0].etag, Some("new".to_string()));
    assert_eq!(patches[0].record_count, Some(99));
}

#[test]
fn test_clear_applied_patches_only_targets_service() {
    let db = create_test_db();
    db.record_applied_patch(
        "HA",
        NaiveDate::from_ymd_opt(2026, 1, 6).unwrap(),
        "tue",
        None,
        None,
    )
    .unwrap();
    db.record_applied_patch(
        "ZA",
        NaiveDate::from_ymd_opt(2026, 1, 6).unwrap(),
        "tue",
        None,
        None,
    )
    .unwrap();

    db.clear_applied_patches("HA").unwrap();

    assert!(db.get_applied_patches("HA").unwrap().is_empty());
    assert_eq!(db.get_applied_patches("ZA").unwrap().len(), 1);
}

#[test]
fn test_get_applied_patches_empty_for_unknown_service() {
    let db = create_test_db();
    assert!(db.get_applied_patches("HA").unwrap().is_empty());
}

// =============================================================================
// Freshness composition and staleness
// =============================================================================

#[test]
fn test_get_freshness_combines_metadata_and_patches() {
    let db = create_test_db();

    db.set_last_updated("2026-01-13 08:00:00 UTC").unwrap();
    db.set_last_weekly_date("HA", NaiveDate::from_ymd_opt(2026, 1, 4).unwrap())
        .unwrap();
    db.record_applied_patch(
        "HA",
        NaiveDate::from_ymd_opt(2026, 1, 6).unwrap(),
        "tue",
        None,
        Some(5),
    )
    .unwrap();

    let freshness = db.get_freshness("HA", 3).unwrap();
    assert_eq!(freshness.service, "HA");
    assert_eq!(
        freshness.last_weekly_date,
        Some(NaiveDate::from_ymd_opt(2026, 1, 4).unwrap())
    );
    assert_eq!(
        freshness.applied_patch_dates,
        vec![NaiveDate::from_ymd_opt(2026, 1, 6).unwrap()]
    );
    assert!(freshness.last_updated.is_some());
}

#[test]
fn test_is_stale_when_no_timestamp() {
    let db = create_test_db();
    // No last_updated metadata means the data is treated as stale.
    assert!(db.is_stale("HA", 3).unwrap());
}

#[test]
fn test_is_stale_false_for_recent_update() {
    let db = create_test_db();
    let now = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();
    db.set_last_updated(&now).unwrap();
    assert!(!db.is_stale("HA", 3).unwrap());
}
