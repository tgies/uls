//! CLI binary integration tests.
//!
//! These tests run the actual `uls` binary with assert_cmd,
//! testing real command-line behavior with stdout/stderr assertions.
//!
//! NOTE: These tests require a pre-populated database or use --help/error cases.
//! For tests requiring data, see setup_test_db() which creates a temp DB with fixtures.

// TODO: Migrate from deprecated Command::cargo_bin to cargo_bin_cmd! macro
#![allow(deprecated)]

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;
use zip::write::FileOptions;
use zip::ZipWriter;

/// Get the path to the fixture directory
fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures/fcc-sample")
}

/// Create a test database with fixture data and return path
fn setup_test_db(services: &[&str]) -> (TempDir, PathBuf) {
    use uls_db::{Database, DatabaseConfig, ImportMode, Importer};

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let config = DatabaseConfig::with_path(&db_path);
    let db = Database::with_config(config).unwrap();
    db.initialize().unwrap();

    let importer = Importer::new(&db);

    for service in services {
        let fixture_dir = fixture_path().join(service);
        let zip_path = temp_dir.path().join(format!("{}.zip", service));

        let file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options: FileOptions<()> = FileOptions::default();

        for entry in fs::read_dir(&fixture_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "dat") {
                let filename = path.file_name().unwrap().to_str().unwrap();
                let contents = fs::read(&path).unwrap();
                zip.start_file(filename, options).unwrap();
                zip.write_all(&contents).unwrap();
            }
        }
        zip.finish().unwrap();

        // Map fixture folder name to service code
        let service_code = match *service {
            "l_amat" => "HA",
            "l_gmrs" => "ZA",
            _ => "HA",
        };

        // Use import_for_service to properly update import_status table
        importer
            .import_for_service(&zip_path, service_code, ImportMode::Full, None)
            .unwrap();
    }

    drop(db);
    (temp_dir, db_path)
}

/// Get first callsign from fixture HD.dat
fn get_first_callsign(service: &str) -> String {
    use std::io::{BufRead, BufReader};

    let hd_path = fixture_path().join(service).join("HD.dat");
    let file = File::open(&hd_path).expect("HD.dat should exist");
    let reader = BufReader::new(file);

    for line in reader.lines().map_while(Result::ok) {
        let fields: Vec<&str> = line.split('|').collect();
        if fields.len() > 4 && !fields[4].is_empty() {
            return fields[4].to_string();
        }
    }
    panic!("No callsign found in fixture");
}

// =============================================================================
// Help and version tests (no database needed)
// =============================================================================

#[test]
fn test_cli_help() {
    Command::cargo_bin("uls")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Universal Licensing System"))
        .stdout(predicate::str::contains("lookup"))
        .stdout(predicate::str::contains("search"));
}

#[test]
fn test_cli_version() {
    Command::cargo_bin("uls")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("uls"));
}

