//! Code definitions and enumerations for ULS data.
//!
//! This module contains all the standardized codes used in ULS records,
//! including radio service codes, application purposes, statuses, etc.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};

/// Radio service codes as defined by the FCC.
/// These identify the type of wireless service a license is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RadioService {
    /// AA - Aviation Auxiliary Group
    AA,
    /// AB - Aural Microwave Booster
    AB,
    /// AC - Aircraft
    AC,
    /// AD - AWS-4 (2000-2020 MHz and 2180-2200 MHz)
    AD,
    /// AF - Aeronautical and Fixed
    AF,
    /// AH - AWS-H Block (1915-1920 MHz and 1995-2000 MHz)
    AH,
    /// AI - Aural Intercity Relay
    AI,
    /// AL - ALL (used for assignments/transfers across services)
    AL,
    /// AN - Antenna Structure Registration
    AN,
    /// AR - Aviation Radionavigation
    AR,
    /// AS - Aural Studio Transmitter Link
    AS,
    /// AT - AWS-3 (1695-1710, 1755-1780, 2155-2180 MHz)
    AT,
    /// AW - AWS (1710-1755 MHz and 2110-2155 MHz)
    AW,
    /// BA - 1390-1392 MHz Band, Market Area
    BA,
    /// BB - 1392-1395 and 1432-1435 MHz Bands, Market Area
    BB,
    /// BC - 1670-1675 MHz Band, Market Area
    BC,
    /// BR - Broadband Radio Service
    BR,
    /// BS - 900 MHz Broadband Service
    BS,
    /// CA - Commercial Air-ground Radiotelephone
    CA,
    /// CB - BETRS
    CB,
    /// CD - Paging and Radiotelephone
    CD,
    /// CE - Digital Electronic Message Service - Common Carrier
    CE,
    /// CF - Common Carrier Fixed Point to Point Microwave
    CF,
    /// CG - General Aviation Air-ground Radiotelephone
    CG,
    /// CJ - Commercial Aviation Air-Ground Radiotelephone (800 MHz)
    CJ,
    /// CL - Cellular
    CL,
    /// CM - Commercial Operator
    CM,
    /// CN - PCS Narrowband
    CN,
    /// CO - Offshore Radiotelephone
    CO,
    /// CP - Part 22 VHF/UHF Paging (excluding 931MHz)
    CP,
    /// CR - Rural Radiotelephone
    CR,
    /// CT - Local Television Transmission
    CT,
    /// CW - PCS Broadband
    CW,
    /// CX - Cellular, Auctioned
    CX,
    /// CY - 1910-1915/1990-1995 MHz Bands, Market Area
    CY,
    /// CZ - Part 22 931 MHz Paging
    CZ,
    /// DV - Multichannel Video Distribution AND Data Service
    DV,
    /// ED - Educational Broadband Service
    ED,
    /// GB - Business, 806-821/851-866 MHz, Conventional
    GB,
    /// GC - 929-931 MHz Band, Auctioned
    GC,
    /// GE - PubSafty/SpecEmer/PubSaftyNtlPlan, 806-817/851-862MHz, Conv
    GE,
    /// GF - Public Safety Ntl Plan, 821-824/866-869 MHz, Conv
    GF,
    /// GI - Other Indust/Land Transp, 896-901/935-940 MHz, Conv
    GI,
    /// GJ - Business/Industrial/Land Trans, 809-824/854-869 MHz, Conv
    GJ,
    /// GL - 900 MHz Conventional SMR (SMR, Site-Specific)
    GL,
    /// GM - 800 MHz Conventional SMR (SMR, Site-specific)
    GM,
    /// GO - Other Indust/Land Transp, 806-821/851-866 MHz, Conv
    GO,
    /// GP - Public Safety/Spec Emerg, 806-821/851-866 MHz, Conv
    GP,
    /// GR - SMR, 896-901/935-940 MHz, Conventional
    GR,
    /// GS - Private Carrier Paging, 929-930 MHz
    GS,
    /// GU - Business, 896-901/935-940 MHz, Conventional
    GU,
    /// GW - General Wireless Communications Service
    GW,
    /// GX - SMR, 806-821/851-866 MHz, Conventional
    GX,
    /// HA - Amateur
    HA,
    /// HV - Vanity (Amateur)
    HV,
    /// IG - Industrial/Business Pool, Conventional
    IG,
    /// IK - Industrial/Business Pool - Commercial, Conventional
    IK,
    /// IQ - Intelligent Transportation Service (Public Safety)
    IQ,
    /// LD - Local Multipoint Distribution Service
    LD,
    /// LN - 902-928 MHz Location Narrowband (Non-multilateration)
    LN,
    /// LP - Broadcast Auxiliary Low Power
    LP,
    /// LS - Location and Monitoring Service, Multilateration (LMS)
    LS,
    /// LV - Low Power Wireless Assist Video Devices
    LV,
    /// LW - 902-928 MHz Location Wideband (Grandfathered AVM)
    LW,
    /// MA - Marine Auxiliary Group
    MA,
    /// MC - Coastal Group
    MC,
    /// MD - Multipoint Distribution Service (MDS and MMDS)
    MD,
    /// MG - Microwave Industrial/Business Pool
    MG,
    /// MK - Alaska Group
    MK,
    /// MM - Millimeter Wave 70/80/90 GHz Service
    MM,
    /// MR - Marine Radiolocation Land
    MR,
    /// MS - Multiple Address Service, Auctioned
    MS,
    /// MW - Microwave Public Safety Pool
    MW,
    /// NC - Nationwide Commercial 5 Channel, 220 MHz
    NC,
    /// NN - 3650-3700 MHz
    NN,
    /// OW - FCC Ownership Disclosure Information
    OW,
    /// PA - Public Safety 4940-4990 MHz Band
    PA,
    /// PB - 4940-4990 MHz Public Safety, Base/Mobile
    PB,
    /// PC - Public Coast Stations, Auctioned
    PC,
    /// PE - Digital Electronic Message Service - Private
    PE,
    /// PF - 4940-4990 MHz Public Safety, Pt-to-Pt, Pt-to-Multi-Pt
    PF,
    /// PK - 3.45 GHz Service
    PK,
    /// PL - 3.5 GHz Band Priority Access License
    PL,
    /// PM - 3.7 GHz Service
    PM,
    /// PW - Public Safety Pool, Conventional
    PW,
    /// QA - 220-222 MHz Band, Auctioned
    QA,
    /// QD - Non-Nationwide Data, 220 MHz
    QD,
    /// QM - Non-Nationwide Public Safety/Mutual Aid, 220 MHz
    QM,
    /// QO - Non-Nationwide Other, 220 MHz
    QO,
    /// QQ - Intelligent Transportation Service (Non-Public Safety)
    QQ,
    /// QT - Non-Nationwide 5 Channel Trunked, 220 MHz
    QT,
    /// RP - Broadcast Auxiliary Remote Pickup
    RP,
    /// RR - Restricted Operator
    RR,
    /// RS - Land Mobile Radiolocation
    RS,
    /// SA - Ship Recreational or Voluntarily Equipped
    SA,
    /// SB - Ship Compulsory Equipped
    SB,
    /// SE - Ship Exemption
    SE,
    /// SG - Conventional Public Safety 700 MHz
    SG,
    /// SL - Public Safety 700 MHz Band-State License
    SL,
    /// SP - 700 MHz Public Safety Broadband Nationwide License
    SP,
    /// SY - Trunked Public Safety 700 MHz
    SY,
    /// TB - TV Microwave Booster
    TB,
    /// TC - MSS Ancillary Terrestrial Component (ATC) Leasing
    TC,
    /// TI - TV Intercity Relay
    TI,
    /// TN - 39 GHz, Auctioned
    TN,
    /// TP - TV Pickup
    TP,
    /// TS - TV Studio Transmitter Link
    TS,
    /// TT - TV Translator Relay
    TT,
    /// TZ - 24 GHz Service
    TZ,
    /// UM - Unlicensed Wireless Microphone Registration
    UM,
    /// UU - Upper Microwave Flexible Use Service
    UU,
    /// VX - Instructional Television Fixed Service
    VX,
    /// WA - Microwave Aviation
    WA,
    /// WM - Microwave Marine
    WM,
    /// WP - 700 MHz Upper Band (Block D)
    WP,
    /// WR - Microwave Radiolocation
    WR,
    /// WS - Wireless Communications Service
    WS,
    /// WT - 600 MHz Band
    WT,
    /// WU - 700 MHz Upper Band (Block C)
    WU,
    /// WX - 700 MHz Guard Band
    WX,
    /// WY - 700 MHz Lower Band (Blocks A, B & E)
    WY,
    /// WZ - 700 MHz Lower Band (Blocks C, D)
    WZ,
    /// YB - Business, 806-821/851-866 MHz, Trunked
    YB,
    /// YC - SMR, 806-821/851-866 MHz, Auctioned
    YC,
    /// YD - SMR, 896-901/935-940 MHz, Auctioned
    YD,
    /// YE - PubSafty/SpecEmer/PubSaftyNtlPlan, 806-817/851-862MHz, Trunked
    YE,
    /// YF - Public Safety Ntl Plan, 821-824/866-869 MHz, Trunked
    YF,
    /// YG - Industrial/Business Pool, Trunked
    YG,
    /// YH - SMR, 806-821/851-866 MHz, Auctioned (Rebanded YC license)
    YH,
    /// YI - Other Indust/Land Transp, 896-901/935-940 MHz, Trunked
    YI,
    /// YJ - Business/Industrial/Land Trans, 809-824/854-869 MHz, Trunked
    YJ,
    /// YK - Industrial/Business Pool - Commercial, Trunked
    YK,
    /// YL - 900 MHz Trunked SMR (SMR, Site-Specific)
    YL,
    /// YM - 800 MHz Trunked SMR (SMR, Site-specific)
    YM,
    /// YO - Other Indust/Land Transp, 806-821/851-866 MHz, Trunked
    YO,
    /// YP - Public Safety/Spec Emerg, 806-821/851-866 MHz, Trunked
    YP,
    /// YS - SMR, 896-901/935-940 MHz, Trunked
    YS,
    /// YU - Business, 896-901/935-940 MHz, Trunked
    YU,
    /// YW - Public Safety Pool, Trunked
    YW,
    /// YX - SMR, 806-821/851-866 MHz, Trunked
    YX,
    /// ZA - General Mobile Radio (GMRS)
    ZA,
    /// ZV - 218-219 MHz Service
    ZV,
}

