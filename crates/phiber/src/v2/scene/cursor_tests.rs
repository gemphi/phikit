//! The 3D cursor over phiphx's `fixtures/overlays.blend`
//! (`truth/blender_overlays.py`: a cursor at (1.5, −2, 0.5) turned in ZXY,
//! saved at frame 12 with Child selected and active).

use super::*;
use crate::phiphx::blend::BlendFile;
use crate::v2::scene::camera::BViewSetup;
use crate::v2::scene::pick::BPickView;
use crate::v2::state::action::apply_action;
use crate::v2::view::prim::BAction;

fn fixture() -> SceneModel {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/overlays.blend");
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    SceneModel::from_blend_owned(scene, &path.to_string_lossy())
}

fn near(a: Vec3, b: Vec3) -> bool {
    (a - b).length() < 1e-3
}

fn op(s: &mut BState, key: OpKey, value: &str) {
    apply_action(s, &BAction::Set(key.path(), value.to_string()));
}

#[test]
fn the_file_s_cursor_until_the_session_moves_it() {
    crate::v2::scene::source::set_loaded_scene(Some(fixture()));
    let mut s = BState::new();
    assert!(near(BCursor::now(&s).location, Vec3::new(1.5, -2.0, 0.5)) && BCursor::edited(&s).is_none());
    let turned = BCursor { location: Vec3::new(1.0, 2.0, 3.0), rotation: Quat::from_rotation_z(0.5) };
    turned.write(&mut s);
    let back = BCursor::now(&s);
    assert!(near(back.location, turned.location) && back.rotation.angle_between(turned.rotation) < 1e-4);
    assert!(BCursor::edited(&s).is_some(), "a save writes it");
    crate::v2::scene::source::set_loaded_scene(None);
}

#[test]
fn the_snap_menu_moves_the_cursor_and_the_selection() {
    let model = fixture();
    crate::v2::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = BState::new();
    crate::v2::scene::select::BSelection { list: vec!["Child".into(), "Track".into()] }.write(&mut s);
    s.set(CtxKey::Object, "Child");
    // Child stands at (2.999, 1.310, 0.5) at the saved frame, Track at (0, −3, 1).
    let child = Vec3::new(2.999_054, 1.310_479, 0.5);
    op(&mut s, OpKey::Cursor, "active");
    assert!(near(BCursor::now(&s).location, child), "{:?}", BCursor::now(&s).location);
    op(&mut s, OpKey::Cursor, "selected");
    assert!(near(BCursor::now(&s).location, (child + Vec3::new(0.0, -3.0, 1.0)) / 2.0), "the median");
    op(&mut s, OpKey::Cursor, "origin");
    assert!(near(BCursor::now(&s).location, Vec3::ZERO));
    let rotation = BCursor::now(&s).rotation;
    assert!(rotation.angle_between(Quat::IDENTITY) > 0.1, "a snap moves the cursor, it does not turn it");
    BCursor { location: Vec3::new(1.0, 2.0, 3.0), rotation }.write(&mut s);
    op(&mut s, OpKey::Cursor, "selection_to_cursor");
    let after = worlds(&s, Some(&model));
    for name in ["Child", "Track"] {
        assert!(near(after[name].w_axis.truncate(), Vec3::new(1.0, 2.0, 3.0)), "{name} at {:?}", after[name].w_axis);
    }
    crate::v2::scene::source::set_loaded_scene(None);
}

#[test]
fn an_added_object_lands_at_the_cursor_and_shift_c_puts_it_back() {
    crate::v2::scene::source::set_loaded_scene(Some(fixture()));
    let mut s = BState::new();
    BCursor { location: Vec3::new(1.0, 2.0, 3.0), rotation: Quat::IDENTITY }.write(&mut s);
    op(&mut s, OpKey::SceneEdit, "add:cube");
    let at = worlds(&s, None).get("Cube").map(|m| m.w_axis.truncate());
    assert!(at.is_some_and(|at| near(at, Vec3::new(1.0, 2.0, 3.0))), "{at:?}");
    op(&mut s, OpKey::Cursor, "center_frame_all");
    assert!(near(BCursor::now(&s).location, Vec3::ZERO));
    crate::v2::scene::source::set_loaded_scene(None);
}

#[test]
fn a_click_puts_the_cursor_on_the_surface_or_on_its_own_plane_turned_to_the_view() {
    let cube = BObject3D::Loaded {
        name: "Cube".into(),
        mesh: crate::v2::scene::edit::primitive_mesh("cube").unwrap(),
        location: [0.0; 3],
        scale: [1.0; 3],
        matrix_world: crate::phiphx::scene::IDENTITY_M4,
        materials: Vec::new(),
        material_indices: Default::default(),
        uvs: Default::default(),
        color: [1.0; 4],
        shadow: Default::default(),
    };
    // Front: the view looks along +Y, right is +X, up is +Z.
    let view = BPickView { yaw: 0.0, pitch: 0.0, pan: (0.0, 0.0), zoom: 1.0, setup: BViewSetup::default(), aspect: 1.0 };
    let now = BCursor { location: Vec3::new(0.0, 5.0, 0.0), rotation: Quat::IDENTITY };
    let on = now.under(std::slice::from_ref(&cube), &view, (0.0, 0.0)).expect("on the cube");
    assert!(near(on.location, Vec3::new(0.0, -1.0, 0.0)), "the cube's front face: {:?}", on.location);
    let (right, up, back) = view.axes();
    assert!(near(on.rotation * Vec3::X, right) && near(on.rotation * Vec3::Y, up) && near(on.rotation * Vec3::Z, back));
    let off = now.under(&[], &view, (0.0, 0.0)).expect("on the plane");
    assert!(near(off.location, Vec3::new(0.0, 5.0, 0.0)), "the plane through the cursor: {:?}", off.location);
}

#[test]
fn a_save_writes_the_cursor_once_it_moved() {
    let model = fixture();
    crate::v2::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = BState::new();
    assert!(crate::v2::scene::save::gather(&s, &model).cursor.is_none());
    op(&mut s, OpKey::Cursor, "origin");
    let saved = crate::v2::scene::save::gather(&s, &model).cursor.expect("written");
    assert!(near(Mat4::from_cols_array_2d(&saved).w_axis.truncate(), Vec3::ZERO));
    crate::v2::scene::source::set_loaded_scene(None);
}
