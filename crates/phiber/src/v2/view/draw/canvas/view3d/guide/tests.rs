//! The Overlays popover's boxes against what the viewport draws, over
//! phiphx's `fixtures/overlays.blend` (`truth/blender_overlays.py`): the
//! floor and axes, the extras and bones, the relationship lines (a hidden
//! parent's left out, an edited parent's carried along), the origins and
//! the 3D cursor.

use super::cursor::{BCursorInk, BCursorMark};
use super::origin::{BOriginDot, BOriginInk};
use super::super::extra::{BExtraPalette, BProjectedLine, project};
use super::super::floor::{BFloorPalette, lines};
use crate::phiphx::blend::BlendFile;
use crate::phiphx::scene::SceneModel;
use crate::v2::scene::camera::BViewSetup;
use crate::v2::scene::pick::extra::BPickExtra;
use crate::v2::scene::select::BSelection;
use crate::v2::scene::{BOverlays, MESH_SCALE, project3_full};
use crate::v2::state::{BKey as _, BState, CtxKey};
use gpui::Hsla;

/// A colour to count lines by.
fn ink(n: u8) -> Hsla {
    Hsla { h: 0.0, s: 0.0, l: f32::from(n) / 16.0, a: 1.0 }
}

const VIEW: (f32, f32, (f32, f32), f32) = (30.0, 30.0, (0.0, 0.0), 0.5);

