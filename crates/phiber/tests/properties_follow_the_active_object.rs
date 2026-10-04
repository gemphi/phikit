//! Blender's Properties editor shows `context.object`: switch the active
//! object and the Object tab's transform and the Material tab's surface
//! values are that object's. Here the switch goes through the reducer, so
//! the rule that follows `ctx.object` is what is under test.

use phiber::phiphx::scene::{SceneMaterial, SceneMesh, SceneModel, SceneNormals, SceneObject};
use phiber::v2::scene::{clear_loaded_scene, set_loaded_scene};
use phiber::v2::state::{BKey as _, BState, ButsKey, CtxKey, action::apply_action};
use phiber::v2::view::BAction;

fn object(name: &str, location: [f32; 3], scale: [f32; 3], color: [f32; 4], roughness: f32) -> SceneObject {
    SceneObject {
        name: name.to_string(),
        mesh: SceneMesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 3.0]],
            indices: vec![0, 1, 2],
            normals: SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]]),
            corners: Default::default(),
        }
        .into(),
        location,
        scale,
        matrix_world: [[scale[0], 0.0, 0.0, 0.0], [0.0, scale[1], 0.0, 0.0], [0.0, 0.0, scale[2], 0.0], [location[0], location[1], location[2], 1.0]],
        materials: vec![SceneMaterial {
            name: format!("{name} material"),
            base_color: color,
            viewport_color: color,
            viewport_pbr: [roughness, 0.25],
            roughness,
            metallic: 0.25,
            // A test's material has no node tree.
            graph: Default::default(),
            base_color_texture: None,
            texture: None,
            bsdf: Default::default(),
            normal_texture: None,
            normal_map: None,
            maps: Default::default(),
        }],
        material_indices: Default::default(),
        uvs: Default::default(),
        color: [1.0; 4],
        shadow: Default::default(),
    }
}

fn value(s: &BState, key: &'static str, axis: &'static str) -> f32 {
    s.get(&ButsKey::Axis(key, axis).path()).and_then(|v| v.parse().ok()).unwrap_or_else(|| panic!("{key}.{axis} is set"))
}

#[test]
fn switching_the_active_object_refills_the_object_and_material_tabs() {
    set_loaded_scene(Some(SceneModel {
        objects: vec![
            object("Cube", [1.0, 2.0, 3.0], [1.0, 1.0, 1.0], [0.8, 0.1, 0.1, 1.0], 0.5),
            object("Lamp Post", [-4.0, 0.0, 0.5], [2.0, 2.0, 2.0], [0.1, 0.2, 0.9, 1.0], 0.9),
        ],
        ..Default::default()
    }));
    let mut s = BState::default();
    apply_action(&mut s, &BAction::Set(CtxKey::Object.path(), "Cube".into()));
    assert_eq!([value(&s, "location", "x"), value(&s, "location", "y"), value(&s, "location", "z")], [1.0, 2.0, 3.0]);
    assert_eq!(value(&s, "scale", "z"), 1.0);
    assert_eq!([value(&s, "dimensions", "x"), value(&s, "dimensions", "y"), value(&s, "dimensions", "z")], [1.0, 2.0, 3.0], "bounding box × scale");
    assert_eq!(s.get(ButsKey::Value("diffuse_color")), Some("0.8 0.1 0.1 1"));
    assert_eq!(s.get(ButsKey::Value("roughness")), Some("0.5"));

    apply_action(&mut s, &BAction::Set(CtxKey::Object.path(), "Lamp Post".into()));
    assert_eq!(value(&s, "location", "x"), -4.0, "the tabs follow the active object");
    assert_eq!(value(&s, "dimensions", "z"), 6.0, "a scale of two doubles the dimensions");
    assert_eq!(s.get(ButsKey::Value("diffuse_color")), Some("0.1 0.2 0.9 1"));
    assert_eq!(s.get(ButsKey::Value("roughness")), Some("0.9"));
    assert_eq!(s.get(ButsKey::Value("metallic")), Some("0.25"));

    apply_action(&mut s, &BAction::Set(CtxKey::Object.path(), "Nobody".into()));
    assert_eq!(value(&s, "location", "x"), -4.0, "an object the scene lacks changes nothing");
    clear_loaded_scene();
}

/// Viewport Display ▸ Shadow and Ray Visibility ▸ Shadow: the panel shows
/// the file's switches for the active object, a click switches that object
/// in the viewport's list and the save carries both bits (`dtx`,
/// `visibility_flag`); the other object keeps the file's.
#[test]
fn the_shadow_checkboxes_switch_the_active_object() {
    use phiber::phiphx::blend::BlendShadowCast;
    use phiber::v2::scene::{BObject3D, scene_source};
    let mut shy = object("Shy", [0.0; 3], [1.0; 3], [0.8; 4], 0.5);
    shy.shadow = BlendShadowCast { display: false, ray: true };
    let model = phiber::phiphx::scene::SceneModel { objects: vec![object("Cube", [0.0; 3], [1.0; 3], [0.8; 4], 0.5), shy], ..Default::default() };
    set_loaded_scene(Some(model.clone()));
    let mut s = BState::default();
    apply_action(&mut s, &BAction::Set(CtxKey::Object.path(), "Shy".into()));
    assert_eq!((s.get(ButsKey::Value("show_shadows")), s.get(ButsKey::Value("visible_shadow"))), (Some("false"), Some("true")), "the file's switches");
    apply_action(&mut s, &BAction::Set(CtxKey::Object.path(), "Cube".into()));
    assert_eq!((s.get(ButsKey::Value("show_shadows")), s.get(ButsKey::Value("visible_shadow"))), (Some("true"), Some("true")));
    apply_action(&mut s, &BAction::Set(ButsKey::Value("visible_shadow").path(), "false".into()));
    let shadow = |s: &BState, name: &str| match scene_source(s).into_iter().find(|o| matches!(o, BObject3D::Loaded { name: n, .. } if n == name)) {
        Some(BObject3D::Loaded { shadow, .. }) => shadow,
        _ => panic!("{name}"),
    };
    assert_eq!(shadow(&s, "Cube"), BlendShadowCast { display: true, ray: false }, "the click switches the active object");
    assert_eq!(shadow(&s, "Shy"), BlendShadowCast { display: false, ray: true }, "the other keeps the file's");
    // Back to Shy and again to Cube: the panel shows the session's switch.
    apply_action(&mut s, &BAction::Set(CtxKey::Object.path(), "Shy".into()));
    apply_action(&mut s, &BAction::Set(CtxKey::Object.path(), "Cube".into()));
    assert_eq!(s.get(ButsKey::Value("visible_shadow")), Some("false"));
    let edits = phiber::v2::scene::save::gather(&s, &model);
    let cube = edits.objects.iter().find(|o| o.name == "Cube").expect("Cube's edit");
    assert_eq!(cube.shadow, Some(BlendShadowCast { display: true, ray: false }), "the save writes both bits");
    assert_eq!(edits.objects.iter().find(|o| o.name == "Shy").and_then(|o| o.shadow), None, "an unswitched object writes nothing");
    clear_loaded_scene();
}
