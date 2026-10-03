use super::*;
use crate::phiphx::scene::IDENTITY_M4;

fn cam(kind: &str) -> SceneCamera {
    SceneCamera {
        name: "Camera".into(),
        location: [0.0; 3],
        matrix_world: IDENTITY_M4,
        kind: kind.into(),
        focal_length_mm: 50.0,
        sensor_width: 36.0,
        ortho_scale: 6.0,
        clip_start: 0.1,
        clip_end: 100.0,
        display_size: 1.0,
        dof: Default::default(),
    }
}

#[test]
fn perspective_camera_is_a_pyramid_pointing_down_minus_z() {
    let segs = camera_shape(&cam("PERSP"), [1.0, 0.5625]);
    // 4 frame edges + 4 apex edges + 3 up-triangle edges.
    assert_eq!(segs.len(), 11);
    // Blender: depth = (drawsize/2) * lens / -(sensor/2) = 0.5 * 50 / -18.
    let depth = segs[0].0[2];
    assert!((depth - (0.5 * 50.0 / -18.0)).abs() < 1e-4, "{depth}");
    assert!(segs.iter().any(|s| s.0 == [0.0; 3]));
}

#[test]
fn ortho_camera_has_no_apex_lines() {
    let segs = camera_shape(&cam("ORTHO"), [1.0, 1.0]);
    assert!(segs.iter().all(|s| s.0 != [0.0; 3]));
}

#[test]
fn empty_circle_lies_in_xz_like_blender() {
    let e = SceneEmpty {
        name: "E".into(),
        matrix_world: IDENTITY_M4,
        display_type: "CIRCLE".into(),
        display_size: 1.0,
    };
    assert!(empty_shape(&e).iter().all(|s| s.0[1] == 0.0 && s.1[1] == 0.0));
}

#[test]
fn empty_display_size_scales_the_shape() {
    let mut model = SceneModel::default();
    model.empties.push(SceneEmpty {
        name: "E".into(),
        matrix_world: IDENTITY_M4,
        display_type: "PLAIN_AXES".into(),
        display_size: 2.0,
    });
    let lines = scene_extras(&model);
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0].b, [2.0, 0.0, 0.0]);
}

/// A rig draws its shown bones (phiphx `rig::display`): the fixture's `Rig`
/// shows two octahedra, three circles and a cube, `Stick` one stick — and a
/// deleted armature draws none.
#[test]
fn every_shown_bone_draws_as_blender_draws_it() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/armature_display.blend");
    let scene = crate::phiphx::blend::BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    let model = SceneModel::from_blend_owned(scene, &path.to_string_lossy());
    crate::v2::scene::source::set_loaded_scene(Some(model.clone()));
    let mut s = crate::v2::state::BState::new();
    let bones = |s: &crate::v2::state::BState| {
        let mut count = std::collections::BTreeMap::new();
        for l in scene_extras_now(s, &model).iter().filter(|l| l.kind == BExtraKind::Bone) {
            *count.entry(l.file_name(&model).unwrap().to_string()).or_insert(0) += 1;
        }
        count
    };
    let drawn = bones(&s);
    assert_eq!(drawn.get("Rig"), Some(&(2 * 12 + 3 * 16 + 12)), "{drawn:?}");
    assert_eq!(drawn.get("Stick"), Some(&1), "the soloed collection's one bone");
    crate::v2::scene::edit::BSceneEdit { deleted: vec!["Stick".into()], ..Default::default() }.write(&mut s);
    assert_eq!(bones(&s).get("Stick"), None);
    crate::v2::scene::source::set_loaded_scene(None);
}
