//! The Playback popover's settings (`TIME_PT_playback`), each an RNA
//! property of the scene or the screen kept under `ctx.buts.<rna name>`
//! like every other property row: read from the file when it opens
//! (`seed`), written back when it saves (`now` → `BlendEdits.playback`).
//! The popover's layout is `panel.rs`.

use crate::phiphx::blend::{BlendLoopMode, BlendPlayback, BlendSyncMode};
use crate::phiphx::blend::{TIME_ALL_3D_WIN, TIME_ALL_ANIM_WIN, TIME_ALL_BUTS_WIN, TIME_ALL_IMAGE_WIN, TIME_CLIPS, TIME_NODES, TIME_REGION, TIME_SEQ, TIME_SPREADSHEETS};
use crate::v2::state::{BKey as _, BPath, BState, ButsKey, val::BValue};

/// `Scene.sync_mode`'s items, as RNA names them.
pub const SYNC_MODES: [(&str, &str, BlendSyncMode); 3] = [
    ("NONE", "Play Every Frame", BlendSyncMode::EveryFrame),
    ("FRAME_DROP", "Frame Dropping", BlendSyncMode::FrameDrop),
    ("AUDIO_SYNC", "Sync to Audio", BlendSyncMode::AudioSync),
];

/// `Scene.playback_loop_mode`'s items.
pub const LOOP_MODES: [(&str, &str, BlendLoopMode); 5] = [
    ("INFINITE", "Infinite", BlendLoopMode::Infinite),
    ("STOP_END_FRAME", "Stop at End Frame", BlendLoopMode::StopEndFrame),
    ("STOP_START_FRAME", "Stop at Start Frame", BlendLoopMode::StopStartFrame),
    ("RESTORE", "Restore Frame", BlendLoopMode::Restore),
    ("BOUNCE", "Bounce", BlendLoopMode::Bounce),
];

/// One checkbox of the popover: its RNA name, its label, and — for the
/// screen's `use_play_*` — the `redraws_flag` bit it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BPlayToggle {
    pub key: &'static str,
    pub label: &'static str,
    pub bit: u32,
}

const fn toggle(key: &'static str, label: &'static str) -> BPlayToggle {
    BPlayToggle { key, label, bit: 0 }
}

const fn play_in(key: &'static str, label: &'static str, bit: u32) -> BPlayToggle {
    BPlayToggle { key, label, bit }
}

/// The popover's column headings and their rows, in Blender's order.
pub const AUDIO: [BPlayToggle; 2] = [toggle("use_audio_scrub", "Scrubbing"), toggle("use_audio", "Play Audio")];
pub const PLAYBACK: [BPlayToggle; 3] = [
    toggle("lock_frame_selection_to_range", "Limit to Frame Range"),
    toggle("allow_preroll", "Allow Preroll"),
    toggle("use_follow", "Follow Current Frame"),
];
pub const PLAY_IN: [BPlayToggle; 9] = [
    play_in("use_play_top_left_3d_editor", "Active Editor", TIME_REGION),
    play_in("use_play_3d_editors", "3D Viewport", TIME_ALL_3D_WIN),
    play_in("use_play_animation_editors", "Animation Editors", TIME_ALL_ANIM_WIN),
    play_in("use_play_image_editors", "Image Editor", TIME_ALL_IMAGE_WIN),
    play_in("use_play_properties_editors", "Properties and Sidebars", TIME_ALL_BUTS_WIN),
    play_in("use_play_clip_editors", "Movie Clip Editor", TIME_CLIPS),
    play_in("use_play_node_editors", "Node Editors", TIME_NODES),
    play_in("use_play_sequence_editors", "Video Sequencer", TIME_SEQ),
    play_in("use_play_spreadsheet_editors", "Spreadsheet", TIME_SPREADSHEETS),
];
pub const SHOW: [BPlayToggle; 1] = [toggle("show_subframe", "Subframes")];
pub const SYNC_KEY: &str = "sync_mode";
pub const LOOP_KEY: &str = "playback_loop_mode";

