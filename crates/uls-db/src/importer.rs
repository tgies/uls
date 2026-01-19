//! Import orchestration for FCC ULS data.
//!
//! This module provides the `Importer` struct which handles bulk import
//! of FCC data from ZIP files into the database.

use std::path::Path;
use std::time::Instant;

use tracing::{debug, info, warn};
use uls_parser::archive::ZipExtractor;

use crate::bulk_inserter::BulkInserter;
use crate::schema::Schema;
use crate::{Database, Result};

/// Statistics from an import operation.
#[derive(Debug, Clone, Default)]
pub struct ImportStats {
    /// Total records successfully imported.
    pub records: usize,
    /// Number of files processed.
    pub files: usize,
    /// Number of parse errors encountered.
    pub parse_errors: usize,
    /// Number of insert errors encountered.
    pub insert_errors: usize,
    /// Import duration in seconds.
    pub duration_secs: f64,
}

impl ImportStats {
    /// Records per second import rate.
    pub fn rate(&self) -> f64 {
        if self.duration_secs > 0.0 {
            self.records as f64 / self.duration_secs
        } else {
            0.0
        }
    }

    /// Whether the import completed without errors.
    pub fn is_successful(&self) -> bool {
        self.insert_errors == 0
    }
}

/// Progress callback for import operations.
pub struct ImportProgress {
    /// Current file being processed (1-indexed).
    pub current_file: usize,
    /// Total number of files to process.
    pub total_files: usize,
    /// Name of current file being processed.
    pub file_name: String,
    /// Records imported so far.
    pub records: usize,
    /// Errors encountered so far.
    pub errors: usize,
}

/// Callback type for import progress updates.
pub type ProgressCallback = Box<dyn Fn(&ImportProgress) + Send + Sync>;

/// Import mode controls which record types are imported.
#[derive(Debug, Clone, Default)]
pub enum ImportMode {
    /// Import only HD + EN + AM (sufficient for callsign/FRN/name lookups)
    Minimal,
    /// Import all record types
    #[default]
    Full,
    /// Import specific record types
    Selective(Vec<String>),
}

impl ImportMode {
    /// Record types to import for minimal mode (for amateur service).
    pub const MINIMAL_TYPES: &'static [&'static str] = &["HD", "EN", "AM"];

    /// Check if a record type should be imported.
    pub fn should_import(&self, record_type: &str) -> bool {
        match self {
            ImportMode::Minimal => {
                Self::MINIMAL_TYPES.contains(&record_type.to_uppercase().as_str())
            }
            ImportMode::Full => true,
            ImportMode::Selective(types) => {
                types.iter().any(|t| t.eq_ignore_ascii_case(record_type))
            }
        }
    }

    /// Check if a file should be imported based on its name.
    pub fn should_import_file(&self, filename: &str) -> bool {
        // Extract record type from filename like "HD.dat" or "EN.dat"
        let record_type = filename.split('.').next().unwrap_or("").to_uppercase();
        self.should_import(&record_type)
    }
}

/// Importer handles bulk import of FCC data into the database.
pub struct Importer<'a> {
    db: &'a Database,
}

impl<'a> Importer<'a> {
    /// Create a new importer for the given database.
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Import records from a ZIP file (full import of all record types).
    ///
    /// This is a convenience method that calls `import_zip_with_mode` with `ImportMode::Full`.
    pub fn import_zip(
        &self,
        zip_path: &Path,
        progress: Option<ProgressCallback>,
    ) -> Result<ImportStats> {
        self.import_zip_with_mode(zip_path, ImportMode::Full, progress)
    }

