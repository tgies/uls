//! Import orchestration for FCC ULS data.
//!
//! This module provides the `Importer` struct which handles bulk import
//! of FCC data from ZIP files into the database.

use std::path::Path;
use std::time::Instant;

use tracing::{info, warn};
use uls_parser::archive::ZipExtractor;

use crate::bulk_inserter::BulkInserter;
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

/// Importer handles bulk import of FCC data into the database.
pub struct Importer<'a> {
    db: &'a Database,
}

impl<'a> Importer<'a> {
    /// Create a new importer for the given database.
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    /// Import records from a ZIP file.
    ///
    /// This method:
    /// 1. Opens the ZIP file
    /// 2. Sorts DAT files by dependency order (HD first, then EN, AM, etc.)
    /// 3. Streams records through the parser
    /// 4. Bulk inserts into the database using prepared statements
    ///
    /// # Arguments
    /// * `zip_path` - Path to the FCC ZIP file
    /// * `progress` - Optional callback for progress updates
    ///
    /// # Returns
    /// Import statistics including record counts and error counts.
    pub fn import_zip(
        &self,
        zip_path: &Path,
        progress: Option<ProgressCallback>,
    ) -> Result<ImportStats> {
        let start = Instant::now();
        
        let mut extractor = ZipExtractor::open(zip_path)?;
        let mut dat_files = extractor.list_dat_files();
        
        // Sort by processing order: HD (licenses) first, then EN (entities), then others
        dat_files.sort_by(|a, b| {
            let priority = |s: &str| -> u8 {
                let upper = s.to_uppercase();
                if upper.contains("HD") { 0 }
                else if upper.contains("EN") { 1 }
                else if upper.contains("AM") { 2 }
                else { 3 }
            };
            priority(a).cmp(&priority(b))
        });
        
        info!("Processing {} DAT files: {:?}", dat_files.len(), dat_files);
        
        // Optimize SQLite for bulk import
        let conn = self.db.conn()?;
        conn.execute_batch(
            "PRAGMA synchronous = OFF;
             PRAGMA journal_mode = MEMORY;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -64000;"
        )?;
        
        // Begin transaction
        conn.execute("BEGIN TRANSACTION", [])?;
        
        // Create bulk inserter with prepared statements (statements compiled ONCE)
        let mut inserter = BulkInserter::new(&conn)?;
        
        let mut stats = ImportStats {
            files: dat_files.len(),
            ..Default::default()
        };
        
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
                            errors: stats.parse_errors + stats.insert_errors + file_parse_errors + file_insert_errors,
                        });
                    }
                }
                true
            })?;
            
            stats.records += file_records;
            stats.parse_errors += file_parse_errors;
            stats.insert_errors += file_insert_errors;
            
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
        
        // Reset SQLite settings
        conn.execute_batch(
            "PRAGMA synchronous = NORMAL;
             PRAGMA journal_mode = WAL;"
        )?;
        
        stats.duration_secs = start.elapsed().as_secs_f64();
        
        info!(
            "Import complete: {} records in {:.1}s ({:.0}/sec)",
            stats.records, stats.duration_secs, stats.rate()
        );
        
        Ok(stats)
    }
}
