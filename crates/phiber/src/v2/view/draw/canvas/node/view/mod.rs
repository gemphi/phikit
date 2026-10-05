//! The node canvas's view: where it is looking and how far in.
//!
//! Ported from Blender 5.2.0 — `View2D` (`DNA_view2d_types.h`), its setup for
//! the node editor in `editors/space_node/space_node.cc:652`, the zoom step in
//! `editors/interface/view2d/view2d_ops.cc:756` and "view all" in
//! `editors/space_node/node_view.cc:78`.
//!
//! ## What this replaces
//!
//! phiber had no view. It FITTED the whole graph to the canvas on every frame,
//! and three separate divergences were recorded against that one absence:
//!
//! * a dragged node's displacement had to be stored in canvas FRACTIONS, because
//!   storing Blender units would feed back into the fit and reframe the graph
//!   under the pointer mid-drag;
//! * a node could not be dragged off the canvas, because with no pan there was
//!   no way back;
//! * the socket hit-test's zoom-compensated slack had nothing to compensate
//!   against, so its scale was pinned at 1.
//!
//! All three were consequences of the same missing thing. With a real view the
//! fit becomes what it is in Blender — the Home key (`NODE_OT_view_all`), one
//! framing among many rather than the only one.
//!
//! ## The representation
//!
//! `cur` is Blender's: the visible rectangle **in view space, Y up**, which is
//! the space `bNode.location` is already in. Keeping it that way means the file's
//! coordinates are never pre-flipped — the single flip happens in
//! [`BNodeView::to_region`], where it belongs, because the canvas's pixels grow
//! downward and Blender's view units grow up.
//!
//! Zoom is not stored. Blender derives it (`ui_view2d_curRect_validate_resize`,
//! and again in the zoom step, with *"better keep in sync!"* written next to the
//! duplicate):
//!
//! ```text
//! zoom = (mask_width + 1) / cur_width
//! ```
//!
//! so it is a function of `cur` and the region, and storing it as well would
//! give two numbers that can disagree. The `+ 1` is Blender's: `mask` is an
//! inclusive pixel rect, so its width in pixels is one more than its extent.

/// `region->v2d.minzoom` / `maxzoom` for the node editor
/// (`space_node.cc:665`). The upper bound is 2.31 and not a round number in
/// Blender either.
pub const MIN_ZOOM: f32 = 0.05;
pub const MAX_ZOOM: f32 = 2.31;

/// `view_zoomfac_default` (`view2d_ops.cc:884`) — one wheel notch.
pub const ZOOM_STEP: f32 = 0.0375;

/// The visible rectangle in view space, Y up: `View2D.cur`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BNodeView {
    pub xmin: f32,
    pub ymin: f32,
    pub xmax: f32,
    pub ymax: f32,
}

impl Default for BNodeView {
    /// `region->v2d.tot`, which `space_node.cc` then copies into `cur`: a
    /// window on the region of view space a new tree is created in, measured in
    /// widget units.
    ///
    /// `-12.8` to `38.4` is not symmetric, and deliberately: a new tree's nodes
    /// are placed at the origin and grow right and up, so the view starts with
    /// more room ahead of them than behind.
    fn default() -> Self {
        Self { xmin: -12.8, ymin: -12.8, xmax: 38.4, ymax: 38.4 }
    }
}

impl BNodeView {
    pub fn width(&self) -> f32 {
        self.xmax - self.xmin
    }

    pub fn height(&self) -> f32 {
        self.ymax - self.ymin
    }

    /// Pixels per view unit — `(mask_width + 1) / cur_width`.
    pub fn zoom(&self, region_width: f32) -> f32 {
        if self.width() <= 0.0 { 1.0 } else { (region_width + 1.0) / self.width() }
    }

    /// A view-space point in canvas pixels.
    ///
    /// The Y flip lives here and nowhere else: view units grow up, canvas pixels
    /// grow down.
    pub fn to_region(&self, at: (f32, f32), region: (f32, f32)) -> (f32, f32) {
        let zoom = self.zoom(region.0);
        ((at.0 - self.xmin) * zoom, (self.ymax - at.1) * zoom)
    }

