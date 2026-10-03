//! Writing the Playback settings back (`scene/playback.rs` reads them):
//! the bits of `Scene.audio.flag`, `Scene.flag` and `Scene.r.flag`, the
//! loop mode where the file's Blender has one, and the active screen's
//! `redraws_flag` — the same places `rna_scene.cc` and `rna_screen.cc` put
//! them.

use crate::blend::file::types::{BlendFile, DnaType};
use crate::blend::scene::playback::*;

fn with_bit(flag: u32, bit: u32, on: bool) -> u32 {
    if on { flag | bit } else { flag & !bit }
}

impl BlendFile {
    /// Write `playback` into scene `scene` and the active screen: how many
    /// fields changed.
    pub(crate) fn write_playback(&mut self, scene: usize, dna: &DnaType, playback: &BlendPlayback) -> usize {
        let mut patched = 0;
        let block = &self.blocks[scene];
        let audio = self.nested_int(block, "Scene", "audio", "AudioData", "flag", dna).unwrap_or(0);
        let audio = with_bit(with_bit(with_bit(audio, AUDIO_SYNC, playback.sync == BlendSyncMode::AudioSync), AUDIO_SCRUB, playback.audio_scrub), AUDIO_MUTE, !playback.audio_play);
        let scene_flag = with_bit(self.int_field(block, "Scene", "flag", dna), SCE_FRAME_DROP, playback.sync == BlendSyncMode::FrameDrop);
        let render = self.nested_int(block, "Scene", "r", "RenderData", "flag", dna).unwrap_or(0);
        let render = with_bit(with_bit(with_bit(render, SCER_LOCK_FRAME_SELECTION, playback.lock_frame_selection), SCER_ALLOW_PREROLL, playback.allow_preroll), SCER_SHOW_SUBFRAME, playback.show_subframe);
        for (field, value) in [("audio.flag", audio), ("flag", scene_flag), ("r.flag", render)] {
            patched += usize::from(self.set_int(scene, "Scene", field, i64::from(value)));
        }
        if self.off("Scene", "playback_loop_mode", dna).is_some() {
            patched += usize::from(self.set_int(scene, "Scene", "playback_loop_mode", i64::from(playback.loop_mode.to_dna())));
        }
        // The transport's popovers: auto keying on the tool settings, the
        // time jump on the render data where the file's Blender has it.
        if let Some(ts) = self.tool_settings_index(&self.blocks[scene], dna) {
            let mode = if playback.auto_key_replace { AUTOKEY_MODE_EDITKEYS } else { AUTOKEY_MODE_NORMAL };
            let mode = if playback.auto_key { mode } else { mode & !AUTOKEY_ON };
            patched += usize::from(self.set_int(ts, "ToolSettings", "autokey_mode", i64::from(mode)));
            if self.off("ToolSettings", "snap_playhead_mode", dna).is_some() {
                let flag = with_bit(self.int_field(&self.blocks[ts], "ToolSettings", "snap_flag_playhead", dna), SCE_SNAP, playback.snap_playhead);
                for (field, value) in [
                    ("snap_flag_playhead", flag),
                    ("snap_playhead_mode", playback.snap_to),
                    ("snap_step_frames", playback.snap_frames),
                    ("snap_step_seconds", playback.snap_seconds),
                    ("playhead_snap_distance", playback.snap_distance),
                ] {
                    patched += usize::from(self.set_int(ts, "ToolSettings", field, i64::from(value)));
                }
            }
            if let Some(field) = self.keying_flag_field(dna) {
                let flag = with_bit(self.int_field(&self.blocks[ts], "ToolSettings", field, dna), AUTOKEY_FLAG_ONLYKEYINGSET, playback.only_keying_set);
                patched += usize::from(self.set_int(ts, "ToolSettings", field, i64::from(flag)));
            }
        }
        if self.nested_int(&self.blocks[scene], "Scene", "r", "RenderData", "time_jump_unit", dna).is_some() {
            patched += usize::from(self.set_int(scene, "Scene", "r.time_jump_unit", i64::from(!playback.jump_in_frames)));
            patched += usize::from(self.set_f32(scene, "Scene", "r.time_jump_delta", playback.jump_delta));
        }
        if let Some(screen) = self.active_screen_index(dna) {
            let redraws = self.int_field(&self.blocks[screen], "bScreen", "redraws_flag", dna);
            let redraws = with_bit((redraws & !PLAY_IN_MASK) | (playback.play_in & PLAY_IN_MASK), TIME_FOLLOW, playback.follow);
            patched += usize::from(self.set_int(screen, "bScreen", "redraws_flag", i64::from(redraws)));
        }
        patched
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blend::write::BlendEdits;

    #[test]
    fn the_playback_settings_read_back_as_written_in_every_layout() {
        let fixtures = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        for name in ["THYMUS-BOLDROID-WAVE-003.blend", "nla_cube.blend"] {
            let mut file = BlendFile::parse_bytes(&std::fs::read(fixtures.join(name)).unwrap()).unwrap();
            let before = file.extract_scene().unwrap().props.playback;
            let wanted = BlendPlayback {
                sync: BlendSyncMode::FrameDrop,
                audio_scrub: true,
                audio_play: false,
                lock_frame_selection: true,
                allow_preroll: true,
                show_subframe: true,
                loop_mode: BlendLoopMode::Bounce,
                follow: true,
                play_in: TIME_REGION | TIME_NODES | TIME_SPREADSHEETS,
                auto_key: true,
                auto_key_replace: true,
                only_keying_set: true,
                jump_in_frames: true,
                jump_delta: 5.0,
                snap_playhead: true,
                snap_to: SCE_SNAP_TO_FRAME | SCE_SNAP_TO_MARKERS,
                snap_frames: 5,
                snap_seconds: 2,
                snap_distance: 12,
            };
            let report = BlendEdits { playback: Some(wanted), ..Default::default() }.apply(&mut file);
            assert!(report.warnings.is_empty(), "{name}: {:?}", report.warnings);
            let again = BlendFile::parse_bytes(&file.to_bytes()).unwrap().extract_scene().unwrap().props.playback;
            // A file from before the loop mode (5.1) has no field for it: it
            // reads back as the factory one.
            let dna = file.dna.as_ref().unwrap();
            let has_loop = file.off("Scene", "playback_loop_mode", dna).is_some();
            let has_jump = file.off("RenderData", "time_jump_delta", dna).is_some();
            let mut expect = if has_loop { wanted } else { BlendPlayback { loop_mode: before.loop_mode, ..wanted } };
            if !has_jump {
                // Before 5.0 the time jump is a second, in no field.
                (expect.jump_in_frames, expect.jump_delta) = (before.jump_in_frames, before.jump_delta);
            }
            if file.off("ToolSettings", "snap_playhead_mode", dna).is_none() {
                // Before playhead snapping (5.0) there are no fields for it.
                expect = BlendPlayback { snap_playhead: before.snap_playhead, snap_to: before.snap_to, snap_frames: before.snap_frames, snap_seconds: before.snap_seconds, snap_distance: before.snap_distance, ..expect };
            }
            assert_eq!(again, expect, "{name}");
            assert_ne!(again, before, "{name}: the fixture did not already have these");
        }
    }
}