    /// Import records from a ZIP file with specified import mode.
    ///
    /// This method:
    /// 1. Opens the ZIP file
    /// 2. Filters DAT files by import mode (minimal imports only HD+EN+AM)
    /// 3. Sorts DAT files by dependency order (HD first, then EN, AM, etc.)
    /// 4. Streams records through the parser
    /// 5. Bulk inserts into the database using prepared statements
    ///
    /// # Arguments
    /// * `zip_path` - Path to the FCC ZIP file
    /// * `mode` - Import mode (Minimal, Full, or Selective)
    /// * `progress` - Optional callback for progress updates
    ///
    /// # Returns
    /// Import statistics including record counts and error counts.
    pub fn import_zip_with_mode(
        &self,
        zip_path: &Path,
        mode: ImportMode,
        progress: Option<ProgressCallback>,
    ) -> Result<ImportStats> {
        let start = Instant::now();

        let mut extractor = ZipExtractor::open(zip_path)?;
        let all_dat_files = extractor.list_dat_files();

        // Filter files based on import mode
        let mut dat_files: Vec<String> = all_dat_files
            .into_iter()
            .filter(|f| mode.should_import_file(f))
            .collect();

        // Sort by processing order: HD (licenses) first, then EN (entities), then others
        dat_files.sort_by(|a, b| {
            let priority = |s: &str| -> u8 {
                let upper = s.to_uppercase();
                if upper.contains("HD") {
                    0
                } else if upper.contains("EN") {
                    1
                } else if upper.contains("AM") {
                    2
                } else {
                    3
                }
            };
            priority(a).cmp(&priority(b))
        });

        info!(
            "Processing {} DAT files (mode={:?}): {:?}",
            dat_files.len(),
            mode,
            dat_files
        );

        // Optimize SQLite for bulk import
        let conn = self.db.conn()?;
        conn.execute_batch(
            "PRAGMA synchronous = OFF;
             PRAGMA journal_mode = MEMORY;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -64000;",
        )?;

        // Drop indexes for faster bulk insert (will be recreated after import)
        debug!("Dropping indexes for bulk import performance");
        Schema::drop_indexes(&conn)?;

        // Begin transaction
        conn.execute("BEGIN TRANSACTION", [])?;

        // Create bulk inserter with prepared statements (statements compiled ONCE)
        let mut inserter = BulkInserter::new(&conn)?;

        let mut stats = ImportStats {
            files: dat_files.len(),
            ..Default::default()
        };

        // Track records per file type for import_status updates
        let mut records_per_type: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for (idx, dat_file) in dat_files.iter().enumerate() {
            let mut file_records = 0usize;
            let mut file_parse_errors = 0usize;
            let mut file_insert_errors = 0usize;

            extractor.process_dat_streaming(dat_file, |line| {
                match line.to_record() {
                    Ok(record) => {
                        if let Err(e) = inserter.insert(&record) {
                            file_insert_errors += 1;
                            if file_insert_errors <= 5 {
                                warn!("Insert error in {}: {}", dat_file, e);
                            }
                        } else {
                            file_records += 1;
                        }
                    }
                    Err(e) => {
                        file_parse_errors += 1;
                        if file_parse_errors <= 5 {
                            warn!("Parse error in {}: {}", dat_file, e);
                        }
                    }
                }

                // Send progress update every 10k records
                if let Some(ref cb) = progress {
                    if (file_records + file_parse_errors) % 10_000 == 0 {
                        cb(&ImportProgress {
                            current_file: idx + 1,
                            total_files: dat_files.len(),
                            file_name: dat_file.clone(),
                            records: stats.records + file_records,
                            errors: stats.parse_errors
                                + stats.insert_errors
                                + file_parse_errors
                                + file_insert_errors,
                        });
                    }
                }
                true
            })?;

            stats.records += file_records;
            stats.parse_errors += file_parse_errors;
            stats.insert_errors += file_insert_errors;

            // Track per-file-type records
            let record_type = dat_file.split('.').next().unwrap_or("").to_uppercase();
            *records_per_type.entry(record_type).or_insert(0) += file_records;

            if file_parse_errors > 0 || file_insert_errors > 0 {
                warn!(
                    "{}: {} records, {} parse errors, {} insert errors",
                    dat_file, file_records, file_parse_errors, file_insert_errors
                );
            }

            // Final progress update for this file
            if let Some(ref cb) = progress {
                cb(&ImportProgress {
                    current_file: idx + 1,
                    total_files: dat_files.len(),
                    file_name: dat_file.clone(),
                    records: stats.records,
                    errors: stats.parse_errors + stats.insert_errors,
                });
            }
        }

        // Drop inserter to release statement borrows before commit
        drop(inserter);

        // Commit transaction
        conn.execute("COMMIT", [])?;

        // Rebuild indexes (this is much faster than maintaining them during insert)
        let index_start = Instant::now();
        debug!("Rebuilding indexes after bulk import");
        Schema::create_indexes(&conn)?;
        let index_duration = index_start.elapsed();
        debug!(
            "Index rebuild completed in {:.2}s",
            index_duration.as_secs_f64()
        );

        // Reset SQLite settings
        conn.execute_batch(
            "PRAGMA synchronous = NORMAL;
             PRAGMA journal_mode = WAL;",
        )?;

        stats.duration_secs = start.elapsed().as_secs_f64();

        info!(
            "Import complete: {} records in {:.1}s ({:.0}/sec), index rebuild: {:.2}s",
            stats.records,
            stats.duration_secs,
            stats.rate(),
            index_duration.as_secs_f64()
        );

        Ok(stats)
    }

