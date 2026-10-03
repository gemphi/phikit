//! `.blend` file header — port of `BLO_readfile_blender_header_decode`
//! (`blenloader_core/intern/blo_core_blend_header.cc`): the legacy 12-byte
//! `BLENDER-v305` header and the format-version-1 17-byte
//! `BLENDER17-01v0502` header Blender 5.0+ writes by default.

use crate::blend::file::types::*;
use crate::{PointCloudError, Result};
use std::io::Read;

/// Legacy header: `BLENDER` + ptr-size char + endian char + 3-digit version.
const HEADER_LEN_V0: usize = 12;
/// Format-version-1 header: `BLENDER` + "17" + `-` + "01" + `v` + 4-digit version.
const HEADER_LEN_V1: usize = 17;

impl BlendFile {
    /// Decode the `BLENDER` file header — port of
    /// `BLO_readfile_blender_header_decode`.
    pub(crate) fn read_header<R: Read>(reader: &mut R) -> Result<BlendHeader> {
        let mut h = [0u8; HEADER_LEN_V1];
        reader.read_exact(&mut h[..HEADER_LEN_V0])?;
        if &h[..7] != BLEND_MAGIC {
            return Err(PointCloudError::InvalidFormat(format!(
                "Invalid .blend magic: {:?}",
                &h[..7]
            )));
        }

        let is_digit = |b: u8| b.is_ascii_digit();

        // ---- Legacy header: BLENDER[_-][vV]NNN --------------------------------
        if h[7] == b'_' || h[7] == b'-' {
            let pointer_size = if h[7] == b'_' { 4 } else { 8 };
            let little_endian = match h[8] {
                b'v' => true,
                b'V' => false,
                other => {
                    return Err(PointCloudError::InvalidFormat(format!(
                        "Unknown .blend endian marker {:?}",
                        other as char
                    )))
                }
            };
            if !(is_digit(h[9]) && is_digit(h[10]) && is_digit(h[11])) {
                return Err(PointCloudError::InvalidFormat(
                    "Unknown .blend header (legacy version digits)".into(),
                ));
            }
            let version = String::from_utf8_lossy(&h[9..12]).to_string();
            let version_number = version.parse::<u32>().unwrap_or(0);
            return Ok(BlendHeader {
                version,
                pointer_size,
                little_endian,
                version_number,
                file_format_version: 0,
            });
        }

        // ---- Format version 1 (Blender 5.0+): BLENDER17-01vNNNN --------------
        if !(is_digit(h[7]) && is_digit(h[8])) {
            return Err(PointCloudError::InvalidFormat(
                "Unknown .blend header (possibly a newer Blender file format)".into(),
            ));
        }
        let header_size = ((h[7] - b'0') as usize) * 10 + (h[8] - b'0') as usize;
        if header_size != HEADER_LEN_V1 {
            return Err(PointCloudError::InvalidFormat(format!(
                "Unsupported .blend header size {header_size} (newer Blender file format?)"
            )));
        }
        reader.read_exact(&mut h[HEADER_LEN_V0..HEADER_LEN_V1])?;
        if h[9] != b'-' {
            return Err(PointCloudError::InvalidFormat(
                "Unknown .blend header (expected 8-byte pointer marker)".into(),
            ));
        }
        if !(is_digit(h[10]) && is_digit(h[11])) {
            return Err(PointCloudError::InvalidFormat(
                "Unknown .blend header (format version digits)".into(),
            ));
        }
        let file_format_version = ((h[10] - b'0') as u8) * 10 + (h[11] - b'0') as u8;
        if file_format_version != 1 {
            return Err(PointCloudError::InvalidFormat(format!(
                "Unsupported .blend file format version {file_format_version}"
            )));
        }
        if h[12] != b'v' {
            return Err(PointCloudError::InvalidFormat(
                "Unknown .blend header (expected little-endian marker)".into(),
            ));
        }
        if !(is_digit(h[13]) && is_digit(h[14]) && is_digit(h[15]) && is_digit(h[16])) {
            return Err(PointCloudError::InvalidFormat(
                "Unknown .blend header (version digits)".into(),
            ));
        }
        let version = String::from_utf8_lossy(&h[13..17]).to_string();
        let version_number = version.parse::<u32>().unwrap_or(0);
        Ok(BlendHeader {
            version,
            pointer_size: 8,
            little_endian: true,
            version_number,
            file_format_version: 1,
        })
    }

}
