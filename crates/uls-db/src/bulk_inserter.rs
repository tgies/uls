//! Bulk inserter with prepared statements for high-performance imports.

use rusqlite::{params, Connection, Statement};

use uls_core::codes::{EntityType, LicenseStatus, OperatorClass, RadioService};
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

        self.stmt_header.execute(params![
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

    fn insert_entity(&mut self, en: &EntityRecord) -> Result<()> {
        // Convert entity_type string to integer code
        let entity_type_code: Option<u8> = en
            .entity_type
            .as_ref()
            .and_then(|s| s.parse::<EntityType>().ok().map(|e| e.to_u8()));

        self.stmt_entity.execute(params![
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

    fn insert_amateur(&mut self, am: &AmateurRecord) -> Result<()> {
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

        self.stmt_amateur.execute(params![
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::Schema;
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        Schema::initialize(&conn).unwrap();
        conn
    }

    // Test fixtures using fake data but realistic FCC record structure.
    // All records share USI=100001 for foreign key consistency.
    const TEST_USI: &str = "100001";
    const TEST_CALLSIGN: &str = "W1TEST";

    fn create_header() -> HeaderRecord {
        // Structure matches: HD|USI|ULS_FILE|EBF|CALLSIGN|STATUS|SERVICE|GRANT|EXPIRE|...
        HeaderRecord::from_fields(&[
            "HD",
            TEST_USI,
            "0000000001",
            "",
            TEST_CALLSIGN,
            "A",
            "HA",
            "01/15/2020",
            "01/15/2030",
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
            "",
            "N",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "",
            "01/15/2020",
            "01/15/2020",
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
            "",
            "",
            "",
        ])
    }

    fn create_entity() -> EntityRecord {
        // Structure matches: EN|USI|ULS_FILE|EBF|CALLSIGN|TYPE|LIC_ID|NAME|FIRST|MI|LAST|...
        EntityRecord::from_fields(&[
            "EN",
            TEST_USI,
            "",
            "",
            TEST_CALLSIGN,
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
        ])
    }

    fn create_amateur() -> AmateurRecord {
        // Structure matches: AM|USI|ULS_FILE|EBF|CALLSIGN|CLASS|GROUP|REGION|...
        AmateurRecord::from_fields(&[
            "AM",
            TEST_USI,
            "",
            "",
            TEST_CALLSIGN,
            "E",
            "D",
            "6",
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
        ])
    }

    fn create_history() -> HistoryRecord {
        // Structure matches: HS|USI|ULS_FILE|CALLSIGN|DATE|CODE
        HistoryRecord::from_fields(&["HS", TEST_USI, "", TEST_CALLSIGN, "01/15/2020", "LIISS"])
    }

    fn create_comment() -> CommentRecord {
        // Structure matches: CO|USI|ULS_FILE|CALLSIGN|DATE|DESCRIPTION|STATUS|STATUS_DATE
        CommentRecord::from_fields(&[
            "CO",
            TEST_USI,
            "",
            TEST_CALLSIGN,
            "01/15/2020",
            "Test comment for unit testing purposes.",
            "",
            "",
        ])
    }

    fn create_special_condition() -> SpecialConditionRecord {
        // Structure matches: SC|USI|ULS_FILE|EBF|CALLSIGN|TYPE|CODE|STATUS|STATUS_DATE
        SpecialConditionRecord::from_fields(&[
            "SC",
            TEST_USI,
            "",
            "",
            TEST_CALLSIGN,
            "P",
            "999",
            "",
            "",
        ])
    }

    /// Helper to insert the parent header record for foreign key constraints
    fn insert_parent_header(inserter: &mut BulkInserter) {
        inserter
            .insert(&UlsRecord::Header(create_header()))
            .unwrap();
    }

    #[test]
    fn test_bulk_inserter_new() {
        let conn = setup_db();
        let inserter = BulkInserter::new(&conn);
        assert!(inserter.is_ok());
    }

    #[test]
    fn test_insert_header() {
        let conn = setup_db();
        let mut inserter = BulkInserter::new(&conn).unwrap();

        let result = inserter.insert(&UlsRecord::Header(create_header()));
        assert!(result.is_ok());

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM licenses WHERE call_sign = ?",
                [TEST_CALLSIGN],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_entity() {
        let conn = setup_db();
        let mut inserter = BulkInserter::new(&conn).unwrap();

        // Insert parent header first for foreign key
        insert_parent_header(&mut inserter);

        let result = inserter.insert(&UlsRecord::Entity(create_entity()));
        assert!(result.is_ok());

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM entities WHERE call_sign = ?",
                [TEST_CALLSIGN],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_amateur() {
        let conn = setup_db();
        let mut inserter = BulkInserter::new(&conn).unwrap();

        insert_parent_header(&mut inserter);

        let result = inserter.insert(&UlsRecord::Amateur(create_amateur()));
        assert!(result.is_ok());

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM amateur_operators WHERE call_sign = ?",
                [TEST_CALLSIGN],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_history() {
        let conn = setup_db();
        let mut inserter = BulkInserter::new(&conn).unwrap();

        insert_parent_header(&mut inserter);

        let result = inserter.insert(&UlsRecord::History(create_history()));
        assert!(result.is_ok());

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM history WHERE callsign = ?",
                [TEST_CALLSIGN],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_comment() {
        let conn = setup_db();
        let mut inserter = BulkInserter::new(&conn).unwrap();

        insert_parent_header(&mut inserter);

        let result = inserter.insert(&UlsRecord::Comment(create_comment()));
        assert!(result.is_ok());

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM comments WHERE callsign = ?",
                [TEST_CALLSIGN],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_special_condition() {
        let conn = setup_db();
        let mut inserter = BulkInserter::new(&conn).unwrap();

        insert_parent_header(&mut inserter);

        let result = inserter.insert(&UlsRecord::SpecialCondition(create_special_condition()));
        assert!(result.is_ok());

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM special_conditions WHERE callsign = ?",
                [TEST_CALLSIGN],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_insert_unsupported_record() {
        use uls_core::codes::RecordType;

        let conn = setup_db();
        let mut inserter = BulkInserter::new(&conn).unwrap();

        // Raw/unsupported records should be skipped silently
        let raw = UlsRecord::Raw {
            record_type: RecordType::AC,
            fields: vec!["AC".to_string(), "123".to_string()],
        };

        let result = inserter.insert(&raw);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bulk_insert_multiple_records() {
        let conn = setup_db();
        let mut inserter = BulkInserter::new(&conn).unwrap();

        // Insert parent first, then all child records
        inserter
            .insert(&UlsRecord::Header(create_header()))
            .unwrap();
        inserter
            .insert(&UlsRecord::Entity(create_entity()))
            .unwrap();
        inserter
            .insert(&UlsRecord::Amateur(create_amateur()))
            .unwrap();
        inserter
            .insert(&UlsRecord::History(create_history()))
            .unwrap();
        inserter
            .insert(&UlsRecord::Comment(create_comment()))
            .unwrap();
        inserter
            .insert(&UlsRecord::SpecialCondition(create_special_condition()))
            .unwrap();

        // Verify all record types were inserted
        let license_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM licenses", [], |r| r.get(0))
            .unwrap();
        let entity_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM entities", [], |r| r.get(0))
            .unwrap();
        let amateur_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM amateur_operators", [], |r| r.get(0))
            .unwrap();
        let history_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM history", [], |r| r.get(0))
            .unwrap();
        let comment_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM comments", [], |r| r.get(0))
            .unwrap();
        let sc_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM special_conditions", [], |r| r.get(0))
            .unwrap();

        assert_eq!(license_count, 1);
        assert_eq!(entity_count, 1);
        assert_eq!(amateur_count, 1);
        assert_eq!(history_count, 1);
        assert_eq!(comment_count, 1);
        assert_eq!(sc_count, 1);
    }

    #[test]
    fn test_insert_replace_behavior() {
        let conn = setup_db();
        let mut inserter = BulkInserter::new(&conn).unwrap();

        // Insert same record twice - should replace, not duplicate
        inserter
            .insert(&UlsRecord::Header(create_header()))
            .unwrap();
        inserter
            .insert(&UlsRecord::Header(create_header()))
            .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM licenses", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1); // Should be 1, not 2
    }
}
