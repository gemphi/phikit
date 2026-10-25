//! Armature deform on the GPU: phiphx batches every rigged mesh of a file into
//! [`SkinJob`]s, [`RaySkin`] runs them in phiray passes, on the process's
//! one device (`Gpu::shared`, which the viewport's tracer uses too) — as
//! many meshes a pass as its bindings hold (`chunks`): six subdivided
//! bodies spread their weights over ~15 million influences, past the 128 MB
//! a binding may cover on most devices. Declines (→ phiphx CPU path) when
//! there is no adapter, the batch is too small to pay off, or one mesh alone
//! will not fit.

use crate::phiphx::phx::{Phx, SKIN_MIN, SkinJob, Skinner};
use phiray::kernels::{Pair, Skin, SkinParams};
use phiray::{Gpu, GpuArray, Pass};
use std::sync::{Arc, Mutex};

/// Below this many vertices the CPU is faster than an upload + readback.
const GPU_MIN_VERTS: usize = 20_000;

#[derive(Default)]
pub struct RaySkin {
    /// The recorded pass, and the device generation it was recorded on
    /// (`Gpu::generation`): a new device starts a new pass.
    pass: Mutex<(u64, Pass)>,
    /// Force the GPU even for small batches (tests).
    pub always: bool,
}

/// All jobs flattened into one set of arrays.
#[derive(Default)]
struct Batch {
    rest: Vec<[f32; 4]>,
    offsets: Vec<u32>,
    pairs: Vec<Pair>,
    bones: Vec<[f32; 16]>,
}

impl Batch {
    fn of(jobs: &[SkinJob]) -> Self {
        let mut b = Batch { offsets: vec![0], ..Default::default() };
        for j in jobs {
            let (pair_base, bone_base) = (b.pairs.len() as u32, b.bones.len() as u32);
            b.rest.extend(j.rest.iter().map(|v| [v[0], v[1], v[2], 1.0]));
            b.offsets.extend(j.offsets[1..].iter().map(|o| o + pair_base));
            b.pairs.extend(j.pairs.iter().map(|p| Pair {
                bone: p.bone + bone_base,
                weight: p.weight,
                next: p.next + bone_base,
                blend: p.blend,
            }));
            b.bones.extend_from_slice(&j.bones);
        }
        b
    }
}

/// The bytes each binding of a pass over `job` takes: rest, offsets,
/// pairs, bones, out.
fn binding_bytes(job: &SkinJob) -> [usize; 5] {
    let n = job.rest.len();
    [n * 16, (n + 1) * 4, job.pairs.len() * std::mem::size_of::<Pair>(), job.bones.len() * 64, n * 16]
}

/// `jobs` split, in order, into runs whose every binding stays within
/// `limit` bytes; `None` when one job alone is over it.
fn chunks(jobs: &[SkinJob], limit: usize) -> Option<Vec<std::ops::Range<usize>>> {
    let mut out = Vec::new();
    let (mut start, mut used) = (0, [0usize; 5]);
    for (i, job) in jobs.iter().enumerate() {
        let cost = binding_bytes(job);
        if cost.iter().any(|&c| c > limit) {
            return None;
        }
        if used.iter().zip(&cost).any(|(u, c)| u + c > limit) {
            out.push(start..i);
            (start, used) = (i, [0; 5]);
        }
        used.iter_mut().zip(&cost).for_each(|(u, c)| *u += c);
    }
    if start < jobs.len() {
        out.push(start..jobs.len());
    }
    Some(out)
}

