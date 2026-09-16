//! Top-level .blend parsing: file header and data blocks.
//!
//! Mirrors Blender's `blenloader_core`:
//! - `BLO_readfile_blender_header_decode` (`blo_core_blend_header.cc`) — both
//!   the legacy 12-byte header (`BLENDER-v305`) and the format-version-1
//!   17-byte header (`BLENDER17-01v0502`) that Blender 5.0+ writes by default.
//! - `BLO_core_bhead.hh` — the three block-header layouts `BHead4`,
//!   `SmallBHead8` and `LargeBHead8`, selected from the file header.
//! - `OldNewMap` — every block's saved address is indexed in a hash map once,
//!   so pointer dereferences are O(1) instead of a linear scan per pointer.

use super::types::*;
use crate::{PointCloudError, Result};
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use std::collections::HashMap;
use std::io::{Read, Seek};

/// Legacy header: `BLENDER` + ptr-size char + endian char + 3-digit version.
const HEADER_LEN_V0: usize = 12;
/// Format-version-1 header: `BLENDER` + "17" + `-` + "01" + `v` + 4-digit version.
const HEADER_LEN_V1: usize = 17;

impl BlendFile {
    /// Zstandard frame magic — Blender 4.5+ writes `.blend` files compressed
    /// with Zstd; `blenloader` transparently decompresses them before reading
    /// the `BLENDER` header (see `BLEN_blendfile_read*/filelist_readjob`).
    const ZSTD_MAGIC: [u8; 4] = [0x28, 0xB5, 0x2F, 0xFD];
    /// Gzip magic — older Blender releases (≤ 4.4) compress saves with gzip.
    const GZIP_MAGIC: [u8; 2] = [0x1F, 0x8B];

    /// Parse a `.blend` file from raw bytes, decompressing a Zstd or gzip
    /// stream when detected (matching Blender's loader behaviour).
    pub fn parse_bytes(data: &[u8]) -> Result<Self> {
        Self::parse_bytes_with_progress(data, &mut |_, _| {})
    }

    /// [`Self::parse_bytes`] with a `(blocks_read, bytes_consumed)` callback.
    pub fn parse_bytes_with_progress(
        data: &[u8],
        progress: &mut dyn FnMut(usize, u64),
    ) -> Result<Self> {
        if data.len() >= 4 && data[0..4] == Self::ZSTD_MAGIC {
            let raw = zstd::stream::decode_all(std::io::Cursor::new(data)).map_err(|e| {
                PointCloudError::InvalidFormat(format!("Zstd .blend decompression failed: {e}"))
            })?;
            return Self::parse_with_progress(&mut std::io::Cursor::new(raw), progress);
        }
        if data.len() >= 2 && data[0..2] == Self::GZIP_MAGIC {
            let mut decoder = flate2::read::MultiGzDecoder::new(data);
            let mut raw = Vec::new();
            decoder.read_to_end(&mut raw).map_err(|e| {
                PointCloudError::InvalidFormat(format!("gzip .blend decompression failed: {e}"))
            })?;
            return Self::parse_with_progress(&mut std::io::Cursor::new(raw), progress);
        }
        let mut cursor = std::io::Cursor::new(data);
        Self::parse_with_progress(&mut cursor, progress)
    }