impl RadioService {
    /// Returns the two-character code for this radio service.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AA => "AA",
            Self::AB => "AB",
            Self::AC => "AC",
            Self::AD => "AD",
            Self::AF => "AF",
            Self::AH => "AH",
            Self::AI => "AI",
            Self::AL => "AL",
            Self::AN => "AN",
            Self::AR => "AR",
            Self::AS => "AS",
            Self::AT => "AT",
            Self::AW => "AW",
            Self::BA => "BA",
            Self::BB => "BB",
            Self::BC => "BC",
            Self::BR => "BR",
            Self::BS => "BS",
            Self::CA => "CA",
            Self::CB => "CB",
            Self::CD => "CD",
            Self::CE => "CE",
            Self::CF => "CF",
            Self::CG => "CG",
            Self::CJ => "CJ",
            Self::CL => "CL",
            Self::CM => "CM",
            Self::CN => "CN",
            Self::CO => "CO",
            Self::CP => "CP",
            Self::CR => "CR",
            Self::CT => "CT",
            Self::CW => "CW",
            Self::CX => "CX",
            Self::CY => "CY",
            Self::CZ => "CZ",
            Self::DV => "DV",
            Self::ED => "ED",
            Self::GB => "GB",
            Self::GC => "GC",
            Self::GE => "GE",
            Self::GF => "GF",
            Self::GI => "GI",
            Self::GJ => "GJ",
            Self::GL => "GL",
            Self::GM => "GM",
            Self::GO => "GO",
            Self::GP => "GP",
            Self::GR => "GR",
            Self::GS => "GS",
            Self::GU => "GU",
            Self::GW => "GW",
            Self::GX => "GX",
            Self::HA => "HA",
            Self::HV => "HV",
            Self::IG => "IG",
            Self::IK => "IK",
            Self::IQ => "IQ",
            Self::LD => "LD",
            Self::LN => "LN",
            Self::LP => "LP",
            Self::LS => "LS",
            Self::LV => "LV",
            Self::LW => "LW",
            Self::MA => "MA",
            Self::MC => "MC",
            Self::MD => "MD",
            Self::MG => "MG",
            Self::MK => "MK",
            Self::MM => "MM",
            Self::MR => "MR",
            Self::MS => "MS",
            Self::MW => "MW",
            Self::NC => "NC",
            Self::NN => "NN",
            Self::OW => "OW",
            Self::PA => "PA",
            Self::PB => "PB",
            Self::PC => "PC",
            Self::PE => "PE",
            Self::PF => "PF",
            Self::PK => "PK",
            Self::PL => "PL",
            Self::PM => "PM",
            Self::PW => "PW",
            Self::QA => "QA",
            Self::QD => "QD",
            Self::QM => "QM",
            Self::QO => "QO",
            Self::QQ => "QQ",
            Self::QT => "QT",
            Self::RP => "RP",
            Self::RR => "RR",
            Self::RS => "RS",
            Self::SA => "SA",
            Self::SB => "SB",
            Self::SE => "SE",
            Self::SG => "SG",
            Self::SL => "SL",
            Self::SP => "SP",
            Self::SY => "SY",
            Self::TB => "TB",
            Self::TC => "TC",
            Self::TI => "TI",
            Self::TN => "TN",
            Self::TP => "TP",
            Self::TS => "TS",
            Self::TT => "TT",
            Self::TZ => "TZ",
            Self::UM => "UM",
            Self::UU => "UU",
            Self::VX => "VX",
            Self::WA => "WA",
            Self::WM => "WM",
            Self::WP => "WP",
            Self::WR => "WR",
            Self::WS => "WS",
            Self::WT => "WT",
            Self::WU => "WU",
            Self::WX => "WX",
            Self::WY => "WY",
            Self::WZ => "WZ",
            Self::YB => "YB",
            Self::YC => "YC",
            Self::YD => "YD",
            Self::YE => "YE",
            Self::YF => "YF",
            Self::YG => "YG",
            Self::YH => "YH",
            Self::YI => "YI",
            Self::YJ => "YJ",
            Self::YK => "YK",
            Self::YL => "YL",
            Self::YM => "YM",
            Self::YO => "YO",
            Self::YP => "YP",
            Self::YS => "YS",
            Self::YU => "YU",
            Self::YW => "YW",
            Self::YX => "YX",
            Self::ZA => "ZA",
            Self::ZV => "ZV",
        }
    }

    /// Returns a human-readable description of this radio service.
    pub fn description(&self) -> &'static str {
        match self {
            Self::AA => "Aviation Auxiliary Group",
            Self::AB => "Aural Microwave Booster",
            Self::AC => "Aircraft",
            Self::AD => "AWS-4 (2000-2020 MHz and 2180-2200 MHz)",
            Self::AF => "Aeronautical and Fixed",
            Self::AH => "AWS-H Block (1915-1920 MHz and 1995-2000 MHz)",
            Self::AI => "Aural Intercity Relay",
            Self::AL => "ALL",
            Self::AN => "Antenna Structure Registration",
            Self::AR => "Aviation Radionavigation",
            Self::AS => "Aural Studio Transmitter Link",
            Self::AT => "AWS-3 (1695-1710, 1755-1780, 2155-2180 MHz)",
            Self::AW => "AWS (1710-1755 MHz and 2110-2155 MHz)",
            Self::BA => "1390-1392 MHz Band, Market Area",
            Self::BB => "1392-1395 and 1432-1435 MHz Bands, Market Area",
            Self::BC => "1670-1675 MHz Band, Market Area",
            Self::BR => "Broadband Radio Service",
            Self::BS => "900 MHz Broadband Service",
            Self::CA => "Commercial Air-ground Radiotelephone",
            Self::CB => "BETRS",
            Self::CD => "Paging and Radiotelephone",
            Self::CE => "Digital Electronic Message Service - Common Carrier",
            Self::CF => "Common Carrier Fixed Point to Point Microwave",
            Self::CG => "General Aviation Air-ground Radiotelephone",
            Self::CJ => "Commercial Aviation Air-Ground Radiotelephone (800 MHz)",
            Self::CL => "Cellular",
            Self::CM => "Commercial Operator",
            Self::CN => "PCS Narrowband",
            Self::CO => "Offshore Radiotelephone",
            Self::CP => "Part 22 VHF/UHF Paging (excluding 931MHz)",
            Self::CR => "Rural Radiotelephone",
            Self::CT => "Local Television Transmission",
            Self::CW => "PCS Broadband",
            Self::CX => "Cellular, Auctioned",
            Self::CY => "1910-1915/1990-1995 MHz Bands, Market Area",
            Self::CZ => "Part 22 931 MHz Paging",
            Self::DV => "Multichannel Video Distribution AND Data Service",
            Self::ED => "Educational Broadband Service",
            Self::GB => "Business, 806-821/851-866 MHz, Conventional",
            Self::GC => "929-931 MHz Band, Auctioned",
            Self::GE => "Public Safety/Special Emergency, 806-817/851-862MHz, Conventional",
            Self::GF => "Public Safety Ntl Plan, 821-824/866-869 MHz, Conventional",
            Self::GI => "Other Industrial/Land Transport, 896-901/935-940 MHz, Conventional",
            Self::GJ => "Business/Industrial/Land Trans, 809-824/854-869 MHz, Conventional",
            Self::GL => "900 MHz Conventional SMR (Site-Specific)",
            Self::GM => "800 MHz Conventional SMR (Site-specific)",
            Self::GO => "Other Industrial/Land Transport, 806-821/851-866 MHz, Conventional",
            Self::GP => "Public Safety/Special Emergency, 806-821/851-866 MHz, Conventional",
            Self::GR => "SMR, 896-901/935-940 MHz, Conventional",
            Self::GS => "Private Carrier Paging, 929-930 MHz",
            Self::GU => "Business, 896-901/935-940 MHz, Conventional",
            Self::GW => "General Wireless Communications Service",
            Self::GX => "SMR, 806-821/851-866 MHz, Conventional",
            Self::HA => "Amateur",
            Self::HV => "Vanity (Amateur)",
            Self::IG => "Industrial/Business Pool, Conventional",
            Self::IK => "Industrial/Business Pool - Commercial, Conventional",
            Self::IQ => "Intelligent Transportation Service (Public Safety)",
            Self::LD => "Local Multipoint Distribution Service",
            Self::LN => "902-928 MHz Location Narrowband (Non-multilateration)",
            Self::LP => "Broadcast Auxiliary Low Power",
            Self::LS => "Location and Monitoring Service, Multilateration (LMS)",
            Self::LV => "Low Power Wireless Assist Video Devices",
            Self::LW => "902-928 MHz Location Wideband (Grandfathered AVM)",
            Self::MA => "Marine Auxiliary Group",
            Self::MC => "Coastal Group",
            Self::MD => "Multipoint Distribution Service (MDS and MMDS)",
            Self::MG => "Microwave Industrial/Business Pool",
            Self::MK => "Alaska Group",
            Self::MM => "Millimeter Wave 70/80/90 GHz Service",
            Self::MR => "Marine Radiolocation Land",
            Self::MS => "Multiple Address Service, Auctioned",
            Self::MW => "Microwave Public Safety Pool",
            Self::NC => "Nationwide Commercial 5 Channel, 220 MHz",
            Self::NN => "3650-3700 MHz",
            Self::OW => "FCC Ownership Disclosure Information",
            Self::PA => "Public Safety 4940-4990 MHz Band",
            Self::PB => "4940-4990 MHz Public Safety, Base/Mobile",
            Self::PC => "Public Coast Stations, Auctioned",
            Self::PE => "Digital Electronic Message Service - Private",
            Self::PF => "4940-4990 MHz Public Safety, Point-to-Point",
            Self::PK => "3.45 GHz Service",
            Self::PL => "3.5 GHz Band Priority Access License",
            Self::PM => "3.7 GHz Service",
            Self::PW => "Public Safety Pool, Conventional",
            Self::QA => "220-222 MHz Band, Auctioned",
            Self::QD => "Non-Nationwide Data, 220 MHz",
            Self::QM => "Non-Nationwide Public Safety/Mutual Aid, 220 MHz",
            Self::QO => "Non-Nationwide Other, 220 MHz",
            Self::QQ => "Intelligent Transportation Service (Non-Public Safety)",
            Self::QT => "Non-Nationwide 5 Channel Trunked, 220 MHz",
            Self::RP => "Broadcast Auxiliary Remote Pickup",
            Self::RR => "Restricted Operator",
            Self::RS => "Land Mobile Radiolocation",
            Self::SA => "Ship Recreational or Voluntarily Equipped",
            Self::SB => "Ship Compulsory Equipped",
            Self::SE => "Ship Exemption",
            Self::SG => "Conventional Public Safety 700 MHz",
            Self::SL => "Public Safety 700 MHz Band-State License",
            Self::SP => "700 MHz Public Safety Broadband Nationwide License",
            Self::SY => "Trunked Public Safety 700 MHz",
            Self::TB => "TV Microwave Booster",
            Self::TC => "MSS Ancillary Terrestrial Component (ATC) Leasing",
            Self::TI => "TV Intercity Relay",
            Self::TN => "39 GHz, Auctioned",
            Self::TP => "TV Pickup",
            Self::TS => "TV Studio Transmitter Link",
            Self::TT => "TV Translator Relay",
            Self::TZ => "24 GHz Service",
            Self::UM => "Unlicensed Wireless Microphone Registration",
            Self::UU => "Upper Microwave Flexible Use Service",
            Self::VX => "Instructional Television Fixed Service",
            Self::WA => "Microwave Aviation",
            Self::WM => "Microwave Marine",
            Self::WP => "700 MHz Upper Band (Block D)",
            Self::WR => "Microwave Radiolocation",
            Self::WS => "Wireless Communications Service",
            Self::WT => "600 MHz Band",
            Self::WU => "700 MHz Upper Band (Block C)",
            Self::WX => "700 MHz Guard Band",
            Self::WY => "700 MHz Lower Band (Blocks A, B & E)",
            Self::WZ => "700 MHz Lower Band (Blocks C, D)",
            Self::YB => "Business, 806-821/851-866 MHz, Trunked",
            Self::YC => "SMR, 806-821/851-866 MHz, Auctioned",
            Self::YD => "SMR, 896-901/935-940 MHz, Auctioned",
            Self::YE => "Public Safety/Special Emergency, 806-817/851-862MHz, Trunked",
            Self::YF => "Public Safety Ntl Plan, 821-824/866-869 MHz, Trunked",
            Self::YG => "Industrial/Business Pool, Trunked",
            Self::YH => "SMR, 806-821/851-866 MHz, Auctioned (Rebanded YC)",
            Self::YI => "Other Industrial/Land Transport, 896-901/935-940 MHz, Trunked",
            Self::YJ => "Business/Industrial/Land Trans, 809-824/854-869 MHz, Trunked",
            Self::YK => "Industrial/Business Pool - Commercial, Trunked",
            Self::YL => "900 MHz Trunked SMR (Site-Specific)",
            Self::YM => "800 MHz Trunked SMR (Site-specific)",
            Self::YO => "Other Industrial/Land Transport, 806-821/851-866 MHz, Trunked",
            Self::YP => "Public Safety/Special Emergency, 806-821/851-866 MHz, Trunked",
            Self::YS => "SMR, 896-901/935-940 MHz, Trunked",
            Self::YU => "Business, 896-901/935-940 MHz, Trunked",
            Self::YW => "Public Safety Pool, Trunked",
            Self::YX => "SMR, 806-821/851-866 MHz, Trunked",
            Self::ZA => "General Mobile Radio (GMRS)",
            Self::ZV => "218-219 MHz Service",
        }
    }

    /// Returns true if this is an amateur radio service.
    pub fn is_amateur(&self) -> bool {
        matches!(self, Self::HA | Self::HV)
    }

    /// Returns true if this is a ship/maritime service.
    pub fn is_maritime(&self) -> bool {
        matches!(self, Self::SA | Self::SB | Self::SE | Self::MA | Self::MC | Self::MK)
    }

    /// Returns true if this is an aircraft service.
    pub fn is_aircraft(&self) -> bool {
        matches!(self, Self::AC | Self::AA | Self::AF | Self::AR | Self::CA | Self::CG | Self::CJ)
    }
}

