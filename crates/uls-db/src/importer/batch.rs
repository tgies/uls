//! Scoped index ownership and atomic source metadata for consecutive imports.

use std::fmt::Display;
use std::path::Path;

use chrono::{Datelike, NaiveDate};
use rusqlite::Connection;
use tracing::{debug, warn};

use super::{ImportMode, ImportStats, Importer, ProgressCallback};
use crate::{Database, DbError, Result, Schema};

/// Source identity committed in the same transaction as an archive's records.
pub struct ImportSource<'a> {
    /// FCC radio service code, for example HA or ZA.
    pub service: &'a str,
    /// Canonical FCC source date, already validated by the update planner.
    pub date: NaiveDate,
    /// Original FCC creation timestamp, when present.
    pub timestamp: Option<&'a str>,
    /// ETag of the archive being imported, when available.
    pub etag: Option<&'a str>,
}

/// Imports using one connection and one scoped index-restoration owner.
///
/// Obtain this only through [`Importer::batch`]. Each archive commits separately;
/// the batch is not an all-or-nothing database transaction.
pub struct ImportBatch<'a> {
    guard: ImportGuard<'a>,
}

impl Importer<'_> {
    /// Run a sequence of archives, deferring index rebuilding between weeklies.
    ///
    /// Cleanup runs on success, error and panic. Normal error returns include
    /// restoration failures; panic cleanup logs them. Callers must own exclusive
    /// access to the database while bulk settings are active, and must use the
    /// supplied batch rather than checking another connection out of this pool.
    /// Daily-only and empty batches do not change indexes or PRAGMAs.
    pub fn batch<T, E: From<DbError> + Display>(
        &self,
        operation: impl FnOnce(&mut ImportBatch<'_>) -> std::result::Result<T, E>,
    ) -> std::result::Result<T, E> {
        let conn = self.db.conn()?;
        let mut batch = ImportBatch {
            guard: ImportGuard::new(&conn),
        };
        let outcome = operation(&mut batch);
        match (outcome, batch.guard.restore()) {
            (outcome, Ok(())) => outcome,
            (Ok(_), Err(cleanup)) => Err(cleanup.into()),
            (Err(original), Err(cleanup)) => {
                Err(DbError::Transaction(format!("{original:#}; additionally, {cleanup}")).into())
            }
        }
    }
}

impl ImportBatch<'_> {
    /// Import a weekly archive without service/source tracking.
    pub fn import_zip_with_mode(
        &mut self,
        path: &Path,
        mode: ImportMode,
        progress: Option<ProgressCallback>,
    ) -> Result<ImportStats> {
        self.guard.prepare()?;
        Importer::import_zip_on_connection(self.guard.conn, path, mode, progress, |_, _, _| Ok(()))
    }

    /// Import a weekly archive and replace import-status rows atomically.
    pub fn import_for_service(
        &mut self,
        path: &Path,
        service: &str,
        mode: ImportMode,
        progress: Option<ProgressCallback>,
    ) -> Result<ImportStats> {
        self.guard.prepare()?;
        Importer::import_zip_on_connection(
            self.guard.conn,
            path,
            mode,
            progress,
            |conn, _, files| replace_import_status(conn, service, files),
        )
    }

    /// Import weekly data, status and its source anchor as one transaction.
    pub fn import_weekly(
        &mut self,
        path: &Path,
        source: &ImportSource<'_>,
        mode: ImportMode,
        progress: Option<ProgressCallback>,
    ) -> Result<ImportStats> {
        self.guard.prepare()?;
        Importer::import_zip_on_connection(
            self.guard.conn,
            path,
            mode,
            progress,
            |conn, _, files| {
                replace_import_status(conn, source.service, files)?;
                if let Some(etag) = source.etag {
                    Schema::set_metadata(conn, &format!("imported_etag_{}", source.service), etag)?;
                }
                Schema::set_metadata(
                    conn,
                    &format!("last_weekly_date_{}", source.service),
                    &source.date.to_string(),
                )?;
                conn.execute(
                    "DELETE FROM applied_patches WHERE radio_service_code = ?1",
                    [source.service],
                )?;
                set_timestamp(conn, source)
            },
        )
    }

    /// Import a daily archive and its source tracking in one transaction.
    pub fn import_daily(
        &mut self,
        path: &Path,
        source: &ImportSource<'_>,
        mode: ImportMode,
        progress: Option<ProgressCallback>,
    ) -> Result<ImportStats> {
        Importer::import_patch_on_connection(
            self.guard.conn,
            path,
            mode,
            progress,
            |conn, stats, _| {
                set_timestamp(conn, source)?;
                let weekday = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"]
                    [source.date.weekday().num_days_from_monday() as usize];
                Database::record_applied_patch_conn(
                    conn,
                    source.service,
                    source.date,
                    weekday,
                    source.etag,
                    Some(stats.records),
                )
            },
        )
    }
}

