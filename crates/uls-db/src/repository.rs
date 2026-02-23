//! Database repository for ULS data operations.
//!
//! Provides high-level methods for inserting, updating, and querying ULS data.

use std::fmt;
use std::path::Path;

use r2d2::{CustomizeConnection, Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection};
use tracing::{debug, info};

use uls_core::codes::{EntityType, LicenseStatus, OperatorClass, RadioService};
use uls_core::records::{
    AmateurRecord, CommentRecord, EntityRecord, HeaderRecord, HistoryRecord,
    SpecialConditionRecord, UlsRecord,
};

use crate::config::DatabaseConfig;
use crate::enum_adapters::{read_license_status, read_operator_class, read_radio_service};
use crate::error::Result;
use crate::models::{License, LicenseStats};
use crate::schema::Schema;

/// Connection customizer that applies PRAGMA settings to each new connection.
///
/// This ensures every connection from the pool has consistent settings,
/// not just the first connection checked out.
#[derive(Clone)]
struct SqliteConnectionCustomizer {
    cache_size: i32,
    foreign_keys: bool,
}

impl fmt::Debug for SqliteConnectionCustomizer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteConnectionCustomizer")
            .field("cache_size", &self.cache_size)
            .field("foreign_keys", &self.foreign_keys)
            .finish()
    }
}

impl CustomizeConnection<Connection, rusqlite::Error> for SqliteConnectionCustomizer {
    fn on_acquire(&self, conn: &mut Connection) -> std::result::Result<(), rusqlite::Error> {
        // Set cache size
        conn.execute_batch(&format!("PRAGMA cache_size = {};", self.cache_size))?;

        // Enable foreign keys if configured
        if self.foreign_keys {
            conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        }

        // Other per-connection optimizations
        conn.execute_batch(
            r#"
            PRAGMA busy_timeout = 5000;
            PRAGMA synchronous = NORMAL;
            PRAGMA temp_store = MEMORY;
            PRAGMA mmap_size = 268435456;
            "#,
        )?;

        Ok(())
    }
}

/// Database connection pool and operations.
pub struct Database {
    pool: Pool<SqliteConnectionManager>,
    config: DatabaseConfig,
}