impl fmt::Display for RadioService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for RadioService {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_uppercase().as_str() {
            "AA" => Ok(Self::AA),
            "AB" => Ok(Self::AB),
            "AC" => Ok(Self::AC),
            "AD" => Ok(Self::AD),
            "AF" => Ok(Self::AF),
            "AH" => Ok(Self::AH),
            "AI" => Ok(Self::AI),
            "AL" => Ok(Self::AL),
            "AN" => Ok(Self::AN),
            "AR" => Ok(Self::AR),
            "AS" => Ok(Self::AS),
            "AT" => Ok(Self::AT),
            "AW" => Ok(Self::AW),
            "BA" => Ok(Self::BA),
            "BB" => Ok(Self::BB),
            "BC" => Ok(Self::BC),
            "BR" => Ok(Self::BR),
            "BS" => Ok(Self::BS),
            "CA" => Ok(Self::CA),
            "CB" => Ok(Self::CB),
            "CD" => Ok(Self::CD),
            "CE" => Ok(Self::CE),
            "CF" => Ok(Self::CF),
            "CG" => Ok(Self::CG),
            "CJ" => Ok(Self::CJ),
            "CL" => Ok(Self::CL),
            "CM" => Ok(Self::CM),
            "CN" => Ok(Self::CN),
            "CO" => Ok(Self::CO),
            "CP" => Ok(Self::CP),
            "CR" => Ok(Self::CR),
            "CT" => Ok(Self::CT),
            "CW" => Ok(Self::CW),
            "CX" => Ok(Self::CX),
            "CY" => Ok(Self::CY),
            "CZ" => Ok(Self::CZ),
            "DV" => Ok(Self::DV),
            "ED" => Ok(Self::ED),
            "GB" => Ok(Self::GB),
            "GC" => Ok(Self::GC),
            "GE" => Ok(Self::GE),
            "GF" => Ok(Self::GF),
            "GI" => Ok(Self::GI),
            "GJ" => Ok(Self::GJ),
            "GL" => Ok(Self::GL),
            "GM" => Ok(Self::GM),
            "GO" => Ok(Self::GO),
            "GP" => Ok(Self::GP),
            "GR" => Ok(Self::GR),
            "GS" => Ok(Self::GS),
            "GU" => Ok(Self::GU),
            "GW" => Ok(Self::GW),
            "GX" => Ok(Self::GX),
            "HA" => Ok(Self::HA),
            "HV" => Ok(Self::HV),
            "IG" => Ok(Self::IG),
            "IK" => Ok(Self::IK),
            "IQ" => Ok(Self::IQ),
            "LD" => Ok(Self::LD),
            "LN" => Ok(Self::LN),
            "LP" => Ok(Self::LP),
            "LS" => Ok(Self::LS),
            "LV" => Ok(Self::LV),
            "LW" => Ok(Self::LW),
            "MA" => Ok(Self::MA),
            "MC" => Ok(Self::MC),
            "MD" => Ok(Self::MD),
            "MG" => Ok(Self::MG),
            "MK" => Ok(Self::MK),
            "MM" => Ok(Self::MM),
            "MR" => Ok(Self::MR),
            "MS" => Ok(Self::MS),
            "MW" => Ok(Self::MW),
            "NC" => Ok(Self::NC),
            "NN" => Ok(Self::NN),
            "OW" => Ok(Self::OW),
            "PA" => Ok(Self::PA),
            "PB" => Ok(Self::PB),
            "PC" => Ok(Self::PC),
            "PE" => Ok(Self::PE),
            "PF" => Ok(Self::PF),
            "PK" => Ok(Self::PK),
            "PL" => Ok(Self::PL),
            "PM" => Ok(Self::PM),
            "PW" => Ok(Self::PW),
            "QA" => Ok(Self::QA),
            "QD" => Ok(Self::QD),
            "QM" => Ok(Self::QM),
            "QO" => Ok(Self::QO),
            "QQ" => Ok(Self::QQ),
            "QT" => Ok(Self::QT),
            "RP" => Ok(Self::RP),
            "RR" => Ok(Self::RR),
            "RS" => Ok(Self::RS),
            "SA" => Ok(Self::SA),
            "SB" => Ok(Self::SB),
            "SE" => Ok(Self::SE),
            "SG" => Ok(Self::SG),
            "SL" => Ok(Self::SL),
            "SP" => Ok(Self::SP),
            "SY" => Ok(Self::SY),
            "TB" => Ok(Self::TB),
            "TC" => Ok(Self::TC),
            "TI" => Ok(Self::TI),
            "TN" => Ok(Self::TN),
            "TP" => Ok(Self::TP),
            "TS" => Ok(Self::TS),
            "TT" => Ok(Self::TT),
            "TZ" => Ok(Self::TZ),
            "UM" => Ok(Self::UM),
            "UU" => Ok(Self::UU),
            "VX" => Ok(Self::VX),
            "WA" => Ok(Self::WA),
            "WM" => Ok(Self::WM),
            "WP" => Ok(Self::WP),
            "WR" => Ok(Self::WR),
            "WS" => Ok(Self::WS),
            "WT" => Ok(Self::WT),
            "WU" => Ok(Self::WU),
            "WX" => Ok(Self::WX),
            "WY" => Ok(Self::WY),
            "WZ" => Ok(Self::WZ),
            "YB" => Ok(Self::YB),
            "YC" => Ok(Self::YC),
            "YD" => Ok(Self::YD),
            "YE" => Ok(Self::YE),
            "YF" => Ok(Self::YF),
            "YG" => Ok(Self::YG),
            "YH" => Ok(Self::YH),
            "YI" => Ok(Self::YI),
            "YJ" => Ok(Self::YJ),
            "YK" => Ok(Self::YK),
            "YL" => Ok(Self::YL),
            "YM" => Ok(Self::YM),
            "YO" => Ok(Self::YO),
            "YP" => Ok(Self::YP),
            "YS" => Ok(Self::YS),
            "YU" => Ok(Self::YU),
            "YW" => Ok(Self::YW),
            "YX" => Ok(Self::YX),
            "ZA" => Ok(Self::ZA),
            "ZV" => Ok(Self::ZV),
            _ => Err(Error::InvalidEnumValue {
                enum_type: "RadioService",
                value: s.to_string(),
            }),
        }
    }
}