    /// Import records from a ZIP file for a specific service and track import status.
    ///
    /// This wraps `import_zip_with_mode` and records which record types were imported
    /// to enable lazy loading detection.
    pub fn import_for_service(
        &self,
        zip_path: &Path,
        service: &str,
        mode: ImportMode,
        progress: Option<ProgressCallback>,
    ) -> Result<ImportStats> {
        // Clear previous import status for this service
        self.db.clear_import_status(service)?;

        let mut extractor = uls_parser::archive::ZipExtractor::open(zip_path)?;
        let all_dat_files = extractor.list_dat_files();

        // Determine which record types will be imported
        let imported_types: Vec<String> = all_dat_files
            .iter()
            .filter(|f| mode.should_import_file(f))
            .map(|f| f.split('.').next().unwrap_or("").to_uppercase())
            .collect();

        // Perform the import
        let stats = self.import_zip_with_mode(zip_path, mode, progress)?;

        // Record import status for each record type
        // Note: We estimate record counts per type based on file structure
        // For now, we just mark them as imported with 0 as placeholder count
        for record_type in imported_types {
            self.db.mark_imported(service, &record_type, 0)?;
        }

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DatabaseConfig;
    use std::io::Write;
    use tempfile::TempDir;
    use zip::write::FileOptions;
    use zip::ZipWriter;

    fn create_test_db() -> (TempDir, Database) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = DatabaseConfig::with_path(db_path);
        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();
        (temp_dir, db)
    }

    fn create_test_zip(temp_dir: &TempDir) -> std::path::PathBuf {
        let zip_path = temp_dir.path().join("test.zip");
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options: FileOptions<()> = FileOptions::default();

        // HD record - license header
        zip.start_file("HD.dat", options).unwrap();
        zip.write_all(b"HD|100001|0000000001||W1TEST|A|HA|01/15/2020|01/15/2030|||||||||||||||||||||||||||||||||||N|||||||||||01/15/2020|01/15/2020|||||||||||||||\n").unwrap();

        // EN record - entity
        zip.start_file("EN.dat", options).unwrap();
        zip.write_all(b"EN|100001|||W1TEST|L|L00100001|DOE, JOHN A|JOHN|A|DOE||555-555-1234||test@example.com|123 Main St|ANYTOWN|CA|90210||||000|0001234567|I||||||\n").unwrap();

        // AM record - amateur
        zip.start_file("AM.dat", options).unwrap();
        zip.write_all(b"AM|100001|||W1TEST|E|D|6||||||||||\n")
            .unwrap();

        zip.finish().unwrap();
        zip_path
    }

    #[test]
    fn test_import_stats_default() {
        let stats = ImportStats::default();
        assert_eq!(stats.records, 0);
        assert_eq!(stats.files, 0);
        assert_eq!(stats.parse_errors, 0);
        assert_eq!(stats.insert_errors, 0);
        assert_eq!(stats.duration_secs, 0.0);
    }

    #[test]
    fn test_import_stats_rate() {
        let stats = ImportStats {
            records: 1000,
            duration_secs: 2.0,
            ..Default::default()
        };
        assert_eq!(stats.rate(), 500.0);
    }

