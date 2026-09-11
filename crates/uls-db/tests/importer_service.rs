//! Integration tests for service-scoped import and incremental patch apply.

use std::io::Write;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use zip::write::FileOptions;
use zip::ZipWriter;

use uls_db::{Database, DatabaseConfig, DbError, ImportMode, Importer};

fn create_test_db() -> (TempDir, Database) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::with_config(DatabaseConfig::with_path(db_path)).unwrap();
    db.initialize().unwrap();
    (temp_dir, db)
}

/// Write a ZIP at `name` containing the given (filename, contents) DAT entries.
fn write_zip(temp_dir: &TempDir, name: &str, entries: &[(&str, &[u8])]) -> PathBuf {
    let zip_path = temp_dir.path().join(name);
    let file = std::fs::File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options: FileOptions<()> = FileOptions::default();
    for (filename, contents) in entries {
        zip.start_file(*filename, options).unwrap();
        zip.write_all(contents).unwrap();
    }
    zip.finish().unwrap();
    zip_path
}

/// Build an HD line for a single license.
fn hd_line(usi: &str, callsign: &str, status: &str) -> String {
    format!(
        "HD|{usi}|0000000001||{callsign}|{status}|HA|01/15/2020|01/15/2030|||||||||||||||||||||||||||||||||||N|||||||||||01/15/2020|01/15/2020|||||||||||||||\n"
    )
}

fn weekly_zip(temp_dir: &TempDir) -> PathBuf {
    write_zip(
        temp_dir,
        "weekly.zip",
        &[
            ("HD.dat", hd_line("100001", "W1WEEK", "A").as_bytes()),
            (
                "EN.dat",
                b"EN|100001|||W1WEEK|L|L00100001|DOE, JOHN|JOHN||DOE||||||||||||000|0001234567|I||||||\n",
            ),
            ("AM.dat", b"AM|100001|||W1WEEK|E|D|6||||||||||\n"),
            ("HS.dat", b"HS|100001||W1WEEK|01/15/2020|LIISS\n"),
        ],
    )
}

// =============================================================================
// import_for_service
// =============================================================================

#[rstest::rstest]
#[case(false)]
#[case(true)]
fn test_import_for_service_records_import_status(#[case] pipelined: bool) {
    let (temp_dir, db) = create_test_db();
    let zip = weekly_zip(&temp_dir);

    let importer = Importer::new(&db).with_pipelined_parsing(pipelined);
    let stats = importer
        .import_for_service(&zip, "HA", ImportMode::Full, None)
        .unwrap();

    assert!(stats.is_successful());
    assert_eq!(stats.records, 4);

    // Every record type present in the ZIP is marked imported for the service.
    for rt in ["HD", "EN", "AM", "HS"] {
        assert!(
            db.has_record_type("HA", rt).unwrap(),
            "{rt} should be marked imported"
        );
    }
    let mut types = db.get_imported_types("HA").unwrap();
    types.sort();
    assert_eq!(types, vec!["AM", "EN", "HD", "HS"]);

    assert!(db.get_license_by_callsign("W1WEEK").unwrap().is_some());
}

#[rstest::rstest]
#[case(false)]
#[case(true)]
fn test_import_for_service_minimal_skips_history(#[case] pipelined: bool) {
    let (temp_dir, db) = create_test_db();
    let zip = weekly_zip(&temp_dir);

    let importer = Importer::new(&db).with_pipelined_parsing(pipelined);
    let stats = importer
        .import_for_service(&zip, "HA", ImportMode::Minimal, None)
        .unwrap();

    // Minimal mode imports HD, EN, AM only.
    assert_eq!(stats.files, 3);
    assert_eq!(stats.records, 3);

    assert!(db.has_record_type("HA", "HD").unwrap());
    assert!(db.has_record_type("HA", "AM").unwrap());
    assert!(!db.has_record_type("HA", "HS").unwrap());
}

#[rstest::rstest]
#[case(false)]
#[case(true)]
fn test_import_for_service_clears_prior_status(#[case] pipelined: bool) {
    let (temp_dir, db) = create_test_db();

    // Pre-seed a stale status entry that a fresh import should clear.
    db.mark_imported("HA", "LA", 999).unwrap();
    assert!(db.has_record_type("HA", "LA").unwrap());

    let zip = weekly_zip(&temp_dir);
    let importer = Importer::new(&db).with_pipelined_parsing(pipelined);
    importer
        .import_for_service(&zip, "HA", ImportMode::Full, None)
        .unwrap();

    // The stale LA entry is gone; only types from this ZIP remain.
    assert!(!db.has_record_type("HA", "LA").unwrap());
    assert!(db.has_record_type("HA", "HD").unwrap());
}

