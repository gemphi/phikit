//! Mesh data must match what Blender itself reads. `truth/mesh_digest.txt`
//! was produced by Blender 5.0.1 (`truth/blender_mesh_digest.py`) for the
//! fixtures, two production files and Blender's upstream `tests/files`;
//! this test computes the
//! same digests with phiphx. Files missing on this machine are skipped.

use phiphx::blend::BlendFile;
use std::path::PathBuf;

fn digest(file: &str, path: &PathBuf) -> Vec<String> {
    let f = BlendFile::parse_bytes(&std::fs::read(path).unwrap()).unwrap();
    let s = f.extract_scene().unwrap();
    s.objects
        .iter()
        .filter(|o| o.object_type == "MESH")
        .map(|o| {
            let ps: f64 = o.vertices.iter().enumerate().map(|(i, v)| (i as f64 + 1.0) * (v[0] as f64 + 2.0 * v[1] as f64 + 3.0 * v[2] as f64)).sum();
            let fs: u64 = o.faces.iter().enumerate().map(|(i, f)| (i as u64 + 1) * f.iter().enumerate().map(|(k, &x)| (k as u64 + 1) * x as u64).sum::<u64>()).sum();
            let uv: f64 = o.loop_uvs.iter().enumerate().map(|(i, u)| ((i % 7) as f64 + 1.0) * (u[0] as f64 + 2.0 * u[1] as f64)).sum();
            let mi: u64 = o.face_materials.iter().enumerate().map(|(i, &m)| (i as u64 + 1) * m as u64).sum();
            let sm = o.face_smooth.iter().filter(|&&b| b).count();
            format!("{file}|{}|{}|{}|{ps:.1}|{fs}|{uv:.1}|{mi}|{sm}", o.name, o.vertices.len(), o.faces.len())
                .replace("|-0.0|", "|0.0|")
        })
        .collect()
}

#[test]
fn meshes_match_blender() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let upstream = root.join("../../../ext/REFS/blender/blender/tests/files");
    // Production files kept outside the repo (see fixtures/README.md).
    let assets = root.join("../phiber/assets/production");
    let files = [
        root.join("tests/fixtures/TEXTURED-PACKED.blend"),
        root.join("tests/fixtures/THYMUS-BOLDROID-WAVE-003.blend"),
        root.join("tests/fixtures/FLORIS_BOLDRIOD_RIG.blend"),
        assets.join("LAND-TYPE-15-01.blend"),
        assets.join("Mannequins-Female--Pose-05.blend"),
        upstream.join("modeling/operators.blend"),
        upstream.join("usd/usd_simple_scene.blend"),
        upstream.join("usd/empty.blend"),
    ];
    let expected: Vec<&str> = include_str!("truth/mesh_digest.txt").lines().collect();
    let mut checked = 0;
    for path in files.iter().filter(|p| p.exists()) {
        let name = path.file_name().unwrap().to_str().unwrap();
        let mut want: Vec<&str> = expected.iter().copied().filter(|l| l.starts_with(&format!("{name}|"))).collect();
        let mut got = digest(name, path);
        want.sort();
        got.sort();
        let diff: Vec<_> = want.iter().zip(&got).filter(|(a, b)| **a != b.as_str()).take(5).collect();
        assert_eq!(want.len(), got.len(), "{name}: mesh count");
        assert!(diff.is_empty(), "{name}: blender vs phiphx {diff:#?}");
        checked += got.len();
    }
    assert!(checked > 0, "no fixture files found");
}
