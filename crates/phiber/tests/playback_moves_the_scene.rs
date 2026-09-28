//! Play moves the scene: the frame the timeline shows is the frame the
//! viewport draws. An object keyed in the file stands where its curves put
//! it at `ctx.frame` — and its child, its light and its camera with it —
//! while a still scene is built once and never again as the playhead runs.

use phiber::phiphx::blend::{BFCurve, BKey, BlendMotion, BlendMotionList, ObjectTransformRaw};
use phiber::phiphx::scene::{
    IDENTITY_M4, SceneCamera, SceneLight, SceneMesh, SceneModel, SceneMotion, SceneNormals, SceneObject,
};
use phiber::v2::{
    scene::{
        BObject3D, current_scene,
        extra::{BExtraKind, scene_extras_at},
        motion::{active_camera_at, lights_at, moving_frame},
        source::{clear_loaded_scene, clear_scene_objects, object_builds, scene_objects, set_loaded_at},
    },
    state::{BKey as _, BState, CtxKey, action::apply_action},
    view::BAction,
};

fn triangle(name: &str, x: f32) -> SceneObject {
    let mut matrix_world = IDENTITY_M4;
    matrix_world[3][0] = x;
    SceneObject {
        name: name.to_string(),
        mesh: SceneMesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            indices: vec![0, 1, 2],
            normals: SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]]),
            corners: Default::default(),
        }
        .into(),
        location: [x, 0.0, 0.0],
        scale: [1.0; 3],
        matrix_world,
        materials: Vec::new(),
        material_indices: Default::default(),
        uvs: Default::default(),
    }
}

fn located(x: f32) -> ObjectTransformRaw {
    ObjectTransformRaw { loc: [x, 0.0, 0.0], ..Default::default() }
}

/// `location.x` keyed linearly from `from` at frame 1 to `to` at frame 11.
fn slide(from: f32, to: f32) -> BFCurve {
    BFCurve::new(
        "location",
        0,
        vec![BKey::new([0.0, from], [1.0, from], [2.0, from], 1), BKey::new([10.0, to], [11.0, to], [12.0, to], 1)],
    )
}

/// A scene of a sliding Cube with a Child riding it, a still Floor, and a
/// keyed Light and Camera — blend order: Cube, Child, Floor, Light, Camera.
fn moving_scene() -> SceneModel {
    let motion = BlendMotionList {
        list: vec![
            BlendMotion { curves: vec![slide(0.0, 10.0)], ..BlendMotion::still("Cube", located(0.0)) },
            BlendMotion { parent: Some(0), ..BlendMotion::still("Child", located(1.0)) },
            BlendMotion::still("Floor", located(-5.0)),
            BlendMotion { curves: vec![slide(2.0, 4.0)], ..BlendMotion::still("Light", located(2.0)) },
            BlendMotion { curves: vec![slide(-3.0, -6.0)], ..BlendMotion::still("Camera", located(-3.0)) },
        ],
    };
    SceneModel {
        objects: vec![triangle("Cube", 0.0), triangle("Child", 1.0), triangle("Floor", -5.0)],
        lights: vec![SceneLight {
            name: "Light".into(),
            location: [2.0, 0.0, 0.0],
            matrix_world: { let mut m = IDENTITY_M4; m[3][0] = 2.0; m },
            color: [1.0; 3],
            energy: 10.0,
            kind: "POINT".into(),
            spot_size: 0.0,
            size: 0.1,
        }],
        cameras: vec![SceneCamera {
            name: "Camera".into(),
            location: [-3.0, 0.0, 0.0],
            matrix_world: { let mut m = IDENTITY_M4; m[3][0] = -3.0; m },
            kind: "PERSP".into(),
            focal_length_mm: 50.0,
            sensor_width: 36.0,
            ortho_scale: 6.0,
            clip_start: 0.1,
            clip_end: 100.0,
            display_size: 1.0,
            dof: Default::default(),
        }],
        motion: SceneMotion { motion, object: vec![0, 1, 2], light: vec![3], camera: vec![4], empty: vec![] },
        ..Default::default()
    }
}

