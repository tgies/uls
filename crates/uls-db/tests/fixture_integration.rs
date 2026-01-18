//! Integration tests using actual fixture files from tests/fixtures/fcc-sample/
//!
//! These tests verify the full pipeline: reading .dat files -> creating ZIP -> importing -> querying

use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use tempfile::TempDir;
use zip::write::FileOptions;
use zip::ZipWriter;

use uls_db::{Database, DatabaseConfig, ImportMode, Importer};

/// Get the path to the fixture directory
fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures/fcc-sample")
}

/// Create a test database in a temp directory
fn create_test_db() -> (TempDir, Database) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let config = DatabaseConfig::with_path(db_path);
    let db = Database::with_config(config).unwrap();
    db.initialize().unwrap();
    (temp_dir, db)
}

/// Create a ZIP file from the fixture .dat files for a given service
fn create_fixture_zip(temp_dir: &TempDir, service: &str) -> PathBuf {
    let fixture_dir = fixture_path().join(service);
    let zip_path = temp_dir.path().join(format!("{}.zip", service));

    let file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options: FileOptions<()> = FileOptions::default();

    // Add each .dat file from the fixture directory
    for entry in fs::read_dir(&fixture_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "dat") {
            let filename = path.file_name().unwrap().to_str().unwrap();
            let contents = fs::read(&path).unwrap();

            zip.start_file(filename, options.clone()).unwrap();
            zip.write_all(&contents).unwrap();
        }
    }

    zip.finish().unwrap();
    zip_path
}

/// Extract callsigns from a fixture HD.dat file (for runtime test data)
fn get_fixture_callsigns(service: &str) -> Vec<String> {
    let hd_path = fixture_path().join(service).join("HD.dat");
    let file = File::open(&hd_path).expect("HD.dat should exist");
    let reader = BufReader::new(file);

    let mut callsigns = Vec::new();
    for line in reader.lines() {
        if let Ok(line) = line {
            let fields: Vec<&str> = line.split('|').collect();
            if fields.len() > 4 && !fields[4].is_empty() {
                callsigns.push(fields[4].to_string());
            }
        }
    }
    callsigns
}

// =============================================================================
// Amateur fixture integration tests
// =============================================================================

#[test]
fn test_amateur_fixture_import_minimal() {
    let (temp_dir, db) = create_test_db();
    let zip_path = create_fixture_zip(&temp_dir, "l_amat");

    let importer = Importer::new(&db);
    let stats = importer
        .import_zip_with_mode(&zip_path, ImportMode::Minimal, None)
        .unwrap();

    // Should import HD, EN, AM files
    assert!(
        stats.files >= 3,
        "Expected at least 3 files, got {}",
        stats.files
    );
    assert!(stats.records > 0, "Should have imported records");
    assert!(stats.is_successful());
}

#[test]
fn test_amateur_fixture_import_full() {
    let (temp_dir, db) = create_test_db();
    let zip_path = create_fixture_zip(&temp_dir, "l_amat");

    let importer = Importer::new(&db);
    let stats = importer
        .import_zip_with_mode(&zip_path, ImportMode::Full, None)
        .unwrap();

    // Full import should get all record types
    assert!(
        stats.files >= 5,
        "Expected at least 5 files, got {}",
        stats.files
    );
    assert!(stats.records > 30, "Expected more records with full import");
    assert!(stats.is_successful());
}

#[test]
fn test_amateur_fixture_query_callsigns() {
    let (temp_dir, db) = create_test_db();
    let zip_path = create_fixture_zip(&temp_dir, "l_amat");

    let importer = Importer::new(&db);
    importer.import_zip(&zip_path, None).unwrap();

    // Get callsigns from the fixture at runtime
    let callsigns = get_fixture_callsigns("l_amat");
    assert!(!callsigns.is_empty(), "Fixture should have callsigns");

    // Verify at least the first few callsigns can be queried
    let mut found = 0;
    for callsign in callsigns.iter().take(5) {
        if let Some(license) = db.get_license_by_callsign(callsign).unwrap() {
            assert_eq!(&license.call_sign, callsign);
            assert!(
                license.radio_service == "HA" || license.radio_service == "HV",
                "Expected amateur service for {}, got {}",
                callsign,
                license.radio_service
            );
            found += 1;
        }
    }
    assert!(found > 0, "Should find at least one amateur license");
}

#[test]
fn test_amateur_fixture_entity_data() {
    let (temp_dir, db) = create_test_db();
    let zip_path = create_fixture_zip(&temp_dir, "l_amat");

    let importer = Importer::new(&db);
    importer.import_zip(&zip_path, None).unwrap();

    // Get a callsign from fixture and verify entity data
    let callsigns = get_fixture_callsigns("l_amat");
    if let Some(callsign) = callsigns.first() {
        if let Some(license) = db.get_license_by_callsign(callsign).unwrap() {
            // Anonymized data should have fake names in LASTNAME, FIRSTNAME format
            let name = &license.licensee_name;
            assert!(
                !name.is_empty(),
                "Should have licensee name for {}",
                callsign
            );
        }
    }
}

// =============================================================================
// GMRS fixture integration tests
// =============================================================================

