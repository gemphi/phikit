//! Playback settings — what the Timeline's Playback popover shows
//! (`TIME_PT_playback`): the scene's sync mode (`Scene.audio.flag`
//! `AUDIO_SYNC`, `Scene.flag` `SCE_FRAME_DROP`), audio scrubbing and mute
//! (`AudioData.flag`), the frame lock, preroll and subframes
//! (`RenderData.flag`), the loop mode (`Scene.playback_loop_mode`, 5.0),
//! and — on the active screen, `bScreen.redraws_flag` — following the
//! frame (`TIME_FOLLOW`) and which editors play (`TIME_ALL_3D_WIN`…). The
//! transport's own two popovers ride along: auto keying
//! (`ToolSettings.autokey_mode`, `keying_flag`) and the time jump
//! (`RenderData.time_jump_unit / _delta`, 5.0 on), and playhead snapping
//! (`ToolSettings.snap_flag_playhead`, `snap_playhead_mode`, the steps and
//! the distance, where the file's Blender has them).

use crate::blend::file::types::*;

/// `AudioData.flag`.
pub const AUDIO_MUTE: u32 = 1 << 0;
pub const AUDIO_SYNC: u32 = 1 << 1;
pub const AUDIO_SCRUB: u32 = 1 << 2;
/// `Scene.flag`.
pub const SCE_FRAME_DROP: u32 = 1 << 3;
/// `RenderData.flag`.
pub const SCER_LOCK_FRAME_SELECTION: u32 = 1 << 1;
pub const SCER_ALLOW_PREROLL: u32 = 1 << 2;
pub const SCER_SHOW_SUBFRAME: u32 = 1 << 3;
/// `ToolSettings.autokey_mode` (`eAutokey_Mode`): on, and which mode.
pub const AUTOKEY_ON: u32 = 1;
pub const AUTOKEY_MODE_EDITKEYS: u32 = 5;
pub const AUTOKEY_MODE_NORMAL: u32 = 3;
/// `ToolSettings.keying_flag` (`autokey_flag` before 4.x).
pub const AUTOKEY_FLAG_ONLYKEYINGSET: u32 = 1 << 6;
/// `ToolSettings.snap_flag_playhead` (`eSnapFlag`) and `snap_playhead_mode`
/// (`eSnapMode`): what the playhead snaps to while it is scrubbed.
pub const SCE_SNAP: u32 = 1 << 0;
pub const SCE_SNAP_TO_FRAME: u32 = 1 << 0;
pub const SCE_SNAP_TO_SECOND: u32 = 1 << 1;
pub const SCE_SNAP_TO_MARKERS: u32 = 1 << 2;
pub const SCE_SNAP_TO_KEYS: u32 = 1 << 3;
pub const SCE_SNAP_TO_STRIPS: u32 = 1 << 4;
/// `bScreen.redraws_flag` (`eScreen_Redraws_Flag`).
pub const TIME_REGION: u32 = 1 << 0;
pub const TIME_ALL_3D_WIN: u32 = 1 << 1;
pub const TIME_ALL_ANIM_WIN: u32 = 1 << 2;
pub const TIME_ALL_BUTS_WIN: u32 = 1 << 3;
pub const TIME_SEQ: u32 = 1 << 5;
pub const TIME_ALL_IMAGE_WIN: u32 = 1 << 6;
pub const TIME_NODES: u32 = 1 << 8;
pub const TIME_CLIPS: u32 = 1 << 9;
pub const TIME_SPREADSHEETS: u32 = 1 << 10;
pub const TIME_FOLLOW: u32 = 1 << 15;
/// The `redraws_flag` bits that are editors playing, not the follow.
pub const PLAY_IN_MASK: u32 = TIME_REGION | TIME_ALL_3D_WIN | TIME_ALL_ANIM_WIN | TIME_ALL_BUTS_WIN | TIME_SEQ | TIME_ALL_IMAGE_WIN | TIME_NODES | TIME_CLIPS | TIME_SPREADSHEETS;

/// `Scene.sync_mode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendSyncMode {
    EveryFrame,
    FrameDrop,
    AudioSync,
}

/// `Scene.playback_loop_mode` (`eScenePlaybackLoopMode`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlendLoopMode {
    Infinite,
    StopEndFrame,
    StopStartFrame,
    Restore,
    Bounce,
}

