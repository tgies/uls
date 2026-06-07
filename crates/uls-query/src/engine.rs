//! Query engine for license lookups and searches.

use std::path::Path;

use rusqlite::params_from_iter;
use tracing::debug;

use uls_db::enum_adapters::{read_license_status, read_operator_class, read_radio_service};
use uls_db::{Database, DatabaseConfig};

use crate::filter::SearchFilter;
use uls_db::models::{License, LicenseStats};

/// Errors from query operations.
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("database error: {0}")]
    Database(#[from] uls_db::DbError),

    #[error("database not initialized - run 'uls update' first")]
    NotInitialized,

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

/// Result type for query operations.
pub type Result<T> = std::result::Result<T, QueryError>;

/// Query engine for ULS data.
pub struct QueryEngine {
    db: Database,
}

impl QueryEngine {
    /// Open a query engine with the given database path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = DatabaseConfig::with_path(path.as_ref());
        let db = Database::with_config(config)?;

        if !db.is_initialized()? {
            return Err(QueryError::NotInitialized);
        }

        Ok(Self { db })
    }

    /// Create a query engine with an existing database.
    pub fn with_database(db: Database) -> Self {
        Self { db }
    }

    /// Look up a license by callsign.
    pub fn lookup(&self, callsign: &str) -> Result<Option<License>> {
        Ok(self.db.get_license_by_callsign(callsign)?)
    }

    /// Look up all licenses by FRN (FCC Registration Number).
    pub fn lookup_by_frn(&self, frn: &str) -> Result<Vec<License>> {
        Ok(self.db.get_licenses_by_frn(frn)?)
    }

    /// Search for licenses matching the given filter.
    pub fn search(&self, filter: SearchFilter) -> Result<Vec<License>> {
        let (where_clause, params) = filter.to_where_clause();
        let order_clause = filter.order_clause();
        let limit_clause = filter.limit_clause();

        let query = format!(
            r#"
            SELECT
                l.unique_system_identifier, l.call_sign,
                e.entity_name, e.first_name, e.middle_initial, e.last_name,
                l.license_status, l.radio_service_code,
                l.grant_date, l.expired_date, l.cancellation_date,
                e.frn, NULL as previous_call_sign,
                e.street_address, e.city, e.state, e.zip_code, e.po_box,
                a.operator_class
            FROM licenses l
            LEFT JOIN entities e ON l.unique_system_identifier = e.unique_system_identifier
            LEFT JOIN amateur_operators a ON l.unique_system_identifier = a.unique_system_identifier
            WHERE {}
            {}
            {}
            "#,
            where_clause, order_clause, limit_clause
        );

        debug!("Search query: {}", query);
        debug!("Params: {:?}", params);

        let conn = self.db.conn()?;

        let mut stmt = conn.prepare(&query)?;
        let iter = stmt.query_map(params_from_iter(params), |row| {
            // Use centralized enum adapter helpers from uls-db
            let status = read_license_status(row, 6)?;
            let radio_service = read_radio_service(row, 7)?;
            let operator_class = read_operator_class(row, 18)?;

            Ok(License {
                unique_system_identifier: row.get(0)?,
                call_sign: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                licensee_name: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                first_name: row.get(3)?,
                middle_initial: row.get(4)?,
                last_name: row.get(5)?,
                status,
                radio_service,
                grant_date: row
                    .get::<_, Option<String>>(8)?
                    .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
                expired_date: row
                    .get::<_, Option<String>>(9)?
                    .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
                cancellation_date: row
                    .get::<_, Option<String>>(10)?
                    .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
                frn: row.get(11)?,
                previous_call_sign: row.get(12)?,
                street_address: row.get(13)?,
                city: row.get(14)?,
                state: row.get(15)?,
                zip_code: row.get(16)?,
                po_box: row.get(17)?,
                operator_class,
            })
        })?;

        let mut results = Vec::new();
        for license in iter {
            results.push(license?);
        }

        Ok(results)
    }

    /// Get database statistics.
    pub fn stats(&self) -> Result<LicenseStats> {
        Ok(self.db.get_stats()?)
    }

    /// Check if the database is ready for queries.
    pub fn is_ready(&self) -> Result<bool> {
        self.db.is_initialized().map_err(Into::into)
    }

    /// Get the count of results for a filter without fetching all data.
    pub fn count(&self, filter: SearchFilter) -> Result<usize> {
        let (where_clause, params) = filter.to_where_clause();

        let query = format!(
            r#"
            SELECT COUNT(*)
            FROM licenses l
            LEFT JOIN entities e ON l.unique_system_identifier = e.unique_system_identifier
            LEFT JOIN amateur_operators a ON l.unique_system_identifier = a.unique_system_identifier
            WHERE {}
            "#,
            where_clause
        );

        let conn = self.db.conn()?;
        let count: usize =
            conn.query_row(&query, params_from_iter(params), |row| row.get::<_, i64>(0))? as usize;
        Ok(count)
    }

    /// Get the underlying database reference.
    pub fn database(&self) -> &Database {
        &self.db
    }

    // ========================================================================
    // Lazy Loading Support
    // ========================================================================

    /// Determine which record types are required for basic queries.
    ///
    /// Returns the minimal set of record types needed:
    /// - HD (licenses) - always needed
    /// - EN (entities) - needed for name/address/FRN
    /// - AM (amateur) - needed if operator_class filter is used
    pub fn required_record_types(filter: &SearchFilter) -> Vec<&'static str> {
        let mut types = vec!["HD", "EN"];
        if filter.operator_class.is_some() {
            types.push("AM");
        }
        types
    }

    /// Check if any required record types are missing for a given service.
    ///
    /// Returns a list of missing record types that need to be imported.
    pub fn missing_data_for_query(
        &self,
        service: &str,
        filter: &SearchFilter,
    ) -> Result<Vec<String>> {
        let required = Self::required_record_types(filter);
        let mut missing = Vec::new();

        for record_type in required {
            if !self.db.has_record_type(service, record_type)? {
                missing.push(record_type.to_string());
            }
        }

        Ok(missing)
    }

    /// Check if data is available for basic queries (HD + EN at minimum).
    pub fn has_basic_data(&self, service: &str) -> Result<bool> {
        let has_hd = self.db.has_record_type(service, "HD")?;
        let has_en = self.db.has_record_type(service, "EN")?;
        Ok(has_hd && has_en)
    }

    /// Get the list of imported record types for a service.
    pub fn imported_types(&self, service: &str) -> Result<Vec<String>> {
        Ok(self.db.get_imported_types(service)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_engine_with_initialized_db() {
        let config = DatabaseConfig::in_memory();
        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();

        let engine = QueryEngine::with_database(db);
        assert!(engine.is_ready().unwrap());
    }

    #[test]
    fn test_query_engine_not_initialized() {
        let config = DatabaseConfig::in_memory();
        let db = Database::with_config(config).unwrap();
        // Don't initialize - should return false, not error

        let engine = QueryEngine::with_database(db);
        assert!(!engine.is_ready().unwrap());
    }

    #[test]
    fn test_lookup_missing() {
        let config = DatabaseConfig::in_memory();
        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();

        let engine = QueryEngine::with_database(db);
        let result = engine.lookup("NONEXISTENT").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_search_empty_db() {
        let config = DatabaseConfig::in_memory();
        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();

        let engine = QueryEngine::with_database(db);
        let filter = SearchFilter::default();
        let results = engine.search(filter).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_count_empty_db() {
        let config = DatabaseConfig::in_memory();
        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();

        let engine = QueryEngine::with_database(db);
        let filter = SearchFilter::default();
        let count = engine.count(filter).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_stats() {
        let config = DatabaseConfig::in_memory();
        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();

        let engine = QueryEngine::with_database(db);
        let stats = engine.stats().unwrap();
        assert_eq!(stats.total_licenses, 0);
    }

    #[test]
    fn test_lookup_by_frn_empty() {
        let config = DatabaseConfig::in_memory();
        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();

        let engine = QueryEngine::with_database(db);
        let results = engine.lookup_by_frn("0001234567").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_database_accessor() {
        let config = DatabaseConfig::in_memory();
        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();

        let engine = QueryEngine::with_database(db);
        assert!(engine.database().is_initialized().unwrap());
    }

    #[test]
    fn test_search_with_data() {
        use uls_core::records::{HeaderRecord, UlsRecord};

        let config = DatabaseConfig::in_memory();
        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();

        // Insert a test license
        let mut header = HeaderRecord::from_fields(&["HD", "12345"]);
        header.unique_system_identifier = 12345;
        header.call_sign = Some("W1TEST".to_string());
        header.license_status = Some('A');
        header.radio_service_code = Some("HA".to_string());
        db.insert_record(&UlsRecord::Header(header)).unwrap();

        let engine = QueryEngine::with_database(db);

        // Search with callsign filter
        let filter = SearchFilter::callsign("W1TEST");
        let results = engine.search(filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].call_sign, "W1TEST");

        // Count should match
        let filter = SearchFilter::callsign("W1TEST");
        let count = engine.count(filter).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_required_record_types_basic() {
        let filter = SearchFilter::default();
        let types = QueryEngine::required_record_types(&filter);
        assert_eq!(types, vec!["HD", "EN"]);
    }

    #[test]
    fn test_required_record_types_with_operator_class() {
        let filter = SearchFilter::new().with_operator_class('E');
        let types = QueryEngine::required_record_types(&filter);
        assert_eq!(types, vec!["HD", "EN", "AM"]);
    }

    #[test]
    fn test_has_basic_data_empty_db() {
        let config = DatabaseConfig::in_memory();
        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();

        let engine = QueryEngine::with_database(db);
        // Empty database has no record types
        let has_data = engine.has_basic_data("HA").unwrap();
        assert!(!has_data);
    }

    #[test]
    fn test_imported_types_empty_db() {
        let config = DatabaseConfig::in_memory();
        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();

        let engine = QueryEngine::with_database(db);
        let types = engine.imported_types("HA").unwrap();
        assert!(types.is_empty());
    }

    #[test]
    fn test_missing_data_for_query_empty_db() {
        let config = DatabaseConfig::in_memory();
        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();

        let engine = QueryEngine::with_database(db);
        let filter = SearchFilter::default();
        let missing = engine.missing_data_for_query("HA", &filter).unwrap();
        // Should be missing HD and EN since db is empty
        assert!(missing.contains(&"HD".to_string()));
        assert!(missing.contains(&"EN".to_string()));
    }

    #[test]
    fn test_missing_data_for_query_with_operator_class() {
        let config = DatabaseConfig::in_memory();
        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();

        let engine = QueryEngine::with_database(db);
        let filter = SearchFilter::new().with_operator_class('E');
        let missing = engine.missing_data_for_query("HA", &filter).unwrap();
        // Should be missing HD, EN, and AM
        assert!(missing.contains(&"HD".to_string()));
        assert!(missing.contains(&"EN".to_string()));
        assert!(missing.contains(&"AM".to_string()));
    }

    #[test]
    fn test_open_uninitialized_db_returns_not_initialized() {
        // A file-backed database that has never had its schema applied reports
        // NotInitialized when opened through the query engine.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.db");

        // Touch the file via with_config so it exists but lacks schema.
        let db = Database::with_config(DatabaseConfig::with_path(&path)).unwrap();
        assert!(!db.is_initialized().unwrap());
        drop(db);

        match QueryEngine::open(&path) {
            Err(QueryError::NotInitialized) => {}
            Err(other) => panic!("expected NotInitialized, got {other}"),
            Ok(_) => panic!("expected NotInitialized error, got an engine"),
        }

        // The error renders the operator-facing guidance.
        let err = QueryError::NotInitialized;
        assert_eq!(
            err.to_string(),
            "database not initialized - run 'uls update' first"
        );
    }

    #[test]
    fn test_open_initialized_db_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ready.db");

        // Initialize the schema, then drop the handle.
        let db = Database::with_config(DatabaseConfig::with_path(&path)).unwrap();
        db.initialize().unwrap();
        drop(db);

        // Opening the same path yields a ready engine.
        let engine = QueryEngine::open(&path).unwrap();
        assert!(engine.is_ready().unwrap());
        let stats = engine.stats().unwrap();
        assert_eq!(stats.total_licenses, 0);
    }

    #[test]
    fn test_search_operator_class_join_returns_class() {
        use uls_core::records::{AmateurRecord, HeaderRecord, UlsRecord};

        let config = DatabaseConfig::in_memory();
        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();

        // Header for the license.
        let mut header = HeaderRecord::from_fields(&["HD", "555"]);
        header.unique_system_identifier = 555;
        header.call_sign = Some("W1EXT".to_string());
        header.license_status = Some('A');
        header.radio_service_code = Some("HA".to_string());
        db.insert_record(&UlsRecord::Header(header)).unwrap();

        // Amateur record carrying the Extra operator class.
        let amateur = AmateurRecord {
            unique_system_identifier: 555,
            operator_class: Some('E'),
            ..AmateurRecord::from_fields(&["AM", "555"])
        };
        db.insert_record(&UlsRecord::Amateur(amateur)).unwrap();

        let engine = QueryEngine::with_database(db);

        // Filtering by operator class exercises the amateur_operators join.
        let filter = SearchFilter::callsign("W1EXT").with_operator_class('E');
        let results = engine.search(filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].operator_class, Some('E'));

        // A different class must not match the joined row.
        let filter = SearchFilter::callsign("W1EXT").with_operator_class('T');
        assert!(engine.search(filter).unwrap().is_empty());
    }

    #[test]
    fn test_search_orders_and_limits_multiple_rows() {
        use uls_core::records::{HeaderRecord, UlsRecord};

        let config = DatabaseConfig::in_memory();
        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();

        for (id, call) in [(1_i64, "W3CCC"), (2, "W1AAA"), (3, "W2BBB")] {
            let mut header = HeaderRecord::from_fields(&["HD", "0"]);
            header.unique_system_identifier = id;
            header.call_sign = Some(call.to_string());
            header.license_status = Some('A');
            header.radio_service_code = Some("HA".to_string());
            db.insert_record(&UlsRecord::Header(header)).unwrap();
        }

        let engine = QueryEngine::with_database(db);

        // Default callsign-ascending sort across all rows.
        let all = engine.search(SearchFilter::default()).unwrap();
        let calls: Vec<&str> = all.iter().map(|l| l.call_sign.as_str()).collect();
        assert_eq!(calls, vec!["W1AAA", "W2BBB", "W3CCC"]);

        // Descending sort with a limit returns the top single row only.
        let filter = SearchFilter::default()
            .with_sort(crate::filter::SortOrder::CallSignDesc)
            .with_limit(1);
        let top = engine.search(filter).unwrap();
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].call_sign, "W3CCC");

        // Count ignores limit and reflects the full match set.
        assert_eq!(engine.count(SearchFilter::default()).unwrap(), 3);
    }

    #[test]
    fn test_lookup_and_lookup_by_frn_return_populated_license() {
        use uls_core::records::{EntityRecord, HeaderRecord, UlsRecord};

        let config = DatabaseConfig::in_memory();
        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();

        let mut header = HeaderRecord::from_fields(&["HD", "777"]);
        header.unique_system_identifier = 777;
        header.call_sign = Some("W1ABC".to_string());
        header.license_status = Some('A');
        header.radio_service_code = Some("HA".to_string());
        db.insert_record(&UlsRecord::Header(header)).unwrap();

        let entity = EntityRecord {
            unique_system_identifier: 777,
            call_sign: Some("W1ABC".to_string()),
            entity_name: Some("Jane Operator".to_string()),
            frn: Some("0009876543".to_string()),
            city: Some("NEWINGTON".to_string()),
            state: Some("CT".to_string()),
            ..EntityRecord::from_fields(&["EN", "777"])
        };
        db.insert_record(&UlsRecord::Entity(entity)).unwrap();

        let engine = QueryEngine::with_database(db);

        // Callsign lookup is case-insensitive and returns the joined entity data.
        let found = engine.lookup("w1abc").unwrap().expect("license present");
        assert_eq!(found.call_sign, "W1ABC");
        assert_eq!(found.frn.as_deref(), Some("0009876543"));
        assert_eq!(found.licensee_name, "Jane Operator");
        assert_eq!(found.city.as_deref(), Some("NEWINGTON"));

        // FRN lookup resolves the same license via the entities inner join.
        let by_frn = engine.lookup_by_frn("0009876543").unwrap();
        assert_eq!(by_frn.len(), 1);
        assert_eq!(by_frn[0].call_sign, "W1ABC");
        assert_eq!(by_frn[0].unique_system_identifier, 777);

        // A non-matching FRN yields no rows.
        assert!(engine.lookup_by_frn("0000000000").unwrap().is_empty());
    }
}
