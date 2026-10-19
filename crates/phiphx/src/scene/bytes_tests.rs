//! `Phx::open_bytes` reads the same scene as `Phx::open` — the guard on the
//! split between reading a file and parsing one.
//!
//! The fixture is one of Blender's own upstream test files, read through
//! both doors and compared. When `REFS` is not checked out the test reports
//! that and stops rather than failing: it is a parity check against an
//! external checkout, and a missing checkout is not a defect in this crate.

use crate::phx::Phx;
use std::path::{Path, PathBuf};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../REFS/blender/blender/tests/files/modeling/operators.blend")
}

#[test]
fn bytes_and_path_read_the_same_scene() {
    let path = fixture();
    let Ok(data) = std::fs::read(&path) else {
        eprintln!("skipped: no REFS checkout at {}", path.display());
        return;
    };

    let from_path = Phx::open(&path).expect("open by path");
    let from_bytes = Phx::open_bytes("operators.blend", &data).expect("open by bytes");

    assert_eq!(from_bytes.objects.len(), from_path.objects.len(), "object count");
    assert!(!from_bytes.objects.is_empty(), "the fixture has objects");

    for (b, p) in from_bytes.objects.iter().zip(&from_path.objects) {
        assert_eq!(b.name, p.name);
        assert_eq!(b.mesh.vertices, p.mesh.vertices, "{} vertices", p.name);
        assert_eq!(b.mesh.indices, p.mesh.indices, "{} indices", p.name);
        assert_eq!(b.uvs, p.uvs, "{} uvs", p.name);
        assert_eq!(b.material_indices, p.material_indices, "{} material ids", p.name);
    }

    // The geometry is identical, so the memory figure must be too — it is
    // computed from those buffers (`SceneModel::heap_bytes`).
    assert_eq!(from_bytes.report.memory_bytes, from_path.report.memory_bytes);

    // What differs, and should: the source a scene reports.
    assert_eq!(from_bytes.source_path.as_deref(), Some("operators.blend"));
}

/// The cheap read — version, frame range, thumbnail — also works from bytes,
/// which is what a browser needs before it decides to parse anything.
#[test]
fn bytes_and_path_peek_the_same() {
    let path = fixture();
    let Ok(data) = std::fs::read(&path) else {
        eprintln!("skipped: no REFS checkout at {}", path.display());
        return;
    };
    let by_path = Phx::peek(&path).expect("peek by path");
    let by_bytes = Phx::peek_bytes(&data).expect("peek by bytes");
    assert_eq!(by_bytes.version, by_path.version);
    assert_eq!(by_bytes.scenes.len(), by_path.scenes.len());
    assert_eq!(
        by_bytes.thumbnail.as_ref().map(|t| (t.width, t.height, t.rgba.len())),
        by_path.thumbnail.as_ref().map(|t| (t.width, t.height, t.rgba.len())),
    );
}
