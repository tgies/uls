//! Integration tests for service-scoped import and incremental patch apply.

use std::io::Write;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use zip::write::FileOptions;
use zip::ZipWriter;

use uls_db::{Database, DatabaseConfig, ImportMode, Importer};

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

#[test]
fn test_import_for_service_records_import_status() {
    let (temp_dir, db) = create_test_db();
    let zip = weekly_zip(&temp_dir);

    let importer = Importer::new(&db);
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

#[test]
fn test_import_for_service_minimal_skips_history() {
    let (temp_dir, db) = create_test_db();
    let zip = weekly_zip(&temp_dir);

    let importer = Importer::new(&db);
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

#[test]
fn test_import_for_service_clears_prior_status() {
    let (temp_dir, db) = create_test_db();

    // Pre-seed a stale status entry that a fresh import should clear.
    db.mark_imported("HA", "LA", 999).unwrap();
    assert!(db.has_record_type("HA", "LA").unwrap());

    let zip = weekly_zip(&temp_dir);
    let importer = Importer::new(&db);
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

// =============================================================================
// Error paths
// =============================================================================

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
