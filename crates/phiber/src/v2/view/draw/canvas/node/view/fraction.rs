//! Fractions of the region: where a card goes before the canvas is measured.
//!
//! Split from `mod.rs`, which converts view space to PIXELS. These three do the
//! same work in fractions of the region, and they are together because they
//! share one fact — that the region's size cancels out of a fraction — and one
//! precondition, [`BNodeView::fitted_to`], without which they disagree with the
//! pixel path on the y axis.
//!
//! The reason any of it exists: GPUI builds the element tree one phase before it
//! measures anything, so a node's card has to be positioned and sized before the
//! canvas's pixel size is known. A fraction is the only answer available then.

use super::BNodeView;

impl BNodeView {
    /// A view-space point as a FRACTION of the region — 0..1 across and down.
    ///
    /// This is [`Self::to_region`] divided by the region's size, and the region
    /// cancels out of it:
    ///
    /// ```text
    /// to_region(x).0 / w = (x - xmin) * zoom / w
    ///                    = (x - xmin) * (w + 1) / (cur_width * w)
    ///                    ≈ (x - xmin) / cur_width
    /// ```
    ///
    /// which is the whole reason it exists. A node's card is a div, placed with
    /// `gpui::relative` while the element tree is built — and the canvas's pixel
    /// size is not known until the paint pass, one phase later. So a fraction is
    /// the only position a card can be given, and this says a fraction needs the
    /// view and nothing else. That is what lets the editor pan and zoom at all:
    /// the framing can change every frame without the cards needing a
    /// measurement they cannot have yet.
    ///
    /// The `+ 1` goes with the region. It is one pixel of Blender's inclusive
    /// `mask` rect, and a fraction has no pixels to keep it in.
    pub fn to_fraction(&self, at: (f32, f32)) -> (f32, f32) {
        let share = |numer: f32, denom: f32| if denom.abs() < 1e-6 { 0.0 } else { numer / denom };
        // The Y flip is the same one `to_region` makes, for the same reason.
        (share(at.0 - self.xmin, self.width()), share(self.ymax - at.1, self.height()))
    }

    /// A fraction of the region back in view space — the inverse of
    /// [`Self::to_fraction`], and needing the region just as little.
    pub fn at_fraction(&self, at: (f32, f32)) -> (f32, f32) {
        (self.xmin + at.0 * self.width(), self.ymax - at.1 * self.height())
    }

    /// `VIEW2D_OT_pan` given as a fraction of the region rather than in pixels.
    ///
    /// The same cancellation as [`Self::to_fraction`]: a pixel pan divides by
    /// the zoom to reach view units, and `pixels / zoom` with
    /// `zoom = w / cur_width` is `(pixels / w) * cur_width` — the fraction times
    /// the view's own width. So a gesture that knows how far ACROSS the canvas
    /// the pointer went does not need to know how wide the canvas is.
    pub fn pan_fraction(&self, by: (f32, f32)) -> Self {
        let (dx, dy) = (by.0 * self.width(), by.1 * self.height());
        Self {
            xmin: self.xmin - dx,
            xmax: self.xmax - dx,
            // Y is flipped, so a downward drag raises the view.
            ymin: self.ymin + dy,
            ymax: self.ymax + dy,
        }
    }

