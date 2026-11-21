//! The stage writer on small hand-built scenes, and the checker on bad layers.
use super::check::UsdCheck;
use super::name::{UsdName, UsdNameBook};
use crate::scene::{SceneCamera, SceneLight, SceneMaterial, SceneMesh, SceneModel, SceneNormals, SceneObject};
use std::sync::Arc;

/// Two triangles forming a quad, with a UV per vertex.
fn quad() -> SceneMesh {
    SceneMesh {
        vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0]],
        indices: vec![0, 1, 2, 0, 2, 3],
        normals: SceneNormals::PerFace(vec![[0.0, 0.0, 1.0]; 2]),
        corners: Default::default(),
    }
}

fn material(name: &str) -> SceneMaterial {
    SceneMaterial { name: name.into(), ..SceneMaterial::neutral() }
}

fn camera() -> SceneCamera {
    SceneCamera {
        name: "Camera".into(),
        location: [0.0; 3],
        matrix_world: crate::scene::IDENTITY_M4,
        kind: "PERSP".into(),
        focal_length_mm: 50.0,
        sensor_width: 36.0,
        sensor_height: 24.0,
        sensor_fit: "AUTO".into(),
        shift: [0.0; 2],
        ortho_scale: 6.0,
        clip_start: 0.1,
        clip_end: 100.0,
        display_size: 1.0,
        dof: Default::default(),
    }
}

fn light(kind: &str, shape: &str) -> SceneLight {
    SceneLight {
        name: format!("L {kind}"),
        location: [0.0; 3],
        matrix_world: crate::scene::IDENTITY_M4,
        color: [1.0, 0.9, 0.8],
        energy: 100.0,
        kind: kind.into(),
        spot_size: 1.0,
        spot_blend: 0.2,
        size: 0.5,
        size_y: 0.25,
        shape: shape.into(),
    }
}

/// "Wall" and "Wall.001" share a mesh; "Wall.001" is parented to "Pole".
fn scene() -> SceneModel {
    let mesh = Arc::new(quad());
    let mut a = SceneObject::identity("Wall", quad());
    a.mesh = mesh.clone();
    a.materials = vec![material("Brick"), material("Paint job")];
    a.material_indices = Arc::new(vec![0, 1]);
    a.uvs = Arc::new(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
    let mut b = a.clone();
    b.name = "Wall.001".into();
    b.matrix_world[3] = [5.0, 0.0, 0.0, 1.0];
    let mut pole = SceneObject::identity("Pole", quad());
    pole.matrix_world[3] = [2.0, 0.0, 0.0, 1.0];
    let mut scene = SceneModel { objects: vec![a, b, pole], cameras: vec![camera()], ..Default::default() };
    scene.lights = vec![light("POINT", ""), light("SUN", ""), light("AREA", "RECTANGLE"), light("SPOT", "")];
    scene.parents = vec![("Wall.001".into(), "Pole".into())];
    scene.world.background.strength = 1.0;
    scene.world.background.color = [0.1, 0.2, 0.3];
    scene
}

#[test]
fn a_scene_writes_a_layer_the_checker_accepts() {
    let text = scene().to_usda();
    let r = UsdCheck::run(&text).unwrap_or_else(|e| panic!("{e}\n{text}"));
    assert_eq!((r.count("Camera"), r.count("SphereLight"), r.count("DistantLight"), r.count("RectLight"), r.count("DomeLight")), (1, 2, 1, 1, 1));
    assert_eq!((r.count("Material"), r.count("GeomSubset")), (2, 2));
    // Wall and Wall.001 share one prototype Mesh; Pole has its own.
    assert_eq!(r.count("Mesh"), 2);
    assert_eq!(r.triangle, 4);
    assert!(text.contains("upAxis = \"Z\"") && text.contains("metersPerUnit = 1") && text.contains("defaultPrim = \"World\""));
}

#[test]
fn a_shared_mesh_is_a_prototype_and_a_parent_nests_its_child() {
    let text = scene().to_usda();
    assert_eq!(text.matches("instanceable = true").count(), 2);
    assert!(text.contains("prepend references = </World/Prototype/Wall>"));
    // Wall.001 sits at x = 5 in the world and under Pole at x = 2: local x = 3.
    let pole = text.find("def Xform \"Pole\"").expect("Pole");
    let child = text.find("def Xform \"Wall_001\"").expect("Wall_001 below Pole");
    assert!(child > pole);
    assert!(text[child..].contains("(3, 0, 0, 1)"), "{}", &text[child..child + 400]);
    assert!(text.contains("displayName = \"Wall.001\"") && text.contains("displayName = \"Paint job\""));
}

#[test]
fn light_units_follow_the_law_in_light_rs() {
    use super::light::UsdLight;
    assert_eq!(UsdLight::intensity(&light("POINT", "")), 100.0);
    assert_eq!(UsdLight::intensity(&light("SUN", "")), 100.0);
    // Area energy is stored as 4/pi of the UI watts; the layer has the UI watts.
    let stored = 500.0 * 4.0 / std::f32::consts::PI;
    let mut area = light("AREA", "SQUARE");
    area.energy = stored;
    assert!((UsdLight::intensity(&area) - 500.0).abs() < 1e-3);
    let spot = scene().to_usda();
    assert!(spot.contains("inputs:shaping:cone:angle = 28.64"), "half of the 1 rad cone, in degrees");
}

#[test]
fn camera_film_follows_sensor_fit() {
    use super::camera::UsdCamera;
    let c = camera();
    assert_eq!(UsdCamera::film(&c, 2.0), (36.0, 18.0));
    assert_eq!(UsdCamera::film(&c, 0.5), (18.0, 36.0));
    let ortho = SceneCamera { kind: "ORTHO".into(), ..camera() };
    assert_eq!(UsdCamera::film(&ortho, 2.0), (60.0, 30.0));
}

#[test]
fn names_become_identifiers_deterministically() {
    assert_eq!(UsdName::id("Lot/Car.001"), "Lot_Car_001");
    assert_eq!(UsdName::id("3D"), "_3D");
    assert_eq!(UsdName::id(""), "_");
    assert_eq!(UsdName::id("Wall é"), "Wall__");
    let mut book = UsdNameBook::default();
    assert_eq!(book.claim("/World", "a b"), "a_b");
    assert_eq!(book.claim("/World", "a.b"), "a_b_1");
    assert_eq!(book.claim("/World/x", "a b"), "a_b");
}

#[test]
fn the_checker_names_what_is_wrong() {
    let good = scene().to_usda();
    assert!(UsdCheck::run("def Xform \"A\"").is_err(), "no header");
    assert!(UsdCheck::run(&good.replacen("}", "", 1)).is_err(), "unbalanced");
    let skew = good.replacen("int[] faceVertexCounts = [3, 3]", "int[] faceVertexCounts = [3, 3, 3]", 1);
    assert!(UsdCheck::run(&skew).unwrap_err().contains("faceVertexCounts"));
    let bad_name = good.replacen("def Xform \"Pole\"", "def Xform \"Po le\"", 1);
    assert!(UsdCheck::run(&bad_name).unwrap_err().contains("not a valid prim name"));
    let twice = good.replacen("def Scope \"Prototype\"", "def Scope \"Material\"", 1);
    assert!(UsdCheck::run(&twice).unwrap_err().contains("twice"));
}