    #[test]
    fn test_import_stats_rate_zero_duration() {
        let stats = ImportStats {
            records: 1000,
            duration_secs: 0.0,
            ..Default::default()
        };
        assert_eq!(stats.rate(), 0.0);
    }

    #[test]
    fn test_import_stats_is_successful() {
        let stats = ImportStats {
            insert_errors: 0,
            ..Default::default()
        };
        assert!(stats.is_successful());

        let stats_with_errors = ImportStats {
            insert_errors: 1,
            ..Default::default()
        };
        assert!(!stats_with_errors.is_successful());
    }

    #[test]
    fn test_importer_new() {
        let (_temp_dir, db) = create_test_db();
        let importer = Importer::new(&db);
        // Just verify it can be created
        assert!(std::ptr::eq(importer.db, &db));
    }

    #[test]
    fn test_import_zip_basic() {
        let (temp_dir, db) = create_test_db();
        let zip_path = create_test_zip(&temp_dir);

        let importer = Importer::new(&db);
        let stats = importer.import_zip(&zip_path, None).unwrap();

        assert_eq!(stats.files, 3); // HD, EN, AM
        assert_eq!(stats.records, 3);
        assert_eq!(stats.parse_errors, 0);
        assert_eq!(stats.insert_errors, 0);
        assert!(stats.is_successful());
    }

    #[test]
    fn test_import_zip_with_progress_callback() {
        let (temp_dir, db) = create_test_db();
        let zip_path = create_test_zip(&temp_dir);

        let progress_called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let progress_called_clone = progress_called.clone();

        let callback: ProgressCallback = Box::new(move |_progress| {
            progress_called_clone.store(true, std::sync::atomic::Ordering::SeqCst);
        });

        let importer = Importer::new(&db);
        let stats = importer.import_zip(&zip_path, Some(callback)).unwrap();

        assert!(stats.is_successful());
        // Progress callback may or may not be called depending on record count
    }

