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
        matches!(
            self,
            Self::SA | Self::SB | Self::SE | Self::MA | Self::MC | Self::MK
        )
    }

    /// Returns true if this is an aircraft service.
    pub fn is_aircraft(&self) -> bool {
        matches!(
            self,
            Self::AC | Self::AA | Self::AF | Self::AR | Self::CA | Self::CG | Self::CJ
        )
    }

    /// Convert to a u8 for compact database storage.
    /// The encoding is stable and matches the alphabetical order of variants.
    pub fn to_u8(&self) -> u8 {
        match self {
            Self::AA => 0,
            Self::AB => 1,
            Self::AC => 2,
            Self::AD => 3,
            Self::AF => 4,
            Self::AH => 5,
            Self::AI => 6,
            Self::AL => 7,
            Self::AN => 8,
            Self::AR => 9,
            Self::AS => 10,
            Self::AT => 11,
            Self::AW => 12,
            Self::BA => 13,
            Self::BB => 14,
            Self::BC => 15,
            Self::BR => 16,
            Self::BS => 17,
            Self::CA => 18,
            Self::CB => 19,
            Self::CD => 20,
            Self::CE => 21,
            Self::CF => 22,
            Self::CG => 23,
            Self::CJ => 24,
            Self::CL => 25,
            Self::CM => 26,
            Self::CN => 27,
            Self::CO => 28,
            Self::CP => 29,
            Self::CR => 30,
            Self::CT => 31,
            Self::CW => 32,
            Self::CX => 33,
            Self::CY => 34,
            Self::CZ => 35,
            Self::DV => 36,
            Self::ED => 37,
            Self::GB => 38,
            Self::GC => 39,
            Self::GE => 40,
            Self::GF => 41,
            Self::GI => 42,
            Self::GJ => 43,
            Self::GL => 44,
            Self::GM => 45,
            Self::GO => 46,
            Self::GP => 47,
            Self::GR => 48,
            Self::GS => 49,
            Self::GU => 50,
            Self::GW => 51,
            Self::GX => 52,
            Self::HA => 53,
            Self::HV => 54,
            Self::IG => 55,
            Self::IK => 56,
            Self::IQ => 57,
            Self::LD => 58,
            Self::LN => 59,
            Self::LP => 60,
            Self::LS => 61,
            Self::LV => 62,
            Self::LW => 63,
            Self::MA => 64,
            Self::MC => 65,
            Self::MD => 66,
            Self::MG => 67,
            Self::MK => 68,
            Self::MM => 69,
            Self::MR => 70,
            Self::MS => 71,
            Self::MW => 72,
            Self::NC => 73,
            Self::NN => 74,
            Self::OW => 75,
            Self::PA => 76,
            Self::PB => 77,
            Self::PC => 78,
            Self::PE => 79,
            Self::PF => 80,
            Self::PK => 81,
            Self::PL => 82,
            Self::PM => 83,
            Self::PW => 84,
            Self::QA => 85,
            Self::QD => 86,
            Self::QM => 87,
            Self::QO => 88,
            Self::QQ => 89,
            Self::QT => 90,
            Self::RP => 91,
            Self::RR => 92,
            Self::RS => 93,
            Self::SA => 94,
            Self::SB => 95,
            Self::SE => 96,
            Self::SG => 97,
            Self::SL => 98,
            Self::SP => 99,
            Self::SY => 100,
            Self::TB => 101,
            Self::TC => 102,
            Self::TI => 103,
            Self::TN => 104,
            Self::TP => 105,
            Self::TS => 106,
            Self::TT => 107,
            Self::TZ => 108,
            Self::UM => 109,
            Self::UU => 110,
            Self::VX => 111,
            Self::WA => 112,
            Self::WM => 113,
            Self::WP => 114,
            Self::WR => 115,
            Self::WS => 116,
            Self::WT => 117,
            Self::WU => 118,
            Self::WX => 119,
            Self::WY => 120,
            Self::WZ => 121,
            Self::YB => 122,
            Self::YC => 123,
            Self::YD => 124,
            Self::YE => 125,
            Self::YF => 126,
            Self::YG => 127,
            Self::YH => 128,
            Self::YI => 129,
            Self::YJ => 130,
            Self::YK => 131,
            Self::YL => 132,
            Self::YM => 133,
            Self::YO => 134,
            Self::YP => 135,
            Self::YS => 136,
            Self::YU => 137,
            Self::YW => 138,
            Self::YX => 139,
            Self::ZA => 140,
            Self::ZV => 141,
        }
    }

    /// Convert from a u8 database value back to RadioService.
    /// Returns None if the value is invalid.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::AA),
            1 => Some(Self::AB),
            2 => Some(Self::AC),
            3 => Some(Self::AD),
            4 => Some(Self::AF),
            5 => Some(Self::AH),
            6 => Some(Self::AI),
            7 => Some(Self::AL),
            8 => Some(Self::AN),
            9 => Some(Self::AR),
            10 => Some(Self::AS),
            11 => Some(Self::AT),
            12 => Some(Self::AW),
            13 => Some(Self::BA),
            14 => Some(Self::BB),
            15 => Some(Self::BC),
            16 => Some(Self::BR),
            17 => Some(Self::BS),
            18 => Some(Self::CA),
            19 => Some(Self::CB),
            20 => Some(Self::CD),
            21 => Some(Self::CE),
            22 => Some(Self::CF),
            23 => Some(Self::CG),
            24 => Some(Self::CJ),
            25 => Some(Self::CL),
            26 => Some(Self::CM),
            27 => Some(Self::CN),
            28 => Some(Self::CO),
            29 => Some(Self::CP),
            30 => Some(Self::CR),
            31 => Some(Self::CT),
            32 => Some(Self::CW),
            33 => Some(Self::CX),
            34 => Some(Self::CY),
            35 => Some(Self::CZ),
            36 => Some(Self::DV),
            37 => Some(Self::ED),
            38 => Some(Self::GB),
            39 => Some(Self::GC),
            40 => Some(Self::GE),
            41 => Some(Self::GF),
            42 => Some(Self::GI),
            43 => Some(Self::GJ),
            44 => Some(Self::GL),
            45 => Some(Self::GM),
            46 => Some(Self::GO),
            47 => Some(Self::GP),
            48 => Some(Self::GR),
            49 => Some(Self::GS),
            50 => Some(Self::GU),
            51 => Some(Self::GW),
            52 => Some(Self::GX),
            53 => Some(Self::HA),
            54 => Some(Self::HV),
            55 => Some(Self::IG),
            56 => Some(Self::IK),
            57 => Some(Self::IQ),
            58 => Some(Self::LD),
            59 => Some(Self::LN),
            60 => Some(Self::LP),
            61 => Some(Self::LS),
            62 => Some(Self::LV),
            63 => Some(Self::LW),
            64 => Some(Self::MA),
            65 => Some(Self::MC),
            66 => Some(Self::MD),
            67 => Some(Self::MG),
            68 => Some(Self::MK),
            69 => Some(Self::MM),
            70 => Some(Self::MR),
            71 => Some(Self::MS),
            72 => Some(Self::MW),
            73 => Some(Self::NC),
            74 => Some(Self::NN),
            75 => Some(Self::OW),
            76 => Some(Self::PA),
            77 => Some(Self::PB),
            78 => Some(Self::PC),
            79 => Some(Self::PE),
            80 => Some(Self::PF),
            81 => Some(Self::PK),
            82 => Some(Self::PL),
            83 => Some(Self::PM),
            84 => Some(Self::PW),
            85 => Some(Self::QA),
            86 => Some(Self::QD),
            87 => Some(Self::QM),
            88 => Some(Self::QO),
            89 => Some(Self::QQ),
            90 => Some(Self::QT),
            91 => Some(Self::RP),
            92 => Some(Self::RR),
            93 => Some(Self::RS),
            94 => Some(Self::SA),
            95 => Some(Self::SB),
            96 => Some(Self::SE),
            97 => Some(Self::SG),
            98 => Some(Self::SL),
            99 => Some(Self::SP),
            100 => Some(Self::SY),
            101 => Some(Self::TB),
            102 => Some(Self::TC),
            103 => Some(Self::TI),
            104 => Some(Self::TN),
            105 => Some(Self::TP),
            106 => Some(Self::TS),
            107 => Some(Self::TT),
            108 => Some(Self::TZ),
            109 => Some(Self::UM),
            110 => Some(Self::UU),
            111 => Some(Self::VX),
            112 => Some(Self::WA),
            113 => Some(Self::WM),
            114 => Some(Self::WP),
            115 => Some(Self::WR),
            116 => Some(Self::WS),
            117 => Some(Self::WT),
            118 => Some(Self::WU),
            119 => Some(Self::WX),
            120 => Some(Self::WY),
            121 => Some(Self::WZ),
            122 => Some(Self::YB),
            123 => Some(Self::YC),
            124 => Some(Self::YD),
            125 => Some(Self::YE),
            126 => Some(Self::YF),
            127 => Some(Self::YG),
            128 => Some(Self::YH),
            129 => Some(Self::YI),
            130 => Some(Self::YJ),
            131 => Some(Self::YK),
            132 => Some(Self::YL),
            133 => Some(Self::YM),
            134 => Some(Self::YO),
            135 => Some(Self::YP),
            136 => Some(Self::YS),
            137 => Some(Self::YU),
            138 => Some(Self::YW),
            139 => Some(Self::YX),
            140 => Some(Self::ZA),
            141 => Some(Self::ZV),
            _ => None,
        }
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

    /// Convert to a u8 for compact database storage.
    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Active => 0,
            Self::Cancelled => 1,
            Self::Expired => 2,
            Self::PendingLegalStatus => 3,
            Self::ParentStationCancelled => 4,
            Self::Terminated => 5,
            Self::TermPending => 6,
        }
    }

    /// Convert from a u8 database value back to LicenseStatus.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Active),
            1 => Some(Self::Cancelled),
            2 => Some(Self::Expired),
            3 => Some(Self::PendingLegalStatus),
            4 => Some(Self::ParentStationCancelled),
            5 => Some(Self::Terminated),
            6 => Some(Self::TermPending),
            _ => None,
        }
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

    /// Convert to a u8 for compact database storage.
    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Advanced => 0,
            Self::Extra => 1,
            Self::General => 2,
            Self::Novice => 3,
            Self::TechnicianPlus => 4,
            Self::Technician => 5,
        }
    }

    /// Convert from a u8 database value back to OperatorClass.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Advanced),
            1 => Some(Self::Extra),
            2 => Some(Self::General),
            3 => Some(Self::Novice),
            4 => Some(Self::TechnicianPlus),
            5 => Some(Self::Technician),
            _ => None,
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

    /// Convert to a u8 for compact database storage.
    pub fn to_u8(&self) -> u8 {
        match self {
            Self::TransfereeContact => 0,
            Self::LicenseeContact => 1,
            Self::AssignorContact => 2,
            Self::LesseeContact => 3,
            Self::Transferee => 4,
            Self::Licensee => 5,
            Self::Owner => 6,
            Self::Assignor => 7,
            Self::Lessee => 8,
        }
    }

    /// Convert from a u8 database value back to EntityType.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::TransfereeContact),
            1 => Some(Self::LicenseeContact),
            2 => Some(Self::AssignorContact),
            3 => Some(Self::LesseeContact),
            4 => Some(Self::Transferee),
            5 => Some(Self::Licensee),
            6 => Some(Self::Owner),
            7 => Some(Self::Assignor),
            8 => Some(Self::Lessee),
            _ => None,
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
    A2,
    A3,
    AC,
    AD,
    AG,
    AH,
    AM,
    AN,
    AP,
    AS,
    AT,
    BC,
    BD,
    BE,
    BF,
    BL,
    BO,
    BT,
    CD,
    CF,
    CG,
    CO,
    CP,
    CS,
    EC,
    EM,
    EN,
    F2,
    F3,
    F4,
    F5,
    F6,
    FA,
    FC,
    FF,
    FR,
    FS,
    FT,
    HD,
    HS,
    IA,
    IF,
    IR,
    L2,
    L3,
    L4,
    L5,
    L6,
    LA,
    LC,
    LD,
    LF,
    LH,
    LL,
    LM,
    LO,
    LS,
    MC,
    ME,
    MF,
    MH,
    MI,
    MK,
    MP,
    MW,
    O2,
    OP,
    P2,
    PA,
    PC,
    RA,
    RC,
    RE,
    RI,
    RZ,
    SC,
    SE,
    SF,
    SG,
    SH,
    SI,
    SR,
    ST,
    SV,
    TA,
    TL,
    TP,
    UA,
    VC,
}

