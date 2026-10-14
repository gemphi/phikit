//! `still.rs`'s tests: a still rendered in bands is the still rendered
//! whole, and a Material Preview still built by hand is lit by the
//! studio image all the same.

use super::still::{still, still_stepwise, with_studio};
use crate::v2::scene::{BObject3D, BShading, BShape};

/// The bands of a 40 × 40 frame are the frame drawn whole, pixel for
/// pixel — outlined in Solid, and anti-aliased by the single pass
/// (`render_aa` 1), which leaves SMAA on the finished frame, so each
/// band is drawn as far over as SMAA reads (`Tracer::seam_rows`) — so
/// a Render larger than the device binds comes out as one image with
/// no seam and no blank band.
#[test]
fn a_still_rendered_in_bands_is_the_still_rendered_whole() {
    if phiray::Ray::gpu().is_none() {
        return;
    }
    let objects = vec![BObject3D::Primitive { name: "Cube".into(), shape: BShape::Cube, x: 0.0, y: 0.0 }, BObject3D::Primitive { name: "Ball".into(), shape: BShape::Sphere, x: -2.0, y: 0.0 }];
    let look = super::world::BRayLook { sky: gpui::black(), grid: gpui::white(), grid_hide: 0, world: None, lights: Vec::new(), wire: None, view: [0.0, 0.0, 1.0, 0.0], environment: None, set: Default::default(), studio: false };
    let setup = crate::v2::scene::camera::setup_from_state(&crate::v2::state::BState::default());
    let cam = phiray::Camera { fx: [1.0, 0.5, 0.7, 1.0], ..super::scene::camera(20.0, 30.0, (0.0, 0.0), 1.0, setup, 1.0, &look) };
    let (whole, _) = still(&objects, super::BStillLook::new(BShading::Solid, gpui::white()), cam, 40, 40, 1, false).expect("a GPU");
    super::bands::force(Some(40 * 16));
    let mut banded = None;
    still_stepwise(&objects, super::BStillLook::new(BShading::Solid, gpui::white()), cam, 40, 40, 1, false, Default::default(), |_, at| {
        if banded.is_none() && at.settled() { banded = at.image(); }
        true
    });
    super::bands::force(None);
    let banded = banded.expect("the finished still");
    assert_eq!(banded.rgba.len(), whole.len());
    let rows: Vec<usize> = (0..40).filter(|y| banded.rgba[y * 160..y * 160 + 160] != whole[y * 160..y * 160 + 160]).collect();
    assert_eq!(banded.rgba, whole, "the bands differ from the frame drawn whole in rows {rows:?}");
    assert!(banded.depth.iter().all(|d| *d > 0.0), "every band's depth was laid in");
}

/// A Material Preview still asked for with a camera built by hand
/// (the examples) is lit by the studio image all the same — `lit`'s
/// words, not a dark mirror — and a camera that has them, or another
/// shading, is left alone.
#[test]
fn a_hand_built_material_preview_camera_is_lit_by_the_studio() {
    let cam = with_studio(phiray::Camera::default(), BShading::Material);
    let expected = super::studio::environment().is_some();
    assert_eq!(cam.preview != [0; 4], expected, "{:?}", cam.preview);
    assert_eq!((cam.env_hidden == 1, cam.envmap > 0), (expected, expected));
    let own = phiray::Camera { preview: [9, 9, 9, 9], ..Default::default() };
    assert_eq!(with_studio(own, BShading::Material).preview, [9, 9, 9, 9]);
    assert_eq!(with_studio(phiray::Camera::default(), BShading::Solid).preview, [0; 4]);
}

