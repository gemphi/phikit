//! `clean_fcurve` (`animation/keyframes_general.cc`), rule for rule.

use super::*;
use crate::phiphx::blend::BFCurve;
use crate::phiphx::scene::SceneAction;

/// A curve of `(frame, value)` keys, keyed as `BFCurve::insert` makes them —
/// a Bézier key with auto-clamped handles, as Blender inserts one.
fn curve(path: &str, index: u32, keys: &[(f32, f32)]) -> BFCurve {
    let mut c = BFCurve::keyed(path.to_string(), index);
    for &(f, v) in keys {
        c.insert(f, v);
    }
    c
}

fn frames_of(c: &BFCurve) -> Vec<f32> {
    c.keys.iter().map(|k| k.co[0]).collect()
}

fn clean(curves: Vec<BFCurve>, step: BKeyClean) -> SceneAction {
    let slot = crate::phiphx::blend::BlendActionSlot { handle: 3, identifier: "OBCube".into(), curves: 0..curves.len() };
    let mut actions = vec![SceneAction { name: "CubeAction".into(), slots: vec![slot], curves, ..Default::default() }];
    step.apply(&mut actions);
    actions.remove(0)
}

fn step(frames: Vec<f32>, channels: bool) -> BKeyClean {
    BKeyClean { action: "CubeAction".into(), slot: "OBCube".into(), frames, threshold: DEFAULT_THRESHOLD, channels }
}

#[test]
fn a_flat_run_collapses_to_the_key_that_reached_it() {
    // `clean_fcurve` compares each key against the last key KEPT and against
    // its immediate next — never against the original previous — so a flat
    // tail goes entirely, last key included: 2 is kept because it differs
    // from 0; 4, 6 and 8 each match both neighbours; and 10 matches the last
    // kept value with no next to differ from. The curve that remains
    // evaluates the same, since it holds its last value onwards
    // (`EXTRAPOLATE_CONSTANT`).
    let c = curve("location", 0, &[(0.0, 0.0), (2.0, 1.0), (4.0, 1.0), (6.0, 1.0), (8.0, 1.0), (10.0, 1.0)]);
    let out = clean(vec![c], step((0..=10).map(|f| f as f32).collect(), false));
    assert_eq!(frames_of(&out.curves[0]), [0.0, 2.0], "the move, and nothing after it");
    // Lossless where it matters: the value at every original frame is what
    // it was before the clean.
    for f in [0.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0] {
        let want = if f < 2.0 { f / 2.0 } else { 1.0 };
        assert!((out.curves[0].evaluate(f) - want).abs() < 1e-5, "frame {f}");
    }
}

#[test]
fn a_key_that_says_something_is_kept() {
    // Every key differs from its neighbours: nothing to simplify.
    let c = curve("location", 0, &[(0.0, 0.0), (5.0, 2.0), (10.0, -1.0), (15.0, 4.0)]);
    let out = clean(vec![c], step(vec![0.0, 5.0, 10.0, 15.0], false));
    assert_eq!(frames_of(&out.curves[0]), [0.0, 5.0, 10.0, 15.0]);
}

#[test]
fn the_threshold_is_inclusive_as_is_eqt() {
    // `IS_EQT(a, b, c)` is `|a − b| <= c`, so a difference exactly at the
    // threshold counts as equal and the key goes — and with the bump gone,
    // frame 10 then matches the one key left and goes too.
    let at = curve("location", 0, &[(0.0, 0.0), (5.0, 0.001), (10.0, 0.0)]);
    let out = clean(vec![at], step(vec![0.0, 5.0, 10.0], false));
    assert_eq!(frames_of(&out.curves[0]), [0.0], "exactly at the threshold is equal");
    // A hair over it is a real change and stands.
    let over = curve("location", 0, &[(0.0, 0.0), (5.0, 0.0011), (10.0, 0.0)]);
    let out = clean(vec![over], step(vec![0.0, 5.0, 10.0], false));
    assert_eq!(frames_of(&out.curves[0]), [0.0, 5.0, 10.0]);
}

#[test]
fn an_unselected_key_is_kept_as_it_stands() {
    // `only_selected_keys`: Clean Keyframes considers only the selection,
    // and an unselected key is kept verbatim — and then serves as the `prev`
    // the next comparison runs against, as it does in Blender.
    let c = curve("location", 0, &[(0.0, 0.0), (2.0, 1.0), (4.0, 1.0), (6.0, 1.0)]);
    // Only frame 4 is picked, so 2 survives untouched though it is flat.
    let out = clean(vec![c], step(vec![4.0], false));
    assert_eq!(frames_of(&out.curves[0]), [0.0, 2.0, 6.0], "only the picked key is weighed");
}

