//! Database schema definition and management.
//!
//! Creates and manages the SQLite schema for ULS data storage.

use rusqlite::Connection;

use crate::error::Result;

/// Current schema version.
pub const SCHEMA_VERSION: i32 = 6;

/// Database schema management.
pub struct Schema;

impl Schema {
    /// Create all tables in the database.
    pub fn create_tables(conn: &Connection) -> Result<()> {
        // Set optimal page size for text-heavy data (must be before first table creation)
        // Smaller pages = less wasted space when storing many short strings
        conn.execute_batch("PRAGMA page_size = 1024;")?;

        // Metadata table for tracking schema version and update times
        // WITHOUT ROWID: Small table with TEXT PRIMARY KEY
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            ) WITHOUT ROWID;
            "#,
        )?;

        // Set schema version
        conn.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES ('schema_version', ?1)",
            [&SCHEMA_VERSION.to_string()],
        )?;

        // Header record (HD) - main license table
        // WITHOUT ROWID: Largest table, integer PK, saves ~8 bytes per row
        // license_status and radio_service_code stored as INTEGER for compactness
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS licenses (
                unique_system_identifier INTEGER PRIMARY KEY,
                uls_file_number TEXT,
                ebf_number TEXT,
                call_sign TEXT COLLATE NOCASE,
                license_status INTEGER,
                radio_service_code INTEGER,
                grant_date TEXT,
                expired_date TEXT,
                cancellation_date TEXT,
                effective_date TEXT,
                last_action_date TEXT,
                revoked_certification TEXT,
                license_revoked TEXT,
                licensee_name TEXT COLLATE NOCASE,
                first_name TEXT COLLATE NOCASE,
                middle_initial TEXT,
                last_name TEXT COLLATE NOCASE,
                suffix TEXT,
                frn TEXT COLLATE NOCASE,
                previous_call_sign TEXT COLLATE NOCASE,
                trustee_call_sign TEXT COLLATE NOCASE,
                trustee_name TEXT COLLATE NOCASE
            ) WITHOUT ROWID;
            "#,
        )?;

        // Entity record (EN)
        // Composite unique key: (unique_system_identifier, entity_type)
        // A license can have multiple entities (Licensee, Contact, etc.)
        // entity_type stored as INTEGER for compactness
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS entities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                unique_system_identifier INTEGER NOT NULL,
                uls_file_number TEXT,
                ebf_number TEXT,
                call_sign TEXT COLLATE NOCASE,
                entity_type INTEGER,
                licensee_id TEXT,
                entity_name TEXT COLLATE NOCASE,
                first_name TEXT COLLATE NOCASE,
                middle_initial TEXT,
                last_name TEXT COLLATE NOCASE,
                suffix TEXT,
                phone TEXT,
                fax TEXT,
                email TEXT,
                street_address TEXT COLLATE NOCASE,
                city TEXT COLLATE NOCASE,
                state TEXT COLLATE NOCASE,
                zip_code TEXT COLLATE NOCASE,
                po_box TEXT,
                attention_line TEXT,
                sgin TEXT,
                frn TEXT COLLATE NOCASE,
                applicant_type_code TEXT,
                status_code TEXT,
                status_date TEXT,
                UNIQUE(unique_system_identifier, entity_type),
                FOREIGN KEY (unique_system_identifier) REFERENCES licenses(unique_system_identifier)
            );
            "#,
        )?;

        // Amateur record (AM)
        // Unique key: unique_system_identifier (one per license)
        // operator_class and previous_operator_class stored as INTEGER
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS amateur_operators (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                unique_system_identifier INTEGER NOT NULL UNIQUE,
                uls_file_number TEXT,
                ebf_number TEXT,
                call_sign TEXT COLLATE NOCASE,
                operator_class INTEGER,
                group_code TEXT,
                region_code INTEGER,
                trustee_call_sign TEXT COLLATE NOCASE,
                trustee_indicator TEXT,
                physician_certification TEXT,
                ve_signature TEXT,
                systematic_call_sign_change TEXT,
                vanity_call_sign_change TEXT,
                vanity_relationship TEXT,
                previous_call_sign TEXT COLLATE NOCASE,
                previous_operator_class INTEGER,
                trustee_name TEXT COLLATE NOCASE,
                FOREIGN KEY (unique_system_identifier) REFERENCES licenses(unique_system_identifier)
            );
            "#,
        )?;

        // History record (HS)
        // Composite unique key: (unique_system_identifier, log_date, code)
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                unique_system_identifier INTEGER NOT NULL,
                uls_file_number TEXT,
                callsign TEXT,
                log_date TEXT,
                code TEXT,
                UNIQUE(unique_system_identifier, log_date, code),
                FOREIGN KEY (unique_system_identifier) REFERENCES licenses(unique_system_identifier)
            );
            "#,
        )?;

        // Comments record (CO)
        // Composite unique key: (unique_system_identifier, comment_date)
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS comments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                unique_system_identifier INTEGER NOT NULL,
                uls_file_number TEXT,
                callsign TEXT,
                comment_date TEXT,
                description TEXT,
                status_code TEXT,
                status_date TEXT,
                UNIQUE(unique_system_identifier, comment_date),
                FOREIGN KEY (unique_system_identifier) REFERENCES licenses(unique_system_identifier)
            );
            "#,
        )?;

        // Special conditions record (SC)
        // Composite unique key: (unique_system_identifier, special_condition_code)
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS special_conditions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                unique_system_identifier INTEGER NOT NULL,
                uls_file_number TEXT,
                ebf_number TEXT,
                callsign TEXT,
                special_condition_type TEXT,
                special_condition_code INTEGER,
                status_code TEXT,
                status_date TEXT,
                UNIQUE(unique_system_identifier, special_condition_code),
                FOREIGN KEY (unique_system_identifier) REFERENCES licenses(unique_system_identifier)
            );
            "#,
        )?;

        // Import status tracking - which record types have been imported per service
        // WITHOUT ROWID: Small table with composite TEXT PRIMARY KEY
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS import_status (
                radio_service_code TEXT NOT NULL,
                record_type TEXT NOT NULL,
                imported_at TEXT,
                record_count INTEGER,
                PRIMARY KEY (radio_service_code, record_type)
            ) WITHOUT ROWID;
            "#,
        )?;

        // Applied patches tracking - which daily files have been applied since last weekly
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS applied_patches (
                radio_service_code TEXT NOT NULL,
                patch_date TEXT NOT NULL,
                patch_weekday TEXT NOT NULL,
                applied_at TEXT NOT NULL,
                etag TEXT,
                record_count INTEGER,
                PRIMARY KEY (radio_service_code, patch_date)
            );
            "#,
        )?;

        Ok(())
    }

    /// Create indexes for efficient queries.
    pub fn create_indexes(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            -- License indexes
            CREATE INDEX IF NOT EXISTS idx_licenses_call_sign ON licenses(call_sign);
            CREATE INDEX IF NOT EXISTS idx_licenses_status ON licenses(license_status);
            CREATE INDEX IF NOT EXISTS idx_licenses_service ON licenses(radio_service_code);
            CREATE INDEX IF NOT EXISTS idx_licenses_frn ON licenses(frn);
            CREATE INDEX IF NOT EXISTS idx_licenses_name ON licenses(licensee_name);
            CREATE INDEX IF NOT EXISTS idx_licenses_grant_date ON licenses(grant_date);
            CREATE INDEX IF NOT EXISTS idx_licenses_expired_date ON licenses(expired_date);
            
            -- Entity indexes
            CREATE INDEX IF NOT EXISTS idx_entities_usi ON entities(unique_system_identifier);
            CREATE INDEX IF NOT EXISTS idx_entities_call_sign ON entities(call_sign);
            CREATE INDEX IF NOT EXISTS idx_entities_frn ON entities(frn);
            CREATE INDEX IF NOT EXISTS idx_entities_city_state ON entities(city, state);
            CREATE INDEX IF NOT EXISTS idx_entities_name ON entities(entity_name);
            CREATE INDEX IF NOT EXISTS idx_entities_last_name ON entities(last_name);
            
            -- Amateur operator indexes
            CREATE INDEX IF NOT EXISTS idx_amateur_usi ON amateur_operators(unique_system_identifier);
            CREATE INDEX IF NOT EXISTS idx_amateur_call_sign ON amateur_operators(call_sign);
            CREATE INDEX IF NOT EXISTS idx_amateur_class ON amateur_operators(operator_class);
            
            -- History indexes
            CREATE INDEX IF NOT EXISTS idx_history_usi ON history(unique_system_identifier);
            CREATE INDEX IF NOT EXISTS idx_history_callsign ON history(callsign);
            
            -- Comments indexes
            CREATE INDEX IF NOT EXISTS idx_comments_usi ON comments(unique_system_identifier);
            CREATE INDEX IF NOT EXISTS idx_comments_callsign ON comments(callsign);
            
            -- Special conditions indexes
            CREATE INDEX IF NOT EXISTS idx_special_cond_usi ON special_conditions(unique_system_identifier);
            "#,
        )?;

        Ok(())
    }

    /// Drop all non-primary indexes (for bulk import performance).
    /// After import completes, call `create_indexes` to rebuild them.
    pub fn drop_indexes(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            r#"
            -- License indexes
            DROP INDEX IF EXISTS idx_licenses_call_sign;
            DROP INDEX IF EXISTS idx_licenses_status;
            DROP INDEX IF EXISTS idx_licenses_service;
            DROP INDEX IF EXISTS idx_licenses_frn;
            DROP INDEX IF EXISTS idx_licenses_name;
            DROP INDEX IF EXISTS idx_licenses_grant_date;
            DROP INDEX IF EXISTS idx_licenses_expired_date;
            
            -- Entity indexes
            DROP INDEX IF EXISTS idx_entities_usi;
            DROP INDEX IF EXISTS idx_entities_call_sign;
            DROP INDEX IF EXISTS idx_entities_frn;
            DROP INDEX IF EXISTS idx_entities_city_state;
            DROP INDEX IF EXISTS idx_entities_name;
            DROP INDEX IF EXISTS idx_entities_last_name;
            
            -- Amateur operator indexes
            DROP INDEX IF EXISTS idx_amateur_usi;
            DROP INDEX IF EXISTS idx_amateur_call_sign;
            DROP INDEX IF EXISTS idx_amateur_class;
            
            -- History indexes
            DROP INDEX IF EXISTS idx_history_usi;
            DROP INDEX IF EXISTS idx_history_callsign;
            
            -- Comments indexes
            DROP INDEX IF EXISTS idx_comments_usi;
            DROP INDEX IF EXISTS idx_comments_callsign;
            
            -- Special conditions indexes
            DROP INDEX IF EXISTS idx_special_cond_usi;
            "#,
        )?;

        Ok(())
    }

    /// Initialize a new database with schema and indexes.
    pub fn initialize(conn: &Connection) -> Result<()> {
        Self::create_tables(conn)?;
        Self::create_indexes(conn)?;
        Ok(())
    }

    /// Get the current schema version from the database.
    /// Returns None if the database is not initialized (table doesn't exist or no rows).
    pub fn get_version(conn: &Connection) -> Result<Option<i32>> {
        let result = conn.query_row(
            "SELECT value FROM metadata WHERE key = 'schema_version'",
            [],
            |row| row.get::<_, String>(0),
        );

        match result {
            Ok(v) => Ok(v.parse().ok()),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(rusqlite::Error::SqliteFailure(_, Some(ref msg)))
                if msg.contains("no such table") =>
            {
                // Table doesn't exist = not initialized
                Ok(None)
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Set a metadata value.
    pub fn set_metadata(conn: &Connection, key: &str, value: &str) -> Result<()> {
        conn.execute(
            "INSERT OR REPLACE INTO metadata (key, value) VALUES (?1, ?2)",
            [key, value],
        )?;
        Ok(())
    }

    /// Get a metadata value.
    pub fn get_metadata(conn: &Connection, key: &str) -> Result<Option<String>> {
        let result = conn.query_row("SELECT value FROM metadata WHERE key = ?1", [key], |row| {
            row.get(0)
        });

        match result {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Migrate the database schema if needed.
    ///
    /// This handles upgrades from older schema versions to the current version.
    pub fn migrate_if_needed(conn: &Connection) -> Result<()> {
        let current_version = Self::get_version(conn)?;

        match current_version {
            None => {
                // Database not initialized, nothing to migrate
                Ok(())
            }
            Some(v) if v >= SCHEMA_VERSION => {
                // Already at or above current version
                Ok(())
            }
            Some(v) => {
                // Need to migrate
                tracing::info!(
                    "Migrating database from schema v{} to v{}",
                    v,
                    SCHEMA_VERSION
                );

                // Apply migrations in order
                if v < 5 {
                    Self::migrate_to_v5(conn)?;
                }

                // Update schema version
                conn.execute(
                    "INSERT OR REPLACE INTO metadata (key, value) VALUES ('schema_version', ?1)",
                    [&SCHEMA_VERSION.to_string()],
                )?;

                Ok(())
            }
        }
    }

    /// Migrate from v4 to v5: Add applied_patches table.
    fn migrate_to_v5(conn: &Connection) -> Result<()> {
        tracing::info!("Applying migration to v5: adding applied_patches table");

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS applied_patches (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                radio_service_code TEXT NOT NULL,
                patch_date TEXT NOT NULL,
                patch_weekday TEXT NOT NULL,
                applied_at TEXT NOT NULL,
                etag TEXT,
                record_count INTEGER,
                UNIQUE(radio_service_code, patch_date)
            );
            
            CREATE INDEX IF NOT EXISTS idx_applied_patches_service 
                ON applied_patches(radio_service_code);
            "#,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_create_schema() {
        let conn = Connection::open_in_memory().unwrap();
        Schema::initialize(&conn).unwrap();

        // Verify tables exist
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='licenses'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Verify schema version
        let version = Schema::get_version(&conn).unwrap();
        assert_eq!(version, Some(SCHEMA_VERSION));
    }

    #[test]
    fn test_metadata() {
        let conn = Connection::open_in_memory().unwrap();
        Schema::initialize(&conn).unwrap();

        Schema::set_metadata(&conn, "test_key", "test_value").unwrap();
        let value = Schema::get_metadata(&conn, "test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        let missing = Schema::get_metadata(&conn, "nonexistent").unwrap();
        assert_eq!(missing, None);
    }

    #[test]
    fn test_unique_constraints() {
        let conn = Connection::open_in_memory().unwrap();
        Schema::initialize(&conn).unwrap();

        // Insert a license
        conn.execute(
            "INSERT INTO licenses (unique_system_identifier, call_sign) VALUES (1, 'W1AW')",
            [],
        )
        .unwrap();

        // Insert entity
        conn.execute(
            "INSERT INTO entities (unique_system_identifier, entity_type, entity_name) VALUES (1, 'L', 'Test')",
            [],
        ).unwrap();

        // Duplicate should fail
        let result = conn.execute(
            "INSERT INTO entities (unique_system_identifier, entity_type, entity_name) VALUES (1, 'L', 'Test2')",
            [],
        );
        assert!(result.is_err());

        // Different entity_type should succeed
        conn.execute(
            "INSERT INTO entities (unique_system_identifier, entity_type, entity_name) VALUES (1, 'C', 'Contact')",
            [],
        ).unwrap();
    }

    #[test]
    fn test_drop_and_recreate_indexes() {
        let conn = Connection::open_in_memory().unwrap();
        Schema::initialize(&conn).unwrap();

        // Count indexes before drop
        let count_before: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(count_before > 0, "Should have indexes after initialize");

        // Drop indexes
        Schema::drop_indexes(&conn).unwrap();

        // Count indexes after drop
        let count_after_drop: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count_after_drop, 0, "All indexes should be dropped");

        // Recreate indexes
        Schema::create_indexes(&conn).unwrap();

        // Count indexes after recreate
        let count_after_recreate: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            count_before, count_after_recreate,
            "All indexes should be recreated"
        );
    }
}