impl Database {
    /// Open a database at the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = DatabaseConfig::with_path(path.as_ref());
        Self::with_config(config)
    }

    /// Open a database with the given configuration.
    pub fn with_config(config: DatabaseConfig) -> Result<Self> {
        // Create parent directory if needed
        if let Some(parent) = config.path.parent() {
            if !parent.exists() && config.path.to_str() != Some(":memory:") {
                std::fs::create_dir_all(parent)?;
            }
        }

        let manager = SqliteConnectionManager::file(&config.path);

        // Create customizer to apply PRAGMAs on every pooled connection
        let customizer = SqliteConnectionCustomizer {
            cache_size: config.cache_size,
            foreign_keys: config.foreign_keys,
        };

        let pool = Pool::builder()
            .max_size(config.max_connections)
            .min_idle(Some(0))
            .connection_timeout(config.connection_timeout)
            .connection_customizer(Box::new(customizer))
            .build(manager)?;

        let db = Self { pool, config };

        // Set WAL mode once (it's a database-wide setting, not per-connection)
        if db.config.enable_wal {
            let conn = db.conn()?;
            conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        }

        Ok(db)
    }

    /// Get a connection from the pool.
    pub fn conn(&self) -> Result<PooledConnection<SqliteConnectionManager>> {
        Ok(self.pool.get()?)
    }

    /// Initialize the database schema.
    pub fn initialize(&self) -> Result<()> {
        let conn = self.conn()?;
        Schema::initialize(&conn)?;
        info!(
            "Database initialized with schema version {}",
            crate::schema::SCHEMA_VERSION
        );
        Ok(())
    }

    /// Check if the database is initialized.
    pub fn is_initialized(&self) -> Result<bool> {
        let conn = self.conn()?;
        Ok(Schema::get_version(&conn)?.is_some())
    }

    /// Migrate the database schema if needed.
    ///
    /// Call this on an existing database to upgrade it to the current schema version.
    pub fn migrate_if_needed(&self) -> Result<()> {
        let conn = self.conn()?;
        Schema::migrate_if_needed(&conn)
    }

    /// Begin a transaction for bulk operations.
    pub fn begin_transaction(&self) -> Result<Transaction> {
        let conn = self.pool.get()?;
        conn.execute("BEGIN TRANSACTION", [])?;
        Ok(Transaction { conn })
    }

    /// Insert a ULS record into the database.
    pub fn insert_record(&self, record: &UlsRecord) -> Result<()> {
        let conn = self.conn()?;
        Self::insert_record_conn(&conn, record)
    }

    /// Insert a record using an existing connection.
    fn insert_record_conn(conn: &Connection, record: &UlsRecord) -> Result<()> {
        match record {
            UlsRecord::Header(hd) => Self::insert_header(conn, hd),
            UlsRecord::Entity(en) => Self::insert_entity(conn, en),
            UlsRecord::Amateur(am) => Self::insert_amateur(conn, am),
            UlsRecord::History(hs) => Self::insert_history(conn, hs),
            UlsRecord::Comment(co) => Self::insert_comment(conn, co),
            UlsRecord::SpecialCondition(sc) => Self::insert_special_condition(conn, sc),
            _ => {
                debug!(
                    "Skipping unsupported record type: {:?}",
                    record.record_type()
                );
                Ok(())
            }
        }
    }

    /// Insert a header record.
    fn insert_header(conn: &Connection, hd: &HeaderRecord) -> Result<()> {
        // Convert license_status char to integer code
        let license_status_code: Option<u8> = hd.license_status.and_then(|c| {
            c.to_string()
                .parse::<LicenseStatus>()
                .ok()
                .map(|s| s.to_u8())
        });
        // Convert radio_service_code string to integer code
        let radio_service_code: Option<u8> = hd
            .radio_service_code
            .as_ref()
            .and_then(|s| s.parse::<RadioService>().ok().map(|r| r.to_u8()));

        let mut stmt = conn.prepare_cached(
            r#"INSERT OR REPLACE INTO licenses (
                unique_system_identifier, uls_file_number, ebf_number, call_sign,
                license_status, radio_service_code, grant_date, expired_date,
                cancellation_date, effective_date, last_action_date
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
        )?;
        stmt.execute(params![
            hd.unique_system_identifier,
            hd.uls_file_number,
            hd.ebf_number,
            hd.call_sign,
            license_status_code,
            radio_service_code,
            hd.grant_date.map(|d| d.to_string()),
            hd.expired_date.map(|d| d.to_string()),
            hd.cancellation_date.map(|d| d.to_string()),
            hd.effective_date.map(|d| d.to_string()),
            hd.last_action_date.map(|d| d.to_string()),
        ])?;
        Ok(())
    }

    /// Insert an entity record.
    fn insert_entity(conn: &Connection, en: &EntityRecord) -> Result<()> {
        // Convert entity_type string to integer code
        let entity_type_code: Option<u8> = en
            .entity_type
            .as_ref()
            .and_then(|s| s.parse::<EntityType>().ok().map(|e| e.to_u8()));

        let mut stmt = conn.prepare_cached(
            r#"INSERT OR REPLACE INTO entities (
                unique_system_identifier, uls_file_number, ebf_number, call_sign,
                entity_type, licensee_id, entity_name, first_name, middle_initial,
                last_name, suffix, phone, fax, email, street_address, city, state,
                zip_code, po_box, attention_line, sgin, frn, applicant_type_code,
                status_code, status_date
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25)"#,
        )?;
        stmt.execute(params![
            en.unique_system_identifier,
            en.uls_file_number,
            en.ebf_number,
            en.call_sign,
            entity_type_code,
            en.licensee_id,
            en.entity_name,
            en.first_name,
            en.mi.map(|c| c.to_string()),
            en.last_name,
            en.suffix,
            en.phone,
            en.fax,
            en.email,
            en.street_address,
            en.city,
            en.state,
            en.zip_code,
            en.po_box,
            en.attention_line,
            en.sgin,
            en.frn,
            en.applicant_type_code.map(|c| c.to_string()),
            en.status_code.map(|c| c.to_string()),
            en.status_date,
        ])?;
        Ok(())
    }

    /// Insert an amateur record.
    fn insert_amateur(conn: &Connection, am: &AmateurRecord) -> Result<()> {
        // Convert operator_class char to integer code
        let operator_class_code: Option<u8> = am.operator_class.and_then(|c| {
            c.to_string()
                .parse::<OperatorClass>()
                .ok()
                .map(|o| o.to_u8())
        });
        let prev_operator_class_code: Option<u8> = am.previous_operator_class.and_then(|c| {
            c.to_string()
                .parse::<OperatorClass>()
                .ok()
                .map(|o| o.to_u8())
        });

        let mut stmt = conn.prepare_cached(
            r#"INSERT OR REPLACE INTO amateur_operators (
                unique_system_identifier, uls_file_number, ebf_number, call_sign,
                operator_class, group_code, region_code, trustee_call_sign,
                trustee_indicator, physician_certification, ve_signature,
                systematic_call_sign_change, vanity_call_sign_change,
                vanity_relationship, previous_call_sign, previous_operator_class,
                trustee_name
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)"#,
        )?;
        stmt.execute(params![
            am.unique_system_identifier,
            am.uls_file_num,
            am.ebf_number,
            am.callsign,
            operator_class_code,
            am.group_code.map(|c| c.to_string()),
            am.region_code,
            am.trustee_callsign,
            am.trustee_indicator.map(|c| c.to_string()),
            am.physician_certification.map(|c| c.to_string()),
            am.ve_signature.map(|c| c.to_string()),
            am.systematic_callsign_change.map(|c| c.to_string()),
            am.vanity_callsign_change.map(|c| c.to_string()),
            am.vanity_relationship,
            am.previous_callsign,
            prev_operator_class_code,
            am.trustee_name,
        ])?;
        Ok(())
    }

    /// Insert a history record.
    fn insert_history(conn: &Connection, hs: &HistoryRecord) -> Result<()> {
        let mut stmt = conn.prepare_cached(
            r#"INSERT OR REPLACE INTO history (
                unique_system_identifier, uls_file_number, callsign, log_date, code
            ) VALUES (?1, ?2, ?3, ?4, ?5)"#,
        )?;
        stmt.execute(params![
            hs.unique_system_identifier,
            hs.uls_file_number,
            hs.callsign,
            hs.log_date,
            hs.code,
        ])?;
        Ok(())
    }

    /// Insert a comment record.
    fn insert_comment(conn: &Connection, co: &CommentRecord) -> Result<()> {
        let mut stmt = conn.prepare_cached(
            r#"INSERT OR REPLACE INTO comments (
                unique_system_identifier, uls_file_number, callsign, comment_date,
                description, status_code, status_date
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
        )?;
        stmt.execute(params![
            co.unique_system_identifier,
            co.uls_file_num,
            co.callsign,
            co.comment_date,
            co.description,
            co.status_code.map(|c| c.to_string()),
            co.status_date,
        ])?;
        Ok(())
    }

    /// Insert a special condition record.
    fn insert_special_condition(conn: &Connection, sc: &SpecialConditionRecord) -> Result<()> {
        let mut stmt = conn.prepare_cached(
            r#"INSERT OR REPLACE INTO special_conditions (
                unique_system_identifier, uls_file_number, ebf_number, callsign,
                special_condition_type, special_condition_code, status_code, status_date
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
        )?;
        stmt.execute(params![
            sc.unique_system_identifier,
            sc.uls_file_number,
            sc.ebf_number,
            sc.callsign,
            sc.special_condition_type.map(|c| c.to_string()),
            sc.special_condition_code,
            sc.status_code.map(|c| c.to_string()),
            sc.status_date,
        ])?;
        Ok(())
    }

    /// Look up a license by call sign.
    pub fn get_license_by_callsign(&self, callsign: &str) -> Result<Option<License>> {
        let conn = self.conn()?;
        let callsign = callsign.to_uppercase();

        let result = conn.query_row(
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
            WHERE l.call_sign = ?1
            ORDER BY l.license_status ASC, l.grant_date DESC
            LIMIT 1
            "#,
            [&callsign],
            |row| {
                // Use centralized enum adapter helpers
                let status = read_license_status(row, 6)?;
                let radio_service = read_radio_service(row, 7)?;
                let operator_class = read_operator_class(row, 17)?;

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
                    operator_class,
                })
            },
        );

        match result {
            Ok(license) => Ok(Some(license)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Look up all licenses by FRN (FCC Registration Number).
    pub fn get_licenses_by_frn(&self, frn: &str) -> Result<Vec<License>> {
        let conn = self.conn()?;
        // Normalize FRN - strip leading zeros for comparison or pad to 10 digits
        let frn = frn.trim();

        let mut stmt = conn.prepare(
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
            INNER JOIN entities e ON l.unique_system_identifier = e.unique_system_identifier
            LEFT JOIN amateur_operators a ON l.unique_system_identifier = a.unique_system_identifier
            WHERE e.frn = ?1
            GROUP BY l.unique_system_identifier
            ORDER BY l.radio_service_code, l.call_sign
            "#,
        )?;

        let licenses = stmt.query_map([frn], |row| {
            // Use centralized enum adapter helpers
            let status = read_license_status(row, 6)?;
            let radio_service = read_radio_service(row, 7)?;
            let operator_class = read_operator_class(row, 17)?;

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
                operator_class,
            })
        })?;

        let mut result = Vec::new();
        for license in licenses {
            result.push(license?);
        }
        Ok(result)
    }

    /// Get database statistics.
    pub fn get_stats(&self) -> Result<LicenseStats> {
        let conn = self.conn()?;

        let total_licenses: u64 = conn.query_row("SELECT COUNT(*) FROM licenses", [], |row| {
            row.get::<_, i64>(0)
        })? as u64;

        // Use integer codes for status comparisons
        let active_code = LicenseStatus::Active.to_u8();
        let expired_code = LicenseStatus::Expired.to_u8();
        let cancelled_code = LicenseStatus::Cancelled.to_u8();

        let active_licenses: u64 = conn.query_row(
            "SELECT COUNT(*) FROM licenses WHERE license_status = ?1",
            [active_code],
            |row| row.get::<_, i64>(0),
        )? as u64;

        let expired_licenses: u64 = conn.query_row(
            "SELECT COUNT(*) FROM licenses WHERE license_status = ?1",
            [expired_code],
            |row| row.get::<_, i64>(0),
        )? as u64;

        let cancelled_licenses: u64 = conn.query_row(
            "SELECT COUNT(*) FROM licenses WHERE license_status = ?1",
            [cancelled_code],
            |row| row.get::<_, i64>(0),
        )? as u64;

        let schema_version = Schema::get_version(&conn)?.unwrap_or(0);
        let last_updated = Schema::get_metadata(&conn, "last_updated")?;

        Ok(LicenseStats {
            total_licenses,
            active_licenses,
            expired_licenses,
            cancelled_licenses,
            by_service: Vec::new(),
            by_operator_class: Vec::new(),
            last_updated,
            schema_version,
        })
    }

    /// Count licenses by radio service code(s).
    /// Pass service codes like ["HA", "HV"] for amateur or ["ZA"] for GMRS.
    pub fn count_by_service(&self, service_codes: &[&str]) -> Result<u64> {
        if service_codes.is_empty() {
            return Ok(0);
        }

        // Convert string service codes to integer codes
        let int_codes: Vec<u8> = service_codes
            .iter()
            .filter_map(|s| s.parse::<RadioService>().ok().map(|r| r.to_u8()))
            .collect();

        if int_codes.is_empty() {
            return Ok(0);
        }

        let conn = self.conn()?;
        let placeholders: String = int_codes.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT COUNT(*) FROM licenses WHERE radio_service_code IN ({})",
            placeholders
        );

        let mut stmt = conn.prepare(&sql)?;
        let count: u64 = stmt.query_row(rusqlite::params_from_iter(int_codes.iter()), |row| {
            row.get::<_, i64>(0)
        })? as u64;

        Ok(count)
    }

    /// Set the last updated timestamp.
    pub fn set_last_updated(&self, timestamp: &str) -> Result<()> {
        let conn = self.conn()?;
        Schema::set_metadata(&conn, "last_updated", timestamp)?;
        Ok(())
    }

    /// Get the ETag of the last imported file for a service.
    pub fn get_imported_etag(&self, service: &str) -> Result<Option<String>> {
        let conn = self.conn()?;
        let key = format!("imported_etag_{}", service);
        Schema::get_metadata(&conn, &key)
    }

    /// Set the ETag of the last imported file for a service.
    pub fn set_imported_etag(&self, service: &str, etag: &str) -> Result<()> {
        let conn = self.conn()?;
        let key = format!("imported_etag_{}", service);
        Schema::set_metadata(&conn, &key, etag)?;
        Ok(())
    }

    // ========================================================================
    // Import Status Tracking
    // ========================================================================

    /// Check if a record type has been imported for a service.
    pub fn has_record_type(&self, service: &str, record_type: &str) -> Result<bool> {
        let conn = self.conn()?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM import_status WHERE radio_service_code = ?1 AND record_type = ?2",
            params![service, record_type],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Get list of imported record types for a service.
    pub fn get_imported_types(&self, service: &str) -> Result<Vec<String>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT record_type FROM import_status WHERE radio_service_code = ?1 ORDER BY record_type"
        )?;
        let iter = stmt.query_map(params![service], |row| row.get(0))?;
        let mut types = Vec::new();
        for record_type in iter {
            types.push(record_type?);
        }
        Ok(types)
    }

    /// Mark record type as imported for a service.
    pub fn mark_imported(&self, service: &str, record_type: &str, count: usize) -> Result<()> {
        let conn = self.conn()?;
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT OR REPLACE INTO import_status (radio_service_code, record_type, imported_at, record_count) 
             VALUES (?1, ?2, ?3, ?4)",
            params![service, record_type, now, count as i64],
        )?;
        Ok(())
    }

    /// Clear import status for a service (used when doing full re-import).
    pub fn clear_import_status(&self, service: &str) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "DELETE FROM import_status WHERE radio_service_code = ?1",
            params![service],
        )?;
        Ok(())
    }

    /// Get record count for an imported record type.
    pub fn get_imported_count(&self, service: &str, record_type: &str) -> Result<Option<usize>> {
        let conn = self.conn()?;
        let result: rusqlite::Result<i64> = conn.query_row(
            "SELECT record_count FROM import_status WHERE radio_service_code = ?1 AND record_type = ?2",
            params![service, record_type],
            |row| row.get(0),
        );
        match result {
            Ok(count) => Ok(Some(count as usize)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    // ========================================================================
    // Data Freshness and Patch Tracking
    // ========================================================================

    /// Get the last updated timestamp for a service.
    pub fn get_last_updated(&self) -> Result<Option<String>> {
        let conn = self.conn()?;
        Schema::get_metadata(&conn, "last_updated")
    }

    /// Get data freshness information for a service.
    pub fn get_freshness(
        &self,
        service: &str,
        threshold_days: i64,
    ) -> Result<crate::freshness::DataFreshness> {
        let last_updated = self.get_last_updated()?;
        let mut freshness = crate::freshness::DataFreshness::from_timestamp(
            service,
            last_updated.as_deref(),
            threshold_days,
        );

        // Get last weekly date from metadata
        let weekly_key = format!("last_weekly_date_{}", service);
        let conn = self.conn()?;
        if let Some(date_str) = Schema::get_metadata(&conn, &weekly_key)? {
            freshness.last_weekly_date =
                chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok();
        }

        // Get applied patches
        freshness.applied_patch_dates = self
            .get_applied_patches(service)?
            .into_iter()
            .map(|p| p.patch_date)
            .collect();

        Ok(freshness)
    }

    /// Check if data for a service is stale.
    pub fn is_stale(&self, service: &str, threshold_days: i64) -> Result<bool> {
        let freshness = self.get_freshness(service, threshold_days)?;
        Ok(freshness.is_stale)
    }

    /// Record that a daily patch has been applied.
    pub fn record_applied_patch(
        &self,
        service: &str,
        patch_date: chrono::NaiveDate,
        weekday: &str,
        etag: Option<&str>,
        record_count: Option<usize>,
    ) -> Result<()> {
        let conn = self.conn()?;
        let now = chrono::Utc::now().to_rfc3339();
        let date_str = patch_date.format("%Y-%m-%d").to_string();

        conn.execute(
            "INSERT OR REPLACE INTO applied_patches 
             (radio_service_code, patch_date, patch_weekday, applied_at, etag, record_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                service,
                date_str,
                weekday,
                now,
                etag,
                record_count.map(|c| c as i64)
            ],
        )?;
        Ok(())
    }

    /// Get all applied patches for a service since last weekly.
    pub fn get_applied_patches(
        &self,
        service: &str,
    ) -> Result<Vec<crate::freshness::AppliedPatch>> {
        let conn = self.conn()?;
        let mut stmt = conn.prepare(
            "SELECT radio_service_code, patch_date, patch_weekday, applied_at, etag, record_count 
             FROM applied_patches 
             WHERE radio_service_code = ?1 
             ORDER BY patch_date",
        )?;

        let iter = stmt.query_map(params![service], |row| {
            let date_str: String = row.get(1)?;
            let applied_at_str: String = row.get(3)?;

            Ok(crate::freshness::AppliedPatch {
                service: row.get(0)?,
                patch_date: chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap()),
                weekday: row.get(2)?,
                applied_at: chrono::DateTime::parse_from_rfc3339(&applied_at_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now()),
                etag: row.get(4)?,
                record_count: row.get::<_, Option<i64>>(5)?.map(|c| c as usize),
            })
        })?;

        let mut patches = Vec::new();
        for patch in iter {
            patches.push(patch?);
        }
        Ok(patches)
    }

    /// Clear applied patches for a service (called when new weekly is imported).
    pub fn clear_applied_patches(&self, service: &str) -> Result<()> {
        let conn = self.conn()?;
        conn.execute(
            "DELETE FROM applied_patches WHERE radio_service_code = ?1",
            params![service],
        )?;
        Ok(())
    }

    /// Set the date of the last weekly import for a service.
    pub fn set_last_weekly_date(&self, service: &str, date: chrono::NaiveDate) -> Result<()> {
        let conn = self.conn()?;
        let key = format!("last_weekly_date_{}", service);
        let date_str = date.format("%Y-%m-%d").to_string();
        Schema::set_metadata(&conn, &key, &date_str)?;
        Ok(())
    }

    /// Get the date of the last weekly import for a service.
    pub fn get_last_weekly_date(&self, service: &str) -> Result<Option<chrono::NaiveDate>> {
        let conn = self.conn()?;
        let key = format!("last_weekly_date_{}", service);
        if let Some(date_str) = Schema::get_metadata(&conn, &key)? {
            Ok(chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").ok())
        } else {
            Ok(None)
        }
    }
}

/// A database transaction for bulk operations.
pub struct Transaction {
    conn: PooledConnection<SqliteConnectionManager>,
}

impl Transaction {
    /// Insert a record within this transaction.
    pub fn insert_record(&self, record: &UlsRecord) -> Result<()> {
        Database::insert_record_conn(&self.conn, record)
    }

    /// Commit the transaction.
    pub fn commit(self) -> Result<()> {
        self.conn.execute("COMMIT", [])?;
        Ok(())
    }

    /// Rollback the transaction.
    pub fn rollback(self) -> Result<()> {
        self.conn.execute("ROLLBACK", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uls_core::records::HeaderRecord;

    fn create_test_db() -> Database {
        let config = DatabaseConfig::in_memory();
        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();
        db
    }

    fn create_test_header() -> HeaderRecord {
        let mut hd = HeaderRecord::from_fields(&["HD", "12345"]);
        hd.unique_system_identifier = 12345;
        hd.call_sign = Some("W1TEST".to_string());
        hd.license_status = Some('A');
        hd.radio_service_code = Some("HA".to_string());
        hd
    }

    #[test]
    fn test_open_database() {
        let db = create_test_db();
        assert!(db.is_initialized().unwrap());
    }

    #[test]
    fn test_insert_and_query() {
        let db = create_test_db();

        let header = create_test_header();
        db.insert_record(&UlsRecord::Header(header)).unwrap();

        let license = db.get_license_by_callsign("W1TEST").unwrap();
        assert!(license.is_some());

        let license = license.unwrap();
        assert_eq!(license.call_sign, "W1TEST");
        assert_eq!(license.status, 'A');
        assert!(license.is_active());
    }

    #[test]
    fn test_case_insensitive_lookup() {
        let db = create_test_db();

        let header = create_test_header();
        db.insert_record(&UlsRecord::Header(header)).unwrap();

        // Should find with different case
        let license = db.get_license_by_callsign("w1test").unwrap();
        assert!(license.is_some());
    }

    #[test]
    fn test_stats() {
        let db = create_test_db();

        let header = create_test_header();
        db.insert_record(&UlsRecord::Header(header)).unwrap();

        let stats = db.get_stats().unwrap();
        assert_eq!(stats.total_licenses, 1);
        assert_eq!(stats.active_licenses, 1);
    }

    #[test]
    fn test_transaction() {
        let db = create_test_db();

        let tx = db.begin_transaction().unwrap();

        let header = create_test_header();
        tx.insert_record(&UlsRecord::Header(header)).unwrap();
        tx.commit().unwrap();

        let license = db.get_license_by_callsign("W1TEST").unwrap();
        assert!(license.is_some());
    }

    #[test]
    fn test_insert_entity() {
        use uls_core::records::EntityRecord;

        let db = create_test_db();

        // Insert header first for FK constraint
        let header = create_test_header();
        db.insert_record(&UlsRecord::Header(header)).unwrap();

        // Insert entity
        let entity = EntityRecord::from_fields(&[
            "EN",
            "12345",
            "",
            "",
            "W1TEST",
            "L",
            "L00100001",
            "DOE, JOHN A",
            "JOHN",
            "A",
            "DOE",
            "",
            "555-555-1234",
            "",
            "test@example.com",
            "123 Main St",
            "ANYTOWN",
            "CA",
            "90210",
            "",
            "",
            "000",
            "0001234567",
            "I",
            "",
            "",
            "",
            "",
            "",
            "",
        ]);
        db.insert_record(&UlsRecord::Entity(entity)).unwrap();
    }

    #[test]
    fn test_insert_amateur() {
        use uls_core::records::AmateurRecord;

        let db = create_test_db();

        let header = create_test_header();
        db.insert_record(&UlsRecord::Header(header)).unwrap();

        let amateur = AmateurRecord::from_fields(&[
            "AM", "12345", "", "", "W1TEST", "E", "D", "6", "", "", "", "", "", "", "", "", "", "",
        ]);
        db.insert_record(&UlsRecord::Amateur(amateur)).unwrap();
    }

    #[test]
    fn test_insert_history() {
        use uls_core::records::HistoryRecord;

        let db = create_test_db();

        let header = create_test_header();
        db.insert_record(&UlsRecord::Header(header)).unwrap();

        let history =
            HistoryRecord::from_fields(&["HS", "12345", "", "W1TEST", "01/01/2020", "LIISS"]);
        db.insert_record(&UlsRecord::History(history)).unwrap();
    }

    #[test]
    fn test_insert_comment() {
        use uls_core::records::CommentRecord;

        let db = create_test_db();

        let header = create_test_header();
        db.insert_record(&UlsRecord::Header(header)).unwrap();

        let comment = CommentRecord::from_fields(&[
            "CO",
            "12345",
            "",
            "W1TEST",
            "01/01/2020",
            "Test comment",
        ]);
        db.insert_record(&UlsRecord::Comment(comment)).unwrap();
    }

    #[test]
    fn test_insert_special_condition() {
        use uls_core::records::SpecialConditionRecord;

        let db = create_test_db();

        let header = create_test_header();
        db.insert_record(&UlsRecord::Header(header)).unwrap();

        let sc = SpecialConditionRecord::from_fields(&[
            "SC", "12345", "", "", "W1TEST", "P", "999", "", "",
        ]);
        db.insert_record(&UlsRecord::SpecialCondition(sc)).unwrap();
    }

    #[test]
    fn test_get_licenses_by_frn() {
        use uls_core::records::EntityRecord;

        let db = create_test_db();

        let header = create_test_header();
        db.insert_record(&UlsRecord::Header(header)).unwrap();

        // Insert entity with FRN
        let entity = EntityRecord::from_fields(&[
            "EN",
            "12345",
            "",
            "",
            "W1TEST",
            "L",
            "L00100001",
            "DOE, JOHN A",
            "JOHN",
            "A",
            "DOE",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "000",
            "0001234567",
            "I",
            "",
            "",
            "",
            "",
            "",
            "",
        ]);
        db.insert_record(&UlsRecord::Entity(entity)).unwrap();

        let licenses = db.get_licenses_by_frn("0001234567").unwrap();
        assert_eq!(licenses.len(), 1);
        assert_eq!(licenses[0].call_sign, "W1TEST");
    }

    #[test]
    fn test_get_licenses_by_frn_not_found() {
        let db = create_test_db();

        let licenses = db.get_licenses_by_frn("9999999999").unwrap();
        assert!(licenses.is_empty());
    }

    #[test]
    fn test_lookup_prefers_active_over_cancelled() {
        let db = create_test_db();

        // Insert a cancelled license first (lower USI = inserted first)
        let mut cancelled = HeaderRecord::from_fields(&["HD", "10001"]);
        cancelled.unique_system_identifier = 10001;
        cancelled.call_sign = Some("K2QA".to_string());
        cancelled.license_status = Some('C');
        cancelled.radio_service_code = Some("HA".to_string());
        db.insert_record(&UlsRecord::Header(cancelled)).unwrap();

        // Insert an active license with the same callsign (higher USI)
        let mut active = HeaderRecord::from_fields(&["HD", "20002"]);
        active.unique_system_identifier = 20002;
        active.call_sign = Some("K2QA".to_string());
        active.license_status = Some('A');
        active.radio_service_code = Some("HA".to_string());
        db.insert_record(&UlsRecord::Header(active)).unwrap();

        // Lookup should return the active record, not the cancelled one
        let license = db.get_license_by_callsign("K2QA").unwrap();
        assert!(license.is_some(), "Should find license for K2QA");
        let license = license.unwrap();
        assert_eq!(
            license.status, 'A',
            "Should return active license, not cancelled (got status='{}')",
            license.status
        );
        assert_eq!(license.unique_system_identifier, 20002);
    }

    #[test]
    fn test_lookup_returns_cancelled_when_no_active() {
        let db = create_test_db();

        // Insert only a cancelled license
        let mut cancelled = HeaderRecord::from_fields(&["HD", "10001"]);
        cancelled.unique_system_identifier = 10001;
        cancelled.call_sign = Some("W9OLD".to_string());
        cancelled.license_status = Some('C');
        cancelled.radio_service_code = Some("HA".to_string());
        db.insert_record(&UlsRecord::Header(cancelled)).unwrap();

        // Should still return the cancelled record when it's the only one
        let license = db.get_license_by_callsign("W9OLD").unwrap();
        assert!(license.is_some(), "Should find cancelled-only license");
        assert_eq!(license.unwrap().status, 'C');
    }

    #[test]
    fn test_lookup_prefers_most_recent_inactive_record() {
        let db = create_test_db();

        // Insert an older expired license (granted 2015)
        let mut older = HeaderRecord::from_fields(&["HD", "10001"]);
        older.unique_system_identifier = 10001;
        older.call_sign = Some("W3OLD".to_string());
        older.license_status = Some('E');
        older.radio_service_code = Some("HA".to_string());
        older.grant_date = Some(chrono::NaiveDate::from_ymd_opt(2015, 3, 1).unwrap());
        older.expired_date = Some(chrono::NaiveDate::from_ymd_opt(2025, 3, 1).unwrap());
        db.insert_record(&UlsRecord::Header(older)).unwrap();

        // Insert a newer expired license (granted 2020)
        let mut newer = HeaderRecord::from_fields(&["HD", "20002"]);
        newer.unique_system_identifier = 20002;
        newer.call_sign = Some("W3OLD".to_string());
        newer.license_status = Some('E');
        newer.radio_service_code = Some("HA".to_string());
        newer.grant_date = Some(chrono::NaiveDate::from_ymd_opt(2020, 6, 15).unwrap());
        newer.expired_date = Some(chrono::NaiveDate::from_ymd_opt(2030, 6, 15).unwrap());
        db.insert_record(&UlsRecord::Header(newer)).unwrap();

        // Should return the more recently granted record
        let license = db.get_license_by_callsign("W3OLD").unwrap();
        assert!(license.is_some(), "Should find expired license for W3OLD");
        let license = license.unwrap();
        assert_eq!(
            license.unique_system_identifier, 20002,
            "Should return the most recently granted expired record"
        );
        assert_eq!(license.status, 'E');
    }

    #[test]
    fn test_count_by_service() {
        let db = create_test_db();

        let header = create_test_header(); // Has radio_service_code = "HA"
        db.insert_record(&UlsRecord::Header(header)).unwrap();

        let count = db.count_by_service(&["HA"]).unwrap();
        assert_eq!(count, 1);

        let count = db.count_by_service(&["ZA"]).unwrap(); // GMRS, shouldn't match
        assert_eq!(count, 0);
    }

    #[test]
    fn test_etag_operations() {
        let db = create_test_db();

        // Initially no etag
        let etag = db.get_imported_etag("l_amat").unwrap();
        assert!(etag.is_none());

        // Set etag
        db.set_imported_etag("l_amat", "abc123").unwrap();

        // Should retrieve it
        let etag = db.get_imported_etag("l_amat").unwrap();
        assert_eq!(etag, Some("abc123".to_string()));

        // Update etag
        db.set_imported_etag("l_amat", "xyz789").unwrap();
        let etag = db.get_imported_etag("l_amat").unwrap();
        assert_eq!(etag, Some("xyz789".to_string()));
    }

    #[test]
    fn test_set_last_updated() {
        let db = create_test_db();

        db.set_last_updated("2025-01-17T12:00:00Z").unwrap();
        // Just verify it doesn't error - metadata retrieval would need Schema::get_metadata
    }

    #[test]
    fn test_license_not_found() {
        let db = create_test_db();

        let license = db.get_license_by_callsign("NOTEXIST").unwrap();
        assert!(license.is_none());
    }

    #[test]
    fn test_transaction_rollback() {
        let db = create_test_db();

        let tx = db.begin_transaction().unwrap();
        let header = create_test_header();
        tx.insert_record(&UlsRecord::Header(header)).unwrap();
        tx.rollback().unwrap();

        // Should not be found after rollback
        let license = db.get_license_by_callsign("W1TEST").unwrap();
        assert!(license.is_none());
    }

    #[test]
    fn test_open_database_with_path() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("subdir").join("test.db");

        // Database::open should work and create parent directory
        let db = Database::open(&db_path).unwrap();
        db.initialize().unwrap();
        assert!(db.is_initialized().unwrap());
        assert!(db_path.parent().unwrap().exists());
    }

    #[test]
    fn test_insert_unsupported_record_type() {
        use uls_core::records::LocationRecord;

        let db = create_test_db();

        // Location is not supported in repository insert_record
        let location = LocationRecord::from_fields(&["LO", "12345", "", "", "W1TEST"]);
        // Should not error, just skip
        db.insert_record(&UlsRecord::Location(location)).unwrap();
    }

    #[test]
    fn test_count_by_service_empty() {
        let db = create_test_db();

        // Empty service codes should return 0
        let count = db.count_by_service(&[]).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_get_imported_types() {
        let db = create_test_db();

        // Initially empty
        let types = db.get_imported_types("HA").unwrap();
        assert!(types.is_empty());

        // Mark some record types as imported
        db.mark_imported("HA", "HD", 100).unwrap();
        db.mark_imported("HA", "EN", 50).unwrap();
        db.mark_imported("HA", "AM", 25).unwrap();

        // Should retrieve them sorted
        let types = db.get_imported_types("HA").unwrap();
        assert_eq!(types, vec!["AM", "EN", "HD"]); // Sorted alphabetically

        // Different service should be empty
        let types = db.get_imported_types("ZA").unwrap();
        assert!(types.is_empty());
    }

    #[test]
    fn test_get_imported_count() {
        let db = create_test_db();

        // Initially not found
        let count = db.get_imported_count("HA", "HD").unwrap();
        assert!(count.is_none());

        // Mark as imported with count
        db.mark_imported("HA", "HD", 500).unwrap();

        // Should retrieve the count
        let count = db.get_imported_count("HA", "HD").unwrap();
        assert_eq!(count, Some(500));

        // Non-existent record type should return None
        let count = db.get_imported_count("HA", "XX").unwrap();
        assert!(count.is_none());

        // Non-existent service should return None
        let count = db.get_imported_count("ZZ", "HD").unwrap();
        assert!(count.is_none());
    }

    #[test]
    fn test_import_status_lifecycle() {
        let db = create_test_db();

        // Mark several types as imported
        db.mark_imported("HA", "HD", 100).unwrap();
        db.mark_imported("HA", "EN", 200).unwrap();

        // Verify they're tracked
        assert!(db.has_record_type("HA", "HD").unwrap());
        assert!(db.has_record_type("HA", "EN").unwrap());
        assert!(!db.has_record_type("HA", "AM").unwrap());

        // Clear import status
        db.clear_import_status("HA").unwrap();

        // All should be cleared
        assert!(!db.has_record_type("HA", "HD").unwrap());
        assert!(!db.has_record_type("HA", "EN").unwrap());

        // get_imported_types should return empty
        let types = db.get_imported_types("HA").unwrap();
        assert!(types.is_empty());
    }

    #[test]
    fn test_pool_pragma_settings_on_all_connections() {
        // Use a path-based database with multiple connections to test pool behavior
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_pool.db");

        let config = crate::config::DatabaseConfig {
            path: db_path.clone(),
            max_connections: 3,
            foreign_keys: true,
            enable_wal: true,
            ..Default::default()
        };

        let db = Database::with_config(config).unwrap();
        db.initialize().unwrap();

        // Get multiple connections and verify each has foreign_keys enabled
        let mut connections = Vec::new();
        for i in 0..3 {
            let conn = db.conn().unwrap();
            let fk_enabled: i32 = conn
                .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
                .unwrap();
            assert_eq!(fk_enabled, 1, "Connection {i} should have foreign_keys ON");
            connections.push(conn);
        }

        // Return connections and get new ones to verify they still work
        drop(connections);

        for i in 0..2 {
            let conn = db.conn().unwrap();
            let fk_enabled: i32 = conn
                .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
                .unwrap();
            assert_eq!(
                fk_enabled, 1,
                "Re-acquired connection {i} should have foreign_keys ON"
            );
        }
    }
}
