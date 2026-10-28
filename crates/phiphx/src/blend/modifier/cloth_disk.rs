//! A point cache kept on disk (`PTCACHE_DISK_CACHE`) — Blender
//! `pointcache.cc`: `BKE_ptcache_path`, `ptcache_filepath`,
//! `ptcache_filepath_ext_append`, `ptcache_file_header_begin_read`,
//! `ptcache_basic_header_read`, `ptcache_file_compressed_read` and
//! `ptcache_disk_frame_to_mem`, with `unfilter_transpose_delta`
//! (`blenlib/intern/compression.cc`).
//!
//! The folder is `blendcache_<file stem>/` beside the `.blend` (or the
//! cache's own `path` for an external cache); each frame is a file named
//! by the owner's ID name in hex (or the cache's `name`), the frame as six
//! digits and the cache's stack index as two: `43 6C 6F 74 68 → "436C6F7468"`,
//! `436C6F7468_000012_00.bphys`. A file starts `BPHYSICS`, a `u32`
//! type-and-flags (the type must be `PTCACHE_TYPE_CLOTH`), then the point
//! count and the data-type bits. A compressed file (`PTCACHE_TYPEFLAG_COMPRESS`)
//! holds each data type's whole array in turn — a compression byte, a
//! size, the Zstd stream — transposed by byte and delta-coded before the
//! Zstd when the byte says `ZSTD_FILTERED`; an uncompressed one holds the
//! points in order, each data type's record in turn. LZO and LZMA caches
//! (removed in 5.0) are not read, as Blender no longer reads them.

use super::cloth_cache::{BCacheFrame, DATA_INDEX, DATA_LOCATION, DATA_SIZE, DATA_VELOCITY};
use crate::blend::file::types::*;
use std::path::PathBuf;

/// `PTCACHE_TYPE_CLOTH`, `PTCACHE_TYPEFLAG_COMPRESS`, `PTCACHE_TYPEFLAG_TYPEMASK`.
const TYPE_CLOTH: u32 = 2;
const TYPEFLAG_COMPRESS: u32 = 1 << 16;
const TYPEFLAG_TYPEMASK: u32 = 0xFFFF;
/// `PointCacheCompression`.
const COMPRESS_NO: u8 = 0;
const COMPRESS_ZSTD_FILTERED: u8 = 3;
const COMPRESS_ZSTD_FAST_DEPRECATED: u8 = 4;
const COMPRESS_ZSTD_SLOW_DEPRECATED: u8 = 8;

/// Where one cache's frame files are.
pub(super) struct BDiskCache {
    dir: PathBuf,
    /// The file name before the frame: the hex ID name or the cache name.
    stem: String,
    /// `"_%02u"` of the stack index, or empty for an external cache that
    /// has none.
    suffix: String,
    start: i32,
    end: i32,
}

impl BDiskCache {
    /// `BKE_ptcache_path` + `ptcache_filepath` for the cache `pc` of `ob`;
    /// `None` when the file came as bytes (no folder to look beside).
    pub(super) fn locate(f: &BlendFile, pc: &BlendBlock, ob: &BlendBlock, dna: &DnaType, external: bool, start: i32, end: i32) -> Option<Self> {
        let blend = crate::blend::shade::image::source_path()?;
        let name = crate::blend::physics::string_at(&pc.data, f.off("PointCache", "name[64]", dna), 64);
        let index = f.uint_field(pc, "PointCache", "index", dna).map_or(0, |i| i as i32);
        let dir = match external {
            true => {
                let path = crate::blend::physics::string_at(&pc.data, f.off("PointCache", "path[1024]", dna), 1024);
                match path.strip_prefix("//") {
                    Some(rel) => blend.parent()?.join(rel),
                    None => PathBuf::from(path),
                }
            }
            false => blend.parent()?.join(format!("blendcache_{}", blend.file_stem()?.to_string_lossy())),
        };
        let stem = match name.is_empty() && !external {
            // "convert chars to hex so they are always a valid filename"
            true => f.extract_name_from_block(ob, "name").ok()?.bytes().map(|b| format!("{b:02X}")).collect(),
            false => name,
        };
        let suffix = match external && index < 0 {
            true => String::new(),
            false => format!("_{:02}", index.max(0)),
        };
        Some(Self { dir, stem, suffix, start, end })
    }

    fn path(&self, frame: i32) -> PathBuf {
        self.dir.join(format!("{}_{:06}{}.bphys", self.stem, frame, self.suffix))
    }

    /// The frames with a file, in the cache's range (`cached_frames`).
    pub(super) fn frames(&self) -> Vec<i32> {
        (self.start..=self.end).filter(|&c| self.path(c).is_file()).collect()
    }

    /// `ptcache_disk_frame_to_mem`.
    pub(super) fn read(&self, frame: i32) -> Option<BCacheFrame> {
        let bytes = std::fs::read(self.path(frame)).ok()?;
        parse(&bytes, frame)
    }
}

/// A little-endian `u32` at `at`, advancing it.
fn u32_at(bytes: &[u8], at: &mut usize) -> Option<u32> {
    let v = u32::from_le_bytes(bytes.get(*at..*at + 4)?.try_into().ok()?);
    *at += 4;
    Some(v)
}

