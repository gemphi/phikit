use super::*;

#[test]
fn formats_like_blender() {
    let r = SceneReadReport { duration_ms: 1410, verts: 754928, faces: 12, tris: 20, objects: 179, ..Default::default() };
    assert_eq!(r.duration_line(), "Blender file read in 0m1.41s");
    assert_eq!(r.stats_line(), "Verts:754,928 | Faces:12 | Tris:20 | Objects:179");
    // With memory set, stats line includes it.
    let r2 = SceneReadReport { memory_bytes: 52_428_800, ..r.clone() };
    assert_eq!(r2.mem_line(), "Mem: 50.0 MiB");
    assert!(r2.stats_line().ends_with("| Mem: 50.0 MiB"));
}
