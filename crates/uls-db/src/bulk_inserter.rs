//! Bulk inserter with prepared statements for high-performance imports.

use rusqlite::{params, Connection, Statement};
use tracing::warn;

use uls_core::records::{
    AmateurRecord, CommentRecord, EntityRecord, HeaderRecord, HistoryRecord,
    SpecialConditionRecord, UlsRecord,
};

use crate::Result;

/// SQL statements for bulk insert operations.
const SQL_INSERT_HEADER: &str = r#"INSERT OR REPLACE INTO licenses (
    unique_system_identifier, uls_file_number, ebf_number, call_sign,
    license_status, radio_service_code, grant_date, expired_date,
    cancellation_date, effective_date, last_action_date
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#;

const SQL_INSERT_ENTITY: &str = r#"INSERT OR REPLACE INTO entities (
    unique_system_identifier, uls_file_number, ebf_number, call_sign,
    entity_type, licensee_id, entity_name, first_name, middle_initial,
    last_name, suffix, phone, fax, email, street_address, city, state,
    zip_code, po_box, attention_line, sgin, frn, applicant_type_code,
    status_code, status_date
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25)"#;

const SQL_INSERT_AMATEUR: &str = r#"INSERT OR REPLACE INTO amateur_operators (
    unique_system_identifier, uls_file_number, ebf_number, call_sign,
    operator_class, group_code, region_code, trustee_call_sign,
    trustee_indicator, physician_certification, ve_signature,
    systematic_call_sign_change, vanity_call_sign_change,
    vanity_relationship, previous_call_sign, previous_operator_class,
    trustee_name
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)"#;

const SQL_INSERT_HISTORY: &str = r#"INSERT OR REPLACE INTO history (
    unique_system_identifier, uls_file_number, callsign, log_date, code
) VALUES (?1, ?2, ?3, ?4, ?5)"#;

const SQL_INSERT_COMMENT: &str = r#"INSERT OR REPLACE INTO comments (
    unique_system_identifier, uls_file_number, callsign, comment_date,
    description, status_code, status_date
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#;

const SQL_INSERT_SPECIAL_CONDITION: &str = r#"INSERT OR REPLACE INTO special_conditions (
    unique_system_identifier, uls_file_number, ebf_number, callsign,
    special_condition_type, special_condition_code, status_code, status_date
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#;

/// Bulk inserter with pre-prepared statements for maximum performance.
/// 
/// Statements are prepared once when the inserter is created, then reused
/// for every insert operation - eliminating SQL parsing overhead.
pub struct BulkInserter<'conn> {
    stmt_header: Statement<'conn>,
    stmt_entity: Statement<'conn>,
    stmt_amateur: Statement<'conn>,
    stmt_history: Statement<'conn>,
    stmt_comment: Statement<'conn>,
    stmt_special_condition: Statement<'conn>,
}

impl<'conn> BulkInserter<'conn> {
    /// Create a new bulk inserter with prepared statements.
    pub fn new(conn: &'conn Connection) -> Result<Self> {
        Ok(Self {
            stmt_header: conn.prepare(SQL_INSERT_HEADER)?,
            stmt_entity: conn.prepare(SQL_INSERT_ENTITY)?,
            stmt_amateur: conn.prepare(SQL_INSERT_AMATEUR)?,
            stmt_history: conn.prepare(SQL_INSERT_HISTORY)?,
            stmt_comment: conn.prepare(SQL_INSERT_COMMENT)?,
            stmt_special_condition: conn.prepare(SQL_INSERT_SPECIAL_CONDITION)?,
        })
    }

    /// Insert a record using the appropriate prepared statement.
    pub fn insert(&mut self, record: &UlsRecord) -> Result<()> {
        match record {
            UlsRecord::Header(hd) => self.insert_header(hd),
            UlsRecord::Entity(en) => self.insert_entity(en),
            UlsRecord::Amateur(am) => self.insert_amateur(am),
            UlsRecord::History(hs) => self.insert_history(hs),
            UlsRecord::Comment(co) => self.insert_comment(co),
            UlsRecord::SpecialCondition(sc) => self.insert_special_condition(sc),
            _ => Ok(()), // Skip unsupported record types silently
        }
    }

    fn insert_header(&mut self, hd: &HeaderRecord) -> Result<()> {
        self.stmt_header.execute(params![
            hd.unique_system_identifier,
            hd.uls_file_number,
            hd.ebf_number,
            hd.call_sign,
            hd.license_status.map(|c| c.to_string()),
            hd.radio_service_code,
            hd.grant_date.map(|d| d.to_string()),
            hd.expired_date.map(|d| d.to_string()),
            hd.cancellation_date.map(|d| d.to_string()),
            hd.effective_date.map(|d| d.to_string()),
            hd.last_action_date.map(|d| d.to_string()),
        ])?;
        Ok(())
    }

    fn insert_entity(&mut self, en: &EntityRecord) -> Result<()> {
        self.stmt_entity.execute(params![
            en.unique_system_identifier,
            en.uls_file_number,
            en.ebf_number,
            en.call_sign,
            en.entity_type,
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

    fn insert_amateur(&mut self, am: &AmateurRecord) -> Result<()> {
        self.stmt_amateur.execute(params![
            am.unique_system_identifier,
            am.uls_file_num,
            am.ebf_number,
            am.callsign,
            am.operator_class.map(|c| c.to_string()),
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
            am.previous_operator_class.map(|c| c.to_string()),
            am.trustee_name,
        ])?;
        Ok(())
    }

    fn insert_history(&mut self, hs: &HistoryRecord) -> Result<()> {
        self.stmt_history.execute(params![
            hs.unique_system_identifier,
            hs.uls_file_number,
            hs.callsign,
            hs.log_date,
            hs.code,
        ])?;
        Ok(())
    }

    fn insert_comment(&mut self, co: &CommentRecord) -> Result<()> {
        self.stmt_comment.execute(params![
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

    fn insert_special_condition(&mut self, sc: &SpecialConditionRecord) -> Result<()> {
        self.stmt_special_condition.execute(params![
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
}