impl ImportBatch<'_> {
    /// Import a daily patch without source tracking, preserving archive order.
    pub fn import_patch(
        &mut self,
        path: &Path,
        mode: ImportMode,
        progress: Option<ProgressCallback>,
    ) -> Result<ImportStats> {
        Importer::import_patch_on_connection(
            self.guard.conn,
            path,
            mode,
            progress,
            |_, _, _| Ok(()),
        )
    }
}

fn replace_import_status(conn: &Connection, service: &str, files: &[String]) -> Result<()> {
    Database::clear_import_status_conn(conn, service)?;
    for file in files {
        let record_type = file.split('.').next().unwrap_or("").to_uppercase();
        Database::mark_imported_conn(conn, service, &record_type, 0)?;
    }
    Ok(())
}

fn set_timestamp(conn: &Connection, source: &ImportSource<'_>) -> Result<()> {
    if let Some(timestamp) = source.timestamp {
        Schema::set_metadata(conn, "last_updated", timestamp)?;
    }
    Ok(())
}

struct ImportGuard<'a> {
    conn: &'a Connection,
    indexes_dropped: bool,
    saved_pragmas: Vec<(&'static str, String)>,
}

impl<'a> ImportGuard<'a> {
    fn new(conn: &'a Connection) -> Self {
        Self {
            conn,
            indexes_dropped: false,
            saved_pragmas: Vec::new(),
        }
    }

    fn prepare(&mut self) -> Result<()> {
        if self.indexes_dropped {
            return Ok(());
        }
        if self.saved_pragmas.is_empty() {
            let mut saved = Vec::new();
            for name in ["journal_mode", "synchronous", "temp_store", "cache_size"] {
                saved.push((name, self.pragma_value(name)?));
            }
            // Keep the original snapshot if the caller retries partial setup.
            self.saved_pragmas = saved;
        }
        // Save before the first mutation: partial setup failures also restore.
        self.conn.execute_batch(
            "PRAGMA synchronous = OFF;
             PRAGMA journal_mode = MEMORY;
             PRAGMA temp_store = MEMORY;
             PRAGMA cache_size = -64000;",
        )?;
        self.indexes_dropped = true;
        debug!("Dropping indexes for bulk import performance");
        Schema::drop_indexes(self.conn)
    }

    fn pragma_value(&self, name: &str) -> Result<String> {
        Ok(self.conn.pragma_query_value(None, name, |row| {
            if name == "journal_mode" {
                row.get::<_, String>(0)
            } else {
                row.get::<_, i64>(0).map(|value| value.to_string())
            }
        })?)
    }

    fn restore(&mut self) -> Result<()> {
        let mut failures = Vec::new();
        if self.indexes_dropped {
            debug!("Rebuilding indexes after bulk import");
            match Schema::create_indexes(self.conn) {
                Ok(()) => self.indexes_dropped = false,
                Err(error) => failures.push(format!("indexes: {error}")),
            }
        }
        // Attempt every setting even if indexes or an earlier setting failed.
        for (name, value) in &self.saved_pragmas {
            let result = self
                .conn
                .pragma_update(None, name, value)
                .map_err(DbError::from)
                .and_then(|()| {
                    let actual = self.pragma_value(name)?;
                    if actual == *value {
                        Ok(())
                    } else {
                        Err(DbError::Transaction(format!(
                            "{name} restored to {actual}, expected {value}"
                        )))
                    }
                });
            if let Err(error) = result {
                failures.push(format!("{name}: {error}"));
            }
        }
        if failures.is_empty() {
            self.saved_pragmas.clear();
            Ok(())
        } else {
            Err(DbError::Transaction(format!(
                "import cleanup failed: {}",
                failures.join("; ")
            )))
        }
    }
}

