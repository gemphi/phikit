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
