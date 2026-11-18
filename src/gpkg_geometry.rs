//! Minimal GeoPackageBinary header validation and WKB payload extraction.

use std::fmt;

const HEADER_LEN: usize = 8;
const MAGIC: &[u8; 2] = b"GP";
const EMPTY_FLAG: u8 = 0x10;

/// Byte order used for the SRS identifier in a GeoPackageBinary header.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ByteOrder {
    BigEndian,
    LittleEndian,
}

/// The validated GeoPackageBinary header and its WKB payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GeoPackageGeometry<'a> {
    pub byte_order: ByteOrder,
    pub srs_id: i32,
    pub empty: bool,
    pub wkb: &'a [u8],
}

/// Why a blob could not be decoded as GeoPackageBinary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeError {
    TruncatedHeader,
    InvalidMagic,
    UnsupportedVersion(u8),
    ReservedFlags,
    InvalidEnvelopeIndicator,
    TruncatedEnvelope,
    EmptyWkb,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TruncatedHeader => write!(f, "GeoPackageBinary header is truncated"),
            Self::InvalidMagic => write!(f, "invalid GeoPackageBinary magic"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported GeoPackageBinary version: {version}")
            }
            Self::ReservedFlags => write!(f, "GeoPackageBinary reserved flags are set"),
            Self::InvalidEnvelopeIndicator => {
                write!(f, "invalid GeoPackageBinary envelope indicator")
            }
            Self::TruncatedEnvelope => write!(f, "GeoPackageBinary envelope is truncated"),
            Self::EmptyWkb => write!(f, "GeoPackageBinary WKB payload is empty"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Validate a GeoPackageBinary header and return its bounded WKB payload.
pub fn decode(blob: &[u8]) -> Result<GeoPackageGeometry<'_>, DecodeError> {
    if blob.len() < HEADER_LEN {
        return Err(DecodeError::TruncatedHeader);
    }
    if &blob[..2] != MAGIC {
        return Err(DecodeError::InvalidMagic);
    }
    if blob[2] != 0 {
        return Err(DecodeError::UnsupportedVersion(blob[2]));
    }

    let flags = blob[3];
    if flags & 0xe0 != 0 {
        return Err(DecodeError::ReservedFlags);
    }
    let envelope_indicator = (flags >> 1) & 0x07;
    let envelope_len = match envelope_indicator {
        0 => 0,
        1 => 32,
        2 | 3 => 48,
        4 => 64,
        _ => return Err(DecodeError::InvalidEnvelopeIndicator),
    };
    let byte_order = if flags & 1 == 0 {
        ByteOrder::BigEndian
    } else {
        ByteOrder::LittleEndian
    };
    let srs_id = match byte_order {
        ByteOrder::BigEndian => i32::from_be_bytes(blob[4..8].try_into().unwrap()),
        ByteOrder::LittleEndian => i32::from_le_bytes(blob[4..8].try_into().unwrap()),
    };

    let wkb_offset = HEADER_LEN
        .checked_add(envelope_len)
        .ok_or(DecodeError::TruncatedEnvelope)?;
    if blob.len() < wkb_offset {
        return Err(DecodeError::TruncatedEnvelope);
    }
    let wkb = &blob[wkb_offset..];
    if wkb.is_empty() {
        return Err(DecodeError::EmptyWkb);
    }

    Ok(GeoPackageGeometry {
        byte_order,
        srs_id,
        empty: flags & EMPTY_FLAG != 0,
        wkb,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(flags: u8, srs_id: i32) -> Vec<u8> {
        let mut bytes = b"GP".to_vec();
        bytes.push(0);
        bytes.push(flags);
        bytes.extend_from_slice(&srs_id.to_le_bytes());
        bytes
    }

    #[test]
    fn decodes_little_and_big_endian_srs_and_extracts_wkb() {
        let mut little = header(1, 4326);
        little.extend_from_slice(&[1, 2, 3]);
        let decoded = decode(&little).unwrap();
        assert_eq!(decoded.byte_order, ByteOrder::LittleEndian);
        assert_eq!(decoded.srs_id, 4326);
        assert_eq!(decoded.wkb, &[1, 2, 3]);

        let mut big = b"GP\0\0".to_vec();
        big.extend_from_slice(&4326_i32.to_be_bytes());
        big.extend_from_slice(&[4, 5]);
        let decoded = decode(&big).unwrap();
        assert_eq!(decoded.byte_order, ByteOrder::BigEndian);
        assert_eq!(decoded.srs_id, 4326);
        assert_eq!(decoded.wkb, &[4, 5]);
    }

    #[test]
    fn skips_each_valid_envelope_size() {
        for (indicator, length) in [(1, 32), (2, 48), (3, 48), (4, 64)] {
            let mut bytes = header(1 | (indicator << 1), 0);
            bytes.resize(HEADER_LEN + length, 0);
            bytes.push(0xaa);
            assert_eq!(decode(&bytes).unwrap().wkb, &[0xaa]);
        }
    }

    #[test]
    fn reports_empty_flag_and_still_returns_wkb() {
        let mut bytes = header(1 | EMPTY_FLAG, 0);
        bytes.push(0);
        let decoded = decode(&bytes).unwrap();
        assert!(decoded.empty);
        assert_eq!(decoded.wkb, &[0]);
    }

    #[test]
    fn rejects_invalid_and_truncated_values() {
        assert_eq!(decode(b"GP\0").unwrap_err(), DecodeError::TruncatedHeader);
        assert_eq!(
            decode(b"XX\0\0\0\0\0\0").unwrap_err(),
            DecodeError::InvalidMagic
        );
        assert_eq!(
            decode(b"GP\x01\0\0\0\0\0").unwrap_err(),
            DecodeError::UnsupportedVersion(1)
        );
        assert_eq!(
            decode(b"GP\0\x40\0\0\0\0").unwrap_err(),
            DecodeError::ReservedFlags
        );
        assert_eq!(
            decode(b"GP\0\x0a\0\0\0\0").unwrap_err(),
            DecodeError::InvalidEnvelopeIndicator
        );
        assert_eq!(
            decode(&header(1 | (4 << 1), 0)).unwrap_err(),
            DecodeError::TruncatedEnvelope
        );
        assert_eq!(decode(&header(1, 0)).unwrap_err(), DecodeError::EmptyWkb);
    }
}
