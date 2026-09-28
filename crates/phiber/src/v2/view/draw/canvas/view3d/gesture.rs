//! What a viewport gesture paints over the canvas — the box (B), the circle
//! brush (C), the lasso (Ctrl right-drag) — read once per frame and drawn
//! in one pair of inks: the canvas shade under the overlay ink, as
//! `wm_gesture_draw` draws each.

use crate::v2::scene::boxsel;
use crate::v2::scene::circlesel::BCircle;
use crate::v2::scene::lasso::BLasso;
use crate::v2::scene::pick::BPickView;
use crate::v2::scene::xform::BTransformModal;
use crate::v2::state::BState;
use gpui::{Bounds, Hsla, Pixels, Window};

/// The lasso's line width and dash length, in pixels.
pub(super) const LINE: f32 = 1.0;
pub(super) const DASH: f32 = 4.0;

pub(super) struct BGestureInk {
    rect: Option<(f32, f32, f32, f32)>,
    circle: Option<BCircle>,
    lasso: Option<BLasso>,
    /// R / S: the pivot (world) and the pointer (canvas), in `view`.
    guide: Option<(glam::Vec3, (f32, f32))>,
    view: BPickView,
    back: Hsla,
    ink: Hsla,
}

impl BGestureInk {
    pub fn now(s: &BState, back: Hsla, ink: Hsla, view: BPickView) -> Self {
        let guide = BTransformModal::now(s).and_then(|modal| modal.guide());
        Self { rect: boxsel::dragged(s), circle: BCircle::now(s), lasso: BLasso::now(s), guide, view, back, ink }
    }

    pub fn paint(&self, bounds: Bounds<Pixels>, window: &mut Window) {
        if let Some(rect) = self.rect {
            super::boxsel::paint(bounds, rect, self.back, self.ink, window);
        }
        if let Some(circle) = self.circle {
            super::circlesel::paint(bounds, circle, self.back, self.ink, window);
        }
        if let Some(lasso) = &self.lasso {
            super::lasso::paint(bounds, lasso, self.back, self.ink, window);
        }
        // `transform_draw_cursor`: a dashed line from the pivot to the pointer.
        if let Some((pivot, at)) = self.guide {
            let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
            let view = BPickView { aspect: w / h.max(1.0), ..self.view };
            if let Some((su, sv)) = view.project(pivot) {
                let from = ((su + 0.5) * w, (sv + 0.5) * h);
                super::lasso::stroke(bounds, &[from, at], false, self.back, self.ink, window);
            }
        }
    }
}