/// The Time Jump popover (`TIME_PT_jump`): `Scene.time_jump_unit` and
/// `time_jump_delta` — one second by default (`DNA_scene_types.h`).
pub const JUMP_UNITS: [(&str, &str); 2] = [("FRAME", "Frame"), ("SECOND", "Second")];
pub const JUMP_UNIT_KEY: &str = "time_jump_unit";
pub const JUMP_DELTA_KEY: &str = "time_jump_delta";
/// The Auto Keyframing popover (`TIME_PT_auto_keyframing`).
pub const AUTO_KEY_MODES: [(&str, &str); 2] = [("ADD_REPLACE_KEYS", "Add & Replace"), ("REPLACE_KEYS", "Replace")];
pub const AUTO_KEY_MODE_KEY: &str = "auto_keying_mode";
pub const AUTO_KEY_SET: BPlayToggle = toggle("use_keyframe_insert_keyingset", "Only Active Keying Set");

/// How many frames a time jump moves: the delta, in seconds at `fps` or in
/// frames, never less than one.
pub fn time_jump_frames(s: &BState, fps: f32) -> i32 {
    let delta = match s.value(&key(JUMP_DELTA_KEY), crate::v2::state::val::BValueKind::Float) {
        Some(BValue::Float(delta)) => delta,
        _ => 1.0,
    };
    let frames = if enum_of(s, JUMP_UNIT_KEY, "SECOND") == "FRAME" { delta } else { delta * fps };
    (frames.round() as i32).max(1)
}

/// The state key of a setting.
pub fn key(rna: &'static str) -> BPath {
    ButsKey::Value(rna).path()
}

/// A toggle's default — the factory file's.
pub fn default_of(rna: &str) -> bool {
    let factory = BlendPlayback::default();
    match rna {
        "use_audio_scrub" => factory.audio_scrub,
        "use_audio" => factory.audio_play,
        "lock_frame_selection_to_range" => factory.lock_frame_selection,
        "allow_preroll" => factory.allow_preroll,
        "use_follow" => factory.follow,
        "show_subframe" => factory.show_subframe,
        _ => PLAY_IN.iter().find(|t| t.key == rna).is_some_and(|t| factory.play_in & t.bit != 0),
    }
}

/// Whether a toggle is on: what the state says, else its default.
pub fn is_on(s: &BState, rna: &'static str) -> bool {
    match s.value(&key(rna), crate::v2::state::val::BValueKind::Bool) {
        Some(BValue::Bool(on)) => on,
        _ => default_of(rna),
    }
}

fn enum_of(s: &BState, rna: &'static str, fallback: &str) -> String {
    match s.value(&key(rna), crate::v2::state::val::BValueKind::Enum) {
        Some(BValue::Enum(v)) => v,
        _ => fallback.to_string(),
    }
}

pub fn sync_mode(s: &BState) -> BlendSyncMode {
    let chosen = enum_of(s, SYNC_KEY, SYNC_MODES[2].0);
    SYNC_MODES.iter().find(|(k, _, _)| *k == chosen).map_or(BlendSyncMode::AudioSync, |(_, _, m)| *m)
}

pub fn loop_mode(s: &BState) -> BlendLoopMode {
    let chosen = enum_of(s, LOOP_KEY, LOOP_MODES[0].0);
    LOOP_MODES.iter().find(|(k, _, _)| *k == chosen).map_or(BlendLoopMode::Infinite, |(_, _, m)| *m)
}

/// The settings as the state has them — what a save writes.
pub fn now(s: &BState) -> BlendPlayback {
    BlendPlayback {
        sync: sync_mode(s),
        audio_scrub: is_on(s, "use_audio_scrub"),
        audio_play: is_on(s, "use_audio"),
        lock_frame_selection: is_on(s, "lock_frame_selection_to_range"),
        allow_preroll: is_on(s, "allow_preroll"),
        show_subframe: is_on(s, "show_subframe"),
        loop_mode: loop_mode(s),
        follow: is_on(s, "use_follow"),
        play_in: PLAY_IN.iter().filter(|t| is_on(s, t.key)).fold(0, |acc, t| acc | t.bit),
        auto_key: s.get(crate::v2::state::CtxKey::AutoKey) == Some(crate::v2::state::val::V_TRUE),
        auto_key_replace: enum_of(s, AUTO_KEY_MODE_KEY, AUTO_KEY_MODES[0].0) == AUTO_KEY_MODES[1].0,
        only_keying_set: is_on(s, AUTO_KEY_SET.key),
        jump_in_frames: enum_of(s, JUMP_UNIT_KEY, "SECOND") == "FRAME",
        jump_delta: match s.value(&key(JUMP_DELTA_KEY), crate::v2::state::val::BValueKind::Float) {
            Some(BValue::Float(delta)) => delta,
            _ => 1.0,
        },
        ..snapped(s)
    }
}

