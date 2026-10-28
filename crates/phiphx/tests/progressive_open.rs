//! Progressive open: the base scene arrives first, the modifier-evaluated
//! meshes swap in later — and the end state is exactly what an eager open
//! produces. THYMUS carries every supported modifier (subsurf, mirror, array,
//! solidify, weld, armature), so it exercises the whole path.

use phiphx::Phx;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

#[test]
fn base_plus_finished_modifiers_equals_an_eager_open() {
    let path = fixture("THYMUS-BOLDROID-WAVE-003.blend");

    let eager = Phx::open(&path).expect("eager open");
    let (mut base, deferred) = Phx::open_progressive(&path, &|_, _| true).expect("progressive open");
    let deferred = deferred.expect("the fixture carries modifiers, so there is deferred work");
    assert!(!deferred.is_empty(), "at least one object has a modifier");

    // Before finishing, the base is pre-modifier: it must differ from eager
    // for at least one object, or the test proves nothing.
    assert_ne!(base.objects, eager.objects, "base meshes are not yet evaluated");
    // And its read report counts the base meshes — lighter than the evaluated
    // scene — so it differs from eager too (the generative modifiers add
    // geometry). `faces` is the file's stored polygon count in both.
    assert_ne!(base.report.tris, eager.report.tris, "base tris are pre-modifier");
    assert_ne!(base.report.verts, eager.report.verts, "base verts are pre-modifier");
    assert_eq!(base.report.faces, eager.report.faces, "stored faces are not evaluated");

    base.apply_deferred(deferred.finish());
    assert_eq!(
        base.objects, eager.objects,
        "progressive base + finished modifiers matches the eager open, object for object"
    );
    // apply_deferred refreshed the report, so its counts now follow the
    // evaluated meshes — the same figures the eager open reports.
    assert_eq!(base.report.verts, eager.report.verts, "report verts follow the evaluated meshes");
    assert_eq!(base.report.tris, eager.report.tris, "report tris follow the evaluated meshes");
    assert_eq!(base.report.faces, eager.report.faces, "faces unchanged by evaluation");
}

#[test]
fn a_file_without_modifiers_has_nothing_to_defer() {
    // crease.blend has creases but no modifier stack — the base is already the
    // final mesh, so the deferred work (if any) is empty and swaps nothing.
    let path = fixture("crease.blend");
    let eager = Phx::open(&path).expect("eager open");
    let (mut base, deferred) = Phx::open_progressive(&path, &|_, _| true).expect("progressive open");
    if let Some(deferred) = deferred {
        base.apply_deferred(deferred.finish());
    }
    assert_eq!(base.objects, eager.objects, "no modifiers → base already equals eager");
}

/// What the viewport draws of a mesh its rig deforms is its pose, at every
/// frame — `SceneMotion::posed_at` over the triangles its skin keeps — and
/// Phiber opens every file progressively. Those triangles used to be the
/// mesh as saved: the progressive read has no evaluated meshes when the
/// skins are made, and the swap never told them. So a figure modelled as a
/// half under Mirror, then Armature, then Subdivision (every garment and
/// accessory in the CW-POSES files) lost its mirrored half the moment it was
/// posed: `Earring pearl` drew 96 of Blender's 192 triangles, `Chatty
/// Pants` its saved cage.
///
/// The counts are Blender 5.2's own (`evaluated_get(depsgraph).to_mesh()`
/// over `mirror_rig.blend`, `fixtures/make_mirror_rig.py`), the twin with a
/// negative determinant as the dancers' second figure has.
#[test]
fn a_mirrored_rigged_half_poses_whole_after_the_swap() {
    let path = fixture("mirror_rig.blend");
    let eager = Phx::open(&path).expect("eager open");
    let (mut base, deferred) = Phx::open_progressive(&path, &|_, _| true).expect("progressive open");
    base.apply_deferred(deferred.expect("Mirror, Armature and Subdivision are deferred").finish());
    let frame = base.timeline.current_frame as f32;
    let (posed, _) = base.motion.posed_at(frame);
    let (truth, _) = eager.motion.posed_at(frame);
    let tris = |model: &phiphx::scene::SceneModel, meshes: &[(usize, phiphx::scene::SceneMesh)], name: &str| {
        let i = model.objects.iter().position(|o| o.name == name).expect(name);
        meshes.iter().find(|(k, _)| *k == i).map(|(_, m)| m.indices.len() / 3).expect("a rig deforms it")
    };
    for (name, blender) in [("Half Body", 192), ("Half Pearl", 48), ("Twin Body", 192)] {
        assert_eq!(tris(&eager, &truth, name), blender, "{name}: eager, posed");
        assert_eq!(tris(&base, &posed, name), blender, "{name}: progressive, posed — the mirrored half and the subdivision drawn");
    }
    assert_eq!(posed, truth, "a progressive open poses exactly as an eager one");
}
