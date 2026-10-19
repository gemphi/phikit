//! The welcome window's mark: the layers that make the hexagon.
//!
//! `manual/deck.rs` already owns this shape and the reasoning behind it —
//! *"the steps read as half a hexagon at the top and half a hexagon at the
//! bottom, and the whole silhouette is a hexagon — phiber's shape, built
//! out of the windows rather than drawn around them"*. The manual draws
//! every figure that way. The welcome window is the one screen where the
//! shape should be the subject rather than the frame, so it reads the same
//! [`BDeck`] rather than carrying a second set of widths that could drift
//! from it.
//!
//! Here the deck is read as a **cascade** instead of concentric layers:
//! the widths step downward, narrowest at the top, and the widest band —
//! the face — carries the mark. Two bands above it, which is the fewest
//! that reads as a series rather than a pair (the manual's own reason for
//! three). There is no outline anywhere; the shape is the bands.
//!
//! No header background: the bands are rows, and the mark sits in the face
//! as content. A titled card here would put a filled header bar across the
//! top of the brand, which is the one place a chrome shade must not be.

use super::set::APP_NAME;
use crate::v2::manual::deck::BDeck;
use crate::v2::view::BView;
use crate::v2::view::prim::{BCard, BRowShade};

/// How many indent units span the mark's full width. The bands' margins are
/// `(1 - width) / 2` of it, so the widths stay [`BDeck`]'s and only the
/// scale is this file's: at 24, `0.58 / 0.80 / 1.00` give margins of 5, 2
/// and 0 units — two clear steps, and the face flush.
pub const SPAN: u16 = 24;

/// The margin, in indent units, that leaves a band `width` of the span.
pub fn margin(width: f32) -> u16 {
    (((1.0 - width.clamp(0.0, 1.0)) / 2.0) * SPAN as f32).round() as u16
}

/// One band of the cascade: `body` centred inside its margins.
fn band(width: f32, shade: BRowShade, body: Vec<BView>) -> BView {
    let pad = margin(width);
    let mut row = vec![BView::indent(pad)];
    row.extend(body);
    row.push(BView::indent(pad));
    BView::list_row(row, shade)
}

/// The mark: two bands, then the face carrying the logo and the wordmark.
///
/// The bands behind are drawn at the stronger stripe tint and the face at
/// the lighter one, which is the deck's own order — *"the cards behind are
/// lighter, as a recessed well is"* — read through the two tints a row has
/// (`BRowShade::Alt` is the stronger band, `Flat` the softer).
pub fn view() -> BView {
    let deck = BDeck::SILVER;
    let [back, middle, face] = deck.widths;
    BView::column(vec![
        band(back, BRowShade::Alt, vec![BView::spacer()]),
        band(middle, BRowShade::Alt, vec![BView::spacer()]),
        band(
            face,
            BRowShade::Flat,
            vec![
                BView::spacer(),
                BView::icon("phiber"),
                BView::label(APP_NAME.to_string()),
                BView::spacer(),
            ],
        ),
    ])
}

/// The mark in a card with no header, so nothing is drawn above the brand.
pub fn card() -> BView {
    BCard::headless().body(vec![view()]).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bands_widen_downward_and_the_face_is_flush() {
        // The cascade is the deck's widths read top-down: each band wider
        // than the one above, the last one full width. A margin that did
        // not fall to zero would leave the face inset and the silhouette
        // would read as a trapezoid with a lip rather than a hexagon.
        let deck = BDeck::SILVER;
        let margins: Vec<u16> = deck.widths.iter().map(|w| margin(*w)).collect();
        assert_eq!(margins.last(), Some(&0), "the face is flush: {margins:?}");
        for pair in margins.windows(2) {
            assert!(pair[0] > pair[1], "each band is wider than the one above: {margins:?}");
        }
    }

    #[test]
    fn the_mark_is_three_bands_and_takes_its_widths_from_the_manuals_deck() {
        // The one claim that matters: this file does not carry its own
        // widths. If the manual's deck changes, the welcome mark changes
        // with it, and a figure in the manual and the brand on the welcome
        // screen stay the same object.
        let BView::Column { list } = view() else { panic!("the mark is a column") };
        assert_eq!(list.len(), BDeck::SILVER.widths.len(), "a band per deck layer");
        assert_eq!(margin(BDeck::SILVER.widths[0]), 5, "0.58 of 24 units leaves 5 either side");
    }
}