impl BlendLoopMode {
    pub const ALL: [Self; 5] = [Self::Infinite, Self::StopEndFrame, Self::StopStartFrame, Self::Restore, Self::Bounce];

    pub fn from_dna(value: u32) -> Self {
        Self::ALL.get(value as usize).copied().unwrap_or(Self::Infinite)
    }

    pub fn to_dna(self) -> u32 {
        Self::ALL.iter().position(|m| *m == self).unwrap_or(0) as u32
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlendPlayback {
    pub sync: BlendSyncMode,
    pub audio_scrub: bool,
    /// `use_audio`: not `AUDIO_MUTE`.
    pub audio_play: bool,
    pub lock_frame_selection: bool,
    pub allow_preroll: bool,
    pub show_subframe: bool,
    pub loop_mode: BlendLoopMode,
    pub follow: bool,
    /// The editors that play: `redraws_flag & PLAY_IN_MASK`.
    pub play_in: u32,
    /// The record button (`use_keyframe_insert_auto`), Replace rather than
    /// Add & Replace, and Only Active Keying Set.
    pub auto_key: bool,
    pub auto_key_replace: bool,
    pub only_keying_set: bool,
    /// The time jump: in frames rather than seconds, and by how much.
    pub jump_in_frames: bool,
    pub jump_delta: f32,
    /// Playhead snapping: on, what to (`SCE_SNAP_TO_*`), the frame and
    /// second steps, and how near in pixels a key or marker must be.
    pub snap_playhead: bool,
    pub snap_to: u32,
    pub snap_frames: u32,
    pub snap_seconds: u32,
    pub snap_distance: u32,
}

impl Default for BlendPlayback {
    /// Blender's factory settings: `AudioData.flag = AUDIO_SYNC`,
    /// `redraws_flag = TIME_ALL_3D_WIN | TIME_ALL_ANIM_WIN`.
    fn default() -> Self {
        Self {
            sync: BlendSyncMode::AudioSync,
            audio_scrub: false,
            audio_play: true,
            lock_frame_selection: false,
            allow_preroll: false,
            show_subframe: false,
            loop_mode: BlendLoopMode::Infinite,
            follow: false,
            play_in: TIME_ALL_3D_WIN | TIME_ALL_ANIM_WIN,
            auto_key: false,
            auto_key_replace: false,
            only_keying_set: false,
            jump_in_frames: false,
            jump_delta: 1.0,
            snap_playhead: false,
            snap_to: SCE_SNAP_TO_KEYS | SCE_SNAP_TO_STRIPS,
            snap_frames: 2,
            snap_seconds: 1,
            snap_distance: 20,
        }
    }
}

impl BlendFile {
    /// A field of a struct embedded in another: `Scene.audio.flag`.
    pub(crate) fn nested_int(&self, block: &BlendBlock, outer: &str, member: &str, inner: &str, field: &str, dna: &DnaType) -> Option<u32> {
        let base = self.off(outer, member, dna)?;
        let ps = self.header.pointer_size as usize;
        let layout = dna.layout(ps);
        let &(inner_off, width) = layout.structure(inner)?.fields.get(field)?;
        let o = base + inner_off;
        Some(match width {
            1 => *block.data.get(o)? as u32,
            2 => self.read_u16(block.data.get(o..o + 2)?) as u32,
            _ => self.read_i32(block.data.get(o..o + 4)?) as u32,
        })
    }

    /// The scene's `ToolSettings` block, by index.
    pub(crate) fn tool_settings_index(&self, sc: &BlendBlock, dna: &DnaType) -> Option<usize> {
        let address = self.ptr(sc, self.off("Scene", "*toolsettings", dna));
        self.blocks.iter().position(|b| address != 0 && b.old_address == address)
    }

    /// The keying flags' field: `keying_flag`, `autokey_flag` before it.
    pub(crate) fn keying_flag_field(&self, dna: &DnaType) -> Option<&'static str> {
        ["keying_flag", "autokey_flag"].into_iter().find(|f| self.off("ToolSettings", f, dna).is_some())
    }

    /// The active screen — the window's workspace layout's, else the
    /// file's `curscreen` — as `extract_ui_state` walks to it.
    pub(crate) fn active_screen_index(&self, dna: &DnaType) -> Option<usize> {
        let wm = self.blocks.iter().find(|b| b.code_str == "WM");
        let win = wm.and_then(|w| self.block_at(self.ptr(w, self.off("wmWindowManager", "windows", dna))));
        let hook = win.and_then(|w| self.block_at(self.ptr(w, self.off("wmWindow", "*workspace_hook", dna))));
        let layout = hook.and_then(|h| self.block_at(self.ptr(h, self.off("WorkSpaceInstanceHook", "*act_layout", dna))));
        let mut address = layout.map(|l| self.ptr(l, self.off("WorkSpaceLayout", "*screen", dna))).unwrap_or(0);
        if self.block_at(address).is_none() {
            let glob = self.blocks.iter().find(|b| b.code_str == "GLOB");
            address = glob.map(|g| self.ptr(g, self.off("FileGlobal", "*curscreen", dna))).unwrap_or(0);
        }
        self.blocks.iter().position(|b| address != 0 && b.old_address == address)
    }

    pub(crate) fn extract_playback(&self, sc: &BlendBlock, dna: &DnaType) -> BlendPlayback {
        let factory = BlendPlayback::default();
        let audio = self.nested_int(sc, "Scene", "audio", "AudioData", "flag", dna).unwrap_or(AUDIO_SYNC);
        let scene_flag = self.int_field(sc, "Scene", "flag", dna);
        let render_flag = self.nested_int(sc, "Scene", "r", "RenderData", "flag", dna).unwrap_or(0);
        let sync = if audio & AUDIO_SYNC != 0 {
            BlendSyncMode::AudioSync
        } else if scene_flag & SCE_FRAME_DROP != 0 {
            BlendSyncMode::FrameDrop
        } else {
            BlendSyncMode::EveryFrame
        };
        let loop_mode = self.off("Scene", "playback_loop_mode", dna).map_or(factory.loop_mode, |_| BlendLoopMode::from_dna(self.int_field(sc, "Scene", "playback_loop_mode", dna)));
        let redraws = self
            .active_screen_index(dna)
            .map(|i| self.int_field(&self.blocks[i], "bScreen", "redraws_flag", dna))
            .unwrap_or(factory.play_in);
        let ts = self.tool_settings_index(sc, dna).map(|i| &self.blocks[i]);
        let autokey = ts.map_or(0, |ts| self.int_field(ts, "ToolSettings", "autokey_mode", dna));
        let keying = ts.zip(self.keying_flag_field(dna)).map_or(0, |(ts, f)| self.int_field(ts, "ToolSettings", f, dna));
        let snaps = ts.filter(|_| self.off("ToolSettings", "snap_playhead_mode", dna).is_some());
        let snap = |field: &str, factory: u32| snaps.map_or(factory, |ts| self.int_field(ts, "ToolSettings", field, dna));
        let r = self.off("Scene", "r", dna);
        let delta = r
            .zip(Self::struct_field_offset_exact("RenderData", "time_jump_delta", dna, self.header.pointer_size as usize))
            .and_then(|(r, f)| sc.data.get(r + f..r + f + 4))
            .map_or(factory.jump_delta, |b| self.read_f32(b));
        BlendPlayback {
            sync,
            audio_scrub: audio & AUDIO_SCRUB != 0,
            audio_play: audio & AUDIO_MUTE == 0,
            lock_frame_selection: render_flag & SCER_LOCK_FRAME_SELECTION != 0,
            allow_preroll: render_flag & SCER_ALLOW_PREROLL != 0,
            show_subframe: render_flag & SCER_SHOW_SUBFRAME != 0,
            loop_mode,
            follow: redraws & TIME_FOLLOW != 0,
            play_in: redraws & PLAY_IN_MASK,
            auto_key: autokey & AUTOKEY_ON != 0,
            auto_key_replace: autokey & !AUTOKEY_ON == AUTOKEY_MODE_EDITKEYS & !AUTOKEY_ON,
            only_keying_set: keying & AUTOKEY_FLAG_ONLYKEYINGSET != 0,
            jump_in_frames: self.nested_int(sc, "Scene", "r", "RenderData", "time_jump_unit", dna).is_some_and(|u| u == 0),
            jump_delta: delta,
            snap_playhead: snap("snap_flag_playhead", 0) & SCE_SNAP != 0,
            snap_to: snap("snap_playhead_mode", factory.snap_to),
            snap_frames: snap("snap_step_frames", factory.snap_frames),
            snap_seconds: snap("snap_step_seconds", factory.snap_seconds),
            snap_distance: snap("playhead_snap_distance", factory.snap_distance),
        }
    }
}