fn overlays(s: &mut BState, list: &[(&'static str, bool)]) -> BOverlays {
    BOverlays::seed(s, list);
    BOverlays::now(s)
}

fn fixture() -> SceneModel {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/overlays.blend");
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    SceneModel::from_blend_owned(scene, &path.to_string_lossy())
}

#[test]
fn the_floor_and_each_axis_draw_as_their_boxes_say() {
    let palette = BFloorPalette { grid: ink(1), axis_x: ink(2), axis_y: ink(3), axis_z: ink(4) };
    let count = |o: BOverlays| {
        let drawn = lines(VIEW, BViewSetup::default(), [0.0; 3], palette, o);
        [1, 2, 3, 4].map(|n| drawn.iter().filter(|l| l.color == ink(n)).count())
    };
    let mut s = BState::new();
    let [grid, x, y, z] = count(BOverlays::now(&s));
    assert!(grid > 0 && x > 0 && y > 0 && z == 0, "Blender's default: floor, X and Y");
    assert_eq!(count(overlays(&mut s, &[("show_floor", false)])), [0, x, y, 0]);
    let [with_line, no_x, ..] = count(overlays(&mut s, &[("show_floor", true), ("show_axis_x", false)]));
    assert!(no_x == 0 && with_line > grid, "the X axis off leaves an ordinary floor line in its place");
    assert!(count(overlays(&mut s, &[("show_axis_z", true)]))[3] > 0);
    assert_eq!(count(overlays(&mut s, &[("show_overlays", false)])), [0, 0, 0, 0]);
}

#[test]
fn the_tracer_is_told_which_guides_to_leave_out() {
    use super::super::rendered::BRayLook;
    let mut s = BState::new();
    assert_eq!(BRayLook::grid_hide(BOverlays::now(&s)), 0);
    assert_eq!(BRayLook::grid_hide(overlays(&mut s, &[("show_floor", false), ("show_axis_z", true)])), 1 | 8);
    assert_eq!(BRayLook::grid_hide(overlays(&mut s, &[("show_overlays", false)])), 1 | 2 | 4);
}

#[test]
fn extras_bones_and_relationship_lines_draw_and_pick_as_their_boxes_say() {
    let model = fixture();
    crate::v2::scene::source::set_loaded_scene(Some(model.clone()));
    let palette = BExtraPalette { camera: ink(1), light: ink(2), empty: ink(3), selected: ink(4), active: ink(5), relation: ink(6), constraint: ink(7) };
    let drawn = |s: &BState| project(s, VIEW.0, VIEW.1, VIEW.2, VIEW.3, palette);
    let count = |l: &[BProjectedLine], n: u8| l.iter().filter(|l| l.color == ink(n)).count();
    let mut s = BState::new();
    // Child → Base, OnTail and OnRel → Rig, Orphan → HiddenParent; Track,
    // Copy and Rider's constraints — Folded's is folded, Clamp To has none.
    assert_eq!((count(&drawn(&s), 6), count(&drawn(&s), 7)), (4, 3), "one segment a line");
    // As the file has it, HiddenParent is hidden: no line runs to it.
    s.set(&crate::v2::part::outliner::key::k_restrict("HiddenParent", "hide_get"), crate::v2::state::val::V_TRUE);
    let all = drawn(&s);
    assert_eq!((count(&all, 6), count(&all, 7)), (3, 3));
    assert!(all.iter().filter(|l| l.color == ink(6) || l.color == ink(7)).all(|l| l.dash));
    let picked = |s: &BState| BPickExtra::list(s, &model).into_iter().map(|e| e.name).collect::<Vec<_>>();
    assert!(picked(&s).contains(&"Rig".to_string()) && picked(&s).contains(&"Track".to_string()));
    overlays(&mut s, &[("show_relationship_lines", false), ("show_bones", false)]);
    let fewer = drawn(&s);
    assert_eq!((count(&fewer, 6), count(&fewer, 7)), (0, 0));
    assert!(count(&fewer, 3) > 0 && !picked(&s).contains(&"Rig".to_string()), "no bones to draw or to click");
    overlays(&mut s, &[("show_extras", false)]);
    assert_eq!(count(&drawn(&s), 3), 0);
    assert!(picked(&s).is_empty(), "a hidden extra cannot be clicked");
    crate::v2::scene::source::set_loaded_scene(None);
}

#[test]
fn a_moved_parent_carries_its_end_of_the_line() {
    use crate::v2::scene::transform::{BTransform, BTransforms};
    let model = fixture();
    crate::v2::scene::source::set_loaded_scene(Some(model.clone()));
    let parent_end = |s: &BState| {
        let lines = crate::v2::scene::extra::scene_extras_now(s, &model);
        let child = model.motion.motion.list.iter().position(|m| m.name == "Child").unwrap();
        lines.iter().find(|l| l.name_index == child && matches!(l.kind, crate::v2::scene::extra::BExtraKind::Parent { .. })).map(|l| l.b).unwrap()
    };
    let mut s = BState::new();
    let before = parent_end(&s);
    let mut edits = BTransforms::now(&s);
    let base = model.empties.iter().find(|e| e.name == "Base").unwrap();
    let (m, n) = (base.matrix_world[3], 1.0);
    edits.set("Base", BTransform { location: [m[0] + n, m[1], m[2]], rotation: [0.0, 0.0, 0.4f32.to_degrees()], scale: [1.0; 3] });
    edits.write(&mut s);
    let after = parent_end(&s);
    assert!((after[0] - before[0] - 1.0).abs() < 1e-3 && (after[1] - before[1]).abs() < 1e-3, "{before:?} → {after:?}");
    crate::v2::scene::source::set_loaded_scene(None);
}

#[test]
fn the_cursor_stands_where_the_file_left_it_a_widget_unit_across() {
    let model = fixture();
    let colors = BCursorInk { red: ink(1), white: ink(2), axis: [ink(3), ink(4), ink(5)] };
    let mark = BCursorMark::of(model.cursor.matrix, VIEW, BViewSetup::default(), colors).expect("in view");
    let at = model.cursor.location();
    let want = project3_full(at[0] * MESH_SCALE, at[2] * MESH_SCALE, at[1] * MESH_SCALE, VIEW.0, VIEW.1, VIEW.2, VIEW.3);
    assert!((mark.center.0 - want.0).abs() < 1e-5 && (mark.center.1 - want.1).abs() < 1e-5);
    assert_eq!(mark.cross.len(), 12, "three axes, both halves, tip and middle each");
    // Unturned, seen from the front: X runs a widget unit to the right.
    let front = BCursorMark::of(glam::Mat4::IDENTITY.to_cols_array_2d(), (0.0, 0.0, (0.0, 0.0), 1.0), BViewSetup::default(), colors).unwrap();
    let (tip, _, color) = front.cross[0];
    assert!((tip.0 - super::set::WIDGET_UNIT).abs() < 1e-2 && tip.1.abs() < 1e-3 && color == ink(3), "{tip:?}");
}

#[test]
fn origins_mark_the_active_object_and_the_selection() {
    let scene = crate::v2::scene::scene_objects(&BState::new());
    let mut s = BState::new();
    BSelection { list: vec!["Cube".into(), "Sphere".into()] }.write(&mut s);
    s.set(CtxKey::Object.path(), "Sphere");
    let colors = BOriginInk { active: ink(1), selected: ink(2), other: ink(3) };
    let fills = |all: bool| {
        let dots = BOriginDot::list(&BSelection::now(&s), "Sphere", all, &scene, &[], VIEW, BViewSetup::default(), colors);
        let mut f: Vec<u8> = dots.iter().map(|d| (d.fill.l * 16.0).round() as u8).collect();
        f.sort_unstable();
        f
    };
    assert_eq!(fills(false), [1, 2], "the active object and the selected one");
    assert_eq!(fills(true), [1, 2, 3], "and, with Origins (All), the Gem");
}