// =============================================================================
// import_patch (daily incremental)
// =============================================================================

#[test]
fn test_import_patch_updates_existing_record() {
    let (temp_dir, db) = create_test_db();

    // Start with an active weekly record.
    let importer = Importer::new(&db);
    importer
        .import_for_service(&weekly_zip(&temp_dir), "HA", ImportMode::Full, None)
        .unwrap();
    assert_eq!(
        db.get_license_by_callsign("W1WEEK")
            .unwrap()
            .unwrap()
            .status,
        'A'
    );

    // Daily patch flips the same license (same USI) to cancelled.
    let patch = write_zip(
        &temp_dir,
        "patch.zip",
        &[("HD.dat", hd_line("100001", "W1WEEK", "C").as_bytes())],
    );

    let stats = importer
        .import_patch(&patch, ImportMode::Full, None)
        .unwrap();
    assert!(stats.is_successful());
    assert_eq!(stats.records, 1);
    assert_eq!(stats.files, 1);

    // INSERT OR REPLACE updated the row rather than adding a duplicate.
    let license = db.get_license_by_callsign("W1WEEK").unwrap().unwrap();
    assert_eq!(license.status, 'C');
    assert_eq!(db.get_stats().unwrap().total_licenses, 1);
}

#[test]
fn test_import_patch_adds_new_record() {
    let (temp_dir, db) = create_test_db();
    let importer = Importer::new(&db);
    importer
        .import_for_service(&weekly_zip(&temp_dir), "HA", ImportMode::Full, None)
        .unwrap();

    let patch = write_zip(
        &temp_dir,
        "patch_new.zip",
        &[("HD.dat", hd_line("200002", "W2NEW", "A").as_bytes())],
    );
    let stats = importer
        .import_patch(&patch, ImportMode::Full, None)
        .unwrap();
    assert_eq!(stats.records, 1);

    assert!(db.get_license_by_callsign("W2NEW").unwrap().is_some());
    assert_eq!(db.get_stats().unwrap().total_licenses, 2);
}

#[test]
fn test_import_patch_does_not_clear_import_status() {
    let (temp_dir, db) = create_test_db();
    let importer = Importer::new(&db);
    importer
        .import_for_service(&weekly_zip(&temp_dir), "HA", ImportMode::Full, None)
        .unwrap();
    assert!(db.has_record_type("HA", "HD").unwrap());

    let patch = write_zip(
        &temp_dir,
        "patch_status.zip",
        &[("HD.dat", hd_line("100001", "W1WEEK", "A").as_bytes())],
    );
    importer
        .import_patch(&patch, ImportMode::Full, None)
        .unwrap();

    // Patches are additive: prior import status remains intact.
    assert!(db.has_record_type("HA", "HD").unwrap());
    assert!(db.has_record_type("HA", "AM").unwrap());
}

#[test]
fn test_import_patch_stream_error_rolls_back() {
    let (temp_dir, db) = create_test_db();
    let importer = Importer::new(&db);
    importer
        .import_for_service(&weekly_zip(&temp_dir), "HA", ImportMode::Full, None)
        .unwrap();

    let mut malformed = hd_line("100001", "W1WEEK", "C").into_bytes();
    malformed.extend_from_slice(hd_line("200002", "W2TEMP", "A").as_bytes());
    malformed.extend_from_slice(&[0xff, b'\n']);
    let patch = write_zip(
        &temp_dir,
        "patch_stream_error.zip",
        &[("HD.dat", &malformed)],
    );

    let error = importer
        .import_patch(&patch, ImportMode::Full, None)
        .unwrap_err();
    assert!(matches!(error, DbError::Parser(_)));
    assert_eq!(
        db.get_license_by_callsign("W1WEEK")
            .unwrap()
            .unwrap()
            .status,
        'A'
    );
    assert!(db.get_license_by_callsign("W2TEMP").unwrap().is_none());

    // The rolled-back transaction must not poison the pooled connection.
    let retry = write_zip(
        &temp_dir,
        "patch_retry.zip",
        &[("HD.dat", hd_line("100001", "W1WEEK", "C").as_bytes())],
    );
    importer
        .import_patch(&retry, ImportMode::Full, None)
        .unwrap();
    assert_eq!(
        db.get_license_by_callsign("W1WEEK")
            .unwrap()
            .unwrap()
            .status,
        'C'
    );
}

