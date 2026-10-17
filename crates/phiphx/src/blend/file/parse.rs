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

use crate::blend::file::types::*;
use crate::{PointCloudError, Result};
use std::collections::HashMap;
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
        let mut dup_index: HashMap<u64, Vec<usize>> = HashMap::new();
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
                let first = *addr_index.entry(block.old_address).or_insert(blocks.len());
                if first != blocks.len() {
                    dup_index.entry(block.old_address).or_insert_with(|| vec![first]).push(blocks.len());
                }
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
            dup_index,
            scope: std::sync::atomic::AtomicUsize::new(usize::MAX),
            cancel_requested: std::sync::atomic::AtomicBool::new(false),
            deform_memo: std::sync::Mutex::default(),
            prefix_box: std::sync::Arc::default(),
            driven_modifiers: std::sync::Mutex::default(),
        })
    }

    /// O(1) block lookup by saved address (Blender `oldnewmap_lookup`).
    #[inline]
    pub fn block_at(&self, old_address: u64) -> Option<&BlendBlock> {
        if old_address == 0 {
            return None;
        }
        if let Some(i) = self.scoped_candidate(old_address) {
            return Some(&self.blocks[i]);
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


}


#[cfg(test)]
#[path = "parse_tests.rs"]
mod header_tests;