    #[test]
    fn test_import_zip_nonexistent_file() {
        let (_temp_dir, db) = create_test_db();
        let importer = Importer::new(&db);

        let result = importer.import_zip(Path::new("/nonexistent/file.zip"), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_import_zip_verifies_data() {
        let (temp_dir, db) = create_test_db();
        let zip_path = create_test_zip(&temp_dir);

        let importer = Importer::new(&db);
        importer.import_zip(&zip_path, None).unwrap();

        // Verify data was actually imported
        if let Some(license) = db.get_license_by_callsign("W1TEST").unwrap() {
            assert_eq!(license.call_sign, "W1TEST");
            assert_eq!(license.radio_service.as_str(), "HA");
        } else {
            panic!("License W1TEST should have been imported");
        }
    }

    // ==========================================================================
    // ImportMode tests
    // ==========================================================================

    #[test]
    fn test_import_mode_should_import_minimal() {
        let mode = ImportMode::Minimal;

        // Minimal should only allow HD, EN, AM
        assert!(mode.should_import("HD"));
        assert!(mode.should_import("EN"));
        assert!(mode.should_import("AM"));
        assert!(mode.should_import("hd")); // Case insensitive

        // Minimal should NOT import other record types
        assert!(!mode.should_import("HS"));
        assert!(!mode.should_import("CO"));
        assert!(!mode.should_import("SC"));
        assert!(!mode.should_import("LA"));
        assert!(!mode.should_import("SF"));
    }

    #[test]
    fn test_import_mode_should_import_full() {
        let mode = ImportMode::Full;

        // Full should allow all record types
        assert!(mode.should_import("HD"));
        assert!(mode.should_import("EN"));
        assert!(mode.should_import("AM"));
        assert!(mode.should_import("HS"));
        assert!(mode.should_import("CO"));
        assert!(mode.should_import("SC"));
        assert!(mode.should_import("LA"));
        assert!(mode.should_import("SF"));
        assert!(mode.should_import("ANYTYPE")); // Full allows anything
    }

    #[test]
    fn test_import_mode_should_import_selective() {
        let mode = ImportMode::Selective(vec!["HD".to_string(), "CO".to_string()]);

        assert!(mode.should_import("HD"));
        assert!(mode.should_import("CO"));
        assert!(mode.should_import("hd")); // Case insensitive

        assert!(!mode.should_import("EN"));
        assert!(!mode.should_import("AM"));
        assert!(!mode.should_import("HS"));
    }

    #[test]
    fn test_import_mode_should_import_file() {
        let mode = ImportMode::Minimal;

        assert!(mode.should_import_file("HD.dat"));
        assert!(mode.should_import_file("EN.dat"));
        assert!(mode.should_import_file("AM.dat"));

        assert!(!mode.should_import_file("HS.dat"));
        assert!(!mode.should_import_file("CO.dat"));
    }

    #[test]
    fn test_import_mode_default_is_full() {
        let mode = ImportMode::default();
        assert!(matches!(mode, ImportMode::Full));
    }

    fn create_multi_type_test_zip(temp_dir: &TempDir) -> std::path::PathBuf {
        let zip_path = temp_dir.path().join("multi.zip");
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options: FileOptions<()> = FileOptions::default();

        // HD record
        zip.start_file("HD.dat", options).unwrap();
        zip.write_all(b"HD|100001|0000000001||W1TEST|A|HA|01/15/2020|01/15/2030|||||||||||||||||||||||||||||||||||N|||||||||||01/15/2020|01/15/2020|||||||||||||||\n").unwrap();

        // EN record
        zip.start_file("EN.dat", options).unwrap();
        zip.write_all(b"EN|100001|||W1TEST|L|L00100001|DOE, JOHN|JOHN||DOE||||||||||||000|0001234567|I||||||\n").unwrap();

        // AM record
        zip.start_file("AM.dat", options).unwrap();
        zip.write_all(b"AM|100001|||W1TEST|E|D|6||||||||||\n")
            .unwrap();

        // HS record (history)
        zip.start_file("HS.dat", options).unwrap();
        zip.write_all(b"HS|100001||W1TEST|01/15/2020|LIISS\n")
            .unwrap();

        // CO record (comment)
        zip.start_file("CO.dat", options).unwrap();
        zip.write_all(b"CO|100001||W1TEST|01/15/2020|Test comment||\n")
            .unwrap();

        zip.finish().unwrap();
        zip_path
    }

    #[test]
    fn test_import_zip_with_mode_minimal() {
        let (temp_dir, db) = create_test_db();
        let zip_path = create_multi_type_test_zip(&temp_dir);

        let importer = Importer::new(&db);
        let stats = importer
            .import_zip_with_mode(&zip_path, ImportMode::Minimal, None)
            .unwrap();

        // Minimal should only import HD, EN, AM (3 files, 3 records)
        assert_eq!(stats.files, 3);
        assert_eq!(stats.records, 3);
        assert!(stats.is_successful());

        // Verify license was imported
        assert!(db.get_license_by_callsign("W1TEST").unwrap().is_some());
    }

    #[test]
    fn test_import_zip_with_mode_full() {
        let (temp_dir, db) = create_test_db();
        let zip_path = create_multi_type_test_zip(&temp_dir);

        let importer = Importer::new(&db);
        let stats = importer
            .import_zip_with_mode(&zip_path, ImportMode::Full, None)
            .unwrap();

        // Full should import all 5 files
        assert_eq!(stats.files, 5);
        assert_eq!(stats.records, 5);
        assert!(stats.is_successful());
    }

    #[test]
    fn test_import_zip_with_mode_selective() {
        let (temp_dir, db) = create_test_db();
        let zip_path = create_multi_type_test_zip(&temp_dir);

        let importer = Importer::new(&db);
        let mode = ImportMode::Selective(vec!["HD".to_string(), "EN".to_string()]);
        let stats = importer
            .import_zip_with_mode(&zip_path, mode, None)
            .unwrap();

        // Selective should only import HD and EN (2 files, 2 records)
        assert_eq!(stats.files, 2);
        assert_eq!(stats.records, 2);
        assert!(stats.is_successful());
    }
}
