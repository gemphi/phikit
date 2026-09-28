//! Looking through the camera: the view the canvas projects with, and the
//! frame and passepartout drawn over it — `view3d_draw.cc::drawviewborder`.
//! The passepartout is `TH_CAMERA_PASSEPARTOUT` (black in Blender's theme,
//! here the ladder's shade 1) at the camera's `passepartalpha`; the frame is
//! a solid line in `TH_BACK` — the canvas shade — with a dashed
//! `TH_VIEW_OVERLAY` line over it.

use crate::v2::scene::camera::{BViewSetup, last_size, setup_from_state, view_pivot};
use crate::v2::scene::{view_pan, view_pitch, view_yaw, view_zoom};
use crate::v2::scene::camera_view::set::BCameraView;
use crate::v2::scene::{MESH_SCALE, source::with_loaded};
use crate::v2::state::BState;
use gpui::{BorderStyle, Bounds, Hsla, Pixels, Window, fill, outline, point, px, size};

/// The camera this viewport looks through, when it does: the scene's active
/// camera framed in the canvas as last painted, at the frame-to-fit zoom.
pub(super) fn through(camera: bool) -> Option<BCameraView> {
    if !camera {
        return None;
    }
    with_loaded(|model| {
        let model = model?;
        let cam = crate::v2::scene::motion::active_camera_at(model, crate::v2::scene::motion::scoped_frame())?;
        let cam = &cam;
        let render = (model.render.resolution_x.max(1) as f32, model.render.resolution_y.max(1) as f32);
        // The zoom the file saved for its camera view (`RegionView3D.camzoom`)
        // — the largest 3D view's, which is the one phiphx reads — else
        // Blender's frame-to-fit zoom.
        let camzoom = model.ui.view.as_ref().filter(|view| view.camera_view).map(|view| view.camzoom);
        Some(BCameraView::of(cam, last_size(), render, camzoom))
    })
}

/// The orbit-model parameters a frame projects with: the saved orbit, or
/// the camera's when looking through it.
pub(super) struct BViewParams {
    pub yaw: f32,
    pub pitch: f32,
    pub pan: (f32, f32),
    pub zoom: f32,
    pub pivot: [f32; 3],
    pub setup: BViewSetup,
}

impl BViewParams {
    pub fn resolve(state: &BState, through: Option<&BCameraView>) -> Self {
        let orbit = setup_from_state(state);
        match through {
            Some(view) => Self {
                yaw: view.yaw,
                pitch: view.pitch,
                pan: (0.0, 0.0),
                zoom: view.zoom,
                pivot: view.pivot,
                setup: BViewSetup {
                    perspective: view.perspective,
                    pivot: (view.pivot[0] * MESH_SCALE, view.pivot[2] * MESH_SCALE, view.pivot[1] * MESH_SCALE),
                    aspect: orbit.aspect,
                    fov: view.fov,
                },
            },
            None => Self {
                yaw: view_yaw(state),
                pitch: view_pitch(state),
                pan: view_pan(state),
                zoom: view_zoom(state),
                pivot: view_pivot(state),
                setup: orbit,
            },
        }
    }
}

/// The colours the frame is drawn in.
#[derive(Clone, Copy)]
pub(super) struct BFramePalette {
    /// `TH_CAMERA_PASSEPARTOUT` at the camera's alpha.
    pub passepartout: Hsla,
    /// `TH_BACK`: the solid line under the dashes.
    pub back: Hsla,
    /// `TH_VIEW_OVERLAY`: the dashed frame.
    pub overlay: Hsla,
}

/// `drawviewborder`: the passepartout in four rectangles around the frame,
/// then the frame's solid and dashed outlines.
pub(super) fn paint(bounds: Bounds<Pixels>, view: &BCameraView, palette: BFramePalette, window: &mut Window) {
    let frame = view.frame.snapped();
    let (width, height) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
    let origin = bounds.origin;
    let rect = |left: f32, top: f32, right: f32, bottom: f32| {
        Bounds::new(point(origin.x + px(left), origin.y + px(top)), size(px((right - left).max(0.0)), px((bottom - top).max(0.0))))
    };
    if frame.left > 0.0 {
        window.paint_quad(fill(rect(0.0, 0.0, frame.left, height), palette.passepartout));
    }
    if frame.right < width {
        window.paint_quad(fill(rect(frame.right, 0.0, width, height), palette.passepartout));
    }
    if frame.top > 0.0 {
        window.paint_quad(fill(rect(frame.left, 0.0, frame.right, frame.top), palette.passepartout));
    }
    if frame.bottom < height {
        window.paint_quad(fill(rect(frame.left, frame.bottom, frame.right, height), palette.passepartout));
    }
    let border = rect(frame.left, frame.top, frame.right, frame.bottom);
    window.paint_quad(outline(border, palette.back, BorderStyle::Solid));
    window.paint_quad(outline(border, palette.overlay, BorderStyle::Dashed));
}