/// Application purpose codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApplicationPurpose {
    /// AA - Assignment of Authorization
    AssignmentOfAuthorization,
    /// AM - Amendment
    Amendment,
    /// AR - DE Annual Report
    DEAnnualReport,
    /// AU - Administrative Update
    AdministrativeUpdate,
    /// CA - Cancellation of License
    CancellationOfLicense,
    /// CB - C Block Election
    CBlockElection,
    /// DC - Data Correction
    DataCorrection,
    /// DU - Duplicate License
    DuplicateLicense,
    /// EX - Request for Extension of Time
    ExtensionOfTime,
    /// HA - HAC Report
    HACReport,
    /// LC - Cancel a Lease
    CancelLease,
    /// LE - Extend Term of a Lease
    ExtendLease,
    /// LM - Modification of a Lease
    ModifyLease,
    /// LN - New Lease
    NewLease,
    /// LT - Transfer of Control of a Lessee
    TransferLessee,
    /// LU - Administrative Update of a Lease
    AdminUpdateLease,
    /// MD - Modification
    Modification,
    /// NE - New
    New,
    /// NT - Required Notification
    RequiredNotification,
    /// RE - DE Reportable Event
    DEReportableEvent,
    /// RL - Register Link/Location
    RegisterLinkLocation,
    /// RM - Renewal/Modification
    RenewalModification,
    /// RO - Renewal Only
    RenewalOnly,
    /// TC - Transfer of Control
    TransferOfControl,
    /// WD - Withdrawal of Application
    Withdrawal,
}

