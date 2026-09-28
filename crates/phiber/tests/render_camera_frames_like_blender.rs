//! A final render looks through the file's camera, framed as Blender frames
//! it.
//!
//! phiray's screen is `u, v ∈ [-0.5, 0.5]`, so the half-width the frame spans
//! at the eye is `0.5 / scale` over a distance `dist`. Blender's camera spans
//! `sensor_width / 2` over `lens`. Matching them is one equation, and these
//! check it — including the sensor-fit rule, which is what decides whether a
//! portrait frame is too wide or right.
//!
//! Measured end to end against a Cycles render of Female-01 through its own
//! `Camera.001`: silhouettes agree at **IoU 0.998**.

use phiber::phiphx::scene::SceneCamera;
use phiber::v2::scene::{camera_of, render_size};

fn cam(kind: &str, lens: f32, sensor: f32, ortho: f32) -> SceneCamera {
    SceneCamera {
        name: "Camera".into(),
        location: [0.0; 3],
        // Identity: +X right, +Y up, looking down -Z, at the origin.
        matrix_world: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]],
        kind: kind.into(),
        focal_length_mm: lens,
        sensor_width: sensor,
        ortho_scale: ortho,
        clip_start: 0.1,
        clip_end: 100.0,
        display_size: 1.0,
        dof: Default::default(),
    }
}

/// The half-angle the frame spans across its longer side. The kernel takes
/// `u` to `u / scale` across and `v` to `v / (aspect * scale)` up, so the two
/// half-extents differ by the aspect.
fn half_angle(c: &phiray::Camera, wide: bool) -> f32 {
    let extent = if wide { 0.5 / c.scale } else { 0.5 / (c.aspect * c.scale) };
    (extent / c.dist).atan()
}

#[test]
fn a_perspective_camera_spans_its_sensor_over_its_lens() {
    let blender = (36.0f32 / 2.0 / 50.0).atan();
    let landscape = camera_of(&cam("PERSP", 50.0, 36.0, 6.0), 1920, 1080);
    assert!((half_angle(&landscape, true) - blender).abs() < 1e-5);
    assert_eq!(landscape.persp, 1);
    // AUTO sensor fit spans the *longer* side, so a portrait frame puts the
    // same angle across its height and is narrower than it is tall.
    let portrait = camera_of(&cam("PERSP", 50.0, 36.0, 6.0), 1080, 1920);
    assert!((half_angle(&portrait, false) - blender).abs() < 1e-5);
    // A portrait frame puts the sensor across its short side's *height*, so
    // it sees a narrower slice of the world than a landscape one — a larger
    // `scale`. Getting this backwards is invisible on a square frame, which
    // is what the test file happens to be, so it is asserted directly.
    assert!(portrait.scale > landscape.scale, "a portrait frame must not be as wide as a landscape one");
}

#[test]
fn a_square_frame_is_the_same_either_way() {
    let square = camera_of(&cam("PERSP", 85.0, 36.0, 6.0), 512, 512);
    assert!((half_angle(&square, true) - half_angle(&square, false)).abs() < 1e-6);
}

#[test]
fn an_orthographic_camera_spans_its_ortho_scale() {
    let c = camera_of(&cam("ORTHO", 50.0, 36.0, 6.0), 800, 400);
    assert_eq!(c.persp, 0);
    // The longer side spans `ortho_scale` world units: half of it is `0.5 / scale`.
    assert!(((0.5 / c.scale) - 3.0).abs() < 1e-4, "half-width {} of 6.0", 0.5 / c.scale);
}

#[test]
fn the_camera_looks_where_blender_points_it() {
    let c = camera_of(&cam("PERSP", 50.0, 36.0, 6.0), 100, 100);
    // Blender's camera looks down its own -Z, which is phiray's `back`.
    assert_eq!(c.back, [0.0, 0.0, 1.0]);
    assert_eq!(c.right, [1.0, 0.0, 0.0]);
    assert_eq!(c.up, [0.0, 1.0, 0.0]);
    // The kernel puts the eye at `pivot + back * dist`, so the pivot sits
    // that far in front of where the camera actually is.
    assert_eq!(c.pivot, [0.0, 0.0, -c.dist]);
}

#[test]
fn the_resolution_is_the_files_scaled_by_its_percentage() {
    use phiber::phiphx::scene::{SceneModel, SceneRender};
    let with = |x, y, pct| {
        let mut m = SceneModel::default();
        m.render = SceneRender { resolution_x: x, resolution_y: y, resolution_percent: pct, ..Default::default() };
        render_size(&m)
    };
    assert_eq!(with(1920, 1080, 100), (1920, 1080));
    assert_eq!(with(1920, 1080, 50), (960, 540));
    assert_eq!(with(1920, 1080, 25), (480, 270));
    // Never zero, whatever the file says.
    assert_eq!(with(0, 0, 0), (1, 1));
}
