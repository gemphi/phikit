//! DNA block parsing (SDNA structure definitions).

use super::types::*;
use crate::phx::{PointCloudError, Result};
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};

impl BlendFile {
    /// Parse DNA block to get structure definitions
    pub(crate) fn parse_dna(data: &[u8], header: &BlendHeader) -> Result<DnaType> {
        let mut cursor = std::io::Cursor::new(data);

        // DNA1 identifier already read
        // Skip "SDNA" identifier
        let mut sdna = [0u8; 4];
        cursor.read_exact(&mut sdna)?;

        if &sdna != b"SDNA" {
            return Err(PointCloudError::InvalidFormat(
                "Invalid SDNA magic".to_string(),
            ));
        }

        // Read NAME chunk
        let mut name_chunk = [0u8; 4];
        cursor.read_exact(&mut name_chunk)?;
        if &name_chunk != b"NAME" {
            return Err(PointCloudError::InvalidFormat(
                "Expected NAME chunk".to_string(),
            ));
        }

        let name_count = if header.little_endian {
            cursor.read_u32::<LittleEndian>()?
        } else {
            cursor.read_u32::<BigEndian>()?
        };

        let mut names = Vec::new();
        for _ in 0..name_count {
            let mut name_buf = Vec::new();
            loop {
                let mut byte = [0u8; 1];
                cursor.read_exact(&mut byte)?;
                if byte[0] == 0 {
                    break;
                }
                name_buf.push(byte[0]);
            }
            names.push(String::from_utf8_lossy(&name_buf).to_string());
        }

        // Align to 4 bytes
        let pos = cursor.position();
        let padding = (4 - (pos % 4)) % 4;
        cursor.seek(SeekFrom::Current(padding as i64))?;

        // Read TYPE chunk
        let mut type_chunk = [0u8; 4];
        cursor.read_exact(&mut type_chunk)?;

        let type_count = if header.little_endian {
            cursor.read_u32::<LittleEndian>()?
        } else {
            cursor.read_u32::<BigEndian>()?
        };

        let mut types = Vec::new();
        for _ in 0..type_count {
            let mut type_buf = Vec::new();
            loop {
                let mut byte = [0u8; 1];
                cursor.read_exact(&mut byte)?;
                if byte[0] == 0 {
                    break;
                }
                type_buf.push(byte[0]);
            }
            types.push(String::from_utf8_lossy(&type_buf).to_string());
        }

        // Align
        let pos = cursor.position();
        let padding = (4 - (pos % 4)) % 4;
        cursor.seek(SeekFrom::Current(padding as i64))?;

        // Read TLEN (type lengths)
        let mut tlen_chunk = [0u8; 4];
        cursor.read_exact(&mut tlen_chunk)?;

        let mut type_sizes = Vec::new();
        for _ in 0..type_count {
            let size = if header.little_endian {
                cursor.read_u16::<LittleEndian>()?
            } else {
                cursor.read_u16::<BigEndian>()?
            };
            type_sizes.push(size);
        }

        // Align
        let pos = cursor.position();
        let padding = (4 - (pos % 4)) % 4;
        cursor.seek(SeekFrom::Current(padding as i64))?;

        // Read STRC (structures)
        let mut strc_chunk = [0u8; 4];
        cursor.read_exact(&mut strc_chunk)?;

        let struct_count = if header.little_endian {
            cursor.read_u32::<LittleEndian>()?
        } else {
            cursor.read_u32::<BigEndian>()?
        };

        let mut structs = Vec::new();
        for _ in 0..struct_count {
            let type_idx = if header.little_endian {
                cursor.read_u16::<LittleEndian>()?
            } else {
                cursor.read_u16::<BigEndian>()?
            };

            let field_count = if header.little_endian {
                cursor.read_u16::<LittleEndian>()?
            } else {
                cursor.read_u16::<BigEndian>()?
            };

            let mut fields = Vec::new();
            for _ in 0..field_count {
                let type_idx_field = if header.little_endian {
                    cursor.read_u16::<LittleEndian>()?
                } else {
                    cursor.read_u16::<BigEndian>()?
                };

                let name_idx_field = if header.little_endian {
                    cursor.read_u16::<LittleEndian>()?
                } else {
                    cursor.read_u16::<BigEndian>()?
                };

                fields.push(DnaField {
                    type_index: type_idx_field,
                    name_index: name_idx_field,
                });
            }

            structs.push(DnaStruct {
                type_index: type_idx,
                fields,
            });
        }

        // println!("[Blend Parser] DNA parsed: {} types, {} structs", types.len(), structs.len());

        // Debug: show first 10 struct names
        let _struct_names: Vec<String> = structs
            .iter()
            .take(10)
            .map(|s| {
                if (s.type_index as usize) < types.len() {
                    types[s.type_index as usize].clone()
                } else {
                    "UNKNOWN".to_string()
                }
            })
            .collect();
        // println!("[Blend Parser] First structs: {:?}", struct_names);

        Ok(DnaType {
            names,
            types,
            type_sizes,
            structs,
        })
    }
}
