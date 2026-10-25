//! End-to-end loading tests for real `.blend` example meshes.
//!
//! Drives `crate::phiphx::Phx::open` — the exact entry
//! point the Viewport/Outliner pipeline uses — over two fixture sets that ship
//! in the workspace:
//!
//!   1. Blender's small upstream test files (`gemphi/ext/REFS/blender/blender/tests/files`).
//!   2. The CW-Fashion example meshes (`gemphi/ext/REFS/zed-repo/.../assets/CW-Fashion`).
//!
//! Every fixture must return `Ok(SceneModel)` or a categorized
//! `Err(SceneLoadError)` WITHOUT panicking (Requirement 2.4, 3.1). When a load
//! succeeds, the produced geometry must satisfy the importer invariants
//! (Requirement 2.5, 2.6): `indices.len() % 3 == 0` and every index is in range.
//!
//! The tests skip gracefully when the optional REFS checkout is absent, so a
//! clean tree without the large fixtures still passes.

use std::path::{Path, PathBuf};

use phiber::phiphx::Phx;
use phiber::phiphx::scene::{SceneModel, SceneNormals};

/// Root of Blender's upstream test files inside REFS (relative to the phiber
/// crate manifest dir). Optional — tests skip when it is absent.
fn blender_tests_files() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../ext/REFS/blender/blender/tests/files")
}

/// Root of the CW-Fashion example meshes inside REFS (relative to the phiber
/// crate manifest dir). Optional — tests skip when it is absent.
fn cw_fashion_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../ext/REFS/zed-repo/crates/phiber/assets/CW-Fashion")
}

/// Assert every object's geometry satisfies the importer invariants: the index
/// buffer is a whole number of triangles and every index is in range; normals
/// are cardinal to the layout (Requirement 2.5, 2.6).
fn assert_scene_invariants(model: &SceneModel, label: &str) {
    for obj in &model.objects {
        let verts = obj.mesh.vertices.len();
        let idx = &obj.mesh.indices;
        assert_eq!(
            idx.len() % 3,
            0,
            "{label} / object `{}`: index buffer not a whole number of triangles ({} indices)",
            obj.name,
            idx.len()
        );
        for &i in idx {
            assert!(
                (i as usize) < verts,
                "{label} / object `{}`: index {i} out of range for {verts} vertices",
                obj.name
            );
        }
        match &obj.mesh.normals {
            SceneNormals::PerVertex(ns) => assert_eq!(
                ns.len(),
                verts,
                "{label} / object `{}`: per-vertex normals ({}) != vertices ({verts})",
                obj.name,
                ns.len()
            ),
            SceneNormals::PerFace(ns) => assert_eq!(
                ns.len(),
                idx.len() / 3,
                "{label} / object `{}`: per-face normals ({}) != triangles ({})",
                obj.name,
                ns.len(),
                idx.len() / 3
            ),
        }
    }
}

/// Load one fixture and report. Returns `Some((objects, verts))` on success,
/// `None` on a categorized error. Panics only propagate (which fails the test)
/// — the loader itself must never panic (Requirement 2.4).
fn load_and_report(path: &Path) -> Option<(usize, usize)> {
    let label = path.file_name().unwrap().to_string_lossy().to_string();
    match Phx::open(path) {
        Ok(model) => {
            assert_scene_invariants(&model, &label);
            let objs = model.objects.len();
            let verts: usize = model.objects.iter().map(|o| o.mesh.vertices.len()).sum();
            let tris: usize = model.objects.iter().map(|o| o.mesh.indices.len() / 3).sum();
            println!("  OK   {label}: {objs} objects, {verts} verts, {tris} tris");
            Some((objs, verts))
        }
        Err(e) => {
            println!("  ERR  {label}: [{}] {}", e.category(), e.message());
            None
        }
    }
}