impl ApplicationPurpose {
    /// Returns the two-character code for this application purpose.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AssignmentOfAuthorization => "AA",
            Self::Amendment => "AM",
            Self::DEAnnualReport => "AR",
            Self::AdministrativeUpdate => "AU",
            Self::CancellationOfLicense => "CA",
            Self::CBlockElection => "CB",
            Self::DataCorrection => "DC",
            Self::DuplicateLicense => "DU",
            Self::ExtensionOfTime => "EX",
            Self::HACReport => "HA",
            Self::CancelLease => "LC",
            Self::ExtendLease => "LE",
            Self::ModifyLease => "LM",
            Self::NewLease => "LN",
            Self::TransferLessee => "LT",
            Self::AdminUpdateLease => "LU",
            Self::Modification => "MD",
            Self::New => "NE",
            Self::RequiredNotification => "NT",
            Self::DEReportableEvent => "RE",
            Self::RegisterLinkLocation => "RL",
            Self::RenewalModification => "RM",
            Self::RenewalOnly => "RO",
            Self::TransferOfControl => "TC",
            Self::Withdrawal => "WD",
        }
    }
}

impl FromStr for ApplicationPurpose {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_uppercase().as_str() {
            "AA" => Ok(Self::AssignmentOfAuthorization),
            "AM" => Ok(Self::Amendment),
            "AR" => Ok(Self::DEAnnualReport),
            "AU" => Ok(Self::AdministrativeUpdate),
            "CA" => Ok(Self::CancellationOfLicense),
            "CB" => Ok(Self::CBlockElection),
            "DC" => Ok(Self::DataCorrection),
            "DU" => Ok(Self::DuplicateLicense),
            "EX" => Ok(Self::ExtensionOfTime),
            "HA" => Ok(Self::HACReport),
            "LC" => Ok(Self::CancelLease),
            "LE" => Ok(Self::ExtendLease),
            "LM" => Ok(Self::ModifyLease),
            "LN" => Ok(Self::NewLease),
            "LT" => Ok(Self::TransferLessee),
            "LU" => Ok(Self::AdminUpdateLease),
            "MD" => Ok(Self::Modification),
            "NE" => Ok(Self::New),
            "NT" => Ok(Self::RequiredNotification),
            "RE" => Ok(Self::DEReportableEvent),
            "RL" => Ok(Self::RegisterLinkLocation),
            "RM" => Ok(Self::RenewalModification),
            "RO" => Ok(Self::RenewalOnly),
            "TC" => Ok(Self::TransferOfControl),
            "WD" => Ok(Self::Withdrawal),
            _ => Err(Error::InvalidEnumValue {
                enum_type: "ApplicationPurpose",
                value: s.to_string(),
            }),
        }
    }
}

