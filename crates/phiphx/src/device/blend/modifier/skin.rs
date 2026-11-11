//! Batched skinning hook. The Armature modifier, when it is last in a stack,
//! becomes a [`SkinJob`]; all jobs of a file run in one [`Skinner`] call
//! (a GPU in Phiber, [`CpuSkin`] otherwise). phiphx itself stays GPU-free.

use bytemuck::{Pod, Zeroable};
use std::sync::{Arc, RwLock};

/// Minimum total weight for a vertex to move (Blender `contrib > 0.0001`).
pub const SKIN_MIN: f32 = 0.0001;

/// One vertex-group influence: an index into [`SkinJob::bones`], and for a
/// B-Bone the next segment's index with how far this vertex leans towards it
/// (`blend` is 0 and `next == bone` for a rigid bone).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct SkinPair {
    pub bone: u32,
    pub weight: f32,
    pub next: u32,
    pub blend: f32,
}

/// One mesh to skin. Influences are CSR: vertex `i` uses `pairs[offsets[i]..offsets[i + 1]]`.
/// `bones` are column-major object-space matrices (`arm⁻¹·chan·arm` folded in).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkinJob {
    pub rest: Vec<[f32; 3]>,
    pub offsets: Vec<u32>,
    pub pairs: Vec<SkinPair>,
    pub bones: Vec<[f32; 16]>,
    /// Filled by the skinner.
    pub out: Vec<[f32; 3]>,
}

/// Runs a batch of jobs; `false` means "not done, fall back to the CPU".
pub trait Skinner: Send + Sync {
    fn run(&self, jobs: &mut [SkinJob]) -> bool;
}

/// Reference skinner (the math every other skinner must match).
pub struct CpuSkin;

impl CpuSkin {
    pub fn vertex(job: &SkinJob, i: usize) -> [f32; 3] {
        let r = job.rest[i];
        let (mut acc, mut total) = ([0.0f32; 3], 0.0f32);
        for p in &job.pairs[job.offsets[i] as usize..job.offsets[i + 1] as usize] {
            let m = &job.bones[p.bone as usize];
            let n = &job.bones[p.next as usize];
            let at = |j: usize| m[j] + (n[j] - m[j]) * p.blend;
            for (k, a) in acc.iter_mut().enumerate() {
                *a += p.weight * (at(k) * r[0] + at(4 + k) * r[1] + at(8 + k) * r[2] + at(12 + k));
            }
            total += p.weight;
        }
        if total > SKIN_MIN { acc.map(|a| a / total) } else { r }
    }
}

impl Skinner for CpuSkin {
    fn run(&self, jobs: &mut [SkinJob]) -> bool {
        use rayon::prelude::*;
        jobs.par_iter_mut().for_each(|j| {
            j.out = (0..j.rest.len()).into_par_iter().map(|i| Self::vertex(j, i)).collect();
        });
        true
    }
}

static SKINNER: RwLock<Option<Arc<dyn Skinner>>> = RwLock::new(None);

/// Install (or clear) the process-wide skinner used while opening files.
pub fn set_skinner(s: Option<Arc<dyn Skinner>>) {
    if let Ok(mut g) = SKINNER.write() {
        *g = s;
    }
}

pub(crate) fn skinner() -> Option<Arc<dyn Skinner>> {
    SKINNER.read().ok()?.clone()
}
