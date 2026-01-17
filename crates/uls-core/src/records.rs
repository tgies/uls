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
