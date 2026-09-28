//! Playhead snapping — the transport's magnet and its popover
//! (`TIME_PT_playhead_snapping`, Blender 5.0): while the playhead is
//! scrubbed it snaps to the nearest target — the next frame step or second
//! step from the start frame, which it always takes, or a marker or a key
//! column, which it takes only within `playhead_snap_distance` pixels
//! (`anim_ops.cc::apply_frame_snap`). Ctrl held while scrubbing flips it.
//!
//! The settings live under `ctx.buts.<rna>` as the rest of the transport's
//! (`settings::key`) and go to `ToolSettings` on save.

use super::settings::{BPlayToggle, key};
use crate::phiphx::blend::{BlendPlayback, SCE_SNAP_TO_FRAME, SCE_SNAP_TO_KEYS, SCE_SNAP_TO_MARKERS, SCE_SNAP_TO_SECOND, SCE_SNAP_TO_STRIPS};
use crate::v2::state::key::{float_or, int_or};
use crate::v2::state::{BState, CtxKey, val::{BValue, BValueKind}};

/// `use_snap_playhead`: the magnet.
pub const ON_KEY: &str = "use_snap_playhead";
pub const FRAME_STEP_KEY: &str = "snap_playhead_frame_step";
pub const SECOND_STEP_KEY: &str = "snap_playhead_second_step";
pub const DISTANCE_KEY: &str = "playhead_snap_distance";

/// `snap_playhead_element`'s items, each a box: key, label, bit. Strips
/// (`SCE_SNAP_TO_STRIPS`) is the Sequencer's and is not shown, but kept.
pub static TARGETS: [BPlayToggle; 4] = [
    BPlayToggle { key: "snap_playhead_to_frame", label: "Frames", bit: SCE_SNAP_TO_FRAME },
    BPlayToggle { key: "snap_playhead_to_second", label: "Seconds", bit: SCE_SNAP_TO_SECOND },
    BPlayToggle { key: "snap_playhead_to_marker", label: "Markers", bit: SCE_SNAP_TO_MARKERS },
    BPlayToggle { key: "snap_playhead_to_key", label: "Keyframes", bit: SCE_SNAP_TO_KEYS },
];
const STRIPS_KEY: &str = "snap_playhead_to_strip";

fn bool_or(s: &BState, rna: &'static str, factory: bool) -> bool {
    match s.value(&key(rna), BValueKind::Bool) {
        Some(BValue::Bool(on)) => on,
        _ => factory,
    }
}

fn int(s: &BState, rna: &'static str, factory: u32) -> u32 {
    match s.value(&key(rna), BValueKind::Int) {
        Some(BValue::Int(n)) => n.max(1) as u32,
        _ => factory,
    }
}

/// The snapping as the state has it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPlayheadSnap {
    pub on: bool,
    pub to: u32,
    pub frames: u32,
    pub seconds: u32,
    pub distance: u32,
}

impl BPlayheadSnap {
    pub fn now(s: &BState) -> Self {
        let f = BlendPlayback::default();
        let strips = if bool_or(s, STRIPS_KEY, f.snap_to & SCE_SNAP_TO_STRIPS != 0) { SCE_SNAP_TO_STRIPS } else { 0 };
        let to = TARGETS.iter().filter(|t| bool_or(s, t.key, f.snap_to & t.bit != 0)).fold(strips, |acc, t| acc | t.bit);
        Self {
            on: bool_or(s, ON_KEY, f.snap_playhead),
            to,
            frames: int(s, FRAME_STEP_KEY, f.snap_frames),
            seconds: int(s, SECOND_STEP_KEY, f.snap_seconds),
            distance: int(s, DISTANCE_KEY, f.snap_distance),
        }
    }

    /// Into what a save writes.
    pub fn write_into(self, p: &mut BlendPlayback) {
        (p.snap_playhead, p.snap_to, p.snap_frames, p.snap_seconds, p.snap_distance) = (self.on, self.to, self.frames, self.seconds, self.distance);
    }

    /// The file's snapping into the state.
    pub fn seed(s: &mut BState, p: &BlendPlayback) {
        s.set(&key(ON_KEY), BValue::Bool(p.snap_playhead).encode());
        for t in &TARGETS {
            s.set(&key(t.key), BValue::Bool(p.snap_to & t.bit != 0).encode());
        }
        s.set(&key(STRIPS_KEY), BValue::Bool(p.snap_to & SCE_SNAP_TO_STRIPS != 0).encode());
        for (rna, n) in [(FRAME_STEP_KEY, p.snap_frames), (SECOND_STEP_KEY, p.snap_seconds), (DISTANCE_KEY, p.snap_distance)] {
            s.set(&key(rna), BValue::Int(i64::from(n)).encode());
        }
    }

