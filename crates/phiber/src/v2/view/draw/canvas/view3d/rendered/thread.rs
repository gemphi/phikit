//! The render thread's loop (`BRayWorker::start` runs it): take the newest
//! job, move or rebuild the tracer for it, refine a sample in slices and
//! publish it — split from `worker.rs`, which holds the handle the UI posts
//! to.

use super::super::mesh::BMeshPalette;
use super::coarse::BCoarse;
use super::pace::{BSlicer, BTally};
use super::posted::BPosted;
use super::build::built;
use super::job::{BRayFrame, BRayJob, BRayShot};
use super::place;
use super::worker::SETTLE;
use crate::v2::scene::BObject3D;
use phiray::{Gpu, Mode, Tracer};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use web_time::Instant;

/// Whether the all-zero-frame warning has been printed this run.
static BLANK_SAID: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(super) fn run(gpu: Gpu, rx: Receiver<BRayJob>, out: Arc<Mutex<Option<BRayFrame>>>, refused: Arc<Mutex<Vec<u64>>>, posted: Arc<BPosted>) {
    let mut tracer: Option<Tracer> = None;
    let mut view: Option<(u64, BRayShot)> = None;
    let mut done = true;
    let mut seq = 0u64;
    let mut shape = 0u64;
    let mut scene_key = 0u64;
    let mut coarse = BCoarse::start(Mode::Solid);
    let mut slicer = BSlicer::new();
    let mut tally = BTally::new();
    let mut warmed = super::compile::BWarmed::new();
    // The scene the tracer holds (its key, objects and palette), and when
    // it was last posed with its trees refitted rather than built.
    let mut held: Option<(u64, Vec<BObject3D>, BMeshPalette)> = None;
    let mut posed: Option<Instant> = None;
    loop {
        // Idle → block (until the posed scene settles); refining → only
        // drain what is already queued.
        let first = if done || tracer.is_none() || view.is_none() {
            let wait = posed.map(|at| SETTLE.saturating_sub(at.elapsed()));
            match wait.map_or_else(|| rx.recv().map_err(|_| RecvTimeoutError::Disconnected), |wait| rx.recv_timeout(wait)) {
                Ok(job) => Some(job),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        } else {
            None
        };
        let (mut next_scene, mut next_view) = (None, None);
        for job in first.into_iter().chain(rx.try_iter()) {
            match job {
                BRayJob::Scene(key, of, objects, palette) => next_scene = Some((key, of, objects, palette)),
                BRayJob::View(key, shot) => next_view = Some((key, shot)),
            }
        }
        if let Some((key, of, objects, palette)) = next_scene {
            let last = held.as_ref().filter(|_| of == shape).map(|(_, o, _)| o.as_slice());
            match tracer.as_mut().filter(|_| of == shape).and_then(|t| place::update(&gpu, t, last, &objects, &palette)) {
                Some(place::BKept::Placed) => {}
                Some(place::BKept::Posed) => posed = Some(Instant::now()),
                None => {
                    tracer = built(&gpu, key, &objects, &palette, &refused);
                    posed = None;
                }
            }
            held = Some((key, objects, palette));
            shape = of;
            scene_key = key;
            done = false;
        }
        if let Some(next) = next_view {
            // Another view: its first samples at a fraction of the size, as
            // Cycles starts a navigated viewport small and doubles it
            // (`start_resolution`), so the first image lands at once.
            if view.is_none_or(|(key, _)| key != next.0) {
                coarse = BCoarse::start_for(next.1.mode, slicer.predicted_ms(next.1.mode, next.1.w, next.1.h));
            }
            view = Some(next);
            done = false;
        }
        // Still for SETTLE since the last pose: a fresh tree, the image and
        // its samples kept (`Tracer::retree`); built again only when the
        // device cannot hold the new tree.
        if posed.take_if(|at| at.elapsed() >= SETTLE).is_some()
            && let Some((key, objects, palette)) = held.as_ref()
        {
            let at = Instant::now();
            if tracer.as_mut().is_some_and(|t| t.retree(&gpu)) {
                crate::v2::app::journal::BJournal::note(format!("Tracer: trees rebuilt over the pose in {:.0}ms", at.elapsed().as_secs_f64() * 1000.0));
            } else {
                tracer = built(&gpu, *key, objects, palette, &refused);
                done = false;
            }
        }
        // Refined already, and nothing new: the same image, not read again.
        if done {
            continue;
        }
        let (Some(t), Some((key, shot))) = (tracer.as_mut(), view.clone()) else { continue };
        let BRayShot { cam, w: full_w, h: full_h, mode, samples, aa, denoise_from, outline } = shot;
        let (w, h) = coarse.pixels(full_w, full_h);
        let cam = phiray::Camera { tile: coarse.tile(cam.tile), ..cam };
        t.set_samples(samples);
        t.set_aa_samples(aa);
        // Which objects the overlay outlines (`Tracer::outline`).
        t.outline(&gpu, &outline);
        // Workbench and the tracer are different kernels over the same scene;
        // switching re-records the pass, which is why the mode rides with the
        // view rather than the scene.
        t.set_mode(mode);
        // In slices when a sample is long, so a view posted meanwhile is
        // taken at the next slice instead of after the whole frame.
        let at = Instant::now();
        let slices = slicer.slices(mode, w, h, coarse);
        let Some((sample, finished)) = t.sample_in(&gpu, cam, w, h, slices, || !posted.newer_than(key, scene_key)) else {
            tally.abandoned += 1;
            continue;
        };
        let traced = at.elapsed();
        slicer.took(mode, w, h, coarse, traced);
        // A coarse image is never the end: the next is twice its size.
        let finished = finished && coarse.full();
        done = finished;
        let skip = coarse.full() && !(finished || sample < 4 || sample % 4 == 3);
        // While views keep arriving — a drag — the next sample stays this
        // small, as Cycles keeps a navigated viewport at its start size; the
        // view grows once the hand rests.
        if !posted.newer_than(key, scene_key) {
            coarse.next();
        }
        // Read back early samples, then every 4th (readback is the slow part).
        if skip {
            tally.sample(traced, Duration::ZERO, coarse.full(), full_w, full_h, mode);
            continue;
        }
        // Viewport Denoising: every frame shown from the start sample on is
        // filtered, the sums left as they are so the next sample lands on
        // the truth (`denoise_shown`). A deterministic mode has no noise and
        // the tracer skips it.
        if denoise_from.is_some_and(|from| sample + 1 >= from.max(1)) {
            t.denoise_shown(&gpu);
        }
        let read_at = Instant::now();
        let mut px = t.pixels(&gpu);
        tally.sample(traced, read_at.elapsed(), (w, h) == (full_w, full_h), full_w, full_h, mode);
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
        super::wake::FRESH.store(true, std::sync::atomic::Ordering::Release);
        // The first image is up: the other kernels compile meanwhile, on a
        // thread of their own (`compile.rs`).
        warmed.once(&gpu);
    }
}