impl Drop for ImportGuard<'_> {
    fn drop(&mut self) {
        if let Err(error) = self.restore() {
            warn!("Failed to restore import state during cleanup: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use tempfile::TempDir;
    use zip::{write::SimpleFileOptions, ZipWriter};

    fn database(dir: &TempDir) -> Database {
        let db = Database::with_config(
            crate::DatabaseConfig::with_path(dir.path().join("test.db")).with_max_connections(1),
        )
        .unwrap();
        db.initialize().unwrap();
        db
    }

    fn archive(dir: &TempDir, service: &str, malformed: bool) -> std::path::PathBuf {
        let name = if malformed { "bad" } else { "good" };
        let path = dir.path().join(format!("{service}-{name}.zip"));
        let mut zip = ZipWriter::new(std::fs::File::create(&path).unwrap());
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/fcc-sample")
            .join(service);
        for file in ["HD.dat", "EN.dat"] {
            zip.start_file(file, SimpleFileOptions::default()).unwrap();
            zip.write_all(&std::fs::read(fixture.join(file)).unwrap())
                .unwrap();
            if malformed && file == "EN.dat" {
                zip.write_all(&[0xff, b'\n']).unwrap();
            }
        }
        zip.finish().unwrap();
        path
    }

    fn source(service: &str) -> ImportSource<'_> {
        ImportSource {
            service,
            date: NaiveDate::from_ymd_opt(2026, 7, 19).unwrap(),
            timestamp: Some("Sun Jul 19 08:00:00 EDT 2026"),
            etag: Some("new-etag"),
        }
    }

    fn index_names(conn: &Connection) -> Vec<String> {
        conn.prepare("SELECT name FROM sqlite_schema WHERE type = 'index' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<std::result::Result<_, _>>()
            .unwrap()
    }

    fn settings(conn: &Connection) -> Vec<String> {
        let guard = ImportGuard::new(conn);
        ["journal_mode", "synchronous", "temp_store", "cache_size"]
            .iter()
            .map(|name| guard.pragma_value(name).unwrap())
            .collect()
    }

    #[test]
    fn repeated_weeklies_share_indexes_and_restore_exact_settings() {
        let dir = TempDir::new().unwrap();
        let db = database(&dir);
        let ha = archive(&dir, "l_amat", false);
        let za = archive(&dir, "l_gmrs", false);
        let (indexes, pragmas) = {
            let conn = db.conn().unwrap();
            conn.execute_batch(
                "PRAGMA synchronous=FULL; PRAGMA temp_store=FILE; PRAGMA cache_size=-777;",
            )
            .unwrap();
            (index_names(&conn), settings(&conn))
        };
        Importer::new(&db)
            .batch(|batch| -> Result<()> {
                batch.import_weekly(&ha, &source("HA"), ImportMode::Full, None)?;
                let after_first = index_names(batch.guard.conn);
                assert!(after_first.len() < indexes.len());
                let schema: i64 =
                    batch
                        .guard
                        .conn
                        .pragma_query_value(None, "schema_version", |row| row.get(0))?;
                batch.import_weekly(&za, &source("ZA"), ImportMode::Full, None)?;
                assert_eq!(index_names(batch.guard.conn), after_first);
                assert_eq!(
                    batch.guard.conn.pragma_query_value::<i64, _>(
                        None,
                        "schema_version",
                        |row| row.get(0)
                    )?,
                    schema
                );
                Ok(())
            })
            .unwrap();
        assert_eq!(
            db.get_last_weekly_date("HA").unwrap(),
            Some(source("HA").date)
        );
        assert_eq!(
            db.get_last_weekly_date("ZA").unwrap(),
            Some(source("ZA").date)
        );
        let conn = db.conn().unwrap();
        assert_eq!(index_names(&conn), indexes);
        assert_eq!(settings(&conn), pragmas);
        assert_eq!(
            conn.query_row::<String, _, _>("PRAGMA quick_check", [], |row| row.get(0))
                .unwrap(),
            "ok"
        );
    }

    #[test]
    fn later_stream_error_keeps_committed_service_and_allows_retry() {
        let dir = TempDir::new().unwrap();
        let db = database(&dir);
        let ha = archive(&dir, "l_amat", false);
        let za = archive(&dir, "l_gmrs", false);
        let bad = archive(&dir, "l_gmrs", true);
        let indexes = index_names(&db.conn().unwrap());
        let error = Importer::new(&db)
            .batch(|batch| -> Result<()> {
                batch.import_weekly(&ha, &source("HA"), ImportMode::Full, None)?;
                batch.import_weekly(&bad, &source("ZA"), ImportMode::Full, None)?;
                Ok(())
            })
            .unwrap_err();
        assert!(error.to_string().contains("parser error"), "{error}");
        assert_eq!(
            db.get_last_weekly_date("HA").unwrap(),
            Some(source("HA").date)
        );
        assert_eq!(db.get_last_weekly_date("ZA").unwrap(), None);
        assert!(db.get_imported_types("ZA").unwrap().is_empty());
        assert_eq!(db.count_by_service(&["ZA"]).unwrap(), 0);
        assert_eq!(index_names(&db.conn().unwrap()), indexes);
        Importer::new(&db)
            .batch(|batch| batch.import_weekly(&za, &source("ZA"), ImportMode::Full, None))
            .unwrap();
        assert!(db.count_by_service(&["ZA"]).unwrap() > 0);
    }

    #[test]
    fn metadata_failure_rolls_back_archive_and_preserves_prior_tracking() {
        let dir = TempDir::new().unwrap();
        let db = database(&dir);
        let ha = archive(&dir, "l_amat", false);
        let za = archive(&dir, "l_gmrs", false);
        let prior = NaiveDate::from_ymd_opt(2026, 7, 12).unwrap();
        db.set_last_weekly_date("ZA", prior).unwrap();
        db.set_imported_etag("ZA", "prior-etag").unwrap();
        db.record_applied_patch("ZA", prior.succ_opt().unwrap(), "mon", None, Some(1))
            .unwrap();
        db.mark_imported("ZA", "PRIOR", 42).unwrap();
        db.conn().unwrap().execute_batch("CREATE TRIGGER deny_weekly BEFORE INSERT ON metadata WHEN NEW.key = 'last_weekly_date_ZA' BEGIN SELECT RAISE(ABORT, 'injected metadata failure'); END;").unwrap();
        let indexes = index_names(&db.conn().unwrap());
        let error = Importer::new(&db)
            .batch(|batch| -> Result<()> {
                batch.import_weekly(&ha, &source("HA"), ImportMode::Full, None)?;
                batch.import_weekly(&za, &source("ZA"), ImportMode::Full, None)?;
                Ok(())
            })
            .unwrap_err();
        assert!(error.to_string().contains("injected metadata failure"));
        assert_eq!(
            db.get_last_weekly_date("HA").unwrap(),
            Some(source("HA").date)
        );
        assert_eq!(db.get_last_weekly_date("ZA").unwrap(), Some(prior));
        assert_eq!(
            db.get_imported_etag("ZA").unwrap().as_deref(),
            Some("prior-etag")
        );
        assert_eq!(db.get_imported_types("ZA").unwrap(), ["PRIOR"]);
        assert_eq!(db.get_applied_patches("ZA").unwrap().len(), 1);
        assert_eq!(db.count_by_service(&["ZA"]).unwrap(), 0);
        assert_eq!(index_names(&db.conn().unwrap()), indexes);
        db.conn()
            .unwrap()
            .execute_batch("DROP TRIGGER deny_weekly;")
            .unwrap();
        Importer::new(&db)
            .batch(|batch| batch.import_weekly(&za, &source("ZA"), ImportMode::Full, None))
            .unwrap();
        assert!(db.get_applied_patches("ZA").unwrap().is_empty());
    }

    #[test]
    fn daily_metadata_failure_keeps_valid_prefix_without_dropping_indexes() {
        let dir = TempDir::new().unwrap();
        let db = database(&dir);
        let ha = archive(&dir, "l_amat", false);
        let za = archive(&dir, "l_gmrs", false);
        let conn = db.conn().unwrap();
        conn.execute_batch("CREATE TRIGGER deny_patch BEFORE INSERT ON applied_patches WHEN NEW.radio_service_code='ZA' BEGIN SELECT RAISE(ABORT, 'injected patch failure'); END;").unwrap();
        let schema: i64 = conn
            .pragma_query_value(None, "schema_version", |row| row.get(0))
            .unwrap();
        let pragmas = settings(&conn);
        drop(conn);
        let error = Importer::new(&db)
            .batch(|batch| -> Result<()> {
                batch.import_daily(&ha, &source("HA"), ImportMode::Full, None)?;
                batch.import_daily(&za, &source("ZA"), ImportMode::Full, None)?;
                Ok(())
            })
            .unwrap_err();
        assert!(error.to_string().contains("injected patch failure"));
        assert_eq!(db.get_applied_patches("HA").unwrap().len(), 1);
        assert_eq!(
            db.get_last_updated().unwrap().as_deref(),
            source("HA").timestamp
        );
        assert!(db.get_applied_patches("ZA").unwrap().is_empty());
        assert_eq!(db.count_by_service(&["ZA"]).unwrap(), 0);
        Importer::new(&db)
            .batch(|_| -> Result<()> { Ok(()) })
            .unwrap();
        let conn = db.conn().unwrap();
        assert_eq!(
            conn.pragma_query_value::<i64, _>(None, "schema_version", |row| row.get(0))
                .unwrap(),
            schema
        );
        assert_eq!(settings(&conn), pragmas);
    }

    #[test]
    fn untracked_patch_imports_records_without_changing_source_or_bulk_settings() {
        let dir = TempDir::new().unwrap();
        let db = database(&dir);
        let ha = archive(&dir, "l_amat", false);
        let indexes = index_names(&db.conn().unwrap());
        let pragmas = settings(&db.conn().unwrap());
        let stats = Importer::new(&db)
            .batch(|batch| batch.import_patch(&ha, ImportMode::Full, None))
            .unwrap();
        assert!(stats.records > 0);
        assert!(db.count_by_service(&["HA"]).unwrap() > 0);
        assert_eq!(db.get_last_weekly_date("HA").unwrap(), None);
        assert!(db.get_applied_patches("HA").unwrap().is_empty());
        assert_eq!(index_names(&db.conn().unwrap()), indexes);
        assert_eq!(settings(&db.conn().unwrap()), pragmas);
    }

    #[test]
    fn locked_database_restores_settings_after_partial_bulk_setup_failure() {
        let dir = TempDir::new().unwrap();
        let db = database(&dir);
        let ha = archive(&dir, "l_amat", false);
        let indexes = index_names(&db.conn().unwrap());
        let pragmas = settings(&db.conn().unwrap());
        let reader = Connection::open(dir.path().join("test.db")).unwrap();
        reader
            .execute_batch("BEGIN; SELECT * FROM licenses;")
            .unwrap();
        let error = Importer::new(&db)
            .batch(|batch| batch.import_weekly(&ha, &source("HA"), ImportMode::Full, None))
            .unwrap_err();
        assert!(error.to_string().contains("locked"), "{error}");
        assert_eq!(db.count_by_service(&["HA"]).unwrap(), 0);
        assert_eq!(index_names(&db.conn().unwrap()), indexes);
        assert_eq!(settings(&db.conn().unwrap()), pragmas);
        reader.execute_batch("ROLLBACK;").unwrap();
        drop(reader);
        Importer::new(&db)
            .batch(|batch| batch.import_weekly(&ha, &source("HA"), ImportMode::Full, None))
            .unwrap();
    }

    #[test]
    fn retry_inside_batch_preserves_settings_from_before_failed_setup() {
        let dir = TempDir::new().unwrap();
        let db = database(&dir);
        let ha = archive(&dir, "l_amat", false);
        let indexes = index_names(&db.conn().unwrap());
        let pragmas = settings(&db.conn().unwrap());
        let reader = Connection::open(dir.path().join("test.db")).unwrap();
        reader
            .execute_batch("BEGIN; SELECT * FROM licenses;")
            .unwrap();
        Importer::new(&db)
            .batch(|batch| -> Result<()> {
                let error = batch
                    .import_weekly(&ha, &source("HA"), ImportMode::Full, None)
                    .unwrap_err();
                assert!(error.to_string().contains("locked"), "{error}");
                reader.execute_batch("ROLLBACK;")?;
                drop(reader);
                batch.import_weekly(&ha, &source("HA"), ImportMode::Full, None)?;
                Ok(())
            })
            .unwrap();
        assert!(db.count_by_service(&["HA"]).unwrap() > 0);
        assert_eq!(index_names(&db.conn().unwrap()), indexes);
        assert_eq!(settings(&db.conn().unwrap()), pragmas);
    }

    #[test]
    fn later_callback_panic_rolls_back_and_restores_batch_state() {
        let dir = TempDir::new().unwrap();
        let db = database(&dir);
        let ha = archive(&dir, "l_amat", false);
        let za = archive(&dir, "l_gmrs", false);
        let indexes = index_names(&db.conn().unwrap());
        let pragmas = settings(&db.conn().unwrap());
        let panic = catch_unwind(AssertUnwindSafe(|| {
            Importer::new(&db).batch(|batch| -> Result<()> {
                batch.import_weekly(&ha, &source("HA"), ImportMode::Full, None)?;
                batch.import_weekly(
                    &za,
                    &source("ZA"),
                    ImportMode::Full,
                    Some(Box::new(|_| panic!("injected callback panic"))),
                )?;
                Ok(())
            })
        }));
        assert!(panic.is_err());
        assert_eq!(
            db.get_last_weekly_date("HA").unwrap(),
            Some(source("HA").date)
        );
        assert_eq!(db.get_last_weekly_date("ZA").unwrap(), None);
        assert_eq!(db.count_by_service(&["ZA"]).unwrap(), 0);
        assert_eq!(index_names(&db.conn().unwrap()), indexes);
        assert_eq!(settings(&db.conn().unwrap()), pragmas);
        Importer::new(&db)
            .batch(|batch| batch.import_weekly(&za, &source("ZA"), ImportMode::Full, None))
            .unwrap();
    }

    #[test]
    fn restoration_failure_is_reported_with_original_error_and_restores_pragmas() {
        for fail_operation in [false, true] {
            let dir = TempDir::new().unwrap();
            let db = database(&dir);
            let ha = archive(&dir, "l_amat", false);
            let pragmas = settings(&db.conn().unwrap());
            let error = Importer::new(&db)
                .batch(|batch| -> Result<()> {
                    batch.import_weekly(&ha, &source("HA"), ImportMode::Full, None)?;
                    // Deliberately make index restoration fail in this disposable DB.
                    batch
                        .guard
                        .conn
                        .execute_batch("ALTER TABLE entities RENAME TO hidden_entities;")?;
                    if fail_operation {
                        Err(DbError::InvalidData("original failure".into()))
                    } else {
                        Ok(())
                    }
                })
                .unwrap_err();
            assert!(
                error.to_string().contains("import cleanup failed"),
                "{error}"
            );
            assert_eq!(
                error.to_string().contains("original failure"),
                fail_operation
            );
            assert_eq!(settings(&db.conn().unwrap()), pragmas);
            let conn = db.conn().unwrap();
            conn.execute_batch("ALTER TABLE hidden_entities RENAME TO entities;")
                .unwrap();
            Schema::create_indexes(&conn).unwrap();
        }
    }
}