/// The defaults with the playhead snapping the state has.
fn snapped(s: &BState) -> BlendPlayback {
    let mut p = BlendPlayback::default();
    super::snap::BPlayheadSnap::now(s).write_into(&mut p);
    p
}

/// The file's settings into the state, when it opens.
pub fn seed(s: &mut BState, p: &BlendPlayback) {
    let sync = SYNC_MODES.iter().find(|(_, _, m)| *m == p.sync).map_or("AUDIO_SYNC", |(k, _, _)| k);
    s.set(&key(SYNC_KEY), BValue::Enum(sync.to_string()).encode());
    let loop_key = LOOP_MODES.iter().find(|(_, _, m)| *m == p.loop_mode).map_or("INFINITE", |(k, _, _)| k);
    s.set(&key(LOOP_KEY), BValue::Enum(loop_key.to_string()).encode());
    for (rna, on) in [
        ("use_audio_scrub", p.audio_scrub),
        ("use_audio", p.audio_play),
        ("lock_frame_selection_to_range", p.lock_frame_selection),
        ("allow_preroll", p.allow_preroll),
        ("use_follow", p.follow),
        ("show_subframe", p.show_subframe),
    ] {
        s.set(&key(rna), BValue::Bool(on).encode());
    }
    for t in PLAY_IN {
        s.set(&key(t.key), BValue::Bool(p.play_in & t.bit != 0).encode());
    }
    // The transport's popovers, and the record button itself.
    s.set(crate::v2::state::CtxKey::AutoKey, BValue::Bool(p.auto_key).encode());
    s.set(&key(AUTO_KEY_MODE_KEY), BValue::Enum(AUTO_KEY_MODES[usize::from(p.auto_key_replace)].0.to_string()).encode());
    s.set(&key(AUTO_KEY_SET.key), BValue::Bool(p.only_keying_set).encode());
    s.set(&key(JUMP_UNIT_KEY), BValue::Enum(if p.jump_in_frames { "FRAME" } else { "SECOND" }.to_string()).encode());
    s.set(&key(JUMP_DELTA_KEY), BValue::Float(p.jump_delta).encode());
    super::snap::BPlayheadSnap::seed(s, p);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_time_jump_is_a_second_until_the_popover_says_otherwise() {
        let mut s = BState::new();
        assert_eq!(time_jump_frames(&s, 24.0), 24);
        s.set(&key(JUMP_DELTA_KEY), BValue::Float(2.5).encode());
        assert_eq!(time_jump_frames(&s, 24.0), 60);
        s.set(&key(JUMP_UNIT_KEY), BValue::Enum("FRAME".into()).encode());
        assert_eq!(time_jump_frames(&s, 24.0), 3, "2.5 frames round to 3");
        s.set(&key(JUMP_DELTA_KEY), BValue::Float(0.0).encode());
        assert_eq!(time_jump_frames(&s, 24.0), 1, "never a jump of nothing");
    }

    #[test]
    fn the_settings_default_to_the_factory_file_and_round_trip_through_the_state() {
        let mut s = BState::new();
        assert_eq!(now(&s), BlendPlayback::default(), "nothing set: the factory settings");
        let wanted = BlendPlayback {
            sync: BlendSyncMode::FrameDrop,
            audio_scrub: true,
            audio_play: false,
            lock_frame_selection: true,
            allow_preroll: true,
            show_subframe: true,
            loop_mode: BlendLoopMode::Restore,
            follow: true,
            play_in: TIME_REGION | TIME_SEQ,
            auto_key: true,
            auto_key_replace: true,
            only_keying_set: true,
            jump_in_frames: true,
            jump_delta: 3.0,
            ..Default::default()
        };
        seed(&mut s, &wanted);
        assert_eq!(now(&s), wanted);
        assert_eq!(s.get(&key(SYNC_KEY)), Some("FRAME_DROP"));
        assert!(is_on(&s, "use_play_sequence_editors") && !is_on(&s, "use_play_3d_editors"));
    }
}
