//! Quick `.blend` peek for file browsers — Blender's
//! `BLO_thumbnail_from_file` / `read_file_thumbnail` (readfile.cc) and the
//! `REND` `RenderInfo` blocks (`write_renderinfo`, writefile.cc).
//!
//! Streams the file (zstd / gzip transparently) and stops at the first block
//! that is neither `REND` nor `TEST`, so a 200 MB scene costs a few KB.

use crate::blend::file::types::*;
use crate::Result;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// `BLEN_THUMB_SIZE * 2` upper bound Blender accepts for stored previews.
const THUMB_MAX: i32 = 512;

/// Preview image, **top-down** RGBA8 rows (Blender stores them bottom-up).
#[derive(Debug, Clone, PartialEq)]
pub struct BlendThumbnail {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// One `RenderInfo` record: a scene written with its frame range.
#[derive(Debug, Clone, PartialEq)]
pub struct BlendRenderInfo {
    pub scene: String,
    pub frame_start: i32,
    pub frame_end: i32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BlendQuickInfo {
    /// "5.2" style version from the header.
    pub version: String,
    pub scenes: Vec<BlendRenderInfo>,
    pub thumbnail: Option<BlendThumbnail>,
}

/// Read-only stream that only reports its position (enough for `read_block`).
struct Counting<R> {
    inner: R,
    pos: u64,
}

impl<R: Read> Read for Counting<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.pos += n as u64;
        Ok(n)
    }
}

impl<R> Seek for Counting<R> {
    fn seek(&mut self, to: SeekFrom) -> std::io::Result<u64> {
        match to {
            SeekFrom::Current(0) => Ok(self.pos),
            _ => Err(std::io::Error::other("quick reader is forward-only")),
        }
    }
}

fn open(path: &Path) -> Result<Box<dyn Read>> {
    decode(BufReader::new(std::fs::File::open(path)?))
}

/// Sniff the first four bytes and wrap the reader in the decompressor they
/// name, as blenloader does (`BLO_read_from_file`): Zstd since Blender 4.5,
/// gzip before it, raw otherwise.
///
/// Generic over the reader, not over a path, so the same sniff serves a file
/// and a buffer — which is what lets [`BlendFile::quick_info_bytes`] exist
/// without a second copy of this.
fn decode<'a, R: Read + 'a>(mut r: R) -> Result<Box<dyn Read + 'a>> {
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;
    let head = std::io::Cursor::new(magic.to_vec()).chain(r);
    Ok(match magic {
        [0x28, 0xB5, 0x2F, 0xFD] => super::zst::stream(head)?,
        [0x1F, 0x8B, ..] => Box::new(flate2::read::MultiGzDecoder::new(head)),
        _ => Box::new(head),
    })
}

fn cstr(b: &[u8]) -> String {
    let n = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    String::from_utf8_lossy(&b[..n]).into_owned()
}

fn thumbnail(data: &[u8], le: bool) -> Option<BlendThumbnail> {
    let int = |o: usize| {
        let b: [u8; 4] = data.get(o..o + 4)?.try_into().ok()?;
        Some(if le { i32::from_le_bytes(b) } else { i32::from_be_bytes(b) })
    };
    let (w, h) = (int(0)?, int(4)?);
    if !(1..=THUMB_MAX).contains(&w) || !(1..=THUMB_MAX).contains(&h) {
        return None;
    }
    let (w, h) = (w as usize, h as usize);
    let px = data.get(8..8 + w * h * 4)?;
    let mut rgba = Vec::with_capacity(px.len());
    for row in (0..h).rev() {
        rgba.extend_from_slice(&px[row * w * 4..(row + 1) * w * 4]);
    }
    Some(BlendThumbnail { width: w as u32, height: h as u32, rgba })
}

impl BlendFile {
    /// Header version, `REND` scenes and the `TEST` thumbnail, without
    /// reading the rest of the file.
    pub fn read_quick_info(path: &Path) -> Result<BlendQuickInfo> {
        Self::quick_info_from(open(path)?)
    }

    /// The same, from bytes already in memory (`Phx::peek_bytes`) — the
    /// version a browser can call on a file it has just been handed, to show
    /// its thumbnail and frame range before parsing anything.
    pub fn quick_info_bytes(data: &[u8]) -> Result<BlendQuickInfo> {
        Self::quick_info_from(decode(data)?)
    }

    fn quick_info_from(reader: impl Read) -> Result<BlendQuickInfo> {
        let mut r = Counting { inner: reader, pos: 0 };
        let header = Self::read_header(&mut r)?;
        let mut info = BlendQuickInfo { version: header.version_label(), ..Default::default() };
        let le = header.little_endian;
        for _ in 0..64 {
            let block = Self::read_block(&mut r, &header)?;
            match block.code_str.as_str() {
                "REND" => {
                    let d = &block.data;
                    let int = |o: usize| {
                        d.get(o..o + 4)
                            .map(|b| {
                                let b: [u8; 4] = b.try_into().unwrap();
                                if le { i32::from_le_bytes(b) } else { i32::from_be_bytes(b) }
                            })
                            .unwrap_or(0)
                    };
                    info.scenes.push(BlendRenderInfo {
                        frame_start: int(0),
                        frame_end: int(4),
                        scene: cstr(d.get(8..).unwrap_or(&[])),
                    });
                }
                "TEST" => info.thumbnail = thumbnail(&block.data, le),
                _ => break,
            }
        }
        Ok(info)
    }
}
