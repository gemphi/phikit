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
