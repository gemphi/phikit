//! The orbit-model view that looks through the scene camera: the camera's
//! eye and lens turned into the yaw, pitch, pivot, field of view and zoom
//! the canvas projects with (`scene/camera.rs`), and the frame the render
//! would fill (`BCameraFrame`). `BKE_camera_params_from_object` supplies the
//! lens side; `SceneView::angles` (phiphx) the orientation.

use super::{BCameraFrame, BSensorFit};
use crate::phiphx::scene::{SceneCamera, SceneView};
use crate::v2::scene::camera::zoom_for;
use crate::v2::scene::{pan_x_key, pan_y_key, perspective_key, pitch_key, yaw_key, zoom_key};
use crate::v2::state::BPath;
use crate::v2::view::prim::BAction;

/// Where the pivot sits ahead of the eye, in Blender units. Any distance
/// gives the same picture — the eye is the camera — so it is the scene's
/// own scale.
const PIVOT_AHEAD: f32 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BCameraView {
    pub yaw: f32,
    pub pitch: f32,
    /// Blender world coordinates.
    pub pivot: [f32; 3],
    pub perspective: bool,
    /// Visible width per unit of eye distance across the whole region.
    pub fov: f32,
    /// The canvas zoom showing the region's width at the pivot.
    pub zoom: f32,
    pub frame: BCameraFrame,
}

impl BCameraView {
    /// The view through `cam` in a region `region` px large, for a render of
    /// `render` px; `camzoom` as saved, or Blender's frame-to-fit zoom.
    pub fn of(cam: &SceneCamera, region: (f32, f32), render: (f32, f32), camzoom: Option<f32>) -> Self {
        let back = cam.matrix_world[2];
        let length = (back[0] * back[0] + back[1] * back[1] + back[2] * back[2]).sqrt().max(1e-9);
        let back = [back[0] / length, back[1] / length, back[2] / length];
        let eye = cam.matrix_world[3];
        let pivot = [eye[0] - back[0] * PIVOT_AHEAD, eye[1] - back[1] * PIVOT_AHEAD, eye[2] - back[2] * PIVOT_AHEAD];
        let (yaw, pitch) = SceneView::angles(back);
        let perspective = cam.kind != "ORTHO";
        // The frame's width in Blender units at the pivot: the sensor fits
        // the render's larger side (`BKE_camera_sensor_fit`, AUTO).
        let aspect = render.0 / render.1.max(1.0);
        let across = match BSensorFit::auto(render.0, render.1) {
            BSensorFit::Horizontal => 1.0,
            BSensorFit::Vertical => aspect,
        };
        let frame_width = if perspective {
            PIVOT_AHEAD * cam.sensor_width.max(1e-3) / cam.focal_length_mm.max(1e-3) * across
        } else {
            cam.ortho_scale.max(1e-3) * across
        };
        let frame = BCameraFrame::of(region, render, camzoom.unwrap_or_else(|| BCameraFrame::fitted_camzoom(region, render)));
        // The region shows the frame and the passepartout around it.
        let region_width = frame_width * region.0 / frame.width().max(1.0);
        Self { yaw, pitch, pivot, perspective, fov: region_width / PIVOT_AHEAD, zoom: zoom_for(region_width), frame }
    }

    /// Orbiting out of the camera view (`ED_view3d_persp_switch_from_camera`):
    /// the orbit takes the camera's view over — yaw, pitch, pivot, zoom and
    /// projection — so the scene stays where it was, and the area stops
    /// looking through the camera (`key`, its `View3dKey::Camera`).
    pub fn leave(&self, key: &BPath) -> Vec<BAction> {
        let pivot = self.pivot;
        vec![
            BAction::Set(yaw_key(), format!("{:.3}", self.yaw)),
            BAction::Set(pitch_key(), format!("{:.3}", self.pitch.clamp(-90.0, 90.0))),
            BAction::Set(zoom_key(), format!("{:.4}", self.zoom)),
            BAction::Set(perspective_key(), self.perspective.to_string()),
            BAction::Set(crate::v2::scene::pivot_key(), format!("{:.5},{:.5},{:.5}", pivot[0], pivot[1], pivot[2])),
            BAction::Set(crate::v2::scene::lens_key(), format!("{:.3}", crate::v2::scene::camera::lens_of_fov(self.fov))),
            BAction::Set(pan_x_key(), "0".to_string()),
            BAction::Set(pan_y_key(), "0".to_string()),
            BAction::Set(key.clone(), "false".to_string()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn camera(kind: &str) -> SceneCamera {
        SceneCamera {
            name: "Camera".into(),
            location: [0.0, -10.0, 0.0],
            // Looking along +Y from y = -10: the back axis is -Y.
            matrix_world: [[1.0, 0.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, -1.0, 0.0, 0.0], [0.0, -10.0, 0.0, 1.0]],
            kind: kind.into(),
            focal_length_mm: 50.0,
            sensor_width: 36.0,
            ortho_scale: 6.0,
            clip_start: 0.1,
            clip_end: 100.0,
            display_size: 1.0,
            dof: Default::default(),
        }
    }

    #[test]
    fn the_eye_is_the_camera_and_the_frame_is_its_lens() {
        let view = BCameraView::of(&camera("PERSP"), (1000.0, 600.0), (1920.0, 1080.0), Some(0.0));
        assert!((view.yaw - 0.0).abs() < 1e-3 && view.pitch.abs() < 1e-3, "front view: {view:?}");
        assert_eq!(view.pivot, [0.0, 0.0, 0.0], "ten units ahead of the eye");
        // The frame is half the region wide at camzoom 0, so the region shows
        // twice the sensor's width at the pivot: 2 × 10 × 36 / 50.
        assert!((view.fov * PIVOT_AHEAD - 2.0 * 10.0 * 36.0 / 50.0).abs() < 1e-3, "{view:?}");
        assert!(view.perspective);
        assert!((view.frame.width() - 500.0).abs() < 1e-3);
    }

    #[test]
    fn an_orthographic_camera_shows_its_ortho_scale_across_the_frame() {
        let view = BCameraView::of(&camera("ORTHO"), (1000.0, 600.0), (1920.0, 1080.0), Some(0.0));
        assert!(!view.perspective);
        assert!((view.fov * PIVOT_AHEAD - 12.0).abs() < 1e-3, "twice 6 across the region");
    }

    #[test]
    fn a_portrait_render_fits_the_sensor_to_its_height() {
        let view = BCameraView::of(&camera("PERSP"), (1000.0, 1000.0), (1080.0, 1920.0), Some(0.0));
        let frame_world = view.fov * PIVOT_AHEAD * view.frame.width() / 1000.0;
        assert!((frame_world - 10.0 * 36.0 / 50.0 * 1080.0 / 1920.0).abs() < 1e-3, "{frame_world}");
    }
}
