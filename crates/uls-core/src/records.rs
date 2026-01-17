//! ULS record type definitions.
//!
//! This module contains struct definitions for all 89 ULS record types.
//! Each record type corresponds to a table in the FCC ULS database.

pub mod common;
pub mod header;
pub mod entity;
pub mod amateur;
pub mod location;
pub mod frequency;
pub mod antenna;
pub mod emission;
pub mod special_conditions;
pub mod market;
pub mod path;
pub mod ship;
pub mod aircraft;
pub mod coast_ground;
pub mod attachment;
pub mod history;
pub mod buildout;
pub mod control_point;
pub mod transfer;
pub mod lease;

pub use common::*;
pub use header::*;
pub use entity::*;
pub use amateur::*;
pub use location::*;
pub use frequency::*;
pub use antenna::*;
pub use emission::*;
pub use special_conditions::*;
pub use market::*;
pub use path::*;
pub use ship::*;
pub use aircraft::*;
pub use coast_ground::*;
pub use attachment::*;
pub use history::*;
pub use buildout::*;
pub use control_point::*;
pub use transfer::*;
pub use lease::*;

use crate::codes::RecordType;

/// A parsed ULS record from a DAT file.
#[derive(Debug, Clone)]
pub enum UlsRecord {
    /// HD - License/Application Header
    Header(HeaderRecord),
    /// EN - Entity (licensee/applicant)
    Entity(EntityRecord),
    /// AM - Amateur operator data
    Amateur(AmateurRecord),
    /// AD - Application details
    ApplicationDetail(ApplicationDetailRecord),
    /// HS - History/status log
    History(HistoryRecord),
    /// CO - FCC comments
    Comment(CommentRecord),
    /// LO - Location data
    Location(LocationRecord),
    /// FR - Frequency data
    Frequency(FrequencyRecord),
    /// AN - Antenna data
    Antenna(AntennaRecord),
    /// EM - Emission data
    Emission(EmissionRecord),
    /// SC - License level special conditions
    SpecialCondition(SpecialConditionRecord),
    /// SF - License level free-form conditions
    FreeformCondition(FreeformConditionRecord),
    /// VC - Vanity call sign request
    VanityCallSign(VanityCallSignRecord),
    /// AC - Aircraft data
    Aircraft(AircraftRecord),
    /// SH - Ship data
    Ship(ShipRecord),
    /// Raw record for types not yet fully implemented
    Raw {
        record_type: RecordType,
        fields: Vec<String>,
    },
}

impl UlsRecord {
    /// Returns the record type code.
    pub fn record_type(&self) -> RecordType {
        match self {
            Self::Header(_) => RecordType::HD,
            Self::Entity(_) => RecordType::EN,
            Self::Amateur(_) => RecordType::AM,
            Self::ApplicationDetail(_) => RecordType::AD,
            Self::History(_) => RecordType::HS,
            Self::Comment(_) => RecordType::CO,
            Self::Location(_) => RecordType::LO,
            Self::Frequency(_) => RecordType::FR,
            Self::Antenna(_) => RecordType::AN,
            Self::Emission(_) => RecordType::EM,
            Self::SpecialCondition(_) => RecordType::SC,
            Self::FreeformCondition(_) => RecordType::SF,
            Self::VanityCallSign(_) => RecordType::VC,
            Self::Aircraft(_) => RecordType::AC,
            Self::Ship(_) => RecordType::SH,
            Self::Raw { record_type, .. } => *record_type,
        }
    }

    /// Returns the unique system identifier if present.
    pub fn unique_system_identifier(&self) -> Option<i64> {
        match self {
            Self::Header(r) => Some(r.unique_system_identifier),
            Self::Entity(r) => Some(r.unique_system_identifier),
            Self::Amateur(r) => Some(r.unique_system_identifier),
            Self::ApplicationDetail(r) => Some(r.unique_system_identifier),
            Self::History(r) => Some(r.unique_system_identifier),
            Self::Comment(r) => Some(r.unique_system_identifier),
            Self::Location(r) => Some(r.unique_system_identifier),
            Self::Frequency(r) => Some(r.unique_system_identifier),
            Self::Antenna(r) => Some(r.unique_system_identifier),
            Self::Emission(r) => Some(r.unique_system_identifier),
            Self::SpecialCondition(r) => Some(r.unique_system_identifier),
            Self::FreeformCondition(r) => r.unique_system_identifier,
            Self::VanityCallSign(r) => Some(r.unique_system_identifier),
            Self::Aircraft(r) => Some(r.unique_system_identifier),
            Self::Ship(r) => r.unique_system_identifier,
            Self::Raw { .. } => None,
        }
    }