    /// `cur` reshaped so its aspect matches the region's — Blender's
    /// `ui_view2d_curRect_validate_resize`, which runs whenever the region
    /// changes and is why `View2D` has only ONE zoom rather than one per axis.
    ///
    /// Everything else here depends on that invariant and nothing else enforced
    /// it. [`Self::to_fraction`] divides each axis by that axis's extent, while
    /// [`Self::to_region`] scales BOTH by the x zoom; the two agree exactly when
    /// the view is shaped like the region and diverge on the y axis when it is
    /// not. Since the cards are placed by fraction and the painter sizes them by
    /// zoom, a mismatch shows up as sockets sliding off the card they belong to.
    ///
    /// The width is kept and the height follows, about the centre: a window
    /// getting shorter should show less, not show the same amount squashed.
    pub fn fitted_to(&self, region: (f32, f32)) -> Self {
        if region.0 <= 0.0 || region.1 <= 0.0 || self.width() <= 0.0 {
            return *self;
        }
        let wanted = self.width() * region.1 / region.0;
        let centre = (self.ymin + self.ymax) / 2.0;
        Self { ymin: centre - wanted / 2.0, ymax: centre + wanted / 2.0, ..*self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_view() -> BNodeView {
        BNodeView { xmin: 0.0, ymin: 0.0, xmax: 1001.0, ymax: 600.0 }
    }

    #[test]
    fn a_fraction_of_the_canvas_needs_the_view_and_nothing_else() {
        // The claim `to_fraction` is built on, and the reason the editor can pan at
        // all: a card's div is placed before the canvas is measured, so its position
        // has to be expressible without the pixel size. Checked against the pixel
        // path, which DOES know the size.
        //
        // It holds for a view SHAPED LIKE the region, which `fitted_to` makes an
        // invariant. The first draft of this test skipped that and compared an
        // arbitrary view against a 37×4001 canvas, where the y answers differ by a
        // factor of a hundred — because `to_region` scales both axes by the x zoom
        // and `to_fraction` divides each by its own extent. That failure is what
        // found the missing aspect correction, so the shape of the check is the
        // point: fit first, then the two agree.
        for region in [(1000.0_f32, 600.0_f32), (37.0, 4001.0), (640.0, 640.0)] {
            let view = BNodeView { xmin: -40.0, ymin: 12.0, xmax: 360.0, ymax: 252.0 }.fitted_to(region);
            for at in [(0.0, 0.0), (100.0, 100.0), (-35.0, 250.0), (359.0, 13.0)] {
                let fraction = view.to_fraction(at);
                let pixels = view.to_region(at, region);
                let (fx, fy) = (pixels.0 / region.0, pixels.1 / region.1);
                // The gap is the `+ 1` of Blender's inclusive `mask`, which is one
                // pixel of the region — so the slack scales with it.
                let slack = 2.0 / region.0.min(region.1);
                assert!(
                    (fraction.0 - fx).abs() < slack && (fraction.1 - fy).abs() < slack,
                    "{at:?} at {region:?}: {fraction:?} vs {:?}",
                    (fx, fy),
                );
            }
        }
    }

    #[test]
    fn fitting_to_a_region_keeps_the_width_and_the_centre() {
        // A window getting shorter shows LESS, rather than the same amount squashed.
        let view = BNodeView { xmin: 0.0, ymin: 0.0, xmax: 400.0, ymax: 400.0 };
        let wide = view.fitted_to((1000.0, 500.0));
        assert_eq!(wide.width(), 400.0, "the width is kept");
        assert_eq!(wide.height(), 200.0, "the height follows the region's shape");
        assert_eq!((wide.ymin + wide.ymax) / 2.0, 200.0, "about the centre");
        // A view already the region's shape is left exactly alone.
        assert_eq!(wide.fitted_to((1000.0, 500.0)), wide);
        // And a degenerate region cannot produce a degenerate view.
        assert_eq!(view.fitted_to((0.0, 0.0)), view);
    }

    #[test]
    fn a_point_survives_the_round_trip_through_fractions_too() {
        let view = BNodeView { xmin: -40.0, ymin: 12.0, xmax: 360.0, ymax: 252.0 };
        for at in [(0.0, 0.0), (100.0, 100.0), (-35.0, 250.0)] {
            let back = view.at_fraction(view.to_fraction(at));
            assert!((back.0 - at.0).abs() < 1e-2 && (back.1 - at.1).abs() < 1e-2, "{at:?} -> {back:?}");
        }
    }

    #[test]
    fn a_pan_in_fractions_is_the_same_pan_as_in_pixels() {
        // `pan_fraction` exists so a gesture does not need the canvas's width. It
        // has to agree with `pan`, which does — otherwise panning would depend on
        // which of the two the caller happened to reach for.
        for region in [(1000.0_f32, 600.0_f32), (320.0, 240.0)] {
            // Same precondition as the fractions themselves: a view shaped like
            // its region, which is the invariant `fitted_to` holds.
            let view = unit_view().fitted_to(region);
            for by in [(100.0_f32, 0.0_f32), (-40.0, 25.0), (0.0, -300.0)] {
                let pixels = view.pan(by, region);
                let fractions = view.pan_fraction((by.0 / region.0, by.1 / region.1));
                let slack = view.width() / region.0;
                assert!(
                    (pixels.xmin - fractions.xmin).abs() < slack
                        && (pixels.ymin - fractions.ymin).abs() < slack,
                    "{by:?} at {region:?}: {pixels:?} vs {fractions:?}",
                );
            }
        }
    }
}
