//! Top-level .blend parsing: file header and data blocks.

use super::types::*;
use crate::phx::{PointCloudError, Result};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::io::{Read, Seek};

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
        if data.len() >= 4 && data[0..4] == Self::ZSTD_MAGIC {
            let raw = zstd::stream::decode_all(std::io::Cursor::new(data)).map_err(|e| {
                PointCloudError::InvalidFormat(format!("Zstd .blend decompression failed: {e}"))
            })?;
            return Self::parse(&mut std::io::Cursor::new(raw));
        }
        if data.len() >= 2 && data[0..2] == Self::GZIP_MAGIC {
            let mut decoder = flate2::read::MultiGzDecoder::new(data);
            let mut raw = Vec::new();
            decoder.read_to_end(&mut raw).map_err(|e| {
                PointCloudError::InvalidFormat(format!("gzip .blend decompression failed: {e}"))
            })?;
            return Self::parse(&mut std::io::Cursor::new(raw));
        }
        let mut cursor = std::io::Cursor::new(data);
        Self::parse(&mut cursor)
    }

    /// Parse a .blend file from a reader
    pub fn parse<R: Read + Seek>(reader: &mut R) -> Result<Self> {
        let header = Self::read_header(reader)?;
        // println!("[Blend Parser] Version: {}, Pointer size: {}, Endian: {}",
        //     header.version, header.pointer_size,
        //     if header.little_endian { "little" } else { "big" });

        let mut blocks = Vec::new();
        let mut dna = None;

        // Read all blocks until ENDB
        loop {
            let block = Self::read_block(reader, &header)?;
            if &block.code == ENDB {
                // println!("[Blend Parser] Found ENDB block, stopping");
                break;
            }

            if &block.code == DNA1 {
                // println!("[Blend Parser] Found DNA block, size: {}", block.size);
                dna = Some(Self::parse_dna(&block.data, &header)?);
            }

            blocks.push(block);
        }

        // println!("[Blend Parser] Parsed {} blocks", blocks.len());

        Ok(BlendFile {
            header,
            blocks,
            dna,
        })
    }

    /// Read the .blend file header
    pub(crate) fn read_header<R: Read>(reader: &mut R) -> Result<BlendHeader> {
        let mut magic = [0u8; 7];
        reader.read_exact(&mut magic)?;

        if &magic != BLEND_MAGIC {
            return Err(PointCloudError::InvalidFormat(format!(
                "Invalid .blend magic: {:?}",
                magic
            )));
        }

        // Pointer size (underscore = 4 bytes, dash = 8 bytes)
        let mut ptr_size_buf = [0u8; 1];
        reader.read_exact(&mut ptr_size_buf)?;
        let pointer_size = if ptr_size_buf[0] == b'_' { 4 } else { 8 };

        // Endianness (v = little, V = big)
        let mut endian_buf = [0u8; 1];
        reader.read_exact(&mut endian_buf)?;
        let little_endian = endian_buf[0] == b'v';

        // Version string (3 bytes)
        let mut version_buf = [0u8; 3];
        reader.read_exact(&mut version_buf)?;
        let version = String::from_utf8_lossy(&version_buf).to_string();
        let version_number = version.parse::<u32>().unwrap_or(0);

        Ok(BlendHeader {
            version,
            pointer_size,
            little_endian,
            version_number,
        })
    }

    /// Read a data block
    pub(crate) fn read_block<R: Read + Seek>(
        reader: &mut R,
        header: &BlendHeader,
    ) -> Result<BlendBlock> {
        let mut code = [0u8; 4];
        reader.read_exact(&mut code)?;
        let code_str = String::from_utf8_lossy(&code)
            .trim_end_matches('\0')
            .trim_end()
            .to_string();

        // Read size
        let size = if header.little_endian {
            reader.read_u32::<LittleEndian>()?
        } else {
            reader.read_u32::<BigEndian>()?
        };

        // Read old memory address (pointer size)
        let old_address = if header.pointer_size == 4 {
            if header.little_endian {
                reader.read_u32::<LittleEndian>()? as u64
            } else {
                reader.read_u32::<BigEndian>()? as u64
            }
        } else {
            if header.little_endian {
                reader.read_u64::<LittleEndian>()?
            } else {
                reader.read_u64::<BigEndian>()?
            }
        };

        // Read SDNA index and count
        let sdna_index = if header.little_endian {
            reader.read_u32::<LittleEndian>()?
        } else {
            reader.read_u32::<BigEndian>()?
        };

        let count = if header.little_endian {
            reader.read_u32::<LittleEndian>()?
        } else {
            reader.read_u32::<BigEndian>()?
        };

        // Sanity check: block sizes should be reasonable (max 512 MB)
        const MAX_BLOCK_SIZE: u32 = 512 * 1024 * 1024;
        if size > MAX_BLOCK_SIZE {
            let offset = reader.stream_position().unwrap_or(0);
            return Err(PointCloudError::InvalidFormat(format!(
                "Block '{}' at offset {} reports unrealistic size {} bytes ({:.1} GB). \
                     This likely indicates a misaligned read or unsupported .blend format.",
                code_str,
                offset,
                size,
                size as f64 / 1_073_741_824.0
            )));
        }

        // Read block data
        let data_offset = reader.stream_position()?;
        let mut data = vec![0u8; size as usize];
        if let Err(e) = reader.read_exact(&mut data) {
            println!(
                "[Blend Parser] Failed to read block '{}' (size {} bytes) at offset {}: {}",
                code_str, size, data_offset, e
            );
            return Err(e.into());
        }

        // NOTE: .blend files do NOT use alignment padding between blocks.
        // The next BHEAD starts immediately after the block data.

        Ok(BlendBlock {
            code,
            code_str,
            size,
            old_address,
            sdna_index,
            count,
            data,
        })
    }
}
