//! What the render in flight looks like from the outside — size, samples,
//! passes carried, time taken — without copying its buffers. The Image
//! editor's header and info bar read this every frame; the picture itself
//! is fetched through `render_job::shot` only when its sequence moves.

use super::render_job::{BRenderShot, job};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BRenderStatus {
    pub w: u32,
    pub h: u32,
    pub taken: u32,
    pub target: u32,
    pub done: bool,
    pub has_image: bool,
    /// Which extra passes the shot carries: depth, normal, albedo.
    pub depth: bool,
    pub normal: bool,
    pub albedo: bool,
    /// Bytes the shot's buffers hold, plus what the renderer is holding on
    /// the device — the `Mem:` of the info bar.
    pub bytes: usize,
    /// The most `bytes` has been this session — the info bar's `Peak:`.
    pub peak: usize,
    /// The sequence number of the shot this describes.
    pub seq: u32,
}

impl BRenderStatus {
    fn of(shot: &BRenderShot, seq: u32) -> Self {
        Self {
            w: shot.w,
            h: shot.h,
            taken: shot.taken,
            target: shot.target,
            done: shot.done,
            has_image: shot.has_image(),
            depth: !shot.depth.is_empty(),
            normal: !shot.normal.is_empty(),
            albedo: !shot.albedo.is_empty(),
            bytes: held(
                shot.bgra.len()
                    + (shot.display.len() + shot.linear.len() + shot.albedo.len() + shot.normal.len()) * 12
                    + shot.depth.len() * 4,
            ),
            peak: PEAK.load(Ordering::Relaxed) as usize,
            seq,
        }
    }

    /// `Mem:` in whole megabytes, rounded up as Blender rounds it.
    pub fn megabytes(&self) -> u64 {
        (self.bytes as u64).div_ceil(1024 * 1024)
    }

    /// `Peak:` in whole megabytes.
    pub fn peak_megabytes(&self) -> u64 {
        (self.peak as u64).div_ceil(1024 * 1024)
    }
}

/// Bytes the renderer is holding on the device, published by the render
/// path (`Tracer::bytes`), and the most it has ever held.
///
/// `Mem:` used to count the shot's host-side buffers alone, which on a
/// 2.4-million-triangle scene read **8M** while the card held hundreds of
/// megabytes of tree, vertices and textures. Blender's `Mem:` is what the
/// renderer holds, so this adds it. `Peak:` used to print the same number
/// as `Mem:` — it was never a peak at all — and now keeps the high-water
/// mark of the session, which is what Blender shows.
static SCENE: AtomicU64 = AtomicU64::new(0);
static PEAK: AtomicU64 = AtomicU64::new(0);

/// What the renderer holds on the device now. Call when a tracer is built
/// or dropped; `0` when there is none.
pub fn set_scene_bytes(bytes: u64) {
    SCENE.store(bytes, Ordering::Relaxed);
}

/// `shot` bytes plus the device's, with the peak kept.
fn held(shot: usize) -> usize {
    let total = shot as u64 + SCENE.load(Ordering::Relaxed);
    PEAK.fetch_max(total, Ordering::Relaxed);
    total as usize
}

/// The render in flight or the last one finished, described without a copy.
pub fn status() -> Option<BRenderStatus> {
    let j = job();
    let seq = j.seq.load(Ordering::Relaxed);
    j.slot(j.active.load(Ordering::Relaxed)).map(|shot| BRenderStatus::of(&shot, seq))
}

/// How long the render took, or how long it has been going.
///
/// Blender's `RenderStats.lastframetime` (`render/intern/pipeline.cc:1510`) is
/// set ONCE when a frame ends — `BLI_time_now_seconds() - starttime` — and
/// `make_renderinfo_string` prints that stored value from then on. So a finished
/// render shows the time it took, frozen.
///
/// This used to return `started.elapsed()` with no finish time kept at all, so
/// the bar counted up for as long as the app stayed open: the header read
/// `Finished` beside a clock at 36 minutes on a render that had taken seconds.
pub fn elapsed() -> Duration {
    let j = job();
    if let Some(took) = j.took.lock().ok().and_then(|took| *took) {
        return took;
    }
    j.started.lock().ok().and_then(|started| started.map(|t| t.elapsed())).unwrap_or_default()
}

/// What is left of a render at the pace it has kept — Cycles'
/// `Session::update_status_time`: `(1 − progress) · elapsed / progress`,
/// which Blender prints as `Remaining:` while the render runs and drops
/// once it is done. `None` before the first sample, and once finished.
pub fn remaining(elapsed: Duration, taken: u32, target: u32, done: bool) -> Option<Duration> {
    if done || taken == 0 || target == 0 || taken >= target {
        return None;
    }
    let progress = taken as f64 / target as f64;
    Some(Duration::from_secs_f64((1.0 - progress) * elapsed.as_secs_f64() / progress))
}

/// `BLI_timecode_string_from_time_simple`: `MM:SS.CC`.
pub fn timecode(duration: Duration) -> String {
    let centis = duration.as_millis() / 10;
    format!("{:02}:{:02}.{:02}", centis / 6000, (centis / 100) % 60, centis % 100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_timecode_is_blenders_simple_form() {
        assert_eq!(timecode(Duration::from_millis(0)), "00:00.00");
        assert_eq!(timecode(Duration::from_millis(12_340)), "00:12.34");
        assert_eq!(timecode(Duration::from_millis(61_500)), "01:01.50");
    }

    #[test]
    fn remaining_is_cycles_pace_estimate_and_vanishes_when_done() {
        let ten = Duration::from_secs(10);
        assert_eq!(remaining(ten, 25, 100, false), Some(Duration::from_secs(30)), "a quarter done in 10 s leaves 30 s");
        assert_eq!(remaining(ten, 0, 100, false), None, "no pace yet");
        assert_eq!(remaining(ten, 100, 100, false), None);
        assert_eq!(remaining(ten, 40, 100, true), None, "an adaptive render that settled early is done");
    }

    #[test]
    fn a_status_counts_its_buffers_without_copying_them() {
        let shot = BRenderShot { w: 4, h: 2, bgra: vec![0; 32], depth: vec![0.0; 8], taken: 3, target: 8, ..Default::default() };
        let status = BRenderStatus::of(&shot, 7);
        assert_eq!((status.w, status.h, status.taken, status.target, status.seq), (4, 2, 3, 8, 7));
        assert!(status.depth && !status.normal && !status.albedo && status.has_image && !status.done);
        assert_eq!(status.bytes, 32 + 32);
        assert_eq!(status.megabytes(), 1);
    }
}
