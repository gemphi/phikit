//! Armature deform on the GPU: phiphx batches every rigged mesh of a file into
//! [`SkinJob`]s, [`RaySkin`] runs them in one phiray pass. Declines (→ phiphx
//! CPU path) when there is no adapter or the batch is too small to pay off.

use crate::phiphx::phx::{Phx, SKIN_MIN, SkinJob, Skinner};
use phiray::kernels::{Pair, Skin, SkinParams};
use phiray::{Gpu, GpuArray, Pass};
use std::sync::{Arc, Mutex, OnceLock};

/// Below this many vertices the CPU is faster than an upload + readback.
const GPU_MIN_VERTS: usize = 20_000;

#[derive(Default)]
pub struct RaySkin {
    gpu: OnceLock<Option<Gpu>>,
    pass: Mutex<Pass>,
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

impl RaySkin {
    /// Make every `Phx::open` in this process skin through phiray, and open
    /// the device now rather than during the first import.
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
        let _ = std::thread::Builder::new()
            .name("phiray.skin.warm".to_string())
            .spawn(move || {
                skin.ready();
            });
    }

    /// Always use the GPU, whatever the batch size (tests, benchmarks).
    pub fn eager() -> Self {
        Self { always: true, ..Default::default() }
    }

    /// Whether a GPU adapter is available.
    pub fn ready(&self) -> bool {
        self.gpu().is_some()
    }

    fn gpu(&self) -> Option<&Gpu> {
        self.gpu.get_or_init(Gpu::new).as_ref()
    }

    /// The device only if it is already open. `run` uses this rather than
    /// [`RaySkin::gpu`]: opening a device on the file-reading thread is what
    /// made import look like a hang (see [`RaySkin::install`]).
    fn opened(&self) -> Option<&Gpu> {
        self.gpu.get()?.as_ref()
    }
}

impl Skinner for RaySkin {
    fn run(&self, jobs: &mut [SkinJob]) -> bool {
        let n: usize = jobs.iter().map(|j| j.rest.len()).sum();
        if (n < GPU_MIN_VERTS && !self.always) || n == 0 {
            return false;
        }
        let Some(gpu) = self.opened() else { return false };
        let Ok(mut pass) = self.pass.lock() else { return false };
        let b = Batch::of(jobs);
        let rest = GpuArray::from(gpu, &b.rest);
        let offsets = GpuArray::from(gpu, &b.offsets);
        let pairs = GpuArray::from(gpu, &b.pairs);
        let bones = GpuArray::from(gpu, &b.bones);
        let out = GpuArray::from(gpu, &vec![[0.0f32; 4]; n]);
        pass.clear();
        let params = SkinParams { min: SKIN_MIN, ..Default::default() };
        let bufs = [rest.buffer(), offsets.buffer(), pairs.buffer(), bones.buffer(), out.buffer()];
        pass.add::<Skin>(gpu, n as u32, params, &bufs);
        pass.run(gpu);
        let moved = out.read(gpu);
        if moved.len() < n {
            // The readback timed out or came back short: decline, so
            // `finish_skins` runs the CPU skinner over the same jobs.
            return false;
        }
        let mut moved = moved.into_iter();
        for j in jobs.iter_mut() {
            j.out = moved.by_ref().take(j.rest.len()).map(|v| [v[0], v[1], v[2]]).collect();
        }
        true
    }
}