    /// A canvas pixel back in view space.
    pub fn to_view(&self, at: (f32, f32), region: (f32, f32)) -> (f32, f32) {
        let zoom = self.zoom(region.0);
        (self.xmin + at.0 / zoom, self.ymax - at.1 / zoom)
    }

    /// `VIEW2D_OT_pan`: the pointer carries the view, so the content follows the
    /// pointer and the view moves the other way.
    pub fn pan(&self, by: (f32, f32), region: (f32, f32)) -> Self {
        let zoom = self.zoom(region.0);
        let (dx, dy) = (by.0 / zoom, by.1 / zoom);
        Self {
            xmin: self.xmin - dx,
            xmax: self.xmax - dx,
            // Y is flipped, so a downward drag raises the view.
            ymin: self.ymin + dy,
            ymax: self.ymax + dy,
        }
    }

    /// One zoom step, `fac` positive to zoom in.
    ///
    /// Blender's `view_zoomstep_apply_ex`, with the asymmetry that matters:
    ///
    /// ```text
    /// fac >= 0:  dx = width * fac
    /// fac <  0:  dx = (width / (1 + 2 * fac)) * fac
    /// cur.xmin += dx;  cur.xmax -= dx
    /// ```
    ///
    /// with the comment *"ensuring symmetry so the old zoom level is restored
    /// after zooming back the same amount"*. Zooming in multiplies the width by
    /// `(1 - 2f)`; the negative branch divides by it. The obvious `(1 + 2f)` for
    /// zooming out is NOT the inverse, and a wheel rolled back and forth would
    /// drift further out every round trip.
    ///
    /// `focus` is the cursor in canvas pixels, for Blender's
    /// `zoom_to_mouse_pos`: the point under the pointer stays under it. Blender
    /// only does that *"if zoom fac is inside minzoom/maxzoom"*, which is why
    /// the limit is checked before the shift and not after.
    pub fn zoom_step(&self, fac: f32, focus: Option<(f32, f32)>, region: (f32, f32)) -> Self {
        let step = |size: f32| if fac >= 0.0 { size * fac } else { (size / (1.0 + 2.0 * fac)) * fac };
        let (dx, dy) = (step(self.width()), step(self.height()));
        let zoomed = Self {
            xmin: self.xmin + dx,
            xmax: self.xmax - dx,
            ymin: self.ymin + dy,
            ymax: self.ymax - dy,
        };
        // `V2D_LIMITZOOM`: refuse the step rather than clamping it, so a wheel
        // at the limit does nothing instead of nudging the framing sideways.
        //
        // Unless the view is already outside the limits, in which case a step
        // back TOWARD them has to be allowed — otherwise the wheel is dead in
        // exactly the case you most want it: a graph whose opening framing is
        // tighter than `MAX_ZOOM` could never be zoomed out of. So the test is
        // on the DISTANCE from the range, not on membership of it.
        let how_far = |zoom: f32| (MIN_ZOOM - zoom).max(zoom - MAX_ZOOM).max(0.0);
        let (now, next) = (how_far(self.zoom(region.0)), how_far(zoomed.zoom(region.0)));
        if next > 0.0 && next >= now {
            return *self;
        }
        match focus {
            None => zoomed,
            Some(focus) => {
                // Keep the point under the cursor where it was.
                let was = self.to_view(focus, region);
                let now = zoomed.to_view(focus, region);
                Self {
                    xmin: zoomed.xmin + (was.0 - now.0),
                    xmax: zoomed.xmax + (was.0 - now.0),
                    ymin: zoomed.ymin + (was.1 - now.1),
                    ymax: zoomed.ymax + (was.1 - now.1),
                }
            }
        }
    }

    /// Whether this view is within the editor's zoom limits for `region`.
    pub fn allowed(&self, region: (f32, f32)) -> bool {
        (MIN_ZOOM..=MAX_ZOOM).contains(&self.zoom(region.0))
    }

}

/// Fractions of the region — how a card is placed before the canvas is measured.
pub mod fraction;
/// Framing a graph — the Home key, and what a node's rectangle is.
pub mod frame;
/// `NODE_OT_view_all` — the Home key, as an operator.
pub mod op;
/// The view as the editor keeps it, and the framing it opens on.
pub mod store;

pub use frame::{graph_bounds, node_bounds, view_all};
pub use store::view_of;

#[cfg(test)]
mod tests;
