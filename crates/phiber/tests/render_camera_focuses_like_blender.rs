//! The camera's depth of field reaches the tracer, and a camera that focuses
//! on an object focuses where Blender says.
//!
//! The fixture is phiphx's `DEPTH-OF-FIELD.blend` (`truth/blender_dof.py`):
//! `TrackingCam` sits at y = −3 looking down +Y and focuses on an Empty at
//! (4, 7, 0) — deliberately *off* the camera axis. The focus distance is the
//! projection onto that axis, 10, and not the straight-line distance, about
//! 10.77. A target on the axis could not tell the two apart, which is why
//! this one is not on it.

use phiber::phiphx::scene::SceneModel;
use phiber::v2::scene::{camera_of, render_job::focus_on};
use std::path::PathBuf;

fn model() -> Option<SceneModel> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/DEPTH-OF-FIELD.blend");
    let bytes = std::fs::read(&path).ok()?;
    let blend = phiber::phiphx::blend::BlendFile::parse_bytes(&bytes).ok()?.extract_scene().ok()?;
    Some(SceneModel::from_blend_owned(blend, &path.to_string_lossy()))
}

#[test]
fn the_aperture_and_focus_reach_the_tracer() {
    let Some(model) = model() else { return };
    let cam = model.cameras.iter().find(|c| c.name == "TrackingCam").unwrap();
    let ray = camera_of(cam, 1920, 1080);
    // 85 mm at f/1.8 — the radius Cycles computes, in metres.
    assert!((ray.aperture - (85.0 * 1e-3) / (2.0 * 1.8)).abs() < 1e-6, "{}", ray.aperture);
    assert_eq!(ray.blades, 6);
    assert!((ray.blade_spin - 15.0_f32.to_radians()).abs() < 1e-4);
    assert!((ray.aperture_ratio - 1.6).abs() < 1e-6);

    // A camera with depth of field switched off is a pinhole, whatever
    // f-stop is saved on it — so the tracer skips the lens sampling entirely.
    let off = model.cameras.iter().find(|c| c.name == "PinholeCam").unwrap();
    assert_eq!(camera_of(off, 1920, 1080).aperture, 0.0);
}

#[test]
fn focusing_on_an_object_measures_along_the_camera_axis() {
    let Some(model) = model() else { return };
    let tracking = model.cameras.iter().find(|c| c.name == "TrackingCam").unwrap();
    let got = focus_on(&model, tracking).expect("TrackingCam focuses on an object");
    assert!((got - 10.0).abs() < 1e-3, "{got}");
    // Spelt out, because this is the whole point: the straight-line distance
    // is a different number, and reading that one would be wrong.
    let straight = (16.0f32 + 100.0).sqrt();
    assert!((got - straight).abs() > 0.5, "the focus distance looks like the straight-line one: {got}");

    // A camera that focuses by distance names no object, so its own
    // `focus_distance` stands — `focus_on` says so by returning nothing
    // rather than by guessing a number.
    let by_distance = model.cameras.iter().find(|c| c.name == "DistanceCam").unwrap();
    assert_eq!(focus_on(&model, by_distance), None);
    assert!((camera_of(by_distance, 1920, 1080).focus - 4.25).abs() < 1e-4);
}