#[test]
fn clean_keyframes_leaves_a_curve_standing_and_clean_channels_drops_it() {
    // A curve flat at zero throughout. Clean Keyframes (`channels = false`,
    // `cleardefault = false`) simplifies it to one key and stops there.
    let flat = || curve("location", 0, &[(0.0, 0.0), (5.0, 0.0), (10.0, 0.0)]);
    let out = clean(vec![flat()], step(vec![0.0, 5.0, 10.0], false));
    assert_eq!(out.curves.len(), 1, "Clean Keyframes never removes a channel");
    assert_eq!(frames_of(&out.curves[0]), [0.0]);
    // Clean Channels (`cleardefault`) walks every key and takes the curve
    // with it, since the one key left sits at `location`'s default of 0.
    let out = clean(vec![flat()], step(Vec::new(), true));
    assert!(out.curves.is_empty(), "a curve left at its default goes");
    assert_eq!(out.slots[0].curves, 0..0, "and the slot's run closes up");
}

#[test]
fn a_curve_left_at_a_non_default_value_is_kept() {
    // `RNA_property_float_get_default_index`: `scale` defaults to 1, so a
    // scale curve flat at 1 is at its default and goes, while one flat at 2
    // is saying something and stays.
    let out = clean(vec![curve("scale", 0, &[(0.0, 1.0), (5.0, 1.0)])], step(Vec::new(), true));
    assert!(out.curves.is_empty(), "scale flat at 1 is its default");
    let out = clean(vec![curve("scale", 0, &[(0.0, 2.0), (5.0, 2.0)])], step(Vec::new(), true));
    assert_eq!(out.curves.len(), 1, "scale flat at 2 is not");
    // A location curve flat at 1 is not at its default either.
    let out = clean(vec![curve("location", 0, &[(0.0, 1.0), (5.0, 1.0)])], step(Vec::new(), true));
    assert_eq!(out.curves.len(), 1);
    // A quaternion's W defaults to 1 and its other components to 0.
    let out = clean(vec![curve("rotation_quaternion", 0, &[(0.0, 1.0), (5.0, 1.0)])], step(Vec::new(), true));
    assert!(out.curves.is_empty(), "quaternion W flat at 1 is its default");
    let out = clean(vec![curve("rotation_quaternion", 1, &[(0.0, 1.0), (5.0, 1.0)])], step(Vec::new(), true));
    assert_eq!(out.curves.len(), 1, "quaternion X flat at 1 is not");
    // A bone's channel resolves by its leaf, not the whole path.
    let out = clean(vec![curve("pose.bones[\"chest\"].scale", 2, &[(0.0, 1.0), (5.0, 1.0)])], step(Vec::new(), true));
    assert!(out.curves.is_empty(), "the leaf of the path is what names the property");
}

#[test]
fn each_curve_is_asked_its_own_question() {
    // The reason this is a step of its own rather than a column delete: X is
    // flat and loses its middle key while Y, moving, keeps all three.
    let x = curve("location", 0, &[(0.0, 0.0), (5.0, 0.0), (10.0, 0.0)]);
    let y = curve("location", 1, &[(0.0, 0.0), (5.0, 3.0), (10.0, 7.0)]);
    let out = clean(vec![x, y], step(vec![0.0, 5.0, 10.0], false));
    assert_eq!(frames_of(&out.curves[0]), [0.0], "X collapses");
    assert_eq!(frames_of(&out.curves[1]), [0.0, 5.0, 10.0], "Y is untouched");
    assert_eq!(out.keys, [0.0, 5.0, 10.0], "the action's keyed frames are recounted over both");
}

#[test]
fn a_clean_round_trips_through_the_step_list() {
    use super::super::retime::BKeyStep;
    let list = vec![
        BKeyStep::Clean(step(vec![1.0, 20.0], false)),
        BKeyStep::Clean(BKeyClean { threshold: 0.25, ..step(Vec::new(), true) }),
    ];
    assert_eq!(BKeyStep::parse_all(&BKeyStep::encode_all(&list)), list);
    // And a clean does not read as one of the other marks.
    let encoded = BKeyStep::encode_all(&list);
    assert!(encoded.starts_with('%'), "{encoded}");
}