#[test]
fn test_import_patch_insert_error_rolls_back() {
    let (temp_dir, db) = create_test_db();
    let importer = Importer::new(&db);
    importer
        .import_for_service(&weekly_zip(&temp_dir), "HA", ImportMode::Full, None)
        .unwrap();

    let patch = write_zip(
        &temp_dir,
        "patch_insert_error.zip",
        &[
            ("HD.dat", hd_line("100001", "W1WEEK", "C").as_bytes()),
            (
                "EN.dat",
                b"EN|999999|||W9ORPHAN|L|L00999999|DOE, JANE|JANE||DOE||||||||||||000|0099999999|I||||||\n",
            ),
        ],
    );

    let error = importer
        .import_patch(&patch, ImportMode::Full, None)
        .unwrap_err();
    assert!(matches!(error, DbError::InvalidData(_)));
    assert_eq!(
        db.get_license_by_callsign("W1WEEK")
            .unwrap()
            .unwrap()
            .status,
        'A'
    );
    assert!(db.get_license_by_callsign("W9ORPHAN").unwrap().is_none());
}

// =============================================================================
// Error paths
// =============================================================================

#[rstest::rstest]
#[case(false)]
#[case(true)]
fn test_weekly_records_remain_ordered_across_batches(#[case] pipelined: bool) {
    let (temp_dir, db) = create_test_db();
    let mut headers = String::new();
    for i in 0..1025 {
        headers.push_str(&hd_line(
            &(100000 + i % 17).to_string(),
            &format!("K{i}A"),
            "A",
        ));
    }
    let zip = write_zip(
        &temp_dir,
        "ordered.zip",
        &[
            ("EN.dat", b"EN|100000|||KFINAL|L||Final name\n"),
            ("HD.dat", headers.as_bytes()),
        ],
    );
    let stats = Importer::new(&db)
        .with_pipelined_parsing(pipelined)
        .import_for_service(&zip, "HA", ImportMode::Full, None)
        .unwrap();
    assert_eq!(stats.records, 1026);
    assert_eq!(db.get_stats().unwrap().total_licenses, 17);
    let conn = db.conn().unwrap();
    for key in 0..17 {
        let last = (0..1025).rev().find(|i| i % 17 == key).unwrap();
        let callsign: String = conn
            .query_row(
                "SELECT call_sign FROM licenses WHERE unique_system_identifier = ?",
                [100000 + key],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(callsign, format!("K{last}A"));
    }
    assert_eq!(
        conn.query_row("SELECT COUNT(*) FROM entities", [], |row| row
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn test_pipelined_progress_panic_cancels_worker_and_rolls_back() {
    let (temp_dir, db) = create_test_db();
    let importer = Importer::new(&db).with_pipelined_parsing(true);
    importer
        .import_for_service(&weekly_zip(&temp_dir), "HA", ImportMode::Full, None)
        .unwrap();
    let mut headers = String::new();
    for i in 0..12000 {
        headers.push_str(&hd_line(&(200000 + i).to_string(), "K1TEMP", "A"));
    }
    let zip = write_zip(
        &temp_dir,
        "cancelled.zip",
        &[("HD.dat", headers.as_bytes())],
    );
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        importer.import_for_service(
            &zip,
            "HA",
            ImportMode::Full,
            Some(Box::new(|_| panic!("progress callback failed"))),
        )
    }));
    assert!(result.is_err());
    assert!(db.get_license_by_callsign("K1TEMP").unwrap().is_none());
    assert_eq!(db.get_stats().unwrap().total_licenses, 1);
    assert!(db.has_record_type("HA", "AM").unwrap());
    {
        let conn = db.conn().unwrap();
        let journal: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(journal, "wal");
        let indexed: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE name = 'idx_licenses_call_sign')",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(indexed);
    }
    importer
        .import_for_service(&weekly_zip(&temp_dir), "HA", ImportMode::Full, None)
        .unwrap();
}

