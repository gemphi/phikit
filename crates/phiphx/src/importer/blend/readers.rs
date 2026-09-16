//! Low-level little-endian byte readers.

use super::types::*;

impl BlendFile {
    /// Read u64 from bytes (little endian)
    pub(crate) fn read_u64(&self, bytes: &[u8]) -> u64 {
        if bytes.len() < 8 {
            return 0;
        }
        (bytes[0] as u64)
            | ((bytes[1] as u64) << 8)
            | ((bytes[2] as u64) << 16)
            | ((bytes[3] as u64) << 24)
            | ((bytes[4] as u64) << 32)
            | ((bytes[5] as u64) << 40)
            | ((bytes[6] as u64) << 48)
            | ((bytes[7] as u64) << 56)
    }

    /// Read f32 from bytes (little endian)
    pub(crate) fn read_f32(&self, bytes: &[u8]) -> f32 {
        if bytes.len() < 4 {
            return 0.0;
        }
        let bits = (bytes[0] as u32)
            | ((bytes[1] as u32) << 8)
            | ((bytes[2] as u32) << 16)
            | ((bytes[3] as u32) << 24);
        f32::from_bits(bits)
    }

    /// Read u16 from bytes (little endian)
    pub(crate) fn read_u16(&self, bytes: &[u8]) -> u16 {
        if bytes.len() < 2 {
            return 0;
        }
        (bytes[0] as u16) | ((bytes[1] as u16) << 8)
    }

    /// Read i32 from bytes (little endian)
    pub(crate) fn read_i32(&self, bytes: &[u8]) -> i32 {
        if bytes.len() < 4 {
            return 0;
        }
        let bits = (bytes[0] as i32)
            | ((bytes[1] as i32) << 8)
            | ((bytes[2] as i32) << 16)
            | ((bytes[3] as i32) << 24);
        bits
    }
}