/// `unfilter_transpose_delta`: undo the byte transpose and delta coding.
fn unfilter(src: &[u8], items: usize, size: usize) -> Vec<u8> {
    let mut dst = vec![0u8; items * size];
    let mut k = 0;
    for ib in 0..size {
        let mut prev = 0u8;
        for it in 0..items {
            let v = src.get(k).copied().unwrap_or(0).wrapping_add(prev);
            prev = v;
            dst[it * size + ib] = v;
            k += 1;
        }
    }
    dst
}

/// `ptcache_file_compressed_read` for one data type's array.
fn compressed_array(bytes: &[u8], at: &mut usize, items: usize, size: usize) -> Option<Vec<u8>> {
    let kind = *bytes.get(*at)?;
    *at += 1;
    if kind == COMPRESS_NO {
        let raw = bytes.get(*at..*at + items * size)?.to_vec();
        *at += items * size;
        return Some(raw);
    }
    let len = u32_at(bytes, at)? as usize;
    let input = bytes.get(*at..*at + len)?;
    *at += len;
    if len == 0 {
        return Some(vec![0u8; items * size]);
    }
    let plain = match kind {
        COMPRESS_ZSTD_FILTERED | COMPRESS_ZSTD_FAST_DEPRECATED | COMPRESS_ZSTD_SLOW_DEPRECATED => crate::blend::file::zst::all(input).ok()?,
        _ => return None,
    };
    Some(match kind {
        COMPRESS_ZSTD_FILTERED => unfilter(&plain, items, size),
        _ => plain,
    })
}

/// One `.bphys` file's cloth frame.
fn parse(bytes: &[u8], frame: i32) -> Option<BCacheFrame> {
    if bytes.get(0..8)? != b"BPHYSICS" {
        return None;
    }
    let mut at = 8;
    let typeflag = u32_at(bytes, &mut at)?;
    if typeflag & TYPEFLAG_TYPEMASK != TYPE_CLOTH {
        return None;
    }
    let totpoint = u32_at(bytes, &mut at)? as usize;
    let types = u32_at(bytes, &mut at)?;
    let mut arrays: [Vec<u8>; 8] = Default::default();
    match typeflag & TYPEFLAG_COMPRESS != 0 {
        true => {
            for (i, a) in arrays.iter_mut().enumerate() {
                if types & (1 << i) != 0 {
                    *a = compressed_array(bytes, &mut at, totpoint, DATA_SIZE[i])?;
                }
            }
        }
        false => {
            for _ in 0..totpoint {
                for (i, a) in arrays.iter_mut().enumerate() {
                    if types & (1 << i) != 0 {
                        a.extend_from_slice(bytes.get(at..at + DATA_SIZE[i])?);
                        at += DATA_SIZE[i];
                    }
                }
            }
        }
    }
    let f32_le = |c: &[u8]| f32::from_le_bytes(c.try_into().unwrap_or([0; 4]));
    let vec3s = |a: &[u8]| -> Vec<[f32; 3]> { a.chunks_exact(12).map(|c| [f32_le(&c[0..4]), f32_le(&c[4..8]), f32_le(&c[8..12])]).collect() };
    let index: Option<Vec<u32>> = (types & (1 << DATA_INDEX) != 0).then(|| arrays[DATA_INDEX as usize].chunks_exact(4).map(|c| u32::from_le_bytes(c.try_into().unwrap_or([0; 4]))).collect());
    let location = vec3s(&arrays[DATA_LOCATION as usize]);
    let velocity = (types & (1 << DATA_VELOCITY) != 0).then(|| vec3s(&arrays[DATA_VELOCITY as usize]));
    Some(BCacheFrame::from_arrays(frame, totpoint, index.as_deref(), &location, velocity.as_deref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_uncompressed_file_reads_point_by_point() {
        let mut b = b"BPHYSICS".to_vec();
        b.extend_from_slice(&TYPE_CLOTH.to_le_bytes());
        b.extend_from_slice(&2u32.to_le_bytes());
        b.extend_from_slice(&(((1u32 << DATA_LOCATION) | (1u32 << DATA_VELOCITY)) as u32).to_le_bytes());
        for p in 0..2 {
            for k in 0..3 {
                b.extend_from_slice(&((p * 3 + k) as f32).to_le_bytes());
            }
            for _ in 0..3 {
                b.extend_from_slice(&0.5f32.to_le_bytes());
            }
        }
        let fr = parse(&b, 7).expect("parses");
        assert_eq!((fr.frame, fr.positions), (7, vec![[0.0, 1.0, 2.0], [3.0, 4.0, 5.0]]));
        assert_eq!(fr.velocities[1], [0.5; 3]);
    }

    #[test]
    fn unfilter_undoes_the_transpose_and_delta() {
        // Two items of three bytes: transposed by byte, delta-coded.
        let plain = [1u8, 10, 100, 2, 20, 200];
        let mut filtered = Vec::new();
        for ib in 0..3 {
            let mut prev = 0u8;
            for it in 0..2 {
                let v = plain[it * 3 + ib];
                filtered.push(v.wrapping_sub(prev));
                prev = v;
            }
        }
        assert_eq!(unfilter(&filtered, 2, 3), plain);
    }
}