impl RecordType {
    /// Returns the two-character code for this record type.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::A2 => "A2",
            Self::A3 => "A3",
            Self::AC => "AC",
            Self::AD => "AD",
            Self::AG => "AG",
            Self::AH => "AH",
            Self::AM => "AM",
            Self::AN => "AN",
            Self::AP => "AP",
            Self::AS => "AS",
            Self::AT => "AT",
            Self::BC => "BC",
            Self::BD => "BD",
            Self::BE => "BE",
            Self::BF => "BF",
            Self::BL => "BL",
            Self::BO => "BO",
            Self::BT => "BT",
            Self::CD => "CD",
            Self::CF => "CF",
            Self::CG => "CG",
            Self::CO => "CO",
            Self::CP => "CP",
            Self::CS => "CS",
            Self::EC => "EC",
            Self::EM => "EM",
            Self::EN => "EN",
            Self::F2 => "F2",
            Self::F3 => "F3",
            Self::F4 => "F4",
            Self::F5 => "F5",
            Self::F6 => "F6",
            Self::FA => "FA",
            Self::FC => "FC",
            Self::FF => "FF",
            Self::FR => "FR",
            Self::FS => "FS",
            Self::FT => "FT",
            Self::HD => "HD",
            Self::HS => "HS",
            Self::IA => "IA",
            Self::IF => "IF",
            Self::IR => "IR",
            Self::L2 => "L2",
            Self::L3 => "L3",
            Self::L4 => "L4",
            Self::L5 => "L5",
            Self::L6 => "L6",
            Self::LA => "LA",
            Self::LC => "LC",
            Self::LD => "LD",
            Self::LF => "LF",
            Self::LH => "LH",
            Self::LL => "LL",
            Self::LM => "LM",
            Self::LO => "LO",
            Self::LS => "LS",
            Self::MC => "MC",
            Self::ME => "ME",
            Self::MF => "MF",
            Self::MH => "MH",
            Self::MI => "MI",
            Self::MK => "MK",
            Self::MP => "MP",
            Self::MW => "MW",
            Self::O2 => "O2",
            Self::OP => "OP",
            Self::P2 => "P2",
            Self::PA => "PA",
            Self::PC => "PC",
            Self::RA => "RA",
            Self::RC => "RC",
            Self::RE => "RE",
            Self::RI => "RI",
            Self::RZ => "RZ",
            Self::SC => "SC",
            Self::SE => "SE",
            Self::SF => "SF",
            Self::SG => "SG",
            Self::SH => "SH",
            Self::SI => "SI",
            Self::SR => "SR",
            Self::ST => "ST",
            Self::SV => "SV",
            Self::TA => "TA",
            Self::TL => "TL",
            Self::TP => "TP",
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
            "A2" => Ok(Self::A2),
            "A3" => Ok(Self::A3),
            "AC" => Ok(Self::AC),
            "AD" => Ok(Self::AD),
            "AG" => Ok(Self::AG),
            "AH" => Ok(Self::AH),
            "AM" => Ok(Self::AM),
            "AN" => Ok(Self::AN),
            "AP" => Ok(Self::AP),
            "AS" => Ok(Self::AS),
            "AT" => Ok(Self::AT),
            "BC" => Ok(Self::BC),
            "BD" => Ok(Self::BD),
            "BE" => Ok(Self::BE),
            "BF" => Ok(Self::BF),
            "BL" => Ok(Self::BL),
            "BO" => Ok(Self::BO),
            "BT" => Ok(Self::BT),
            "CD" => Ok(Self::CD),
            "CF" => Ok(Self::CF),
            "CG" => Ok(Self::CG),
            "CO" => Ok(Self::CO),
            "CP" => Ok(Self::CP),
            "CS" => Ok(Self::CS),
            "EC" => Ok(Self::EC),
            "EM" => Ok(Self::EM),
            "EN" => Ok(Self::EN),
            "F2" => Ok(Self::F2),
            "F3" => Ok(Self::F3),
            "F4" => Ok(Self::F4),
            "F5" => Ok(Self::F5),
            "F6" => Ok(Self::F6),
            "FA" => Ok(Self::FA),
            "FC" => Ok(Self::FC),
            "FF" => Ok(Self::FF),
            "FR" => Ok(Self::FR),
            "FS" => Ok(Self::FS),
            "FT" => Ok(Self::FT),
            "HD" => Ok(Self::HD),
            "HS" => Ok(Self::HS),
            "IA" => Ok(Self::IA),
            "IF" => Ok(Self::IF),
            "IR" => Ok(Self::IR),
            "L2" => Ok(Self::L2),
            "L3" => Ok(Self::L3),
            "L4" => Ok(Self::L4),
            "L5" => Ok(Self::L5),
            "L6" => Ok(Self::L6),
            "LA" => Ok(Self::LA),
            "LC" => Ok(Self::LC),
            "LD" => Ok(Self::LD),
            "LF" => Ok(Self::LF),
            "LH" => Ok(Self::LH),
            "LL" => Ok(Self::LL),
            "LM" => Ok(Self::LM),
            "LO" => Ok(Self::LO),
            "LS" => Ok(Self::LS),
            "MC" => Ok(Self::MC),
            "ME" => Ok(Self::ME),
            "MF" => Ok(Self::MF),
            "MH" => Ok(Self::MH),
            "MI" => Ok(Self::MI),
            "MK" => Ok(Self::MK),
            "MP" => Ok(Self::MP),
            "MW" => Ok(Self::MW),
            "O2" => Ok(Self::O2),
            "OP" => Ok(Self::OP),
            "P2" => Ok(Self::P2),
            "PA" => Ok(Self::PA),
            "PC" => Ok(Self::PC),
            "RA" => Ok(Self::RA),
            "RC" => Ok(Self::RC),
            "RE" => Ok(Self::RE),
            "RI" => Ok(Self::RI),
            "RZ" => Ok(Self::RZ),
            "SC" => Ok(Self::SC),
            "SE" => Ok(Self::SE),
            "SF" => Ok(Self::SF),
            "SG" => Ok(Self::SG),
            "SH" => Ok(Self::SH),
            "SI" => Ok(Self::SI),
            "SR" => Ok(Self::SR),
            "ST" => Ok(Self::ST),
            "SV" => Ok(Self::SV),
            "TA" => Ok(Self::TA),
            "TL" => Ok(Self::TL),
            "TP" => Ok(Self::TP),
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
        // Test ALL RadioService variants for complete coverage
        let codes = [
            "AA", "AB", "AC", "AD", "AF", "AH", "AI", "AL", "AN", "AR", "AS", "AT", "AW", "BA",
            "BB", "BC", "BR", "BS", "CA", "CB", "CD", "CE", "CF", "CG", "CJ", "CL", "CM", "CN",
            "CO", "CP", "CR", "CT", "CW", "CX", "CY", "CZ", "DV", "ED", "GB", "GC", "GE", "GF",
            "GI", "GJ", "GL", "GM", "GO", "GP", "GR", "GS", "GU", "GW", "GX", "HA", "HV", "IG",
            "IK", "IQ", "LD", "LN", "LP", "LS", "LV", "LW", "MA", "MC", "MD", "MG", "MK", "MM",
            "MR", "MS", "MW", "NC", "NN", "OW", "PA", "PB", "PC", "PE", "PF", "PK", "PL", "PM",
            "PW", "QA", "QD", "QM", "QO", "QQ", "QT", "RP", "RR", "RS", "SA", "SB", "SE", "SG",
            "SL", "SP", "SY", "TB", "TC", "TI", "TN", "TP", "TS", "TT", "TZ", "UM", "UU", "VX",
            "WA", "WM", "WP", "WR", "WS", "WT", "WU", "WX", "WY", "WZ", "YB", "YC", "YD", "YE",
            "YF", "YG", "YH", "YI", "YJ", "YK", "YL", "YM", "YO", "YP", "YS", "YU", "YW", "YX",
            "ZA", "ZV",
        ];
        for code in codes {
            let service: RadioService = code
                .parse()
                .unwrap_or_else(|_| panic!("Failed to parse RadioService {}", code));
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
    fn test_radio_service_maritime() {
        assert!(RadioService::SA.is_maritime());
        assert!(RadioService::SB.is_maritime());
        assert!(RadioService::SE.is_maritime());
        assert!(RadioService::MA.is_maritime());
        assert!(RadioService::MC.is_maritime());
        assert!(!RadioService::HA.is_maritime());
        assert!(!RadioService::AC.is_maritime());
    }

    #[test]
    fn test_radio_service_aircraft() {
        assert!(RadioService::AC.is_aircraft());
        assert!(RadioService::AF.is_aircraft());
        assert!(!RadioService::HA.is_aircraft());
        assert!(!RadioService::SA.is_aircraft());
    }

    #[test]
    fn test_radio_service_description() {
        // Test that ALL RadioService variants have non-empty descriptions
        let all_services = [
            RadioService::AA,
            RadioService::AB,
            RadioService::AC,
            RadioService::AD,
            RadioService::AF,
            RadioService::AH,
            RadioService::AI,
            RadioService::AL,
            RadioService::AN,
            RadioService::AR,
            RadioService::AS,
            RadioService::AT,
            RadioService::AW,
            RadioService::BA,
            RadioService::BB,
            RadioService::BC,
            RadioService::BR,
            RadioService::BS,
            RadioService::CA,
            RadioService::CB,
            RadioService::CD,
            RadioService::CE,
            RadioService::CF,
            RadioService::CG,
            RadioService::CJ,
            RadioService::CL,
            RadioService::CM,
            RadioService::CN,
            RadioService::CO,
            RadioService::CP,
            RadioService::CR,
            RadioService::CT,
            RadioService::CW,
            RadioService::CX,
            RadioService::CY,
            RadioService::CZ,
            RadioService::DV,
            RadioService::ED,
            RadioService::GB,
            RadioService::GC,
            RadioService::GE,
            RadioService::GF,
            RadioService::GI,
            RadioService::GJ,
            RadioService::GL,
            RadioService::GM,
            RadioService::GO,
            RadioService::GP,
            RadioService::GR,
            RadioService::GS,
            RadioService::GU,
            RadioService::GW,
            RadioService::GX,
            RadioService::HA,
            RadioService::HV,
            RadioService::IG,
            RadioService::IK,
            RadioService::IQ,
            RadioService::LD,
            RadioService::LN,
            RadioService::LP,
            RadioService::LS,
            RadioService::LV,
            RadioService::LW,
            RadioService::MA,
            RadioService::MC,
            RadioService::MD,
            RadioService::MG,
            RadioService::MK,
            RadioService::MM,
            RadioService::MR,
            RadioService::MS,
            RadioService::MW,
            RadioService::NC,
            RadioService::NN,
            RadioService::OW,
            RadioService::PA,
            RadioService::PB,
            RadioService::PC,
            RadioService::PE,
            RadioService::PF,
            RadioService::PK,
            RadioService::PL,
            RadioService::PM,
            RadioService::PW,
            RadioService::QA,
            RadioService::QD,
            RadioService::QM,
            RadioService::QO,
            RadioService::QQ,
            RadioService::QT,
            RadioService::RP,
            RadioService::RR,
            RadioService::RS,
            RadioService::SA,
            RadioService::SB,
            RadioService::SE,
            RadioService::SG,
            RadioService::SL,
            RadioService::SP,
            RadioService::SY,
            RadioService::TB,
            RadioService::TC,
            RadioService::TI,
            RadioService::TN,
            RadioService::TP,
            RadioService::TS,
            RadioService::TT,
            RadioService::TZ,
            RadioService::UM,
            RadioService::UU,
            RadioService::VX,
            RadioService::WA,
            RadioService::WM,
            RadioService::WP,
            RadioService::WR,
            RadioService::WS,
            RadioService::WT,
            RadioService::WU,
            RadioService::WX,
            RadioService::WY,
            RadioService::WZ,
            RadioService::YB,
            RadioService::YC,
            RadioService::YD,
            RadioService::YE,
            RadioService::YF,
            RadioService::YG,
            RadioService::YH,
            RadioService::YI,
            RadioService::YJ,
            RadioService::YK,
            RadioService::YL,
            RadioService::YM,
            RadioService::YO,
            RadioService::YP,
            RadioService::YS,
            RadioService::YU,
            RadioService::YW,
            RadioService::YX,
            RadioService::ZA,
            RadioService::ZV,
        ];
        for service in all_services {
            let desc = service.description();
            assert!(
                !desc.is_empty(),
                "RadioService::{:?} has empty description",
                service
            );
        }

        // Spot check some specific descriptions
        assert_eq!(RadioService::HA.description(), "Amateur");
        assert_eq!(RadioService::HV.description(), "Vanity (Amateur)");
        assert_eq!(
            RadioService::ZA.description(),
            "General Mobile Radio (GMRS)"
        );
        assert_eq!(RadioService::AC.description(), "Aircraft");
    }

    #[test]
    fn test_radio_service_invalid() {
        let result: Result<RadioService> = "XX".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_radio_service_case_insensitive() {
        let upper: RadioService = "HA".parse().unwrap();
        let lower: RadioService = "ha".parse().unwrap();
        let mixed: RadioService = "Ha".parse().unwrap();
        assert_eq!(upper, lower);
        assert_eq!(lower, mixed);
    }

    #[test]
    fn test_operator_class_roundtrip() {
        for code in ["A", "E", "G", "N", "P", "T"] {
            let class: OperatorClass = code.parse().unwrap();
            assert_eq!(class.as_str(), code);
            assert_eq!(class.to_string(), code);
        }
    }

    #[test]
    fn test_operator_class_description() {
        assert_eq!(OperatorClass::Extra.description(), "Amateur Extra");
        assert_eq!(OperatorClass::Advanced.description(), "Advanced");
        assert_eq!(OperatorClass::General.description(), "General");
        assert_eq!(OperatorClass::Technician.description(), "Technician");
        assert_eq!(
            OperatorClass::TechnicianPlus.description(),
            "Technician Plus"
        );
        assert_eq!(OperatorClass::Novice.description(), "Novice");
    }

    #[test]
    fn test_operator_class_invalid() {
        let result: Result<OperatorClass> = "X".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_operator_class_case_insensitive() {
        let upper: OperatorClass = "E".parse().unwrap();
        let lower: OperatorClass = "e".parse().unwrap();
        assert_eq!(upper, lower);
    }

    #[test]
    fn test_license_status_roundtrip() {
        for code in ["A", "C", "E", "L", "P", "T", "X"] {
            let status: LicenseStatus = code.parse().unwrap();
            assert_eq!(status.as_str(), code);
            assert_eq!(status.to_string(), code);
        }
    }

    #[test]
    fn test_license_status_is_active() {
        assert!(LicenseStatus::Active.is_active());
        assert!(!LicenseStatus::Expired.is_active());
        assert!(!LicenseStatus::Cancelled.is_active());
        assert!(!LicenseStatus::Terminated.is_active());
        assert!(!LicenseStatus::PendingLegalStatus.is_active());
        assert!(!LicenseStatus::ParentStationCancelled.is_active());
        assert!(!LicenseStatus::TermPending.is_active());
    }

    #[test]
    fn test_license_status_invalid() {
        let result: Result<LicenseStatus> = "Z".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_license_status_case_insensitive() {
        let upper: LicenseStatus = "A".parse().unwrap();
        let lower: LicenseStatus = "a".parse().unwrap();
        assert_eq!(upper, lower);
    }

    #[test]
    fn test_application_purpose_roundtrip() {
        let codes = [
            "AA", "AM", "AR", "AU", "CA", "CB", "DC", "DU", "EX", "HA", "LC", "LE", "LM", "LN",
            "LT", "LU", "MD", "NE", "NT", "RE", "RL", "RM", "RO", "TC", "WD",
        ];
        for code in codes {
            let purpose: ApplicationPurpose = code
                .parse()
                .unwrap_or_else(|_| panic!("Failed to parse ApplicationPurpose {}", code));
            assert_eq!(purpose.as_str(), code);
            assert_eq!(purpose.to_string(), code);
        }
    }

    #[test]
    fn test_application_purpose_specific_values() {
        assert_eq!(ApplicationPurpose::AssignmentOfAuthorization.as_str(), "AA");
        assert_eq!(ApplicationPurpose::New.as_str(), "NE");
        assert_eq!(ApplicationPurpose::RenewalOnly.as_str(), "RO");
        assert_eq!(ApplicationPurpose::Modification.as_str(), "MD");
        assert_eq!(ApplicationPurpose::Withdrawal.as_str(), "WD");
    }

    #[test]
    fn test_application_purpose_invalid() {
        let result: Result<ApplicationPurpose> = "XX".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_application_purpose_case_insensitive() {
        let upper: ApplicationPurpose = "NE".parse().unwrap();
        let lower: ApplicationPurpose = "ne".parse().unwrap();
        let mixed: ApplicationPurpose = "Ne".parse().unwrap();
        assert_eq!(upper, lower);
        assert_eq!(lower, mixed);
    }

    #[test]
    fn test_application_status_roundtrip() {
        let codes = [
            "1", "2", "A", "C", "D", "E", "G", "H", "I", "J", "K", "M", "N", "P", "Q", "R", "S",
            "T", "U", "W", "X", "Y",
        ];
        for code in codes {
            let status: ApplicationStatus = code
                .parse()
                .unwrap_or_else(|_| panic!("Failed to parse ApplicationStatus {}", code));
            assert_eq!(status.as_str(), code);
            assert_eq!(status.to_string(), code);
        }
    }

    #[test]
    fn test_application_status_specific_values() {
        assert_eq!(ApplicationStatus::Submitted.as_str(), "1");
        assert_eq!(ApplicationStatus::Pending.as_str(), "2");
        assert_eq!(ApplicationStatus::Granted.as_str(), "G");
        assert_eq!(ApplicationStatus::Withdrawn.as_str(), "W");
        assert_eq!(ApplicationStatus::Terminated.as_str(), "T");
        assert_eq!(ApplicationStatus::HasProblems.as_str(), "Y");
    }

    #[test]
    fn test_application_status_invalid() {
        let result: Result<ApplicationStatus> = "Z".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_application_status_case_insensitive() {
        let upper: ApplicationStatus = "G".parse().unwrap();
        let lower: ApplicationStatus = "g".parse().unwrap();
        assert_eq!(upper, lower);
    }

    #[test]
    fn test_entity_type_roundtrip() {
        let codes = ["CE", "CL", "CR", "CS", "E", "L", "O", "R", "S"];
        for code in codes {
            let entity: EntityType = code
                .parse()
                .unwrap_or_else(|_| panic!("Failed to parse EntityType {}", code));
            assert_eq!(entity.as_str(), code);
            assert_eq!(entity.to_string(), code);
        }
    }

    #[test]
    fn test_entity_type_specific_values() {
        assert_eq!(EntityType::Licensee.as_str(), "L");
        assert_eq!(EntityType::LicenseeContact.as_str(), "CL");
        assert_eq!(EntityType::Transferee.as_str(), "E");
        assert_eq!(EntityType::TransfereeContact.as_str(), "CE");
        assert_eq!(EntityType::Owner.as_str(), "O");
        assert_eq!(EntityType::Assignor.as_str(), "R");
        assert_eq!(EntityType::Lessee.as_str(), "S");
    }

    #[test]
    fn test_entity_type_invalid() {
        let result: Result<EntityType> = "XX".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_entity_type_case_insensitive() {
        let upper: EntityType = "L".parse().unwrap();
        let lower: EntityType = "l".parse().unwrap();
        assert_eq!(upper, lower);
    }

    #[test]
    fn test_record_type_dat_filename() {
        assert_eq!(RecordType::HD.dat_filename(), "HD.dat");
        assert_eq!(RecordType::AM.dat_filename(), "AM.dat");
        assert_eq!(RecordType::EN.dat_filename(), "EN.dat");
    }

    #[test]
    fn test_all_record_types_parse() {
        let codes = [
            "A2", "A3", "AC", "AD", "AG", "AH", "AM", "AN", "AP", "AS", "AT", "BC", "BD", "BE",
            "BF", "BL", "BO", "BT", "CD", "CF", "CG", "CO", "CP", "CS", "EC", "EM", "EN", "F2",
            "F3", "F4", "F5", "F6", "FA", "FC", "FF", "FR", "FS", "FT", "HD", "HS", "IA", "IF",
            "IR", "L2", "L3", "L4", "L5", "L6", "LA", "LC", "LD", "LF", "LH", "LL", "LM", "LO",
            "LS", "MC", "ME", "MF", "MH", "MI", "MK", "MP", "MW", "O2", "OP", "P2", "PA", "PC",
            "RA", "RC", "RE", "RI", "RZ", "SC", "SE", "SF", "SG", "SH", "SI", "SR", "ST", "SV",
            "TA", "TL", "TP", "UA", "VC",
        ];

        for code in codes {
            let rt: RecordType = code
                .parse()
                .unwrap_or_else(|_| panic!("Failed to parse {}", code));
            assert_eq!(rt.as_str(), code);
            assert_eq!(rt.to_string(), code);
        }
    }

    #[test]
    fn test_record_type_invalid() {
        let result: Result<RecordType> = "XX".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_record_type_case_insensitive() {
        let upper: RecordType = "HD".parse().unwrap();
        let lower: RecordType = "hd".parse().unwrap();
        let mixed: RecordType = "Hd".parse().unwrap();
        assert_eq!(upper, lower);
        assert_eq!(lower, mixed);
    }
}
