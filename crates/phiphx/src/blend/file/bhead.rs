//! Block headers — `BHead4`, `SmallBHead8`, `LargeBHead8`
//! (`blenloader_core/BLO_core_bhead.hh`), chosen by `BlendHeader::bhead_type`.

use crate::blend::file::types::*;
use crate::{PointCloudError, Result};
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use std::io::{Read, Seek};

impl BlendFile {
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
    }}