    /// Does a scrub snap now — the magnet, flipped while Ctrl is held?
    pub fn active(self, ctrl: bool) -> bool {
        self.on != ctrl
    }

    /// `frame` snapped: the nearest marker or key within `reach` frames
    /// (the distance in pixels, at the window's zoom) if there is one, else
    /// the nearest frame or second step, else `frame` as it is.
    pub fn snap(self, frame: f32, reach: f32, targets: &BSnapTargets) -> f32 {
        let nearest = |list: &mut dyn Iterator<Item = f32>| list.min_by(|a, b| (a - frame).abs().total_cmp(&(b - frame).abs()));
        let step = |every: f32| targets.start + ((frame - targets.start) / every).round() * every;
        let mut close = Vec::new();
        if self.to & SCE_SNAP_TO_MARKERS != 0 {
            close.extend(nearest(&mut targets.markers.iter().copied()));
        }
        if self.to & SCE_SNAP_TO_KEYS != 0 {
            close.extend(nearest(&mut targets.keys.iter().copied()));
        }
        let mut hard = Vec::new();
        if self.to & SCE_SNAP_TO_SECOND != 0 {
            hard.push(step(targets.fps * self.seconds as f32));
        }
        if self.to & SCE_SNAP_TO_FRAME != 0 {
            hard.push(step(self.frames as f32));
        }
        let all = nearest(&mut close.iter().chain(&hard).copied());
        if let Some(best) = all.filter(|best| (best - frame).abs() < reach) {
            return best;
        }
        nearest(&mut hard.into_iter()).unwrap_or(frame)
    }
}

/// What a scrub can snap to, read once per render of the window.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BSnapTargets {
    pub start: f32,
    pub fps: f32,
    pub markers: Vec<f32>,
    pub keys: Vec<f32>,
}

impl BSnapTargets {
    pub fn now(s: &BState) -> Self {
        Self {
            start: int_or(s, CtxKey::FrameStart, 1) as f32,
            fps: float_or(s, CtxKey::Fps, 24.0),
            markers: crate::v2::scene::marker::BMarkerList::now(s).list.iter().map(|(f, _)| *f as f32).collect(),
            keys: crate::v2::scene::BShownKeys::now(s).columns().to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_and_markers_snap_near_steps_always_and_the_file_round_trips() {
        let targets = BSnapTargets { start: 1.0, fps: 24.0, markers: vec![40.0], keys: vec![10.0, 30.0] };
        let factory = BPlayheadSnap::now(&BState::new());
        assert!(!factory.on && factory.to == SCE_SNAP_TO_KEYS | SCE_SNAP_TO_STRIPS && factory.frames == 2 && factory.distance == 20);
        // Keyframes only: within reach the key wins, beyond it the frame stays.
        assert_eq!(factory.snap(28.6, 3.0, &targets), 30.0);
        assert_eq!(factory.snap(20.0, 3.0, &targets), 20.0);
        // Frames every 2 from frame 1: always taken when nothing is nearer.
        let frames = BPlayheadSnap { to: SCE_SNAP_TO_FRAME, ..factory };
        assert_eq!(frames.snap(20.2, 3.0, &targets), 21.0);
        let seconds = BPlayheadSnap { to: SCE_SNAP_TO_SECOND | SCE_SNAP_TO_MARKERS, ..factory };
        assert_eq!(seconds.snap(38.5, 3.0, &targets), 40.0, "the marker is nearer than second 25");
        assert_eq!(seconds.snap(30.0, 3.0, &targets), 25.0, "a second from the start");
        assert!(factory.active(true) && !factory.active(false), "Ctrl flips the magnet");

        let mut s = BState::new();
        let wanted = BlendPlayback { snap_playhead: true, snap_to: SCE_SNAP_TO_FRAME | SCE_SNAP_TO_MARKERS, snap_frames: 5, snap_seconds: 3, snap_distance: 9, ..Default::default() };
        BPlayheadSnap::seed(&mut s, &wanted);
        let mut back = BlendPlayback::default();
        BPlayheadSnap::now(&s).write_into(&mut back);
        assert_eq!(back, wanted);
    }
}