fn x_of(object: &BObject3D) -> f32 {
    match object {
        BObject3D::Loaded { matrix_world, .. } => matrix_world[3][0],
        BObject3D::Primitive { x, .. } => *x,
    }
}

fn at_frame(frame: &str) -> BState {
    let mut state = BState::new();
    state.set(CtxKey::Frame, frame);
    state
}

#[test]
fn the_object_list_stands_where_the_curves_put_it_at_the_shown_frame() {
    let model = moving_scene();
    let objects = current_scene(&at_frame("1"), Some(&model));
    assert_eq!(objects.iter().map(x_of).collect::<Vec<_>>(), [0.0, 1.0, -5.0]);
    let objects = current_scene(&at_frame("6"), Some(&model));
    let xs: Vec<f32> = objects.iter().map(x_of).collect();
    assert!((xs[0] - 5.0).abs() < 1e-5, "the Cube is half way: {xs:?}");
    assert!((xs[1] - 6.0).abs() < 1e-5, "the Child rides it: {xs:?}");
    assert_eq!(xs[2], -5.0, "the Floor never moves");
    let objects = current_scene(&at_frame("250"), Some(&model));
    assert!((x_of(&objects[0]) - 10.0).abs() < 1e-5, "past the last key it holds");
    // No frame in state: the file's rest pose.
    assert_eq!(x_of(&current_scene(&BState::new(), Some(&model))[0]), 0.0);
}

#[test]
fn lights_cameras_and_the_overlay_move_with_the_frame() {
    let model = moving_scene();
    assert!((lights_at(&model, Some(6.0))[0].matrix_world[3][0] - 3.0).abs() < 1e-5);
    assert_eq!(lights_at(&model, None)[0].matrix_world[3][0], model.lights[0].matrix_world[3][0], "no frame: as loaded");
    let camera = active_camera_at(&model, Some(11.0)).expect("the one camera");
    assert!((camera.matrix_world[3][0] + 6.0).abs() < 1e-5, "the camera is where its curve puts it");
    let lines = scene_extras_at(&model, Some(6.0));
    let light_x = lines.iter().find(|l| l.kind == BExtraKind::Light).map(|l| l.a[0]).expect("a light line");
    let still = scene_extras_at(&model, None);
    let still_x = still.iter().find(|l| l.kind == BExtraKind::Light).map(|l| l.a[0]).unwrap();
    assert!((light_x - still_x - 1.0).abs() < 1e-3, "the light's drawing moved a unit with it: {light_x} vs {still_x}");
}

#[test]
fn a_moving_scene_is_rebuilt_per_frame_and_a_still_one_only_per_model() {
    clear_loaded_scene();
    clear_scene_objects();
    let moving = std::sync::Arc::new(moving_scene());
    let mut state = at_frame("1");
    set_loaded_at(Some(moving.clone()), &state);
    let before = object_builds();
    let first = scene_objects(&state);
    scene_objects(&state);
    assert_eq!(object_builds(), before + 1, "one build per frame, shared across a pass");
    // The playhead steps: a new build, standing further along.
    apply_action(&mut state, &BAction::Set(CtxKey::FrameStep.path(), "+5".into()));
    set_loaded_at(Some(moving.clone()), &state);
    let later = scene_objects(&state);
    assert_eq!(object_builds(), before + 2);
    assert!(x_of(&later[0]) > x_of(&first[0]), "the Cube moved on");

    // A still scene: the frame is not part of its key.
    let still = std::sync::Arc::new(SceneModel { objects: vec![triangle("Floor", -5.0)], ..Default::default() });
    assert_eq!(moving_frame(&still, Some(6.0)), None);
    set_loaded_at(Some(still.clone()), &state);
    scene_objects(&state);
    let built = object_builds();
    for frame in ["2", "3", "4"] {
        state.set(CtxKey::Frame, frame);
        set_loaded_at(Some(still.clone()), &state);
        scene_objects(&state);
    }
    assert_eq!(object_builds(), built, "a still scene never rebuilds as the playhead runs");
    clear_loaded_scene();
}
