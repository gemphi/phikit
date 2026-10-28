//! The point cache's frames for a Cloth — `BKE_ptcache_read`,
//! `ptcache_find_frames_around`, `ptcache_read`, `ptcache_interpolate` and
//! `ptcache_cloth_read` / `ptcache_cloth_interpolate` (`pointcache.cc`),
//! over the file's own `PointCache.mem_cache` (a list of `PTCacheMem`, one
//! per frame, each data type's array written as its own block) or the disk
//! cache (`cloth_disk.rs`).
//!
//! A cloth frame carries `BPHYS_DATA_LOCATION`, `_VELOCITY` and `_XCONST`
//! per vertex (world space). Between two cached frames the position is the
//! cubic Hermite of the two (`interp_cubic_v3`, the velocities scaled by
//! the frame gap), as `psys_interpolate_particle(-1, …)` does for cloth.

use super::cloth::{PTCACHE_DISK_CACHE, PTCACHE_EXTERNAL};
use super::cloth_disk;
use crate::blend::file::types::*;
use glam::Vec3;

/// `ePointCache_DataType`.
pub(super) const DATA_INDEX: u32 = 0;
pub(super) const DATA_LOCATION: u32 = 1;
pub(super) const DATA_VELOCITY: u32 = 2;
/// Bytes per point of each data type (`ptcache_data_size`; the last is a
/// `BoidData`).
pub(super) const DATA_SIZE: [usize; 8] = [4, 12, 12, 16, 12, 4, 12, 20];

/// One cached frame: the positions (and velocities) by vertex.
#[derive(Clone, Debug, Default)]
pub(crate) struct BCacheFrame {
    pub frame: i32,
    pub positions: Vec<[f32; 3]>,
    pub velocities: Vec<[f32; 3]>,
}

impl BCacheFrame {
    /// `ptcache_read` over a frame's arrays: points land by their own
    /// index when the frame carries one (`BPHYS_DATA_INDEX`), else in
    /// order.
    pub(super) fn from_arrays(frame: i32, totpoint: usize, index: Option<&[u32]>, location: &[[f32; 3]], velocity: Option<&[[f32; 3]]>) -> Self {
        let n = match index {
            Some(ix) => ix.iter().take(totpoint).map(|&i| i as usize + 1).max().unwrap_or(0),
            None => totpoint.min(location.len()),
        };
        let mut out = Self { frame, positions: vec![[0.0; 3]; n], velocities: vec![[0.0; 3]; n] };
        for k in 0..totpoint.min(location.len()) {
            let at = index.map_or(k, |ix| ix.get(k).map_or(k, |&i| i as usize));
            if let Some(p) = out.positions.get_mut(at) {
                *p = location[k];
            }
            if let (Some(v), Some(vel)) = (out.velocities.get_mut(at), velocity.and_then(|v| v.get(k))) {
                *v = *vel;
            }
        }
        out
    }
}

/// What `BKE_ptcache_read` found for the frame.
#[derive(Clone, Debug)]
pub(crate) enum BClothRead {
    /// `PTCACHE_READ_EXACT`, or `_OLD` (the nearest cached frame).
    Frame(BCacheFrame),
    /// `PTCACHE_READ_INTERPOLATED` between two cached frames at `t`.
    Between(BCacheFrame, BCacheFrame, f32),
}

impl BClothRead {
    /// The positions to show, world space.
    pub(crate) fn positions_at(&self) -> Vec<[f32; 3]> {
        match self {
            Self::Frame(f) => f.positions.clone(),
            Self::Between(a, b, t) => {
                let dfra = (b.frame - a.frame) as f32;
                a.positions
                    .iter()
                    .zip(&b.positions)
                    .enumerate()
                    .map(|(i, (x1, x2))| {
                        let v = |f: &BCacheFrame| f.velocities.get(i).map_or(Vec3::ZERO, |v| Vec3::from(*v) * dfra);
                        interp_cubic(Vec3::from(*x1), v(a), Vec3::from(*x2), v(b), *t).to_array()
                    })
                    .collect()
            }
        }
    }
}

/// `interp_cubic_v3`: the Hermite curve through `x1` at 0 and `x2` at 1
/// with those tangents.
fn interp_cubic(x1: Vec3, v1: Vec3, x2: Vec3, v2: Vec3, t: f32) -> Vec3 {
    let (t2, t3) = (t * t, t * t * t);
    let a = v1 + v2 + 2.0 * (x1 - x2);
    let b = -2.0 * v1 - v2 - 3.0 * (x1 - x2);
    a * t3 + b * t2 + v1 * t + x1
}

/// Where the frames come from: the file's list or the folder beside it.
enum BSource<'a> {
    Memory(Vec<&'a BlendBlock>),
    Disk(cloth_disk::BDiskCache),
}

