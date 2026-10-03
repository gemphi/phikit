//! The NLA the editor lists is Blender's: every animated object's action
//! line and tracks, with each strip's range, repeat, flags and action —
//! against `truth/nla_cube.txt`, made by `truth/blender_nla.py` from the
//! fixture `truth/blender_nla_fixture.py` builds.

use phiphx::blend::{BNlaStrip, BNlaTrack, BlendNla, EXTEND_HOLD, STRIP_CLIP, STRIP_MUTED, STRIP_SELECT, TRACK_MUTED};
use phiphx::scene::SceneModel;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/nla_cube.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/truth/nla_cube.txt");

fn model() -> SceneModel {
    let bytes = std::fs::read(FILE).expect("fixture");
    let blend = phiphx::blend::BlendFile::parse_bytes(&bytes).expect("parse").extract_scene().expect("scene");
    SceneModel::from_blend_owned(blend, FILE)
}

/// Blender's spelling of a strip, from phiphx's.
fn strip_line(strip: &BNlaStrip) -> String {
    let kind = match strip.kind {
        STRIP_CLIP => "CLIP",
        1 => "TRANSITION",
        2 => "META",
        _ => "SOUND",
    };
    let extend = match strip.extend {
        EXTEND_HOLD => "HOLD",
        1 => "HOLD_FORWARD",
        _ => "NOTHING",
    };
    format!(
        "strip\t{}\t{kind}\t{}\t{}\t{}\t{}\t{}\t{}\t{extend}\tsel={}\tmute={}\t{}",
        strip.name,
        strip.start,
        strip.end,
        strip.actstart,
        strip.actend,
        strip.repeat,
        strip.scale,
        u8::from(strip.flag & STRIP_SELECT != 0),
        u8::from(strip.flag & STRIP_MUTED != 0),
        strip.action.as_deref().unwrap_or("-")
    )
}

fn track_line(track: &BNlaTrack) -> String {
    format!("track\t{}\tmute={}\tsolo={}", track.name, u8::from(track.flag & TRACK_MUTED != 0), u8::from(track.flag & phiphx::blend::TRACK_SOLO != 0))
}

/// The NLA as the truth file spells it: tracks last first, as the editor lists them.
fn lines(nla: &BlendNla) -> Vec<String> {
    let extend = match nla.extend {
        EXTEND_HOLD => "HOLD",
        1 => "HOLD_FORWARD",
        _ => "NOTHING",
    };
    let mut out = vec![format!("object\t{}\t{}\t{extend}\t{}", nla.object, nla.action.as_deref().unwrap_or("-"), nla.tracks.len())];
    for track in nla.tracks.iter().rev() {
        out.push(track_line(track));
        out.extend(track.strips.iter().map(strip_line));
    }
    out
}

#[test]
fn every_animated_object_lists_blenders_action_line_tracks_and_strips() {
    let model = model();
    let want: Vec<String> = std::fs::read_to_string(TRUTH).expect("truth").lines().map(str::to_string).collect();
    let have: Vec<String> = model.nla.iter().flat_map(lines).collect();
    assert_eq!(have, want);
    let cube = model.nla.iter().find(|nla| nla.object == "Cube").expect("the cube has animation data");
    assert_eq!(cube.action.as_deref(), Some("Idle"), "the action line is the active action");
    assert_eq!(cube.tracks.len(), 2, "two tracks, stored first to last");
    assert_eq!(cube.tracks[0].strips[0].repeat, 2.0, "the lower strip repeats, so it runs 1 .. 97");
    assert_eq!((cube.tracks[0].strips[0].start, cube.tracks[0].strips[0].end), (1.0, 97.0));
    let sphere = model.nla.iter().find(|nla| nla.object == "Sphere").expect("animation data without an action is listed too");
    assert!(sphere.action.is_none() && sphere.tracks.is_empty());
}