#[test]
fn test_lookup_help() {
    Command::cargo_bin("uls")
        .unwrap()
        .args(["lookup", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("callsign"));
}

#[test]
fn test_search_help() {
    Command::cargo_bin("uls")
        .unwrap()
        .args(["search", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("name"))
        .stdout(predicate::str::contains("city"));
}

#[test]
fn test_stats_help() {
    Command::cargo_bin("uls")
        .unwrap()
        .args(["stats", "--help"])
        .assert()
        .success();
}

// =============================================================================
// Lookup command tests (with database)
// =============================================================================

#[test]
fn test_lookup_with_fixture_data() {
    let (_temp_dir, db_path) = setup_test_db(&["l_amat"]);
    let callsign = get_first_callsign("l_amat");

    Command::cargo_bin("uls")
        .unwrap()
        .env("ULS_DB_PATH", &db_path)
        .args(["lookup", &callsign])
        .assert()
        .success()
        .stdout(predicate::str::contains(&callsign));
}

#[test]
fn test_lookup_case_insensitive() {
    let (_temp_dir, db_path) = setup_test_db(&["l_amat"]);
    let callsign = get_first_callsign("l_amat");
    let lowercase = callsign.to_lowercase();

    Command::cargo_bin("uls")
        .unwrap()
        .env("ULS_DB_PATH", &db_path)
        .args(["lookup", &lowercase])
        .assert()
        .success()
        .stdout(predicate::str::contains(&callsign));
}

#[test]
fn test_lookup_not_found() {
    let (_temp_dir, db_path) = setup_test_db(&["l_amat"]);

    Command::cargo_bin("uls")
        .unwrap()
        .env("ULS_DB_PATH", &db_path)
        .args(["lookup", "ZZZZZZ"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No license found"));
}

#[test]
fn test_lookup_json_output() {
    let (_temp_dir, db_path) = setup_test_db(&["l_amat"]);
    let callsign = get_first_callsign("l_amat");

    // Lookup now returns array for consistency with multi-callsign support
    Command::cargo_bin("uls")
        .unwrap()
        .env("ULS_DB_PATH", &db_path)
        .args(["lookup", &callsign, "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["))
        .stdout(predicate::str::contains("call_sign"));
}

// =============================================================================
// Search command tests (with database)
// =============================================================================

#[test]
fn test_search_with_name() {
    let (_temp_dir, db_path) = setup_test_db(&["l_amat"]);

    // Search with wildcard to get any results
    Command::cargo_bin("uls")
        .unwrap()
        .env("ULS_DB_PATH", &db_path)
        .args(["search", "--name", "*", "--limit", "5"])
        .assert()
        .success()
        .stdout(predicate::str::contains("result"));
}

#[test]
fn test_search_no_results() {
    let (_temp_dir, db_path) = setup_test_db(&["l_amat"]);

    Command::cargo_bin("uls")
        .unwrap()
        .env("ULS_DB_PATH", &db_path)
        .args(["search", "--name", "ZZZZNONEXISTENT"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No results"));
}

#[test]
fn test_search_json_output() {
    let (_temp_dir, db_path) = setup_test_db(&["l_amat"]);

    Command::cargo_bin("uls")
        .unwrap()
        .env("ULS_DB_PATH", &db_path)
        .args(["search", "--name", "*", "--limit", "2", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));
}

#[test]
fn test_search_csv_output() {
    let (_temp_dir, db_path) = setup_test_db(&["l_amat"]);

    Command::cargo_bin("uls")
        .unwrap()
        .env("ULS_DB_PATH", &db_path)
        .args(["search", "--name", "*", "--limit", "2", "--format", "csv"])
        .assert()
        .success()
        .stdout(predicate::str::contains("call_sign"));
}

// =============================================================================
// Stats command tests (with database)
// =============================================================================

#[test]
fn test_stats_with_data() {
    let (_temp_dir, db_path) = setup_test_db(&["l_amat", "l_gmrs"]);

    Command::cargo_bin("uls")
        .unwrap()
        .env("ULS_DB_PATH", &db_path)
        .args(["stats"])
        .assert()
        .success()
        .stdout(predicate::str::contains("licenses").or(predicate::str::contains("records")));
}

// =============================================================================
// Shorthand callsign lookup tests
// =============================================================================

#[test]
fn test_shorthand_callsign_lookup() {
    let (_temp_dir, db_path) = setup_test_db(&["l_amat"]);
    let callsign = get_first_callsign("l_amat");

    // uls <callsign> should work as shorthand for uls lookup <callsign>
    Command::cargo_bin("uls")
        .unwrap()
        .env("ULS_DB_PATH", &db_path)
        .arg(&callsign)
        .assert()
        .success()
        .stdout(predicate::str::contains(&callsign));
}

#[test]
fn test_multi_callsign_lookup() {
    let (_temp_dir, db_path) = setup_test_db(&["l_amat"]);
    let callsign = get_first_callsign("l_amat");

    // Multiple callsigns should all be looked up
    Command::cargo_bin("uls")
        .unwrap()
        .env("ULS_DB_PATH", &db_path)
        .args(["lookup", &callsign, &callsign, "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));
}

#[test]
fn test_multi_callsign_partial_failure() {
    let (_temp_dir, db_path) = setup_test_db(&["l_amat"]);
    let callsign = get_first_callsign("l_amat");

    // One valid, one invalid - should still succeed with warning
    Command::cargo_bin("uls")
        .unwrap()
        .env("ULS_DB_PATH", &db_path)
        .args(["lookup", &callsign, "ZZZZZZ"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&callsign))
        .stderr(predicate::str::contains("No license found"));
}

// =============================================================================
// FRN lookup tests
// =============================================================================

#[test]
fn test_frn_lookup_not_found() {
    let (_temp_dir, db_path) = setup_test_db(&["l_amat"]);

    Command::cargo_bin("uls")
        .unwrap()
        .env("ULS_DB_PATH", &db_path)
        .args(["frn", "0000000000"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No licenses found"));
}

#[test]
fn test_frn_help() {
    Command::cargo_bin("uls")
        .unwrap()
        .args(["frn", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("FRN").or(predicate::str::contains("frn")));
}

#[test]
fn test_multi_frn_partial_failure() {
    let (_temp_dir, db_path) = setup_test_db(&["l_amat"]);

    // One invalid FRN - should fail with warning
    Command::cargo_bin("uls")
        .unwrap()
        .env("ULS_DB_PATH", &db_path)
        .args(["frn", "0000000000", "9999999999"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No licenses found"));
}

// =============================================================================
// Database management tests
// =============================================================================

#[test]
fn test_db_init() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("new.db");

    Command::cargo_bin("uls")
        .unwrap()
        .args(["db", "init", "--path", db_path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("initialized"));

    assert!(db_path.exists());
}

#[test]
fn test_db_info() {
    let (_temp_dir, db_path) = setup_test_db(&["l_amat"]);

    Command::cargo_bin("uls")
        .unwrap()
        .env("ULS_DB_PATH", &db_path)
        .args(["db", "info"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Database Info").or(predicate::str::contains("path")));
}

#[test]
fn test_db_info_json_format() {
    let (_temp_dir, db_path) = setup_test_db(&["l_amat"]);

    Command::cargo_bin("uls")
        .unwrap()
        .env("ULS_DB_PATH", &db_path)
        .args(["db", "info", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("{"))
        .stdout(predicate::str::contains("\"path\""));
}

#[test]
fn test_db_vacuum() {
    let (_temp_dir, db_path) = setup_test_db(&["l_amat"]);

    Command::cargo_bin("uls")
        .unwrap()
        .env("ULS_DB_PATH", &db_path)
        .args(["db", "vacuum"])
        .assert()
        .success()
        .stdout(predicate::str::contains("optimized"));
}

#[test]
fn test_db_help() {
    Command::cargo_bin("uls")
        .unwrap()
        .args(["db", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("info"))
        .stdout(predicate::str::contains("vacuum"));
}