    /// Returns the call sign if present.
    pub fn call_sign(&self) -> Option<&str> {
        match self {
            Self::Header(r) => r.call_sign.as_deref(),
            Self::Entity(r) => r.call_sign.as_deref(),
            Self::Amateur(r) => r.callsign.as_deref(),
            Self::ApplicationDetail(_) => None,
            Self::History(r) => r.callsign.as_deref(),
            Self::Comment(r) => r.callsign.as_deref(),
            Self::Location(r) => r.call_sign.as_deref(),
            Self::Frequency(r) => r.call_sign.as_deref(),
            Self::Antenna(r) => r.call_sign.as_deref(),
            Self::Emission(r) => r.call_sign.as_deref(),
            Self::SpecialCondition(r) => r.callsign.as_deref(),
            Self::FreeformCondition(r) => r.callsign.as_deref(),
            Self::VanityCallSign(_) => None,
            Self::Aircraft(r) => r.call_sign.as_deref(),
            Self::Ship(r) => r.callsign.as_deref(),
            Self::Raw { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create minimal records for testing dispatch methods
    fn header_record() -> UlsRecord {
        UlsRecord::Header(HeaderRecord::from_fields(&[
            "HD", "100001", "", "", "W1TEST", "A", "HA",
        ]))
    }

    fn entity_record() -> UlsRecord {
        UlsRecord::Entity(EntityRecord::from_fields(&[
            "EN", "100002", "", "", "W2TEST", "L",
        ]))
    }

    fn amateur_record() -> UlsRecord {
        UlsRecord::Amateur(AmateurRecord::from_fields(&[
            "AM", "100003", "", "", "W3TEST", "E", "D", "6",
        ]))
    }

    fn history_record() -> UlsRecord {
        UlsRecord::History(HistoryRecord::from_fields(&[
            "HS", "100004", "", "W4TEST", "01/01/2020", "LIISS",
        ]))
    }

    fn comment_record() -> UlsRecord {
        UlsRecord::Comment(CommentRecord::from_fields(&[
            "CO", "100005", "", "W5TEST", "01/01/2020", "Test comment",
        ]))
    }

    fn location_record() -> UlsRecord {
        UlsRecord::Location(LocationRecord::from_fields(&[
            "LO", "100006", "", "", "W6TEST",
        ]))
    }

    fn frequency_record() -> UlsRecord {
        UlsRecord::Frequency(FrequencyRecord::from_fields(&[
            "FR", "100007", "", "", "W7TEST",
        ]))
    }

    fn special_condition_record() -> UlsRecord {
        UlsRecord::SpecialCondition(SpecialConditionRecord::from_fields(&[
            "SC", "100008", "", "", "W8TEST", "P", "999",
        ]))
    }

    fn raw_record() -> UlsRecord {
        UlsRecord::Raw {
            record_type: RecordType::BC,
            fields: vec!["BC".to_string(), "12345".to_string()],
        }
    }

    #[test]
    fn test_record_type_dispatch() {
        assert_eq!(header_record().record_type(), RecordType::HD);
        assert_eq!(entity_record().record_type(), RecordType::EN);
        assert_eq!(amateur_record().record_type(), RecordType::AM);
        assert_eq!(history_record().record_type(), RecordType::HS);
        assert_eq!(comment_record().record_type(), RecordType::CO);
        assert_eq!(location_record().record_type(), RecordType::LO);
        assert_eq!(frequency_record().record_type(), RecordType::FR);
        assert_eq!(special_condition_record().record_type(), RecordType::SC);
        assert_eq!(raw_record().record_type(), RecordType::BC);
    }

    #[test]
    fn test_unique_system_identifier_dispatch() {
        assert_eq!(header_record().unique_system_identifier(), Some(100001));
        assert_eq!(entity_record().unique_system_identifier(), Some(100002));
        assert_eq!(amateur_record().unique_system_identifier(), Some(100003));
        assert_eq!(history_record().unique_system_identifier(), Some(100004));
        assert_eq!(comment_record().unique_system_identifier(), Some(100005));
        assert_eq!(location_record().unique_system_identifier(), Some(100006));
        assert_eq!(frequency_record().unique_system_identifier(), Some(100007));
        assert_eq!(special_condition_record().unique_system_identifier(), Some(100008));
        assert_eq!(raw_record().unique_system_identifier(), None);
    }

    #[test]
    fn test_call_sign_dispatch() {
        assert_eq!(header_record().call_sign(), Some("W1TEST"));
        assert_eq!(entity_record().call_sign(), Some("W2TEST"));
        assert_eq!(amateur_record().call_sign(), Some("W3TEST"));
        assert_eq!(history_record().call_sign(), Some("W4TEST"));
        assert_eq!(comment_record().call_sign(), Some("W5TEST"));
        assert_eq!(location_record().call_sign(), Some("W6TEST"));
        assert_eq!(frequency_record().call_sign(), Some("W7TEST"));
        assert_eq!(special_condition_record().call_sign(), Some("W8TEST"));
        assert_eq!(raw_record().call_sign(), None);
    }
}

