//! Custom PHX binary format exporter and helper base64 utilities.

use crate::phx::{PointCloud, Result};
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

/// Implements custom binary format and base64 helper operations.
pub struct BinaryExporter;

impl BinaryExporter {
    /// Export to custom binary format (returns base64 encoded string)
    pub fn export_binary(
        cloud: &PointCloud,
        include_normals: bool,
        include_colors: bool,
    ) -> Result<String> {
        let mut buffer: Vec<u8> = Vec::new();

        buffer.write_all(b"PHX\x00")?;
        buffer.write_u32::<LittleEndian>(1)?;

        let flags: u32 =
            (if include_normals { 1 } else { 0 }) | (if include_colors { 2 } else { 0 });
        buffer.write_u32::<LittleEndian>(flags)?;
        buffer.write_u32::<LittleEndian>(cloud.vertex_count() as u32)?;

        for vertex in cloud.vertices() {
            buffer.write_f32::<LittleEndian>(vertex.position[0])?;
            buffer.write_f32::<LittleEndian>(vertex.position[1])?;
            buffer.write_f32::<LittleEndian>(vertex.position[2])?;

            if include_normals {
                if let Some(n) = vertex.normal {
                    buffer.write_f32::<LittleEndian>(n[0])?;
                    buffer.write_f32::<LittleEndian>(n[1])?;
                    buffer.write_f32::<LittleEndian>(n[2])?;
                } else {
                    buffer.write_f32::<LittleEndian>(0.0)?;
                    buffer.write_f32::<LittleEndian>(0.0)?;
                    buffer.write_f32::<LittleEndian>(0.0)?;
                }
            }

            if include_colors {
                if let Some(c) = vertex.color {
                    buffer.write_u8(c[0])?;
                    buffer.write_u8(c[1])?;
                    buffer.write_u8(c[2])?;
                    buffer.write_u8(c[3])?;
                } else {
                    buffer.write_u8(255)?;
                    buffer.write_u8(255)?;
                    buffer.write_u8(255)?;
                    buffer.write_u8(255)?;
                }
            }
        }

        Ok(Self::base64_encode(&buffer))
    }

    /// Export to binary bytes
    pub fn export_binary_to_bytes(
        cloud: &PointCloud,
        include_normals: bool,
        include_colors: bool,
    ) -> Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();

        buffer.write_all(b"PHX\x00")?;
        buffer.write_u32::<LittleEndian>(1)?;

        let flags: u32 =
            (if include_normals { 1 } else { 0 }) | (if include_colors { 2 } else { 0 });
        buffer.write_u32::<LittleEndian>(flags)?;
        buffer.write_u32::<LittleEndian>(cloud.vertex_count() as u32)?;

        for vertex in cloud.vertices() {
            buffer.write_f32::<LittleEndian>(vertex.position[0])?;
            buffer.write_f32::<LittleEndian>(vertex.position[1])?;
            buffer.write_f32::<LittleEndian>(vertex.position[2])?;

            if include_normals {
                let n = vertex.normal.unwrap_or([0.0, 0.0, 0.0]);
                buffer.write_f32::<LittleEndian>(n[0])?;
                buffer.write_f32::<LittleEndian>(n[1])?;
                buffer.write_f32::<LittleEndian>(n[2])?;
            }

            if include_colors {
                let c = vertex.color.unwrap_or([255, 255, 255, 255]);
                buffer.write_u8(c[0])?;
                buffer.write_u8(c[1])?;
                buffer.write_u8(c[2])?;
                buffer.write_u8(c[3])?;
            }
        }

        Ok(buffer)
    }

    /// Simple base64 encoding (without external crate)
    pub fn base64_encode(data: &[u8]) -> String {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut result = String::with_capacity((data.len() + 2) / 3 * 4);

        for chunk in data.chunks(3) {
            let b = match chunk.len() {
                1 => [chunk[0], 0, 0],
                2 => [chunk[0], chunk[1], 0],
                3 => [chunk[0], chunk[1], chunk[2]],
                _ => unreachable!(),
            };

            let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32);

            result.push(ALPHABET[((n >> 18) & 0x3F) as usize] as char);
            result.push(ALPHABET[((n >> 12) & 0x3F) as usize] as char);

            if chunk.len() > 1 {
                result.push(ALPHABET[((n >> 6) & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }

            if chunk.len() > 2 {
                result.push(ALPHABET[(n & 0x3F) as usize] as char);
            } else {
                result.push('=');
            }
        }

        result
    }

    /// Simple base64 decoding (without external crate)
    pub fn base64_decode(data: &str, expected_len: usize) -> Result<Vec<u8>> {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let data: String = data.chars().filter(|c| !c.is_whitespace()).collect();
        let data = data.trim_end_matches('=');

        let mut result = Vec::with_capacity(data.len() * 3 / 4);

        for chunk in data.as_bytes().chunks(4) {
            let mut n: u32 = 0;
            let mut valid_chars = 0;

            for (i, &c) in chunk.iter().enumerate() {
                let c = match c {
                    b'-' => b'+',
                    b'_' => b'/',
                    _ => c,
                };
                if let Some(pos) = ALPHABET.iter().position(|&x| x == c) {
                    n |= (pos as u32) << (18 - i * 6);
                    valid_chars += 1;
                }
            }

            if valid_chars > 1 {
                result.push((n >> 16) as u8);
            }
            if valid_chars > 2 {
                result.push(((n >> 8) & 0xFF) as u8);
            }
            if valid_chars > 3 {
                result.push((n & 0xFF) as u8);
            }
        }

        result.truncate(expected_len);
        while result.len() < expected_len {
            result.push(0);
        }

        Ok(result)
    }
}
