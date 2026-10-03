//! Cameras, lights and empties take the Transform panel's and G / R / S's
//! edits like meshes do: a light parented to a moved cube comes along, G
//! on a light or a camera moves it, and the panel shows an object's own
//! transform — a child's relative to its parent, a light's too. Over the
//! factory scene with the Light and Arm parented to the Cube and the empty
//! Hand to Arm (`tests/fixtures/make_parented.py`).

use glam::Vec3;
use phiber::phiphx::blend::BlendFile;
use phiber::phiphx::scene::SceneModel;
use phiber::v2::scene::camera::BViewSetup;
use phiber::v2::scene::extra::{BExtraKind, scene_extras_now};
use phiber::v2::scene::pick::BPickView;
use phiber::v2::scene::source::set_loaded_scene;
use phiber::v2::scene::transform::{BTransform, BTransforms};
use phiber::v2::scene::xform::{BTransformModal, BTransformMode};
use phiber::v2::state::action::apply_action;
use phiber::v2::state::{BKey as _, BState, ButsKey, CtxKey};
use phiber::v2::view::BAction;
use std::path::PathBuf;

fn load() -> SceneModel {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parented.blend");
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    let model = SceneModel::from_blend_owned(scene, &path.to_string_lossy());
    set_loaded_scene(Some(model.clone()));
    model
}

/// The middle of a kind's gizmo lines — where the extra stands.
fn centre(s: &BState, model: &SceneModel, kind: BExtraKind) -> Vec3 {
    let lines: Vec<_> = scene_extras_now(s, model).iter().filter(|l| l.kind == kind).copied().collect();
    let sum: Vec3 = lines.iter().map(|l| Vec3::from(l.a) + Vec3::from(l.b)).sum();
    sum / (2 * lines.len().max(1)) as f32
}

#[test]
fn a_light_parented_to_a_moved_cube_comes_along() {
    let model = load();
    let mut s = BState::new();
    let light0 = centre(&s, &model, BExtraKind::Light);
    let camera0 = centre(&s, &model, BExtraKind::Camera);
    let moved = BTransform { location: [2.0, 0.0, 1.0], rotation: [0.0; 3], scale: [1.0; 3] };
    BTransforms { list: vec![("Cube".into(), moved)] }.write(&mut s);
    let light1 = centre(&s, &model, BExtraKind::Light);
    assert!((light1 - light0 - Vec3::new(2.0, 0.0, 1.0)).length() < 1e-4, "the Light follows the Cube: {light0} → {light1}");
    assert!((centre(&s, &model, BExtraKind::Camera) - camera0).length() < 1e-5, "the Camera has no parent: it stays");
    // The empty Hand, parented through Arm to the Cube, comes along too.
    let hand = centre(&s, &model, BExtraKind::Empty);
    BTransforms::default().write(&mut s);
    assert!((hand - centre(&s, &model, BExtraKind::Empty) - Vec3::new(2.0, 0.0, 1.0)).length() < 1e-4);
}

#[test]
fn g_moves_a_light_and_the_panel_shows_its_own_transform() {
    let model = load();
    let mut s = BState::new();
    // Picking the Light fills the panel with its own values.
    apply_action(&mut s, &BAction::Set(CtxKey::Object.path(), "Light".into()));
    let x = |s: &BState| s.get(ButsKey::Axis("location", "x")).and_then(|v| v.parse::<f32>().ok()).unwrap_or(f32::NAN);
    assert!((x(&s) - 4.076).abs() < 1e-3, "the Light's location: {}", x(&s));

    // Moving its parent leaves its own location as it was.
    BTransforms { list: vec![("Cube".into(), BTransform { location: [2.0, 0.0, 0.0], rotation: [0.0; 3], scale: [1.0; 3] })] }.write(&mut s);
    apply_action(&mut s, &BAction::Set(CtxKey::Object.path(), "Cube".into()));
    apply_action(&mut s, &BAction::Set(CtxKey::Object.path(), "Light".into()));
    assert!((x(&s) - 4.076).abs() < 1e-3, "relative to its parent, as Blender's panel shows it: {}", x(&s));

    // G on the Light: it moves; the Cube does not.
    let light0 = centre(&s, &model, BExtraKind::Light);
    BTransformModal::start(&mut s, BTransformMode::Translate);
    let modal = BTransformModal::now(&s).expect("a light can be moved");
    assert_eq!(modal.list.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), ["Light"]);
    let view = BPickView { yaw: 0.0, pitch: 0.0, pan: (0.0, 0.0), zoom: 1.0, setup: BViewSetup::default(), aspect: 1.0 };
    modal.moved(&view, (200.0, 200.0), (400.0, 400.0)).iter().for_each(|a| apply_action(&mut s, a));
    let modal = BTransformModal::now(&s).unwrap();
    modal.moved(&view, (260.0, 200.0), (400.0, 400.0)).iter().for_each(|a| apply_action(&mut s, a));
    let light1 = centre(&s, &model, BExtraKind::Light);
    assert!(light1.x - light0.x > 0.1 && (light1.z - light0.z).abs() < 1e-4, "right in front view is +X: {light0} → {light1}");
    let cube = BTransforms::now(&s).get("Cube").expect("the Cube's own edit");
    assert_eq!(cube.location, [2.0, 0.0, 0.0], "the Cube stays");
}
