//! Depth of field must read off the file exactly what Blender reports.
//!
//! `truth/depth_of_field.txt` was produced by Blender 5.0.1
//! (`truth/blender_dof.py`), which also writes the `DEPTH-OF-FIELD.blend`
//! fixture: one camera focusing on an Empty with a hexagonal anamorphic
//! iris, one focusing by distance, and one with depth of field switched off.
//! None of them is left at a default, so a field read at the wrong offset in
//! `CameraDOFSettings` cannot pass by accident.
//!
//! The aperture radius is checked here too, because that is where the
//! f-stop stops being a ratio and becomes a length — and the formula
//! differs between a perspective and an orthographic camera.

use phiphx::blend::BlendFile;
use std::path::PathBuf;

fn model_of(path: &std::path::Path) -> phiphx::scene::SceneModel {
    let blend = BlendFile::parse_bytes(&std::fs::read(path).unwrap()).unwrap().extract_scene().unwrap();
    phiphx::scene::SceneModel::from_blend_owned(blend, &path.to_string_lossy())
}

fn row_of(cam: &phiphx::scene::SceneCamera) -> String {
    let dof = &cam.dof;
    format!(
        "{}|{}|{:.4}|{}|{:.4}|{}|{:.4}|{:.4}|{:.4}",
        cam.name,
        if dof.on { "on" } else { "off" },
        dof.focus_distance,
        dof.focus_object,
        dof.fstop,
        dof.blades,
        dof.rotation,
        dof.ratio,
        cam.focal_length_mm,
    )
}

#[test]
fn depth_of_field_matches_blender() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("tests/fixtures/DEPTH-OF-FIELD.blend");
    let truth = std::fs::read_to_string(root.join("tests/truth/depth_of_field.txt")).unwrap();
    let scene = model_of(&path);
    let mut got: Vec<String> = scene.cameras.iter().map(row_of).collect();
    got.sort();
    let want: Vec<&str> = truth.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(got.len(), want.len(), "read {} cameras, Blender reports {}", got.len(), want.len());
    for (got, want) in got.iter().zip(want) {
        assert_eq!(got, want);
    }
}

#[test]
fn the_f_stop_becomes_an_aperture_radius_the_way_cycles_does_it() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("tests/fixtures/DEPTH-OF-FIELD.blend");
    let scene = model_of(&path);
    let cam = |name: &str| scene.cameras.iter().find(|c| c.name == name).unwrap().clone();

    // `blender_camera_sync`: (lens in metres) / (2 × f-stop).
    let tracking = cam("TrackingCam");
    let want = (85.0 * 1e-3) / (2.0 * 1.8);
    assert!((tracking.dof.aperture(tracking.focal_length_mm, false) - want).abs() < 1e-9, "{}", tracking.dof.aperture(tracking.focal_length_mm, false));

    // An orthographic camera has no lens to scale by, so Cycles drops the
    // factor rather than treating the lens as 1 mm — a distinction worth a
    // line, since getting it wrong shrinks the aperture by a thousand.
    assert!((tracking.dof.aperture(tracking.focal_length_mm, true) - 1.0 / (2.0 * 1.8)).abs() < 1e-9);

    let distance = cam("DistanceCam");
    assert!((distance.dof.aperture(distance.focal_length_mm, false) - (35.0 * 1e-3) / (2.0 * 5.6)).abs() < 1e-9);

    // Depth of field off is a pinhole whatever the f-stop says, which is
    // what keeps a camera with a wide aperture saved on it from softening
    // every render made through it.
    let pinhole = cam("PinholeCam");
    assert_eq!(pinhole.dof.aperture(pinhole.focal_length_mm, false), 0.0);
}