impl RaySkin {
    /// Make every `Phx::open` in this process skin through phiray, and open
    /// the shared device now rather than during the first import — or the
    /// first viewport frame, which waits for it.
    ///
    /// `Gpu::new` requests an adapter and a device, which takes seconds. Doing
    /// it lazily inside `run` put that cost — and any driver stall — on the
    /// thread reading the `.blend`, after the last progress report and past
    /// the last Cancel check: the import looked hung. The warm-up runs on its
    /// own thread, and until it finishes `run` declines and phiphx's CPU
    /// skinner does the work.
    pub fn install() {
        let skin = Arc::new(RaySkin::default());
        let handle: Arc<dyn Skinner> = skin.clone();
        Phx::use_skinner(Some(handle));
        // Declines in a browser, where there is no second thread and a
        // WebGPU handle cannot leave this one; `run` then declines too and
        // phiphx's CPU skinner does the work, which is the same fallback as
        // a machine with no adapter.
        crate::v2::app::offload::offload("phiray.skin.warm", move || {
            skin.ready();
        });
    }

    /// Always use the GPU, whatever the batch size (tests, benchmarks).
    pub fn eager() -> Self {
        Self { always: true, ..Default::default() }
    }

    /// Whether a GPU adapter is available (opening the shared device).
    pub fn ready(&self) -> bool {
        Gpu::shared().is_some()
    }
}

impl Skinner for RaySkin {
    fn run(&self, jobs: &mut [SkinJob]) -> bool {
        let n: usize = jobs.iter().map(|j| j.rest.len()).sum();
        if (n < GPU_MIN_VERTS && !self.always) || n == 0 {
            return false;
        }
        // Only a device already open: opening one on the file-reading thread
        // is what made import look like a hang (see [`RaySkin::install`]).
        let Some(gpu) = Gpu::opened() else { return false };
        let Ok(mut slot) = self.pass.lock() else { return false };
        if slot.0 != Gpu::generation() {
            *slot = (Gpu::generation(), Pass::default());
        }
        let (_, pass) = &mut *slot;
        let gpu = &gpu;
        let Some(runs) = chunks(jobs, gpu.max_binding() as usize) else { return false };
        for run in runs {
            let part = &mut jobs[run];
            let count: usize = part.iter().map(|j| j.rest.len()).sum();
            let b = Batch::of(part);
            let rest = GpuArray::from(gpu, &b.rest);
            let offsets = GpuArray::from(gpu, &b.offsets);
            let pairs = GpuArray::from(gpu, &b.pairs);
            let bones = GpuArray::from(gpu, &b.bones);
            let out = GpuArray::from(gpu, &vec![[0.0f32; 4]; count]);
            pass.clear();
            let params = SkinParams { min: SKIN_MIN, ..Default::default() };
            let bufs = [rest.buffer(), offsets.buffer(), pairs.buffer(), bones.buffer(), out.buffer()];
            pass.add::<Skin>(gpu, count as u32, params, &bufs);
            pass.run(gpu);
            let moved = out.read(gpu);
            if moved.len() < count {
                // The readback timed out or came back short: decline, so
                // `finish_skins` runs the CPU skinner over the same jobs.
                return false;
            }
            let mut moved = moved.into_iter();
            for j in part.iter_mut() {
                j.out = moved.by_ref().take(j.rest.len()).map(|v| [v[0], v[1], v[2]]).collect();
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(verts: usize, pairs: usize) -> SkinJob {
        SkinJob { rest: vec![[0.0; 3]; verts], offsets: vec![0; verts + 1], pairs: vec![Default::default(); pairs], bones: vec![[0.0; 16]; 2], ..Default::default() }
    }

    /// Meshes go to a pass whole, as many as its bindings hold; a mesh
    /// too big for a binding on its own sends the batch to the CPU.
    #[test]
    fn a_batch_splits_where_a_binding_would_overflow_and_never_splits_a_mesh() {
        // Each job's pairs take 1600 bytes: two fit in 4000, not three.
        let jobs: Vec<SkinJob> = (0..5).map(|_| job(10, 100)).collect();
        assert_eq!(chunks(&jobs, 4000), Some(vec![0..2, 2..4, 4..5]));
        assert_eq!(chunks(&jobs, 1 << 20), Some(vec![0..5]), "all in one pass when they fit");
        assert_eq!(chunks(&jobs, 1000), None, "one mesh over the limit: the CPU");
        assert_eq!(chunks(&[], 1000), Some(vec![]));
    }
}
