// Copyright 2025 contributors to the GeoPlegma project.
// Originally authored by Michael Jendryke (GeoInsight GmbH, michael.jendryke@geoinsight.ai)
//
// Licenced under the Apache Licence, Version 2.0 <LICENCE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENCE-MIT or http://opensource.org/licenses/MIT>, at your
// discretion. This file may not be copied, modified, or distributed
// except according to those terms.
use crate::error::port::GeoPlegmaError;
use geo::{Point, Polygon};
use std::convert::{From, TryFrom};
use std::fmt;

#[derive(Debug)]
pub struct Zone {
    pub id: ZoneId,
    pub region: Polygon,
    pub center: Point,
    pub vertex_count: u32,
    pub children: Option<Vec<ZoneId>>,
    pub neighbors: Option<Vec<ZoneId>>,
}

#[derive(Debug)]
pub struct Zones {
    pub zones: Vec<Zone>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ZoneId {
    StrId(String),
    HexId(HexString),
    IntId(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HexString(String);

impl HexString {
    pub fn new(s: &str) -> Result<Self, String> {
        if s.len() == 16 && s.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(Self(s.to_string()))
        } else {
            Err("HexId must be exactly 16 hexadecimal characters.".to_string())
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HexString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ZoneId {
    /// 1 to 32 character ZoneID
    pub fn new_str(s: &str) -> Result<Self, GeoPlegmaError> {
        if (1..=32).contains(&s.len()) {
            Ok(ZoneId::StrId(s.to_string()))
        } else {
            Err(GeoPlegmaError::UnsupportedZoneIdFormat(format!(
                "StrId must be between 1 and 32 characters got '{}'",
                s
            )))
        }
    }

    /// Hexadecimal ZoneId
    pub fn new_hex(s: &str) -> Result<Self, GeoPlegmaError> {
        HexString::new(s)
            .map(ZoneId::HexId)
            .map_err(|e| GeoPlegmaError::InvalidHexId(e))
    }

    /// 64 bit Integer ZoneId
    pub fn new_int(id: u64) -> Self {
        ZoneId::IntId(id)
    }

    /// convert ZoneId::StrId as String
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ZoneId::StrId(s) => Some(s),
            _ => None,
        }
    }

    /// convert ZoneId::HexId as String
    pub fn as_hex(&self) -> Option<&HexString> {
        match self {
            ZoneId::HexId(h) => Some(h),
            _ => None,
        }
    }

    /// convert ZoneId::IntId as String
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            ZoneId::IntId(i) => Some(*i),
            _ => None,
        }
    }
}

impl Default for ZoneId {
    fn default() -> Self {
        ZoneId::HexId(HexString::new("0000000000000000").unwrap())
    }
}

impl fmt::Display for ZoneId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZoneId::StrId(s) => write!(f, "{s}"),
            ZoneId::HexId(s) => write!(f, "{s}"),
            ZoneId::IntId(i) => write!(f, "{i}"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RefinementLevel(i32);

impl RefinementLevel {
    pub fn new(value: i32) -> Result<Self, GeoPlegmaError> {
        if value < 0 {
            Err(GeoPlegmaError::DepthBelowZero(value))
        } else {
            Ok(Self(value))
        }
    }

    pub fn get(self) -> i32 {
        self.0
    }
}

// i32 → Depth (fallible)
impl TryFrom<i32> for RefinementLevel {
    type Error = GeoPlegmaError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        RefinementLevel::new(value)
    }
}

// u8 → Depth (infallible)
impl From<u8> for RefinementLevel {
    fn from(value: u8) -> Self {
        Self(value as i32)
    }
}

// u32 → Depth (infallible)
impl From<u32> for RefinementLevel {
    fn from(value: u32) -> Self {
        Self(value as i32)
    }
}

// Depth → i32
impl From<RefinementLevel> for i32 {
    fn from(d: RefinementLevel) -> Self {
        d.0
    }
}

// Depth → u8 (fallible)
impl TryFrom<RefinementLevel> for u8 {
    type Error = GeoPlegmaError;

    fn try_from(d: RefinementLevel) -> Result<Self, Self::Error> {
        u8::try_from(d.0).map_err(|_| GeoPlegmaError::DepthTooLarge(d))
    }
}

// Display for Depth
impl fmt::Display for RefinementLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelativeDepth(i32);

impl RelativeDepth {
    pub fn new(value: i32) -> Result<Self, GeoPlegmaError> {
        if value < 0 {
            Err(GeoPlegmaError::RelativeDepthBelowZero(value))
        } else {
            Ok(Self(value))
        }
    }

    pub fn get(self) -> i32 {
        self.0
    }
}

// i32 → RelativeDepth (fallible)
impl TryFrom<i32> for RelativeDepth {
    type Error = GeoPlegmaError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        RelativeDepth::new(value)
    }
}

// u8 → RelativeDepth (infallible)
impl From<u8> for RelativeDepth {
    fn from(value: u8) -> Self {
        Self(value as i32)
    }
}

// u32 → RelativeDepth (infallible)
impl From<u32> for RelativeDepth {
    fn from(value: u32) -> Self {
        Self(value as i32)
    }
}

// RelativeDepth → i32
impl From<RelativeDepth> for i32 {
    fn from(rd: RelativeDepth) -> Self {
        rd.0
    }
}

// RelativeDepth → u8 (fallible)
impl TryFrom<RelativeDepth> for u8 {
    type Error = GeoPlegmaError;

    fn try_from(rd: RelativeDepth) -> Result<Self, Self::Error> {
        u8::try_from(rd.0).map_err(|_| GeoPlegmaError::RelativeDepthTooLarge(rd))
    }
}

// Display for RelativeDepth
impl fmt::Display for RelativeDepth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