#[rstest::rstest]
#[case(false)]
#[case(true)]
fn test_import_for_service_stream_error_rolls_back_and_preserves_status(#[case] pipelined: bool) {
    let (temp_dir, db) = create_test_db();
    let importer = Importer::new(&db).with_pipelined_parsing(pipelined);
    importer
        .import_for_service(&weekly_zip(&temp_dir), "HA", ImportMode::Full, None)
        .unwrap();
    db.mark_imported("HA", "LA", 999).unwrap();

    let mut malformed = hd_line("200002", "W2BAD", "A").into_bytes();
    // Fail after several buffer-reuse cycles, with a final partial batch.
    for i in 0..2048 {
        malformed.extend_from_slice(hd_line(&(300000 + i).to_string(), "K1TEMP", "A").as_bytes());
    }
    malformed.extend_from_slice(hd_line("200003", "W3TEMP", "A").as_bytes());
    malformed.extend_from_slice(&[0xff, b'\n']);
    let bad_weekly = write_zip(&temp_dir, "bad_weekly.zip", &[("HD.dat", &malformed)]);

    let error = importer
        .import_for_service(&bad_weekly, "HA", ImportMode::Full, None)
        .unwrap_err();
    assert!(matches!(error, DbError::Parser(_)));
    assert!(db.get_license_by_callsign("W2BAD").unwrap().is_none());
    assert!(db.get_license_by_callsign("W3TEMP").unwrap().is_none());
    assert!(db.get_license_by_callsign("W1WEEK").unwrap().is_some());
    assert_eq!(db.get_stats().unwrap().total_licenses, 1);
    assert!(db.has_record_type("HA", "LA").unwrap());
}

#[rstest::rstest]
#[case(false)]
#[case(true)]
fn test_import_for_service_insert_error_rolls_back_and_preserves_status(#[case] pipelined: bool) {
    let (temp_dir, db) = create_test_db();
    let importer = Importer::new(&db).with_pipelined_parsing(pipelined);
    importer
        .import_for_service(&weekly_zip(&temp_dir), "HA", ImportMode::Full, None)
        .unwrap();
    db.mark_imported("HA", "LA", 999).unwrap();

    let bad_weekly = write_zip(
        &temp_dir,
        "bad_weekly_insert.zip",
        &[
            ("HD.dat", hd_line("200002", "W2BAD", "A").as_bytes()),
            (
                "EN.dat",
                b"EN|999999|||W9ORPHAN|L|L00999999|DOE, JANE|JANE||DOE||||||||||||000|0099999999|I||||||\n",
            ),
        ],
    );

    let error = importer
        .import_for_service(&bad_weekly, "HA", ImportMode::Full, None)
        .unwrap_err();
    assert!(matches!(error, DbError::InvalidData(_)));
    assert!(db.get_license_by_callsign("W2BAD").unwrap().is_none());
    assert!(db.get_license_by_callsign("W1WEEK").unwrap().is_some());
    assert!(db.has_record_type("HA", "LA").unwrap());
}

#[test]
fn test_import_for_service_bad_zip_errors() {
    let (temp_dir, db) = create_test_db();
    let bogus = temp_dir.path().join("notazip.zip");
    std::fs::write(&bogus, b"this is not a zip archive").unwrap();

    let importer = Importer::new(&db);
    let result = importer.import_for_service(&bogus, "HA", ImportMode::Full, None);
    assert!(result.is_err());
}

#[test]
fn test_import_patch_nonexistent_file_errors() {
    let (_temp_dir, db) = create_test_db();
    let importer = Importer::new(&db);

    let result = importer.import_patch(Path::new("/nonexistent/patch.zip"), ImportMode::Full, None);
    assert!(result.is_err());
}

#[test]
fn test_import_zip_with_no_matching_files_imports_nothing() {
    let (temp_dir, db) = create_test_db();

    // ZIP contains only a record type that Minimal mode excludes.
    let zip = write_zip(
        &temp_dir,
        "history_only.zip",
        &[("HS.dat", b"HS|100001||W1WEEK|01/15/2020|LIISS\n")],
    );

    let importer = Importer::new(&db);
    let stats = importer
        .import_zip_with_mode(&zip, ImportMode::Minimal, None)
        .unwrap();
    assert_eq!(stats.files, 0);
    assert_eq!(stats.records, 0);
    assert!(stats.is_successful());
}
