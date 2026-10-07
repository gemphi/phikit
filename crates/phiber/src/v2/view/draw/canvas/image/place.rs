//! The Image and UV editors' geometry: where the picture lands in the
//! window at the zoom in force, and where the UV editor's 0..1 square does.
//! One concern, one file — and the reason `mod.rs` no longer carries it.
//!
//! Blender ref: `image_view_all` and `image_main_region_set_view2d`.

use super::{BImageView, UV_TILE, ZOOM_MAX, ZOOM_MIN};
use gpui::{Bounds, Pixels, point, px, size};

impl BImageView {
    /// `image_view_all`: a picture that fits shows at 1:1; a larger one at
    /// the largest power-of-two fraction that fits the window.
    pub fn view_all(w: f32, h: f32, winx: f32, winy: f32) -> f32 {
        if w <= winx && h <= winy {
            return 1.0;
        }
        let fit = (winx / w.max(1.0)).min(winy / h.max(1.0));
        1.0 / (1.0 / fit).log2().ceil().exp2()
    }

    /// The zoom in effect for a `w × h` image in a `winx × winy` window.
    pub fn zoom_for(&self, w: f32, h: f32, winx: f32, winy: f32) -> f32 {
        self.zoom.unwrap_or_else(|| Self::view_all(w, h, winx, winy)).clamp(ZOOM_MIN, ZOOM_MAX)
    }

    /// The UV editor's 0..1 square in the window `at`: a `UV_TILE` image at
    /// the view's zoom (1 when fitting), centred and moved by the offset.
    pub fn uv_square(&self, at: Bounds<Pixels>) -> Bounds<Pixels> {
        Self { status: None, zoom: Some(self.zoom.unwrap_or(1.0)), ..self.clone() }.rect(UV_TILE, UV_TILE, at)
    }

    /// `image_main_region_set_view2d`: the image centred, then moved by the
    /// offset, in the window `at`.
    pub fn rect(&self, w: f32, h: f32, at: Bounds<Pixels>) -> Bounds<Pixels> {
        let (winx, winy) = (f32::from(at.size.width), f32::from(at.size.height));
        let zoom = self.zoom_for(w, h, winx, winy);
        let x = (winx - zoom * w) / 2.0 - zoom * self.pan.0;
        let y = (winy - zoom * h) / 2.0 + zoom * self.pan.1;
        Bounds::new(point(at.origin.x + px(x.round()), at.origin.y + px(y.round())), size(px((zoom * w).round()), px((zoom * h).round())))
    }
}
