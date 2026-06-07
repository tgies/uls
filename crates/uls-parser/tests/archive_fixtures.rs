//! Integration tests for ZipExtractor over file-backed archives.
//!
//! These exercise the public `open` path against ZIPs built from the shared
//! fixture .dat files, plus error paths for missing and corrupt archives.

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use uls_parser::archive::{ArchiveStats, ZipError};
use uls_parser::ParseError;
use uls_parser::ZipExtractor;

/// Path to the shared fixture directory at the workspace root.
fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests/fixtures/fcc-sample")
}

/// Build a ZIP from every .dat file in a fixture service directory.
fn create_fixture_zip(temp_dir: &TempDir, service: &str) -> PathBuf {
    let fixture_dir = fixture_path().join(service);
    let zip_path = temp_dir.path().join(format!("{service}.zip"));

    let file = File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

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
    zip_path
}

/// Count records in a fixture .dat file the way the parser does: each non-empty
/// line whose first field parses as a known FCC record type begins a record
/// (continuation lines fold into it). Cross-checks count_all_records using the
/// uls-core RecordType enum rather than a hand-maintained type list.
fn expected_record_count(service: &str, dat: &str) -> usize {
    use std::io::{BufRead, BufReader};
    let path = fixture_path().join(service).join(dat);
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let mut count = 0;
    for line in reader.lines().map_while(Result::ok) {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            continue;
        }
        let first = trimmed.split('|').next().unwrap_or("");
        if first.parse::<uls_core::codes::RecordType>().is_ok() {
            count += 1;
        }
    }
    count
}

#[test]
fn test_open_fixture_zip_lists_dat_files() {
    let temp = TempDir::new().unwrap();
    let zip_path = create_fixture_zip(&temp, "l_amat");

    let mut extractor = ZipExtractor::open(&zip_path).unwrap();
    let mut dat_files = extractor.list_dat_files();
    dat_files.sort();

    assert_eq!(
        dat_files,
        vec!["AM.dat", "CO.dat", "EN.dat", "HD.dat", "HS.dat", "LA.dat", "SC.dat",]
    );
}

#[test]
fn test_open_nonexistent_path_errors_io() {
    let result = ZipExtractor::open("/no/such/path/missing.zip");
    match result {
        Err(ZipError::Io(_)) => {}
        Err(other) => panic!("expected Io error, got {other:?}"),
        Ok(_) => panic!("expected error opening missing zip"),
    }
}

#[test]
fn test_open_non_zip_file_errors_zip() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("not-a-zip.zip");
    let mut f = File::create(&path).unwrap();
    f.write_all(b"this is plainly not a zip archive at all")
        .unwrap();
    drop(f);

    let result = ZipExtractor::open(&path);
    match result {
        Err(ZipError::Zip(_)) => {}
        Err(other) => panic!("expected Zip error, got {other:?}"),
        Ok(_) => panic!("expected error opening non-zip file"),
    }
}

#[test]
fn test_open_truncated_zip_errors_zip() {
    let temp = TempDir::new().unwrap();
    let good = create_fixture_zip(&temp, "l_amat");
    let bytes = fs::read(&good).unwrap();

    // Keep only the leading half so the central directory is gone.
    let truncated_path = temp.path().join("truncated.zip");
    let mut f = File::create(&truncated_path).unwrap();
    f.write_all(&bytes[..bytes.len() / 2]).unwrap();
    drop(f);

    let result = ZipExtractor::open(&truncated_path);
    assert!(
        matches!(result, Err(ZipError::Zip(_))),
        "truncated archive should be a Zip error"
    );
}