    /// Parse a .blend file from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        Self::parse_with_progress(reader, &mut |_, _| {})
    }

    /// Parse a .blend file, reporting `(blocks_read, bytes_consumed)` after
    /// every block so a caller can drive a real progress bar.
    pub fn parse_with_progress<R: Read + Seek>(
        reader: &mut R,
        progress: &mut dyn FnMut(usize, u64),
    ) -> Result<Self> {
        let header = Self::read_header(reader)?;

        let mut blocks: Vec<BlendBlock> = Vec::new();
        let mut addr_index: HashMap<u64, usize> = HashMap::new();
        let mut dna = None;

        // Read all blocks until ENDB
        loop {
            let block = Self::read_block(reader, &header)?;
            if &block.code == ENDB {
                break;
            }

            if &block.code == DNA1 {
                dna = Some(Self::parse_dna(&block.data, &header)?);
            }

            // OldNewMap: first writer wins, matching the previous `.find`
            // semantics (Blender's `oldnewmap_insert` likewise keeps one
            // entry per old address).
            if block.old_address != 0 {
                addr_index.entry(block.old_address).or_insert(blocks.len());
            }
            blocks.push(block);

            if blocks.len() % 256 == 0 {
                progress(blocks.len(), reader.stream_position().unwrap_or(0));
            }
        }
        progress(blocks.len(), reader.stream_position().unwrap_or(0));

        Ok(BlendFile {
            header,
            blocks,
            dna,
            addr_index,
            cancel_requested: std::sync::atomic::AtomicBool::new(false),
        })
    }

    /// O(1) block lookup by saved address (Blender `oldnewmap_lookup`).
    #[inline]
    pub fn block_at(&self, old_address: u64) -> Option<&BlendBlock> {
        if old_address == 0 {
            return None;
        }
        self.addr_index
            .get(&old_address)
            .map(|&i| &self.blocks[i])
    }

    /// Block lookup by saved address that must also carry block code `code`
    /// (e.g. `"ME"`, `"MA"`). Falls back to a scan only when the indexed
    /// block has a different code (DATA blocks may legitimately reuse an
    /// address inside another ID).
    pub fn block_at_code(&self, old_address: u64, code: &str) -> Option<&BlendBlock> {
        match self.block_at(old_address) {
            Some(b) if b.code_str == code => Some(b),
            Some(_) => self
                .blocks
                .iter()
                .find(|b| b.old_address == old_address && b.code_str == code),
            None => None,
        }
    }

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

    /// Read one block header + payload, using the `BHead` layout implied by
    /// the file header (`BlenderHeader::bhead_type`).
    pub(crate) fn read_block<R: Read + Seek>(
        reader: &mut R,
        header: &BlendHeader,
    ) -> Result<BlendBlock> {
        let le = header.little_endian;
        let rd_i32 = |b: &[u8]| -> i32 {
            if le {
                LittleEndian::read_i32(b)
            } else {
                BigEndian::read_i32(b)
            }
        };
        let rd_u32 = |b: &[u8]| -> u32 {
            if le {
                LittleEndian::read_u32(b)
            } else {
                BigEndian::read_u32(b)
            }
        };
        let rd_u64 = |b: &[u8]| -> u64 {
            if le {
                LittleEndian::read_u64(b)
            } else {
                BigEndian::read_u64(b)
            }
        };
        let rd_i64 = |b: &[u8]| -> i64 {
            if le {
                LittleEndian::read_i64(b)
            } else {
                BigEndian::read_i64(b)
            }
        };

        let mut code = [0u8; 4];
        let (size, old_address, sdna_index, count): (i64, u64, u32, i64) =
            match header.bhead_type() {
                // BHead4 { int code, len; uint old; int SDNAnr, nr; }
                BHeadType::BHead4 => {
                    let mut buf = [0u8; 20];
                    reader.read_exact(&mut buf)?;
                    code.copy_from_slice(&buf[0..4]);
                    (
                        rd_i32(&buf[4..8]) as i64,
                        rd_u32(&buf[8..12]) as u64,
                        rd_u32(&buf[12..16]),
                        rd_i32(&buf[16..20]) as i64,
                    )
                }
                // SmallBHead8 { int code, len; uint64 old; int SDNAnr, nr; }
                BHeadType::SmallBHead8 => {
                    let mut buf = [0u8; 24];
                    reader.read_exact(&mut buf)?;
                    code.copy_from_slice(&buf[0..4]);
                    (
                        rd_i32(&buf[4..8]) as i64,
                        rd_u64(&buf[8..16]),
                        rd_u32(&buf[16..20]),
                        rd_i32(&buf[20..24]) as i64,
                    )
                }
                // LargeBHead8 { int code; int SDNAnr; uint64 old; int64 len; int64 nr; }
                BHeadType::LargeBHead8 => {
                    let mut buf = [0u8; 32];
                    reader.read_exact(&mut buf)?;
                    code.copy_from_slice(&buf[0..4]);
                    (
                        rd_i64(&buf[16..24]),
                        rd_u64(&buf[8..16]),
                        rd_u32(&buf[4..8]),
                        rd_i64(&buf[24..32]),
                    )
                }
            };

        let code_str = String::from_utf8_lossy(&code)
            .trim_end_matches('\0')
            .trim_end()
            .to_string();

        // Sanity limit, as before: a block larger than this is a misaligned
        // read rather than real data.
        const MAX_BLOCK_SIZE: i64 = 4 * 1024 * 1024 * 1024;
        if size < 0 || size > MAX_BLOCK_SIZE || count < 0 {
            let offset = reader.stream_position().unwrap_or(0);
            return Err(PointCloudError::InvalidFormat(format!(
                "Block '{}' at offset {} reports unrealistic size {} bytes / count {}. \
                 This likely indicates a misaligned read or unsupported .blend format.",
                code_str, offset, size, count
            )));
        }

        let data_offset = reader.stream_position()?;
        let mut data = vec![0u8; size as usize];
        if let Err(e) = reader.read_exact(&mut data) {
            eprintln!(
                "[Blend Parser] Failed to read block '{}' (size {} bytes) at offset {}: {}",
                code_str, size, data_offset, e
            );
            return Err(e.into());
        }

        Ok(BlendBlock {
            code,
            code_str,
            size: size as u64,
            old_address,
            sdna_index,
            count: count as u64,
            data,
        })
    }
}