impl BSource<'_> {
    /// The cached frame numbers, ascending (`BKE_ptcache_id_exist` over
    /// the range).
    fn frames(&self, f: &BlendFile, dna: &DnaType) -> Vec<i32> {
        match self {
            Self::Memory(list) => list.iter().map(|pm| f.uint_field(pm, "PTCacheMem", "frame", dna).unwrap_or(0) as i32).collect(),
            Self::Disk(d) => d.frames(),
        }
    }

    fn read(&self, f: &BlendFile, dna: &DnaType, frame: i32) -> Option<BCacheFrame> {
        match self {
            Self::Memory(list) => list.iter().find(|pm| f.uint_field(pm, "PTCacheMem", "frame", dna).unwrap_or(0) as i32 == frame).and_then(|pm| mem_frame(f, pm, dna)),
            Self::Disk(d) => d.read(frame),
        }
    }
}

/// `BKE_ptcache_read(pid, framenr, can_simulate = false)` for the cloth
/// cache `pc` of `ob`: `None` when nothing is cached for the frame.
pub(super) fn frame_read(f: &BlendFile, pc: &BlendBlock, ob: &BlendBlock, dna: &DnaType, framenr: i32, start: i32, end: i32) -> Option<BClothRead> {
    let flag = f.int_field(pc, "PointCache", "flag", dna);
    let source = match flag & PTCACHE_DISK_CACHE != 0 {
        true => BSource::Disk(cloth_disk::BDiskCache::locate(f, pc, ob, dna, flag & PTCACHE_EXTERNAL != 0, start, end)?),
        false => {
            let first = f.ptr(pc, f.off("PointCache", "mem_cache", dna));
            BSource::Memory(f.list(first, f.off("PTCacheMem", "*next", dna)))
        }
    };
    let frames = source.frames(f, dna);
    let exact = frames.contains(&framenr) && (start..=end).contains(&framenr);
    // `ptcache_find_frames_around`.
    let (cfra1, cfra2) = match exact {
        true => (framenr, 0),
        false => {
            let below = frames.iter().copied().filter(|&c| c <= framenr).max();
            let above = frames.iter().copied().filter(|&c| c > framenr).min();
            match (below, above) {
                (Some(b), None) => (0, b),
                (Some(b), Some(a)) => (b, a),
                (None, Some(a)) => (0, a),
                (None, None) => return None,
            }
        }
    };
    let one = |c: i32| source.read(f, dna, c);
    match (cfra1, cfra2) {
        (0, 0) => None,
        (c, 0) | (0, c) => one(c).map(BClothRead::Frame),
        (a, b) => {
            let t = (framenr - a) as f32 / (b - a) as f32;
            Some(BClothRead::Between(one(a)?, one(b)?, t))
        }
    }
}

/// A `PTCacheMem`'s arrays: `data[i]` is a block of `totpoint` records of
/// `DATA_SIZE[i]` bytes when bit `i` of `data_types` is set.
fn mem_frame(f: &BlendFile, pm: &BlendBlock, dna: &DnaType) -> Option<BCacheFrame> {
    let ps = f.header.pointer_size as usize;
    let frame = f.uint_field(pm, "PTCacheMem", "frame", dna)? as i32;
    let totpoint = f.uint_field(pm, "PTCacheMem", "totpoint", dna)? as usize;
    let types = f.uint_field(pm, "PTCacheMem", "data_types", dna)?;
    let base = f.off("PTCacheMem", "*data[8]", dna).or_else(|| f.off("PTCacheMem", "data[8]", dna))?;
    let array = |i: u32| (types & (1 << i) != 0).then(|| f.block_at(BlendFile::read_ptr(&pm.data, base + i as usize * ps, ps, f.header.little_endian))).flatten();
    let index: Option<Vec<u32>> = array(DATA_INDEX).map(|b| b.data.chunks_exact(4).take(totpoint).map(|c| f.read_i32(c).max(0) as u32).collect());
    let vec3s = |b: &BlendBlock| -> Vec<[f32; 3]> { b.data.chunks_exact(12).take(totpoint).map(|c| [f.read_f32(&c[0..4]), f.read_f32(&c[4..8]), f.read_f32(&c[8..12])]).collect() };
    let location = vec3s(array(DATA_LOCATION)?);
    let velocity = array(DATA_VELOCITY).map(vec3s);
    Some(BCacheFrame::from_arrays(frame, totpoint, index.as_deref(), &location, velocity.as_deref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hermite_passes_through_both_ends() {
        let (x1, x2) = (Vec3::new(0.0, 1.0, 2.0), Vec3::new(3.0, -1.0, 0.5));
        let (v1, v2) = (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 2.0, 0.0));
        assert!((interp_cubic(x1, v1, x2, v2, 0.0) - x1).length() < 1e-6);
        assert!((interp_cubic(x1, v1, x2, v2, 1.0) - x2).length() < 1e-6);
        // Straight-line motion at constant velocity is reproduced exactly.
        let v = x2 - x1;
        assert!((interp_cubic(x1, v, x2, v, 0.25) - (x1 + v * 0.25)).length() < 1e-6);
    }

    #[test]
    fn an_index_array_places_points_by_index() {
        let fr = BCacheFrame::from_arrays(3, 2, Some(&[2, 0]), &[[1.0; 3], [2.0; 3]], None);
        assert_eq!(fr.positions, vec![[2.0; 3], [0.0; 3], [1.0; 3]]);
    }
}