#[test]
fn test_gmrs_fixture_import() {
    let (temp_dir, db) = create_test_db();
    let zip_path = create_fixture_zip(&temp_dir, "l_gmrs");

    let importer = Importer::new(&db);
    let stats = importer.import_zip(&zip_path, None).unwrap();

    assert!(stats.records > 0, "Should have imported GMRS records");
    assert!(stats.is_successful());
}

#[test]
fn test_gmrs_fixture_query_callsigns() {
    let (temp_dir, db) = create_test_db();
    let zip_path = create_fixture_zip(&temp_dir, "l_gmrs");

    let importer = Importer::new(&db);
    importer.import_zip(&zip_path, None).unwrap();

    // Get callsigns from the fixture at runtime
    let callsigns = get_fixture_callsigns("l_gmrs");
    assert!(!callsigns.is_empty(), "Fixture should have GMRS callsigns");

    // Verify callsigns can be queried and have correct service
    let mut found = 0;
    for callsign in callsigns.iter().take(5) {
        if let Some(license) = db.get_license_by_callsign(callsign).unwrap() {
            assert_eq!(&license.call_sign, callsign);
            assert_eq!(
                license.radio_service, "ZA",
                "Expected GMRS (ZA) service for {}, got {}",
                callsign, license.radio_service
            );
            found += 1;
        }
    }
    assert!(found > 0, "Should find at least one GMRS license");
}

#[test]
fn test_callsign_formats_are_correct() {
    // Verify amateur callsigns have letter-digit-letter pattern
    let amat_callsigns = get_fixture_callsigns("l_amat");
    for cs in amat_callsigns.iter().take(10) {
        // Amateur: should have digit in middle with letters after
        let chars: Vec<char> = cs.chars().collect();
        let has_digit = chars.iter().any(|c| c.is_ascii_digit());
        let digit_pos = chars.iter().position(|c| c.is_ascii_digit());

        if let Some(pos) = digit_pos {
            let after_digit = &chars[pos + 1..];
            assert!(
                after_digit.iter().any(|c| c.is_ascii_alphabetic()),
                "Amateur callsign {} should have letters after digit",
                cs
            );
        } else {
            panic!("Amateur callsign {} should have a digit", cs);
        }
        assert!(has_digit, "Amateur callsign {} should have digit", cs);
    }

    // Verify GMRS callsigns have letters+digits pattern (no embedded digit)
    let gmrs_callsigns = get_fixture_callsigns("l_gmrs");
    for cs in gmrs_callsigns.iter().take(10) {
        // GMRS: all letters first, then all digits
        let chars: Vec<char> = cs.chars().collect();
        let first_digit = chars.iter().position(|c| c.is_ascii_digit());

        if let Some(pos) = first_digit {
            // Everything before should be letters
            assert!(
                chars[..pos].iter().all(|c| c.is_ascii_alphabetic()),
                "GMRS callsign {} prefix should be all letters",
                cs
            );
            // Everything after (including pos) should be digits
            assert!(
                chars[pos..].iter().all(|c| c.is_ascii_digit()),
                "GMRS callsign {} suffix should be all digits",
                cs
            );
        } else {
            panic!("GMRS callsign {} should have digits", cs);
        }
    }
}

// =============================================================================
// Multi-service combined tests
// =============================================================================

#[test]
fn test_combined_amateur_and_gmrs_import() {
    let (temp_dir, db) = create_test_db();

    // Import both services
    let amat_zip = create_fixture_zip(&temp_dir, "l_amat");
    let gmrs_zip = create_fixture_zip(&temp_dir, "l_gmrs");

    let importer = Importer::new(&db);
    let amat_stats = importer.import_zip(&amat_zip, None).unwrap();
    let gmrs_stats = importer.import_zip(&gmrs_zip, None).unwrap();

    assert!(amat_stats.is_successful());
    assert!(gmrs_stats.is_successful());

    // Get callsigns from fixtures
    let amat_callsigns = get_fixture_callsigns("l_amat");
    let gmrs_callsigns = get_fixture_callsigns("l_gmrs");

    // Verify we can query from both services
    let mut found_amat = false;
    for cs in amat_callsigns.iter().take(3) {
        if db.get_license_by_callsign(cs).unwrap().is_some() {
            found_amat = true;
            break;
        }
    }

    let mut found_gmrs = false;
    for cs in gmrs_callsigns.iter().take(3) {
        if db.get_license_by_callsign(cs).unwrap().is_some() {
            found_gmrs = true;
            break;
        }
    }

    assert!(
        found_amat,
        "Should find amateur callsigns after combined import"
    );
    assert!(
        found_gmrs,
        "Should find GMRS callsigns after combined import"
    );
}

#[test]
fn test_service_counts_after_import() {
    let (temp_dir, db) = create_test_db();

    let amat_zip = create_fixture_zip(&temp_dir, "l_amat");
    let gmrs_zip = create_fixture_zip(&temp_dir, "l_gmrs");

    let importer = Importer::new(&db);
    importer.import_zip(&amat_zip, None).unwrap();
    importer.import_zip(&gmrs_zip, None).unwrap();

    // Verify counts are tracked separately
    let amat_count = db.count_by_service(&["HA", "HV"]).unwrap_or(0);
    let gmrs_count = db.count_by_service(&["ZA"]).unwrap_or(0);

    assert!(amat_count > 0, "Should have amateur records");
    assert!(gmrs_count > 0, "Should have GMRS records");
}
