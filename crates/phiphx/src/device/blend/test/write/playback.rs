use super::*;
use crate::device::blend::write::BlendEditList;

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
        let report = BlendEditList { playback: Some(wanted), ..Default::default() }.apply(&mut file);
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
