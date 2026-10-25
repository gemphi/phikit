//! The render thread: holds the process's GPU (`Gpu::shared`, opened at
//! start-up) and owns the tracer, so the UI never waits. Of what is queued
//! it takes only the newest scene and view — a drag or a playing animation
//! queues one per step. A scene of the same shape (objects moved, a frame
//! played) moves the tracer (`place.rs`: objects placed by new matrices,
//! or the vertices in place with the trees refitted); once a posed scene
//! has been still for [`SETTLE`], the trees are built fresh
//! (`Tracer::retree`), the image keeping its samples.
//! The UI posts the latest scene and view; the thread keeps refining the
//! newest view and publishes each finished sample as a BGRA frame.

use super::posted::BPosted;
pub(super) use super::job::{BRayFrame, BRayJob, BRayShot};
use phiray::Gpu;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use web_time::Instant;

/// How long a posed scene stays still before its tree is built fresh.
pub(super) const SETTLE: Duration = Duration::from_millis(400);

pub(super) struct BRayWorker {
    tx: Sender<BRayJob>,
    latest: Arc<Mutex<Option<BRayFrame>>>,
    /// The newest view and scene keys posted, read by the thread between
    /// the slices of a sample: a newer one waiting ends the sample early.
    posted: Arc<BPosted>,
    /// Scene keys the GPU could not take (too large): the painter falls back.
    refused: Arc<Mutex<Vec<u64>>>,
    /// The shared device it draws on (`Gpu::generation`): when Preferences
    /// ▸ System ▸ Render Devices opens another, a new worker takes over.
    pub generation: u64,
}

impl BRayWorker {
    /// Start the thread; `None` when there is no GPU adapter.
    pub fn start() -> Option<Self> {
        let (tx, rx) = channel();
        let (ready_tx, ready_rx) = channel();
        let latest = Arc::new(Mutex::new(None));
        let refused = Arc::new(Mutex::new(Vec::new()));
        let posted = Arc::new(BPosted::default());
        let (out, no, seen) = (latest.clone(), refused.clone(), posted.clone());
        std::thread::Builder::new()
            .name("phiray".into())
            .spawn(move || {
                let gpu = Gpu::shared();
                let _ = ready_tx.send(gpu.is_some());
                if let Some(gpu) = gpu {
                    super::thread::run(gpu, rx, out, no, seen);
                }
            })
            .ok()?;
        let generation = Gpu::generation();
        ready_rx.recv().ok()?.then_some(Self { tx, latest, refused, posted, generation })
    }

    pub fn send(&self, job: BRayJob) {
        use std::sync::atomic::Ordering::Relaxed;
        match &job {
            BRayJob::View(key, _) => self.posted.view.store(*key, Relaxed),
            BRayJob::Scene(key, ..) => self.posted.scene.store(*key, Relaxed),
        }
        let _ = self.tx.send(job);
    }

    /// Whether the render thread gave up on scene `key`.
    pub fn refused(&self, key: u64) -> bool {
        self.refused.lock().is_ok_and(|r| r.contains(&key))
    }

    /// Take the newest frame if it is newer than `seq` and of `view` — the
    /// viewport that posted the view takes its frames, and no other one, so
    /// an image is never replaced (and its texture dropped) after another
    /// viewport has already painted it this frame.
    pub fn take(&self, view: u64, seq: u64) -> Option<BRayFrame> {
        let mut slot = self.latest.lock().ok()?;
        if slot.as_ref().is_some_and(|f| f.seq > seq && f.view == view) { slot.take() } else { None }
    }

    /// [`Self::take`], waiting up to `cap` for the frame to land: playback
    /// draws a frame's image in the frame itself, as Blender's draw loop
    /// does, instead of the image of a frame the playhead has left.
    pub fn take_within(&self, view: u64, seq: u64, cap: Duration) -> Option<BRayFrame> {
        let until = Instant::now() + cap;
        loop {
            if let Some(frame) = self.take(view, seq) {
                return Some(frame);
            }
            if Instant::now() >= until {
                return None;
            }
            std::thread::sleep(Duration::from_micros(250));
        }
    }
}