/// Curated small Blender upstream fixtures load without panicking, and the ones
/// that carry mesh geometry produce valid, in-range triangle buffers.
#[test]
fn curated_blender_fixtures_load_without_panicking() {
    let root = blender_tests_files();
    if !root.exists() {
        eprintln!(
            "skip: {} not present (optional REFS checkout)",
            root.display()
        );
        return;
    }

    // A spread of small fixtures: empty scene, simple meshes, UI test files.
    let curated = [
        "usd/empty.blend",
        "usd/usd_simple_scene.blend",
        "usd/usd_mesh_subd.blend",
        "usd/usd_hierarchy_collision.blend",
        "usd/nested_instancing_test.blend",
        "ui_tests/allselect.blend",
        "ui_tests/flippedmatrixes.blend",
        "ui_tests/space_types.blend",
        "ui_tests/outlinertest.blend",
    ];

    let mut tested = 0usize;
    let mut with_geometry = 0usize;
    println!("curated Blender fixtures:");
    for rel in curated {
        let path = root.join(rel);
        if !path.exists() {
            continue;
        }
        tested += 1;
        if let Some((_objs, verts)) = load_and_report(&path) {
            if verts > 0 {
                with_geometry += 1;
            }
        }
    }

    if tested == 0 {
        eprintln!("skip: no curated fixtures found under {}", root.display());
        return;
    }
    // The simple mesh scenes in the set must yield real geometry, proving the
    // hardened parser produces vertices/indices (not just empty scenes).
    assert!(
        with_geometry > 0,
        "expected at least one curated fixture to yield mesh geometry"
    );
    println!("curated: {tested} loaded, {with_geometry} produced geometry");
}

/// The `usd_simple_scene.blend` fixture specifically must yield at least one
/// object with real vertices and a valid triangle buffer — the smoking-gun
/// proof that the modern-layout parser path works end to end.
#[test]
fn simple_scene_fixture_yields_real_geometry() {
    let path = blender_tests_files().join("usd/usd_simple_scene.blend");
    if !path.exists() {
        eprintln!("skip: {} not present", path.display());
        return;
    }
    let model = Phx::open(&path).expect("usd_simple_scene.blend must load");
    assert_scene_invariants(&model, "usd_simple_scene.blend");
    let verts: usize = model.objects.iter().map(|o| o.mesh.vertices.len()).sum();
    let tris: usize = model.objects.iter().map(|o| o.mesh.indices.len() / 3).sum();
    assert!(!model.objects.is_empty(), "scene should contain objects");
    assert!(verts > 0, "scene should contain real vertices");
    assert!(tris > 0, "scene should contain real triangles");
    println!(
        "usd_simple_scene.blend: {} objects, {verts} verts, {tris} tris",
        model.objects.len()
    );
}

/// The CW-Fashion example `.blend` meshes load through the same pipeline
/// without panicking. These are large production files (20–178 MB); the native
/// parser is O(n²) in block lookups and can be slow/stack-heavy on the biggest
/// ones, so this test is `#[ignore]` by default and skips files above a size
/// cap. Run explicitly with:
///   cargo test -p phiber --test blend_fixture_loading cw_fashion -- --ignored --nocapture
#[test]
#[ignore = "large production .blend files (20-178 MB); run with --ignored"]
fn cw_fashion_example_meshes_load() {
    let root = cw_fashion_root();
    if !root.exists() {
        eprintln!(
            "skip: {} not present (optional REFS checkout)",
            root.display()
        );
        return;
    }

    // Cap the file size we attempt so the default `--ignored` run stays bounded;
    // the very large files (120-178 MB) are the ones the importer explicitly
    // warns can stack-overflow — those are covered by the `.phx` sidecar path,
    // not the native parser exercised here.
    const MAX_BYTES: u64 = 96 * 1024 * 1024;

    let mut blends: Vec<PathBuf> = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "blend") {
                out.push(p);
            }
        }
    }
    walk(&root, &mut blends);
    blends.sort();

    println!("CW-Fashion example meshes:");
    let mut attempted = 0usize;
    let mut loaded = 0usize;
    for path in &blends {
        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(u64::MAX);
        let label = path.file_name().unwrap().to_string_lossy().to_string();
        if size > MAX_BYTES {
            println!("  SKIP {label}: {size} bytes exceeds cap ({MAX_BYTES})");
            continue;
        }
        attempted += 1;
        // Phx::open wraps the native parse in catch_unwind and maps a
        // panic to Err(Malformed), so a problematic file surfaces as an error
        // rather than aborting the test process (Requirement 2.4).
        if load_and_report(path).is_some() {
            loaded += 1;
        }
    }

    if attempted == 0 {
        eprintln!("skip: no CW-Fashion .blend files within the {MAX_BYTES}-byte cap");
        return;
    }
    println!("CW-Fashion: {attempted} attempted, {loaded} loaded Ok");
}