impl fmt::Display for ApplicationPurpose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Application status codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApplicationStatus {
    /// 1 - Submitted
    Submitted,
    /// 2 - Pending
    Pending,
    /// A - Granted (alternative)
    AGranted,
    /// C - Consented To
    ConsentedTo,
    /// D - Dismissed
    Dismissed,
    /// E - Eliminate
    Eliminate,
    /// G - Granted
    Granted,
    /// H - History Only
    HistoryOnly,
    /// I - Inactive
    Inactive,
    /// J - HAC Submitted
    HACSubmitted,
    /// K - Killed
    Killed,
    /// M - Consummated
    Consummated,
    /// N - Granted in Part
    GrantedInPart,
    /// P - Pending Pack Filing
    PendingPackFiling,
    /// Q - Accepted
    Accepted,
    /// R - Returned
    Returned,
    /// S - Saved
    Saved,
    /// T - Terminated
    Terminated,
    /// U - Unprocessable
    Unprocessable,
    /// W - Withdrawn
    Withdrawn,
    /// X - Not Applicable
    NotApplicable,
    /// Y - Application has problems
    HasProblems,
}

impl ApplicationStatus {
    /// Returns the single-character code for this status.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Submitted => "1",
            Self::Pending => "2",
            Self::AGranted => "A",
            Self::ConsentedTo => "C",
            Self::Dismissed => "D",
            Self::Eliminate => "E",
            Self::Granted => "G",
            Self::HistoryOnly => "H",
            Self::Inactive => "I",
            Self::HACSubmitted => "J",
            Self::Killed => "K",
            Self::Consummated => "M",
            Self::GrantedInPart => "N",
            Self::PendingPackFiling => "P",
            Self::Accepted => "Q",
            Self::Returned => "R",
            Self::Saved => "S",
            Self::Terminated => "T",
            Self::Unprocessable => "U",
            Self::Withdrawn => "W",
            Self::NotApplicable => "X",
            Self::HasProblems => "Y",
        }
    }
}

impl FromStr for ApplicationStatus {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_uppercase().as_str() {
            "1" => Ok(Self::Submitted),
            "2" => Ok(Self::Pending),
            "A" => Ok(Self::AGranted),
            "C" => Ok(Self::ConsentedTo),
            "D" => Ok(Self::Dismissed),
            "E" => Ok(Self::Eliminate),
            "G" => Ok(Self::Granted),
            "H" => Ok(Self::HistoryOnly),
            "I" => Ok(Self::Inactive),
            "J" => Ok(Self::HACSubmitted),
            "K" => Ok(Self::Killed),
            "M" => Ok(Self::Consummated),
            "N" => Ok(Self::GrantedInPart),
            "P" => Ok(Self::PendingPackFiling),
            "Q" => Ok(Self::Accepted),
            "R" => Ok(Self::Returned),
            "S" => Ok(Self::Saved),
            "T" => Ok(Self::Terminated),
            "U" => Ok(Self::Unprocessable),
            "W" => Ok(Self::Withdrawn),
            "X" => Ok(Self::NotApplicable),
            "Y" => Ok(Self::HasProblems),
            _ => Err(Error::InvalidEnumValue {
                enum_type: "ApplicationStatus",
                value: s.to_string(),
            }),
        }
    }
}

impl fmt::Display for ApplicationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// License status codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LicenseStatus {
    /// A - Active
    Active,
    /// C - Cancelled
    Cancelled,
    /// E - Expired
    Expired,
    /// L - Pending Legal Status
    PendingLegalStatus,
    /// P - Parent Station Cancelled
    ParentStationCancelled,
    /// T - Terminated
    Terminated,
    /// X - Term Pending
    TermPending,
}

impl LicenseStatus {
    /// Returns the single-character code for this status.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "A",
            Self::Cancelled => "C",
            Self::Expired => "E",
            Self::PendingLegalStatus => "L",
            Self::ParentStationCancelled => "P",
            Self::Terminated => "T",
            Self::TermPending => "X",
        }
    }

    /// Returns true if this license is considered active/valid.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
}