#[test]
fn test_stats_reports_counts_and_nonzero_size() {
    let temp = TempDir::new().unwrap();
    let zip_path = create_fixture_zip(&temp, "l_amat");
    let mut extractor = ZipExtractor::open(&zip_path).unwrap();

    let ArchiveStats {
        total_files,
        dat_files,
        total_size_bytes,
    } = extractor.stats().unwrap();

    // The l_amat fixture has seven .dat files and nothing else.
    assert_eq!(total_files, 7);
    assert_eq!(dat_files.len(), 7);
    assert!(total_size_bytes > 0);

    // total_size_bytes is the sum of each entry's uncompressed size.
    let expected_total: u64 = dat_files
        .iter()
        .map(|name| extractor.file_size(name).unwrap())
        .sum();
    assert_eq!(total_size_bytes, expected_total);
}

#[test]
fn test_count_all_records_matches_fixture_lines() {
    let temp = TempDir::new().unwrap();
    let zip_path = create_fixture_zip(&temp, "l_amat");
    let mut extractor = ZipExtractor::open(&zip_path).unwrap();

    let counts = extractor.count_all_records().unwrap();
    assert_eq!(counts.len(), 7);

    for (dat, count) in &counts {
        assert_eq!(
            *count,
            expected_record_count("l_amat", dat),
            "record count mismatch for {dat}"
        );
    }

    assert_eq!(counts["HD.dat"], 34);
    assert_eq!(counts["AM.dat"], 34);
    assert_eq!(counts["LA.dat"], 1);
}

#[test]
fn test_count_all_records_gmrs_fixture() {
    let temp = TempDir::new().unwrap();
    let zip_path = create_fixture_zip(&temp, "l_gmrs");
    let mut extractor = ZipExtractor::open(&zip_path).unwrap();

    let counts = extractor.count_all_records().unwrap();
    assert_eq!(counts["HD.dat"], 33);
    assert_eq!(counts["EN.dat"], 33);
}

#[test]
fn test_file_size_matches_uncompressed_fixture_bytes() {
    let temp = TempDir::new().unwrap();
    let zip_path = create_fixture_zip(&temp, "l_amat");
    let mut extractor = ZipExtractor::open(&zip_path).unwrap();

    let on_disk = fs::metadata(fixture_path().join("l_amat").join("HD.dat"))
        .unwrap()
        .len();
    assert_eq!(extractor.file_size("HD.dat").unwrap(), on_disk);
}

#[test]
fn test_process_dat_streaming_visits_every_record() {
    let temp = TempDir::new().unwrap();
    let zip_path = create_fixture_zip(&temp, "l_amat");
    let mut extractor = ZipExtractor::open(&zip_path).unwrap();

    let mut callsigns = Vec::new();
    let count = extractor
        .process_dat_streaming("HD.dat", |line| {
            assert_eq!(line.record_type, "HD");
            callsigns.push(line.field(4).to_string());
            true
        })
        .unwrap();

    assert_eq!(count, 34);
    assert_eq!(callsigns.len(), 34);
    assert!(callsigns.iter().all(|c| !c.is_empty()));
}

#[test]
fn test_process_dat_streaming_missing_file_is_parse_error() {
    let temp = TempDir::new().unwrap();
    let zip_path = create_fixture_zip(&temp, "l_amat");
    let mut extractor = ZipExtractor::open(&zip_path).unwrap();

    let result = extractor.process_dat_streaming("DOES_NOT_EXIST.dat", |_| true);
    match result {
        Err(ParseError::InvalidFormat { line, message }) => {
            assert_eq!(line, 0);
            assert!(message.contains("DOES_NOT_EXIST.dat"));
        }
        other => panic!("expected InvalidFormat parse error, got {other:?}"),
    }
}

#[test]
fn test_get_file_creation_date_counts_without_date_line() {
    let temp = TempDir::new().unwrap();
    let zip_path = temp.path().join("counts-no-date.zip");

    {
        let file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        zip.start_file("counts", options).unwrap();
        zip.write_all(b"  1234 /some/path/AM.dat\n  5678 /some/path/HD.dat\n")
            .unwrap();
        zip.finish().unwrap();
    }

    let mut extractor = ZipExtractor::open(&zip_path).unwrap();
    assert!(extractor.get_file_creation_date().is_none());
}
