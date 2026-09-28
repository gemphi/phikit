//! The render thread: owns the GPU and the tracer so the UI never waits.
//! The UI posts the latest scene and view; the thread keeps refining the
//! newest view and publishes each finished sample as a BGRA frame.

use super::super::mesh::BMeshPalette;
use super::scene;
use crate::v2::scene::BObject3D;
use phiray::{Camera, Gpu, Mode, Tracer};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};

/// Whether the all-zero-frame warning has been printed this run.
static BLANK_SAID: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(super) enum BRayJob {
    Scene(u64, Vec<BObject3D>, BMeshPalette),
    View(u64, BRayShot),
}

/// One frame to trace: the camera and size, which kernel draws it, and what
/// the Render panel asked for.
#[derive(Clone, Copy, Debug)]
pub(super) struct BRayShot {
    pub cam: Camera,
    pub w: u32,
    pub h: u32,
    pub mode: Mode,
    /// How far to refine (Blender's Max Samples).
    pub samples: u32,
    /// Filter the frame once it has settled.
    pub denoise: bool,
}

/// A published image (BGRA8, row 0 at the top).
pub(super) struct BRayFrame {
    pub seq: u64,
    pub view: u64,
    pub w: u32,
    pub h: u32,
    pub bgra: Vec<u8>,
    pub done: bool,
}

pub(super) struct BRayWorker {
    tx: Sender<BRayJob>,
    latest: Arc<Mutex<Option<BRayFrame>>>,
    /// Scene keys the GPU could not take (too large): the painter falls back.
    refused: Arc<Mutex<Vec<u64>>>,
}

impl BRayWorker {
    /// Start the thread; `None` when there is no GPU adapter.
    pub fn start() -> Option<Self> {
        let (tx, rx) = channel();
        let (ready_tx, ready_rx) = channel();
        let latest = Arc::new(Mutex::new(None));
        let refused = Arc::new(Mutex::new(Vec::new()));
        let (out, no) = (latest.clone(), refused.clone());
        std::thread::Builder::new()
            .name("phiray".into())
            .spawn(move || {
                let gpu = Gpu::new();
                let _ = ready_tx.send(gpu.is_some());
                if let Some(gpu) = gpu {
                    run(gpu, rx, out, no);
                }
            })
            .ok()?;
        ready_rx.recv().ok()?.then_some(Self { tx, latest, refused })
    }

    pub fn send(&self, job: BRayJob) {
        let _ = self.tx.send(job);
    }

    /// Whether the render thread gave up on scene `key`.
    pub fn refused(&self, key: u64) -> bool {
        self.refused.lock().is_ok_and(|r| r.contains(&key))
    }

    /// Take the newest frame if it is newer than `seq`.
    pub fn take(&self, seq: u64) -> Option<BRayFrame> {
        let mut slot = self.latest.lock().ok()?;
        if slot.as_ref().is_some_and(|f| f.seq > seq) { slot.take() } else { None }
    }
}

fn run(gpu: Gpu, rx: Receiver<BRayJob>, out: Arc<Mutex<Option<BRayFrame>>>, refused: Arc<Mutex<Vec<u64>>>) {
    let mut tracer: Option<Tracer> = None;
    let mut view: Option<(u64, BRayShot)> = None;
    let mut done = true;
    let mut seq = 0u64;
    loop {
        // Idle → block; refining → only drain what is already queued.
        let first = if done || tracer.is_none() || view.is_none() {
            match rx.recv() {
                Ok(job) => Some(job),
                Err(_) => return,
            }
        } else {
            None
        };
        for job in first.into_iter().chain(rx.try_iter()) {
            match job {
                BRayJob::Scene(key, objects, palette) => {
                    tracer = Tracer::from_scene(&gpu, scene::build(&objects, &palette));
                    if tracer.is_none()
                        && let Ok(mut r) = refused.lock()
                    {
                        r.push(key);
                    }
                    done = false;
                }
                BRayJob::View(key, shot) => {
                    view = Some((key, shot));
                    done = false;
                }
            }
        }
        let (Some(t), Some((key, shot))) = (tracer.as_mut(), view) else { continue };
        let BRayShot { cam, w, h, mode, samples, denoise } = shot;
        t.set_samples(samples);
        // Workbench and the tracer are different kernels over the same scene;
        // switching re-records the pass, which is why the mode rides with the
        // view rather than the scene.
        t.set_mode(mode);
        let (sample, finished) = t.sample(&gpu, cam, w, h);
        done = finished;
        // Read back early samples, then every 4th (readback is the slow part).
        if !(finished || sample < 4 || sample % 4 == 3) {
            continue;
        }
        // Denoising reads the guides the tracer wrote and replaces the shown
        // image, so it runs once the frame has settled — another sample would
        // overwrite it.
        if finished && denoise {
            t.denoise(&gpu);
        }
        let mut px = t.pixels(&gpu);
        // A frame the kernel never wrote comes back all zero — no sky in it,
        // where every real frame carries the theme's sky. It is published
        // empty: the painter keeps the theme canvas and says so once, rather
        // than painting the viewport black.
        if px.iter().all(|byte| *byte == 0) {
            if !BLANK_SAID.swap(true, std::sync::atomic::Ordering::Relaxed) {
                eprintln!("phiray: {} returned an all-zero {w}×{h} frame ({mode:?}, sample {sample}); the viewport keeps the theme canvas", gpu.info.name);
            }
            px.clear();
        }
        px.chunks_exact_mut(4).for_each(|p| p.swap(0, 2));
        seq += 1;
        if let Ok(mut slot) = out.lock() {
            *slot = Some(BRayFrame { seq, view: key, w, h, bgra: px, done });
        }
    }
}
