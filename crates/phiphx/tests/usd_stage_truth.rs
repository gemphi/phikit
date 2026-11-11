//! The USD writer on real `.blend` fixtures: the layer has what the
//! `SceneModel` reports (prims, triangles) and passes the structural check.
//!
//! "Truth" here is the scene's own count, not a Blender dump: the writer
//! must not lose or invent geometry on the way out. Loading the layer in
//! `pxr` (usd-core) is an extra check that runs when `python -c "import
//! pxr"` works and is skipped, loudly, when it does not.
use phiphx::device::blend::BlendFile;
use phiphx::device::usd::write::scene::check::{UsdCheck, UsdReport};
use phiphx::scene::SceneModel;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

fn open(name: &str) -> SceneModel {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
    let scene = BlendFile::parse_bytes(&std::fs::read(&path).unwrap()).unwrap().extract_scene().unwrap();
    SceneModel::from_blend_owned(scene, &path.to_string_lossy())
}

/// Write `name`'s scene and check it; also load it in pxr when there is one.
fn layer(name: &str) -> (SceneModel, String, UsdReport) {
    let scene = open(name);
    let text = scene.to_usda();
    let report = UsdCheck::run(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
    pxr_load(name, &text);
    (scene, text, report)
}

/// Meshes the scene holds once each (a shared mesh counts once) and the
/// triangles in them.
fn unique(scene: &SceneModel) -> (usize, usize) {
    let mut seen = HashSet::new();
    let mut tri = 0;
    for o in scene.objects.iter().filter(|o| !o.mesh.vertices.is_empty()) {
        if seen.insert(Arc::as_ptr(&o.mesh)) {
            tri += o.mesh.indices.len() / 3;
        }
    }
    (seen.len(), tri)
}

fn pxr_load(name: &str, text: &str) {
    let have = std::process::Command::new("python").args(["-c", "import pxr"]).output().is_ok_and(|o| o.status.success());
    if !have {
        eprintln!("pxr (usd-core) not available: {name} was checked structurally only");
        return;
    }
    let file = std::env::temp_dir().join(format!("phiphx_{name}.usda"));
    std::fs::write(&file, text).unwrap();
    let code = "import sys\nfrom pxr import Usd\ns = Usd.Stage.Open(sys.argv[1])\nassert s.GetDefaultPrim(), 'no default prim'\nlist(s.Traverse())";
    let out = std::process::Command::new("python").args(["-c", code]).arg(&file).output().unwrap();
    let _ = std::fs::remove_file(&file);
    assert!(out.status.success(), "{name}: pxr refused the layer: {}", String::from_utf8_lossy(&out.stderr));
}

#[test]
fn lights_keep_their_kinds() {
    let (scene, text, r) = layer("LIGHTS.blend");
    let of = |k: &str| scene.lights.iter().filter(|l| l.kind == k).count();
    assert_eq!(r.count("DistantLight"), of("SUN"));
    assert_eq!(r.count("SphereLight"), of("POINT") + of("SPOT"));
    assert_eq!(r.count("RectLight") + r.count("DiskLight"), of("AREA"));
    assert_eq!(r.count("SphereLight") + r.count("DistantLight") + r.count("RectLight") + r.count("DiskLight"), scene.lights.len());
    assert!(of("SUN") > 0 && of("POINT") > 0 && of("SPOT") > 0 && of("AREA") > 0, "the fixture has every kind");
    assert_eq!(text.matches("ShapingAPI").count(), of("SPOT"));
    assert_eq!(r.count("Camera"), scene.cameras.len());
}

#[test]
fn an_area_light_is_written_in_ui_watts() {
    let (scene, text, _) = layer("LIGHTS.blend");
    let panel = scene.lights.iter().find(|l| l.name == "Panel").expect("Panel");
    let watts = panel.energy * std::f32::consts::PI / 4.0;
    assert!((watts - 500.0).abs() < 0.1, "the file's 500 W: {watts}");
    assert!(text.contains(&format!("float inputs:intensity = {}", watts)), "the layer carries {watts}");
}

#[test]
fn a_depth_of_field_camera_keeps_its_lens() {
    let (scene, text, r) = layer("DEPTH-OF-FIELD.blend");
    assert_eq!(r.count("Camera"), scene.cameras.len());
    let cam = scene.cameras.iter().find(|c| c.dof.on).expect("a camera with depth of field");
    assert!(text.contains(&format!("float focusDistance = {}", cam.dof.focus_distance)));
    assert!(text.contains(&format!("float focalLength = {}", cam.focal_length_mm)));
}

#[test]
fn meshes_and_triangles_match_the_scene() {
    for name in ["collections.blend", "array_merge.blend", "shrinkwrap.blend", "NORMAL-MAP.blend"] {
        let (scene, _, r) = layer(name);
        let (meshes, tri) = unique(&scene);
        assert_eq!((r.count("Mesh"), r.triangle), (meshes, tri), "{name}");
        assert!(meshes > 0, "{name} has geometry");
        assert_eq!(r.corner, tri * 3, "{name}: every face is a triangle");
        let names: HashSet<_> = scene.objects.iter().map(|o| &o.name).collect();
        assert!(names.len() == scene.objects.len(), "{name}: object names are unique");
    }
}

#[test]
fn collection_instances_share_a_prototype() {
    let (scene, text, r) = layer("collection_instances.blend");
    let (meshes, tri) = unique(&scene);
    assert_eq!((r.count("Mesh"), r.triangle), (meshes, tri));
    let used: usize = scene.objects.iter().filter(|o| !o.mesh.vertices.is_empty()).count();
    assert!(used > meshes, "the fixture draws a mesh more than once ({used} objects, {meshes} meshes)");
    let shared = text.matches("instanceable = true").count();
    assert!(shared >= 2, "{shared} instances");
    // Drawn triangles: each instance counts its prototype's.
    let drawn: usize = scene.objects.iter().map(|o| o.mesh.indices.len() / 3).sum();
    assert!(drawn > tri, "{drawn} drawn from {tri} written");
}

#[test]
fn collections_become_scopes_and_every_object_is_placed() {
    let (scene, text, r) = layer("collections.blend");
    let nested: usize = {
        fn count(c: &phiphx::scene::SceneCollection) -> usize {
            c.children.iter().map(|k| 1 + count(k)).sum()
        }
        scene.collections.iter().map(count).sum()
    };
    assert!(nested > 0);
    // Scopes: one per nested collection, plus Material and Prototype when written.
    let extra = usize::from(text.contains("def Scope \"Material\"")) + usize::from(text.contains("def Scope \"Prototype\""));
    assert_eq!(r.count("Scope"), nested + extra);
    let placed = r.count("Xform") - 1;
    assert!(placed >= scene.objects.len(), "{placed} Xforms for {} objects", scene.objects.len());
}

#[test]
fn materials_are_preview_surfaces() {
    for name in ["NODES.blend", "NORMAL-MAP.blend", "TEXTURED-PACKED.blend"] {
        let (scene, text, r) = layer(name);
        let names: HashSet<_> = scene.objects.iter().flat_map(|o| o.materials.iter().map(|m| m.name.as_str())).collect();
        assert_eq!(r.count("Material"), names.len(), "{name}");
        assert_eq!(text.matches("\"UsdPreviewSurface\"").count(), names.len(), "{name}");
        let textured: usize = scene.objects.iter().flat_map(|o| &o.materials).filter(|m| m.base_color_texture.is_some()).count();
        if textured > 0 {
            assert!(text.contains("\"UsdUVTexture\"") && text.contains("UsdPrimvarReader_float2"), "{name}");
        }
    }
}

#[test]
fn the_world_is_a_dome_light() {
    let (scene, text, r) = layer("WORLD-CAMERA-SPLIT.blend");
    let bg = &scene.world.background;
    let lit = bg.texture.is_some() || (bg.strength > 0.0 && bg.color != [0.0; 3]);
    assert_eq!(r.count("DomeLight"), usize::from(lit));
    if lit {
        assert!(text.contains(&format!("float inputs:intensity = {}", bg.strength)));
    }
}