impl FromStr for LicenseStatus {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_uppercase().as_str() {
            "A" => Ok(Self::Active),
            "C" => Ok(Self::Cancelled),
            "E" => Ok(Self::Expired),
            "L" => Ok(Self::PendingLegalStatus),
            "P" => Ok(Self::ParentStationCancelled),
            "T" => Ok(Self::Terminated),
            "X" => Ok(Self::TermPending),
            _ => Err(Error::InvalidEnumValue {
                enum_type: "LicenseStatus",
                value: s.to_string(),
            }),
        }
    }
}

impl fmt::Display for LicenseStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Amateur operator class codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperatorClass {
    /// A - Advanced
    Advanced,
    /// E - Amateur Extra
    Extra,
    /// G - General
    General,
    /// N - Novice
    Novice,
    /// P - Technician Plus
    TechnicianPlus,
    /// T - Technician
    Technician,
}

impl OperatorClass {
    /// Returns the single-character code for this operator class.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Advanced => "A",
            Self::Extra => "E",
            Self::General => "G",
            Self::Novice => "N",
            Self::TechnicianPlus => "P",
            Self::Technician => "T",
        }
    }

    /// Returns a human-readable description of this operator class.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Advanced => "Advanced",
            Self::Extra => "Amateur Extra",
            Self::General => "General",
            Self::Novice => "Novice",
            Self::TechnicianPlus => "Technician Plus",
            Self::Technician => "Technician",
        }
    }
}

impl FromStr for OperatorClass {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_uppercase().as_str() {
            "A" => Ok(Self::Advanced),
            "E" => Ok(Self::Extra),
            "G" => Ok(Self::General),
            "N" => Ok(Self::Novice),
            "P" => Ok(Self::TechnicianPlus),
            "T" => Ok(Self::Technician),
            _ => Err(Error::InvalidEnumValue {
                enum_type: "OperatorClass",
                value: s.to_string(),
            }),
        }
    }
}

impl fmt::Display for OperatorClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Entity type codes for the EN (Entity) record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    /// CE - Transferee Contact
    TransfereeContact,
    /// CL - Licensee Contact
    LicenseeContact,
    /// CR - Assignor Contact
    AssignorContact,
    /// CS - Lessee Contact
    LesseeContact,
    /// E - Transferee
    Transferee,
    /// L - Licensee
    Licensee,
    /// O - Owner
    Owner,
    /// R - Assignor or Transferor
    Assignor,
    /// S - Lessee
    Lessee,
}

impl EntityType {
    /// Returns the code for this entity type.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TransfereeContact => "CE",
            Self::LicenseeContact => "CL",
            Self::AssignorContact => "CR",
            Self::LesseeContact => "CS",
            Self::Transferee => "E",
            Self::Licensee => "L",
            Self::Owner => "O",
            Self::Assignor => "R",
            Self::Lessee => "S",
        }
    }
}

impl FromStr for EntityType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_uppercase().as_str() {
            "CE" => Ok(Self::TransfereeContact),
            "CL" => Ok(Self::LicenseeContact),
            "CR" => Ok(Self::AssignorContact),
            "CS" => Ok(Self::LesseeContact),
            "E" => Ok(Self::Transferee),
            "L" => Ok(Self::Licensee),
            "O" => Ok(Self::Owner),
            "R" => Ok(Self::Assignor),
            "S" => Ok(Self::Lessee),
            _ => Err(Error::InvalidEnumValue {
                enum_type: "EntityType",
                value: s.to_string(),
            }),
        }
    }
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Record type codes identifying the type of record in a DAT file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecordType {
    A2, A3, AC, AD, AG, AH, AM, AN, AP, AS, AT,
    BC, BD, BE, BF, BL, BO, BT,
    CD, CF, CG, CO, CP, CS,
    EC, EM, EN,
    F2, F3, F4, F5, F6, FA, FC, FF, FR, FS, FT,
    HD, HS,
    IA, IF, IR,
    L2, L3, L4, L5, L6, LA, LC, LD, LF, LH, LL, LM, LO, LS,
    MC, ME, MF, MH, MI, MK, MP, MW,
    O2, OP,
    P2, PA, PC,
    RA, RC, RE, RI, RZ,
    SC, SE, SF, SG, SH, SI, SR, ST, SV,
    TA, TL, TP,
    UA,
    VC,
}

impl RecordType {
    /// Returns the two-character code for this record type.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::A2 => "A2", Self::A3 => "A3", Self::AC => "AC", Self::AD => "AD",
            Self::AG => "AG", Self::AH => "AH", Self::AM => "AM", Self::AN => "AN",
            Self::AP => "AP", Self::AS => "AS", Self::AT => "AT",
            Self::BC => "BC", Self::BD => "BD", Self::BE => "BE", Self::BF => "BF",
            Self::BL => "BL", Self::BO => "BO", Self::BT => "BT",
            Self::CD => "CD", Self::CF => "CF", Self::CG => "CG", Self::CO => "CO",
            Self::CP => "CP", Self::CS => "CS",
            Self::EC => "EC", Self::EM => "EM", Self::EN => "EN",
            Self::F2 => "F2", Self::F3 => "F3", Self::F4 => "F4", Self::F5 => "F5",
            Self::F6 => "F6", Self::FA => "FA", Self::FC => "FC", Self::FF => "FF",
            Self::FR => "FR", Self::FS => "FS", Self::FT => "FT",
            Self::HD => "HD", Self::HS => "HS",
            Self::IA => "IA", Self::IF => "IF", Self::IR => "IR",
            Self::L2 => "L2", Self::L3 => "L3", Self::L4 => "L4", Self::L5 => "L5",
            Self::L6 => "L6", Self::LA => "LA", Self::LC => "LC", Self::LD => "LD",
            Self::LF => "LF", Self::LH => "LH", Self::LL => "LL", Self::LM => "LM",
            Self::LO => "LO", Self::LS => "LS",
            Self::MC => "MC", Self::ME => "ME", Self::MF => "MF", Self::MH => "MH",
            Self::MI => "MI", Self::MK => "MK", Self::MP => "MP", Self::MW => "MW",
            Self::O2 => "O2", Self::OP => "OP",
            Self::P2 => "P2", Self::PA => "PA", Self::PC => "PC",
            Self::RA => "RA", Self::RC => "RC", Self::RE => "RE", Self::RI => "RI",
            Self::RZ => "RZ",
            Self::SC => "SC", Self::SE => "SE", Self::SF => "SF", Self::SG => "SG",
            Self::SH => "SH", Self::SI => "SI", Self::SR => "SR", Self::ST => "ST",
            Self::SV => "SV",
            Self::TA => "TA", Self::TL => "TL", Self::TP => "TP",
            Self::UA => "UA",
            Self::VC => "VC",
        }
    }

    /// Returns the corresponding DAT filename for this record type.
    pub fn dat_filename(&self) -> String {
        format!("{}.dat", self.as_str())
    }
}

