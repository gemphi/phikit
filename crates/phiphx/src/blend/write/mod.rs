//! Writing a `.blend` back — the file that was read, block for block, with
//! the fields the session changed patched in place, and a new block where
//! one is needed (a marker added).
//!
//! Blender ref: `writefile.cc` (`BLO_write_file`) lays a file out as the
//! header, then every block as a `BHead` and its bytes, then `DNA1` and
//! `ENDB`. The reader (`blo_read_file_internal`) resolves pointers through
//! the saved addresses, so a file whose blocks keep their addresses and
//! their DNA reads back exactly; a block that is not pointed at is never
//! read, so an orphaned one costs nothing. That is what lets a saved file
//! be the opened file with the changes written through it, rather than a
//! second writer of every struct Blender has.
//!
//! The pieces: [`BlendFile::to_bytes`] and [`BlendFile::write_to`] here,
//! the field writers in `field.rs`, the view layer's blocks in `layer.rs`,
//! the marker list in `marker.rs`, and the session's edits as one value in
//! `edit.rs` (`BlendEdits`).

pub mod action;
mod cursor;
pub mod edit;
pub mod keys;
pub mod field;
pub mod layer;
pub mod link;
pub mod marker;
pub mod nla;
pub mod mesh;
pub mod object;
mod owned;
pub mod playback;
pub mod relink;
mod render;

use crate::blend::file::types::*;
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use std::io::Write;

pub use edit::{BlendCollectionEdit, BlendDuplicate, BlendEdits, BlendNewObject, BlendObjectEdit, BlendSaveReport};
pub use action::{BlendActionAssign, BlendActionRange};
pub use keys::BlendKeyInsert;
pub use nla::BlendNlaStack;
pub use mesh::BlendNewMesh;

/// How the file on disk is wrapped (`BLO_write_file`'s `use_compress`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendCompression {
    None,
    /// Blender 4.5+ (`(\xb5/\xfd`).
    Zstd,
    /// Blender ≤ 4.4 (`\x1f\x8b`).
    Gzip,
}

impl BlendCompression {
    /// What the bytes on disk are wrapped in.
    pub fn of(bytes: &[u8]) -> Self {
        if bytes.len() >= 4 && bytes[0..4] == [0x28, 0xB5, 0x2F, 0xFD] {
            Self::Zstd
        } else if bytes.len() >= 2 && bytes[0..2] == [0x1F, 0x8B] {
            Self::Gzip
        } else {
            Self::None
        }
    }

    /// `raw` wrapped this way.
    pub fn wrap(self, raw: Vec<u8>) -> std::io::Result<Vec<u8>> {
        match self {
            Self::None => Ok(raw),
            Self::Zstd => zstd::stream::encode_all(std::io::Cursor::new(raw), 3),
            Self::Gzip => {
                let mut out = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
                out.write_all(&raw)?;
                out.finish()
            }
        }
    }
}

impl BlendHeader {
    /// The header as the file spells it: `BLENDER-v306` or `BLENDER17-01v0502`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = BLEND_MAGIC.to_vec();
        if self.file_format_version == 0 {
            out.push(if self.pointer_size == 4 { b'_' } else { b'-' });
            out.push(if self.little_endian { b'v' } else { b'V' });
            out.extend_from_slice(format!("{:0>3}", self.version).as_bytes());
        } else {
            out.extend_from_slice(b"17-01v");
            out.extend_from_slice(format!("{:0>4}", self.version).as_bytes());
        }
        out
    }
}

impl BlendFile {
    /// The whole file: header, every block, `ENDB`.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = self.header.to_bytes();
        for block in &self.blocks {
            self.write_bhead(&mut out, block.code, block.size, block.old_address, block.sdna_index, block.count);
            out.extend_from_slice(&block.data);
        }
        self.write_bhead(&mut out, *ENDB, 0, 0, 0, 0);
        out
    }

    /// One block header in the layout the file header calls for.
    fn write_bhead(&self, out: &mut Vec<u8>, code: [u8; 4], size: u64, old: u64, sdna: u32, count: u64) {
        let le = self.header.little_endian;
        let i32s = |out: &mut Vec<u8>, v: i32| {
            let mut b = [0u8; 4];
            if le { LittleEndian::write_i32(&mut b, v) } else { BigEndian::write_i32(&mut b, v) }
            out.extend_from_slice(&b);
        };
        let u64s = |out: &mut Vec<u8>, v: u64| {
            let mut b = [0u8; 8];
            if le { LittleEndian::write_u64(&mut b, v) } else { BigEndian::write_u64(&mut b, v) }
            out.extend_from_slice(&b);
        };
        out.extend_from_slice(&code);
        match self.header.bhead_type() {
            BHeadType::BHead4 => {
                i32s(out, size as i32);
                i32s(out, old as u32 as i32);
                i32s(out, sdna as i32);
                i32s(out, count as i32);
            }
            BHeadType::SmallBHead8 => {
                i32s(out, size as i32);
                u64s(out, old);
                i32s(out, sdna as i32);
                i32s(out, count as i32);
            }
            BHeadType::LargeBHead8 => {
                i32s(out, sdna as i32);
                u64s(out, old);
                u64s(out, size);
                u64s(out, count);
            }
        }
    }

    /// Write the file to `path`, wrapped as `compression` says. An existing
    /// file at `path` is first renamed to `<path>1` (Blender's
    /// `use_save_versions`: `file.blend` → `file.blend1`).
    pub fn write_to(&self, path: &std::path::Path, compression: BlendCompression) -> std::io::Result<()> {
        let bytes = compression.wrap(self.to_bytes())?;
        if path.is_file() {
            let mut backup = path.as_os_str().to_owned();
            backup.push("1");
            std::fs::rename(path, std::path::Path::new(&backup))?;
        }
        std::fs::write(path, bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_parsed_file_written_back_is_the_same_bytes() {
        for name in ["nla_cube.blend", "THYMUS-BOLDROID-WAVE-003.blend"] {
            let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
            let bytes = std::fs::read(&path).expect("fixture");
            let file = BlendFile::parse_bytes(&bytes).expect("parse");
            let again = file.to_bytes();
            assert_eq!(again.len(), bytes.len(), "{name}: the same length");
            assert!(again == bytes, "{name}: byte for byte");
        }
    }

    #[test]
    fn a_compressed_file_round_trips_through_its_own_wrapping() {
        let path = format!("{}/tests/fixtures/DEPTH-OF-FIELD.blend", env!("CARGO_MANIFEST_DIR"));
        let bytes = std::fs::read(&path).expect("fixture");
        assert_eq!(BlendCompression::of(&bytes), BlendCompression::Zstd);
        let file = BlendFile::parse_bytes(&bytes).expect("parse");
        let wrapped = BlendCompression::Zstd.wrap(file.to_bytes()).expect("zstd");
        let again = BlendFile::parse_bytes(&wrapped).expect("parse again");
        assert_eq!(again.blocks.len(), file.blocks.len());
        assert_eq!(again.to_bytes(), file.to_bytes());
        assert_eq!(BlendHeader::to_bytes(&file.header).len(), if file.header.file_format_version == 0 { 12 } else { 17 });
    }
}


