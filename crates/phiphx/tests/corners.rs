//! Corner normals: smooth / split-normal meshes carry one unit normal per
//! triangle corner; all-flat meshes carry none.

use phiphx::phx::Phx;
use std::path::PathBuf;

#[test]
fn smooth_meshes_have_corner_normals_flat_ones_do_not() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let rig = Phx::open(&root.join("THYMUS-BOLDROID-WAVE-003.blend")).unwrap();
    let with: Vec<_> = rig.objects.iter().filter(|o| !o.mesh.corners.is_empty()).collect();
    assert!(!with.is_empty());
    for o in with {
        assert_eq!(o.mesh.corners.len(), o.mesh.indices.len(), "{}", o.name);
        let unit = o.mesh.corners.iter().filter(|c| *c != [0.0; 3]).all(|c| ((c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt() - 1.0).abs() < 1e-3);
        assert!(unit, "{}", o.name);
        assert_eq!(o.mesh.corners.normals[0], [0.0; 3]);
        assert!(o.mesh.corners.normals.len() <= o.mesh.vertices.len() + 1 || o.mesh.corners.normals.len() < o.mesh.corners.len());
    }
    let planes = Phx::open(&root.join("NORMAL-MAP.blend")).unwrap();
    assert!(planes.objects.iter().all(|o| o.mesh.corners.is_empty()));
}
