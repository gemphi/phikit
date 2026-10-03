//! What the render in flight looks like from the outside — size, samples,
//! passes carried, time taken — without copying its buffers. The Image
//! editor's header and info bar read this every frame; the picture itself
//! is fetched through `render_job::shot` only when its sequence moves.

use super::render_job::{BRenderShot, job};
use std::sync::atomic::Ordering;
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
    /// Bytes the shot's buffers hold — the `Mem:` of the info bar.
    pub bytes: usize,
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
            bytes: shot.bgra.len()
                + (shot.display.len() + shot.linear.len() + shot.albedo.len() + shot.normal.len()) * 12
                + shot.depth.len() * 4,
            seq,
        }
    }

    /// `Mem:` in whole megabytes, rounded up as Blender rounds it.
    pub fn megabytes(&self) -> u64 {
        (self.bytes as u64).div_ceil(1024 * 1024)
    }
}

/// The render in flight or the last one finished, described without a copy.
pub fn status() -> Option<BRenderStatus> {
    let j = job();
    let seq = j.seq.load(Ordering::Relaxed);
    j.shot.lock().ok().and_then(|shot| shot.as_ref().map(|shot| BRenderStatus::of(shot, seq)))
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
    fn a_status_counts_its_buffers_without_copying_them() {
        let shot = BRenderShot { w: 4, h: 2, bgra: vec![0; 32], depth: vec![0.0; 8], taken: 3, target: 8, ..Default::default() };
        let status = BRenderStatus::of(&shot, 7);
        assert_eq!((status.w, status.h, status.taken, status.target, status.seq), (4, 2, 3, 8, 7));
        assert!(status.depth && !status.normal && !status.albedo && status.has_image && !status.done);
        assert_eq!(status.bytes, 32 + 32);
        assert_eq!(status.megabytes(), 1);
    }
}
