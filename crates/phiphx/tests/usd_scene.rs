//! A whole scene as USD and back, through the one convert path
//! (`Phx::convert`, `SceneTarget`, `SceneModel::to_usda`): every shown
//! object named, every triangle written, and the layer read back into the
//! same scene in the same place. Run on small fixtures.

use phiphx::Phx;
use phiphx::export::SceneTarget;
use phiphx::scene::SceneModel;
use std::path::{Path, PathBuf};

const FIXTURE: &[&str] = &[
    "MATERIALS-BSDF.blend",
    "collection_instances.blend",
    "instance_visibility.blend",
    "skin_rig.blend",
    "text.blend",
    "crease.blend",
];

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("phiphx_usd_scene");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

/// World-space box of the shown objects (in the scene's own Z-up space).
fn world_box(scene: &SceneModel) -> [glam::Vec3; 2] {
    let mesh = scene.world_mesh(glam::Mat4::IDENTITY);
    mesh.positions.iter().fold([glam::Vec3::splat(f32::MAX), glam::Vec3::splat(f32::MIN)], |[lo, hi], p| {
        [lo.min((*p).into()), hi.max((*p).into())]
    })
}

fn shown_triangle(scene: &SceneModel) -> usize {
    (0..scene.objects.len()).filter(|&i| scene.shows(i)).map(|i| scene.objects[i].mesh.indices.len() / 3).sum()
}

#[test]
fn blend_scene_to_usda_names_every_object_and_reads_back() {
    for name in FIXTURE {
        let input = fixture(name);
        let scene = Phx::open(&input).expect("open fixture");
        let usda = scene.to_usda();
        let shown: Vec<usize> = (0..scene.objects.len()).filter(|&i| scene.shows(i)).collect();
        assert!(!shown.is_empty(), "{name}: has meshes to show");
        assert_eq!(usda.mesh_count, shown.len(), "{name}: a mesh prim per shown object");
        assert_eq!(usda.triangle_count, shown_triangle(&scene), "{name}: every triangle");
        for prim in &usda.prim {
            assert!(usda.text.contains(&format!("def Xform \"{prim}\"")), "{name}: no Xform {prim}");
            assert!(usda.text.contains(&format!("def Mesh \"{prim}\"")), "{name}: no Mesh {prim}");
        }
        for needle in ["#usda 1.0", "metersPerUnit = 1", "upAxis = \"Y\"", "xformOp:transform"] {
            assert!(usda.text.contains(needle), "{name}: no {needle}");
        }

        let out = scratch(&name.replace(".blend", ".usda"));
        let c = Phx::convert(&input, &out).expect("convert");
        assert!(!c.copied);
        assert_eq!(c.triangle_count, usda.triangle_count, "{name}: convert writes the same scene");
        let back = Phx::open(&out).expect("open the written USD");
        assert_eq!(back.objects.len(), usda.mesh_count, "{name}: objects read back");
        assert_eq!(back.report.tris, usda.triangle_count, "{name}: triangles read back");
        let ([a0, a1], [b0, b1]) = (world_box(&scene), world_box(&back));
        let size = (a1 - a0).length().max(1e-3);
        assert!((a0 - b0).length() < size * 1e-3 && (a1 - b1).length() < size * 1e-3, "{name}: read back in place ({a0} {a1} vs {b0} {b1})");
    }
}

#[test]
fn usd_to_usd_is_copied_as_it_is() {
    let first = scratch("copy_in.usda");
    Phx::convert(&fixture(FIXTURE[0]), &first).expect("write usda");
    let second = scratch("copy_out.usd");
    let c = Phx::convert(&first, &second).expect("copy");
    assert!(c.copied);
    assert_eq!(std::fs::read(&first).unwrap(), std::fs::read(&second).unwrap());
    assert!(c.triangle_count > 0, "counted from the opened copy");
}

#[test]
fn every_target_writes_and_phx_reads_back() {
    let input = fixture(FIXTURE[0]);
    let scene = Phx::open(&input).unwrap();
    for ext in ["usda", "usd", "phx", "gltf", "ply"] {
        let out = scratch(&format!("target.{ext}"));
        assert!(SceneTarget::of_path(&out).is_some(), "{ext} is a target");
        let c = Phx::convert(&input, &out).unwrap_or_else(|e| panic!("{ext}: {e:?}"));
        assert!(c.byte_count > 0 && std::fs::metadata(&out).unwrap().len() as usize == c.byte_count);
    }
    let back = Phx::open(&scratch("target.phx")).expect("open the written PHX");
    assert_eq!(back.report.tris, shown_triangle(&scene));
    assert!(SceneTarget::of_path(Path::new("x.usdz")).is_none());
    assert!(Phx::convert(&input, &scratch("x.fbx")).is_err());
}
