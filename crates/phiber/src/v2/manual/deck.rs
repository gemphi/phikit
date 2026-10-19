//! The deck — how every figure in the manual is drawn.
//!
//! A manual full of flat rectangular screenshots says nothing about the
//! app it documents; the pictures could be of anything. phiber's figures
//! are drawn as a **deck**: windows of decreasing width stacked and
//! centred, each one behind showing a thin bar of itself above *and*
//! below the one in front, the way a deck of cards shows the edges of the
//! cards under the top one.
//!
//! The stack is symmetric about its middle, so the steps read as half a
//! hexagon at the top and half a hexagon at the bottom, and the whole
//! silhouette is a hexagon — phiber's shape, built out of the windows
//! rather than drawn around them.
//!
//! Three things come out of that, and all three are the point:
//!
//! - The shape is made of the layers; there is no outline anywhere.
//! - It reads as depth with no projection and no perspective: three
//!   widths and one bar height.
//! - Repeated across pages the decks line up into a series, the same
//!   three widths at the same bar on every page, with only the content of
//!   the face changing. That repetition is the mark, rather than a logo
//!   stamped in a corner.
//!
//! It is a deck and not a cascade: the layers do not step sideways, they
//! stay centred and only their width changes. A sideways step reads as
//! sheets of paper pushed askew; centred widths read as one object with
//! depth.
//!
//! Three layers because three is the fewest that reads as a series rather
//! than a pair, and because the editor already nests exactly this far: a
//! region, a panel inside it, a well inside that
//! (`theme::BSurfaceTone::Nested`). Each layer back sinks one nesting
//! level, so a figure's depth and the editor's depth are the same ladder —
//! the picture is drawn in the shades the screen is.
//!
//! The palette is Silver (`BPaletteName::Silver`), which is also the
//! editor's default, so a figure and the screen it was taken from are the
//! same grey.

use crate::v2::theme::BPaletteName;

/// The figure's geometry: the layer widths and how much of each shows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BDeck {
    /// Each layer's width as a share of the frame, **back to front** — the
    /// narrowest card first, the full face last. They increase, so the
    /// stack widens downward into a trapezoid.
    pub widths: [f32; 3],
    /// How much of a layer behind shows above — and below — the one in
    /// front, as a share of the frame's height. A card in a deck shows an
    /// edge, not a panel, so this is thin on purpose.
    pub bar: f32,
    /// The shade-ladder step each layer is drawn at, back to front: the
    /// cards behind are lighter, as a recessed well is.
    pub sink: [u8; 3],
    pub palette: BPaletteName,
}

impl BDeck {
    /// The manual's figure: the default every page uses unless it says
    /// otherwise.
    ///
    /// The widths climb 0.58 → 0.80 → 1.00: a step of 0.22 and then 0.20,
    /// so the two steps are nearly equal while the face still reads as the
    /// widest thing by a clear margin.
    pub const SILVER: Self = Self {
        widths: [0.58, 0.80, 1.00],
        bar: 0.040,
        sink: [2, 1, 0],
        palette: BPaletteName::Silver,
    };

    /// The layers, **back to front**, as `(x, y, width, height)` in shares
    /// of the frame — what a figure's renderer lays its content into.
    ///
    /// Every layer is centred both ways. A layer further back is narrower
    /// and *taller*: it reaches `bar` higher and `bar` lower than the one
    /// in front, which is the edge that shows. Drawn back to front that
    /// leaves a step at the top and the same step at the bottom — the
    /// hexagon.
    pub fn layers(&self) -> [(f32, f32, f32, f32); 3] {
        let mut out = [(0.0, 0.0, 0.0, 0.0); 3];
        for (i, width) in self.widths.iter().enumerate() {
            // The face sits inside by every step; each layer behind it
            // reclaims one of them, top and bottom alike.
            let inset = i as f32 * self.bar;
            out[i] = ((1.0 - width) / 2.0, inset, *width, 1.0 - 2.0 * inset);
        }
        out
    }

    /// The face — the layer the screen is drawn into.
    pub fn face(&self) -> (f32, f32, f32, f32) {
        self.layers()[self.widths.len() - 1]
    }

    /// The shade step a layer is drawn at.
    pub fn shade(&self, layer: usize) -> u8 {
        self.sink.get(layer).copied().unwrap_or(0)
    }
}

impl Default for BDeck {
    fn default() -> Self {
        Self::SILVER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stack_widens_toward_the_face() {
        let d = BDeck::SILVER;
        assert!(d.widths[0] < d.widths[1] && d.widths[1] < d.widths[2], "{:?}", d.widths);
        assert_eq!(d.widths[2], 1.00, "the face is the full width");
    }

    #[test]
    fn every_layer_is_centred_so_the_silhouette_is_a_hexagon() {
        // A deck, not a cascade: the layers do not step sideways. Centred
        // widths read as one object with depth; a sideways step reads as
        // sheets of paper pushed askew.
        for (x, _, w, _) in BDeck::SILVER.layers() {
            assert!((x - (1.0 - w) / 2.0).abs() < 1e-6, "layer at {x} is not centred for width {w}");
        }
        let face = BDeck::SILVER.face();
        assert_eq!((face.0, face.2), (0.0, 1.00), "the face spans the frame's width");
        // Centred vertically too: the face is inset by both steps, and
        // the same amount is left above it as below.
        assert!((face.1 - (1.0 - face.1 - face.3)).abs() < 1e-6, "the face is not centred vertically: {face:?}");
    }

    #[test]
    fn each_card_behind_shows_one_bar_above_and_one_below() {
        let d = BDeck::SILVER;
        let l = d.layers();
        for i in 1..l.len() {
            let above = l[i].1 - l[i - 1].1;
            let below = (l[i - 1].1 + l[i - 1].3) - (l[i].1 + l[i].3);
            assert!((above - d.bar).abs() < 1e-6, "layer {i} shows {above} above, wanted {}", d.bar);
            assert!((below - d.bar).abs() < 1e-6, "layer {i} shows {below} below, wanted {}", d.bar);
        }
        // Thin on purpose — an edge, not a panel.
        assert!(d.bar < 0.08, "the bar is an edge, got {}", d.bar);
        // The back layer is the tallest: it is the one that reaches the
        // frame top and bottom, and its step is the hexagon's point.
        assert!((l[0].1).abs() < 1e-6 && (l[0].1 + l[0].3 - 1.0).abs() < 1e-6, "the back layer spans the frame: {:?}", l[0]);
    }

    #[test]
    fn depth_in_the_picture_is_depth_on_the_screen() {
        // Back to front, so the card furthest back is the deepest well.
        assert_eq!(BDeck::SILVER.sink, [2, 1, 0]);
    }

    #[test]
    fn the_figure_is_drawn_in_the_editors_own_default_theme() {
        assert_eq!(BDeck::SILVER.palette, BPaletteName::Silver);
        assert_eq!(BDeck::SILVER.palette, BPaletteName::DEFAULT, "a figure and the screen it came from are the same grey");
    }
}