#[cfg(test)]
mod header_tests {
    use super::*;

    #[test]
    fn legacy_header_decodes() {
        let mut c = std::io::Cursor::new(b"BLENDER-v305".to_vec());
        let h = BlendFile::read_header(&mut c).unwrap();
        assert_eq!(h.pointer_size, 8);
        assert!(h.little_endian);
        assert_eq!(h.version_number, 305);
        assert_eq!(h.file_format_version, 0);
        assert!(matches!(h.bhead_type(), BHeadType::SmallBHead8));
    }

    #[test]
    fn legacy_32bit_header_decodes() {
        let mut c = std::io::Cursor::new(b"BLENDER_V249".to_vec());
        let h = BlendFile::read_header(&mut c).unwrap();
        assert_eq!(h.pointer_size, 4);
        assert!(!h.little_endian);
        assert!(matches!(h.bhead_type(), BHeadType::BHead4));
    }

    #[test]
    fn v1_header_decodes() {
        let mut c = std::io::Cursor::new(b"BLENDER17-01v0502".to_vec());
        let h = BlendFile::read_header(&mut c).unwrap();
        assert_eq!(h.pointer_size, 8);
        assert!(h.little_endian);
        assert_eq!(h.version_number, 502);
        assert_eq!(h.file_format_version, 1);
        assert!(matches!(h.bhead_type(), BHeadType::LargeBHead8));
    }

    #[test]
    fn unknown_header_is_rejected() {
        let mut c = std::io::Cursor::new(b"BLENDER99-01v0502".to_vec());
        assert!(BlendFile::read_header(&mut c).is_err());
    }

    #[test]
    fn large_bhead8_block_roundtrip() {
        // Header + one LargeBHead8 'DATA' block of 8 bytes + ENDB.
        let mut bytes = b"BLENDER17-01v0502".to_vec();
        bytes.extend_from_slice(b"DATA");
        bytes.extend_from_slice(&7u32.to_le_bytes()); // SDNAnr
        bytes.extend_from_slice(&0xDEADBEEFu64.to_le_bytes()); // old
        bytes.extend_from_slice(&8i64.to_le_bytes()); // len
        bytes.extend_from_slice(&1i64.to_le_bytes()); // nr
        bytes.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8]);
        bytes.extend_from_slice(b"ENDB");
        bytes.extend_from_slice(&[0u8; 28]);
        let f = BlendFile::parse_bytes(&bytes).unwrap();
        assert_eq!(f.blocks.len(), 1);
        let b = f.block_at(0xDEADBEEF).unwrap();
        assert_eq!(b.code_str, "DATA");
        assert_eq!(b.sdna_index, 7);
        assert_eq!(b.data, vec![1, 2, 3, 4, 5, 6, 7, 8]);
    }
}
