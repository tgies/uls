//! Query engine for license lookups and searches.

use std::path::Path;

use rusqlite::params_from_iter;
use tracing::debug;

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
                e.street_address, e.city, e.state, e.zip_code,
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
            Ok(License {
                unique_system_identifier: row.get(0)?,
                call_sign: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                licensee_name: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                first_name: row.get(3)?,
                middle_initial: row.get(4)?,
                last_name: row.get(5)?,
                status: row
                    .get::<_, Option<String>>(6)?
                    .and_then(|s| s.chars().next())
                    .unwrap_or('?'),
                radio_service: row.get::<_, Option<String>>(7)?.unwrap_or_default(),
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
                operator_class: row
                    .get::<_, Option<String>>(17)?
                    .and_then(|s| s.chars().next()),
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
        let count: usize = conn.query_row(&query, params_from_iter(params), |row| row.get(0))?;
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
}
