//! Where the last traced image belongs on the canvas while a newer view is
//! being traced.
//!
//! A zoom, a pan or a window resize changes the view at once, but the
//! tracer's new image lands a frame or more later. Drawn over the canvas as
//! it was, the old image showed the meshes at their old size, and during a
//! window drag stretched them out of shape, until the new image arrived. The
//! outlines, bones and other overlays had already moved, so they and the
//! meshes under them disagreed.
//!
//! Every view posted to the tracer is noted with what decides where things
//! land on screen (`scene::camera::view_project`): the zoom, the pan and
//! the traced image's shape. On the canvas an object is `SCREEN · zoom ·
//! width` pixels a unit across, centred on `0.5 + pan` of the width — so
//! an image traced for one view maps onto another by one scale and one
//! shift. That is exact for an orthographic view and for the depth of the
//! pivot in a perspective one, and close everywhere else, and the new image
//! replaces it as soon as it lands. An orbit (another yaw, pitch or pivot)
//! cannot be reached by a scale, so the old image is drawn as it was framed.

use gpui::{Bounds, Pixels, point, px, size};

/// What decides where an image's pixels land.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct BShotView {
    pub zoom: f32,
    pub pan: (f32, f32),
    /// Width over height of the traced image.
    pub aspect: f32,
    /// Yaw, pitch and the orbit's pivot: the view's direction.
    pub orbit: (f32, f32, (f32, f32, f32), bool),
    /// The part of the frame the image is: `(x, y, w, h)` as fractions of
    /// the frame, y down — the viewport's own render region — or the whole.
    pub part: Option<(f32, f32, f32, f32)>,
}

impl BShotView {
    /// The rectangle within `bounds` that an image traced for `self` covers
    /// in the view `now`, drawn on a canvas `bounds` wide.
    pub(super) fn place(&self, now: &Self, bounds: Bounds<Pixels>) -> Bounds<Pixels> {
        let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
        let zoom = if self.orbit == now.orbit && self.zoom > 0.0 { now.zoom / self.zoom } else { 1.0 };
        let pan = if self.orbit == now.orbit { (now.pan, self.pan) } else { ((0.0, 0.0), (0.0, 0.0)) };
        // The image's width on the canvas, so that a unit is as many pixels
        // across in it as in the view now; its height keeps its own shape.
        let wide = w * zoom;
        let tall = wide / self.aspect.max(1e-6);
        let left = 0.5 * w + pan.0.0 * w - 0.5 * wide - pan.1.0 * wide;
        let top = 0.5 * h + pan.0.1 * w - 0.5 * tall - pan.1.1 * wide;
        let (px0, py0, pw, ph) = self.part.unwrap_or((0.0, 0.0, 1.0, 1.0));
        Bounds::new(point(bounds.origin.x + px(left + px0 * wide), bounds.origin.y + px(top + py0 * tall)), size(px(pw * wide), px(ph * tall)))
    }
}

/// The views posted lately, by key — enough to cover the images in flight.
#[derive(Default)]
pub(super) struct BShotLog(Vec<(u64, BShotView)>);

impl BShotLog {
    const KEEP: usize = 16;

    pub(super) fn note(&mut self, key: u64, view: BShotView) {
        self.0.retain(|(k, _)| *k != key);
        self.0.push((key, view));
        if self.0.len() > Self::KEEP {
            self.0.remove(0);
        }
    }

    pub(super) fn get(&self, key: u64) -> Option<BShotView> {
        self.0.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(zoom: f32, pan: (f32, f32), aspect: f32) -> BShotView {
        BShotView { zoom, pan, aspect, orbit: (30.0, 20.0, (0.0, 0.0, 0.0), true), part: None }
    }

    fn rect(b: Bounds<Pixels>) -> [f32; 4] {
        [f32::from(b.origin.x), f32::from(b.origin.y), f32::from(b.size.width), f32::from(b.size.height)]
    }

    fn near(a: [f32; 4], b: [f32; 4]) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-3)
    }

    const WIDE: f32 = 800.0;
    const TALL: f32 = 600.0;
    const NONE: f32 = 0.0;
    const CANVAS: Bounds<Pixels> = Bounds { origin: gpui::Point { x: px(NONE), y: px(NONE) }, size: gpui::Size { width: px(WIDE), height: px(TALL) } };

    #[test]
    fn the_same_view_fills_the_canvas() {
        let v = view(1.0, (0.0, 0.0), 800.0 / 600.0);
        assert!(near(rect(v.place(&v, CANVAS)), [0.0, 0.0, 800.0, 600.0]));
    }

    #[test]
    fn a_zoom_scales_the_image_about_the_pan_centre() {
        // Twice the zoom: twice as wide, still centred.
        let (old, now) = (view(1.0, (0.0, 0.0), 800.0 / 600.0), view(2.0, (0.0, 0.0), 800.0 / 600.0));
        assert!(near(rect(old.place(&now, CANVAS)), [-400.0, -300.0, 1600.0, 1200.0]));
        // A point a quarter of the width right of centre stays under itself
        // once both its views are drawn: at 0.25 · zoom in each.
        let (old, now) = (view(1.0, (0.1, 0.0), 800.0 / 600.0), view(1.0, (0.2, 0.0), 800.0 / 600.0));
        assert!(near(rect(old.place(&now, CANVAS)), [80.0, 0.0, 800.0, 600.0]), "a pan shifts it by the pan");
    }

    #[test]
    fn a_wider_window_keeps_the_meshes_in_shape() {
        // Traced 800 × 600, the window now 1000 × 600: the image is drawn
        // 1000 × 750, centred — not stretched to 1000 × 600.
        let old = view(1.0, (0.0, 0.0), 800.0 / 600.0);
        let window = 1000.0;
        let canvas = Bounds { size: gpui::Size { width: px(window), height: px(TALL) }, ..CANVAS };
        assert!(near(rect(old.place(&view(1.0, (0.0, 0.0), 1000.0 / 600.0), canvas)), [0.0, -75.0, 1000.0, 750.0]));
    }

    #[test]
    fn a_part_of_the_frame_lands_where_the_frame_puts_it() {
        // The right half's lower half, traced alone, drawn in that quarter —
        // and twice as large with the zoom, about the same centre.
        let v = BShotView { part: Some((0.5, 0.5, 0.5, 0.5)), ..view(1.0, (0.0, 0.0), 800.0 / 600.0) };
        assert!(near(rect(v.place(&v, CANVAS)), [400.0, 300.0, 400.0, 300.0]));
        assert!(near(rect(v.place(&view(2.0, (0.0, 0.0), 800.0 / 600.0), CANVAS)), [400.0, 300.0, 800.0, 600.0]));
    }

    #[test]
    fn an_orbit_is_not_scaled() {
        let old = view(1.0, (0.0, 0.0), 800.0 / 600.0);
        let turned = BShotView { orbit: (40.0, 20.0, (0.0, 0.0, 0.0), true), zoom: 3.0, ..old };
        assert!(near(rect(old.place(&turned, CANVAS)), [0.0, 0.0, 800.0, 600.0]));
    }
}
