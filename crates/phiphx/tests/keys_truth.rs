//! The key columns the Timeline draws are Blender's: every action's
//! distinct keyed frames, curve count and users, against
//! `truth/thymus_keys.txt` (made by `truth/blender_keys.py`).

use phiphx::scene::SceneModel;
use std::collections::HashMap;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/THYMUS-BOLDROID-WAVE-003.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/truth/thymus_keys.txt");

/// `name → (fcurve count, users, keys, curve state)`; the state is Blender's
/// `selected=…;rainbow=…;…;keysel=…;free=…;…;clamped=…;hsel=…` counts as written.
fn truth() -> HashMap<String, (usize, Vec<String>, Vec<f32>, String)> {
    std::fs::read_to_string(TRUTH)
        .expect("truth file")
        .lines()
        .filter_map(|line| {
            let mut column = line.split('\t');
            let (name, count, users, keys, state) = (column.next()?, column.next()?, column.next()?, column.next()?, column.next()?);
            let users = users.split(',').filter(|u| !u.is_empty()).map(str::to_string).collect();
            let keys = keys.split(' ').filter_map(|k| k.parse().ok()).collect();
            Some((name.to_string(), (count.parse().ok()?, users, keys, state.to_string())))
        })
        .collect()
}

/// The same counts from phiphx's curves, in Blender's spelling.
fn state_of(action: &phiphx::scene::SceneAction) -> String {
    use phiphx::blend::{COLOR_AUTO_RAINBOW, COLOR_AUTO_RGB, COLOR_AUTO_YRGB, COLOR_CUSTOM, HANDLE_ALIGN, HANDLE_AUTO, HANDLE_AUTO_ANIM, HANDLE_FREE, HANDLE_VECT};
    let count = |mode: u32| action.curves.iter().filter(|c| c.color_mode == mode).count();
    let keys = || action.curves.iter().flat_map(|c| c.keys.iter());
    let handles = |kind: u8| keys().map(|k| usize::from(k.h1_type == kind) + usize::from(k.h2_type == kind)).sum::<usize>();
    format!(
        "selected={};rainbow={};rgb={};yrgb={};custom={};keysel={};free={};auto={};vector={};aligned={};clamped={};hsel={}",
        action.curves.iter().filter(|c| c.selected()).count(),
        count(COLOR_AUTO_RAINBOW),
        count(COLOR_AUTO_RGB),
        count(COLOR_AUTO_YRGB),
        count(COLOR_CUSTOM),
        keys().filter(|k| k.selected).count(),
        handles(HANDLE_FREE),
        handles(HANDLE_AUTO),
        handles(HANDLE_VECT),
        handles(HANDLE_ALIGN),
        handles(HANDLE_AUTO_ANIM),
        keys().map(|k| usize::from(k.h1_selected) + usize::from(k.h2_selected)).sum::<usize>()
    )
}

#[test]
fn every_action_has_blenders_key_columns_and_users() {
    let bytes = std::fs::read(FILE).expect("fixture");
    let blend = phiphx::blend::BlendFile::parse_bytes(&bytes).expect("parse").extract_scene().expect("scene");
    let model = SceneModel::from_blend_owned(blend, FILE);
    let want = truth();
    assert_eq!(want.len(), 3, "the fixture carries three actions");
    assert_eq!(model.actions.len(), want.len(), "every action is listed once");
    for action in &model.actions {
        let (count, users, keys, state) = want.get(&action.name).unwrap_or_else(|| panic!("{} is not in Blender's list", action.name));
        assert_eq!(action.fcurve_count, *count, "{}: curve count", action.name);
        assert_eq!(action.curves.len(), *count, "{}: every curve is carried", action.name);
        assert_eq!(&state_of(action), state, "{}: selection and colour modes", action.name);
        assert_eq!(&action.users, users, "{}: users", action.name);
        assert_eq!(&action.keys, keys, "{}: key columns", action.name);
        let (start, end) = keys.first().zip(keys.last()).map_or((0.0, 0.0), |(a, b)| (*a, *b));
        assert_eq!((action.frame_start, action.frame_end), (start, end), "{}: range", action.name);
    }
    let rig = model.actions.iter().find(|a| a.used_by("rig")).expect("the rig is animated");
    assert_eq!(rig.name, "rigAction");
}