impl FromStr for RecordType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_uppercase().as_str() {
            "A2" => Ok(Self::A2), "A3" => Ok(Self::A3), "AC" => Ok(Self::AC),
            "AD" => Ok(Self::AD), "AG" => Ok(Self::AG), "AH" => Ok(Self::AH),
            "AM" => Ok(Self::AM), "AN" => Ok(Self::AN), "AP" => Ok(Self::AP),
            "AS" => Ok(Self::AS), "AT" => Ok(Self::AT),
            "BC" => Ok(Self::BC), "BD" => Ok(Self::BD), "BE" => Ok(Self::BE),
            "BF" => Ok(Self::BF), "BL" => Ok(Self::BL), "BO" => Ok(Self::BO),
            "BT" => Ok(Self::BT),
            "CD" => Ok(Self::CD), "CF" => Ok(Self::CF), "CG" => Ok(Self::CG),
            "CO" => Ok(Self::CO), "CP" => Ok(Self::CP), "CS" => Ok(Self::CS),
            "EC" => Ok(Self::EC), "EM" => Ok(Self::EM), "EN" => Ok(Self::EN),
            "F2" => Ok(Self::F2), "F3" => Ok(Self::F3), "F4" => Ok(Self::F4),
            "F5" => Ok(Self::F5), "F6" => Ok(Self::F6), "FA" => Ok(Self::FA),
            "FC" => Ok(Self::FC), "FF" => Ok(Self::FF), "FR" => Ok(Self::FR),
            "FS" => Ok(Self::FS), "FT" => Ok(Self::FT),
            "HD" => Ok(Self::HD), "HS" => Ok(Self::HS),
            "IA" => Ok(Self::IA), "IF" => Ok(Self::IF), "IR" => Ok(Self::IR),
            "L2" => Ok(Self::L2), "L3" => Ok(Self::L3), "L4" => Ok(Self::L4),
            "L5" => Ok(Self::L5), "L6" => Ok(Self::L6), "LA" => Ok(Self::LA),
            "LC" => Ok(Self::LC), "LD" => Ok(Self::LD), "LF" => Ok(Self::LF),
            "LH" => Ok(Self::LH), "LL" => Ok(Self::LL), "LM" => Ok(Self::LM),
            "LO" => Ok(Self::LO), "LS" => Ok(Self::LS),
            "MC" => Ok(Self::MC), "ME" => Ok(Self::ME), "MF" => Ok(Self::MF),
            "MH" => Ok(Self::MH), "MI" => Ok(Self::MI), "MK" => Ok(Self::MK),
            "MP" => Ok(Self::MP), "MW" => Ok(Self::MW),
            "O2" => Ok(Self::O2), "OP" => Ok(Self::OP),
            "P2" => Ok(Self::P2), "PA" => Ok(Self::PA), "PC" => Ok(Self::PC),
            "RA" => Ok(Self::RA), "RC" => Ok(Self::RC), "RE" => Ok(Self::RE),
            "RI" => Ok(Self::RI), "RZ" => Ok(Self::RZ),
            "SC" => Ok(Self::SC), "SE" => Ok(Self::SE), "SF" => Ok(Self::SF),
            "SG" => Ok(Self::SG), "SH" => Ok(Self::SH), "SI" => Ok(Self::SI),
            "SR" => Ok(Self::SR), "ST" => Ok(Self::ST), "SV" => Ok(Self::SV),
            "TA" => Ok(Self::TA), "TL" => Ok(Self::TL), "TP" => Ok(Self::TP),
            "UA" => Ok(Self::UA),
            "VC" => Ok(Self::VC),
            _ => Err(Error::InvalidRecordType(s.to_string())),
        }
    }
}

impl fmt::Display for RecordType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radio_service_roundtrip() {
        for code in ["HA", "HV", "AC", "ZA", "CL", "CW"] {
            let service: RadioService = code.parse().unwrap();
            assert_eq!(service.as_str(), code);
            assert_eq!(service.to_string(), code);
        }
    }

    #[test]
    fn test_radio_service_amateur() {
        assert!(RadioService::HA.is_amateur());
        assert!(RadioService::HV.is_amateur());
        assert!(!RadioService::AC.is_amateur());
    }

    #[test]
    fn test_radio_service_invalid() {
        let result: Result<RadioService> = "XX".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_operator_class_roundtrip() {
        for code in ["A", "E", "G", "N", "P", "T"] {
            let class: OperatorClass = code.parse().unwrap();
            assert_eq!(class.as_str(), code);
        }
    }

    #[test]
    fn test_license_status() {
        assert!(LicenseStatus::Active.is_active());
        assert!(!LicenseStatus::Expired.is_active());
        assert!(!LicenseStatus::Cancelled.is_active());
    }

    #[test]
    fn test_record_type_dat_filename() {
        assert_eq!(RecordType::HD.dat_filename(), "HD.dat");
        assert_eq!(RecordType::AM.dat_filename(), "AM.dat");
    }

    #[test]
    fn test_all_record_types_parse() {
        let codes = [
            "A2", "A3", "AC", "AD", "AG", "AH", "AM", "AN", "AP", "AS", "AT",
            "BC", "BD", "BE", "BF", "BL", "BO", "BT",
            "CD", "CF", "CG", "CO", "CP", "CS",
            "EC", "EM", "EN",
            "F2", "F3", "F4", "F5", "F6", "FA", "FC", "FF", "FR", "FS", "FT",
            "HD", "HS",
            "IA", "IF", "IR",
            "L2", "L3", "L4", "L5", "L6", "LA", "LC", "LD", "LF", "LH", "LL", "LM", "LO", "LS",
            "MC", "ME", "MF", "MH", "MI", "MK", "MP", "MW",
            "O2", "OP",
            "P2", "PA", "PC",
            "RA", "RC", "RE", "RI", "RZ",
            "SC", "SE", "SF", "SG", "SH", "SI", "SR", "ST", "SV",
            "TA", "TL", "TP",
            "UA",
            "VC",
        ];

        for code in codes {
            let rt: RecordType = code.parse().expect(&format!("Failed to parse {}", code));
            assert_eq!(rt.as_str(), code);
        }
    }
}
