//! Row / column / list-row / spacer / indent — the linear layout
//! primitives and their shading tag.

use crate::v2::host::BHostType;
use crate::v2::theme::{BColor, BResolvedUiStyle};
use crate::v2::view::BView;

/// What a row has to say about itself — and, because the shade *is* the
/// question, what it paints at rest and under the pointer.
///
/// The tree, the file list and the table each used to map these four states
/// themselves, and disagreed: the outliner painted an alternating row in the
/// hover colour, so a striped row read as permanently hovered, and a selected
/// tree row answered the pointer with nothing. The mapping belongs to the
/// type, not to each of the three widgets that asks it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BRowShade {
    Flat,
    Alt,
    Selected,
    /// The active one among the selected (`TH_ACTIVE` over `TH_SELECT`):
    /// a step lighter, as Blender lights the active object's row.
    Active,
}

/// The two opacities the alternating bands carry. Both are the stripe colour
/// (`row_alt`) at reduced strength, so every row is tinted and the swept
/// surface still reads through: the even rows a touch fainter, the odd rows a
/// touch stronger — a gentle zebra where neither band goes opaque. A flat row
/// used to paint nothing, which left the two rows too alike to tell apart.
const ROW_FLAT_ALPHA: f32 = 0.40;
const ROW_BAND_ALPHA: f32 = 0.80;

impl BRowShade {
    /// What a row of this shade paints at rest.
    ///
    /// The two resting bands are the stripe colour at two alphas (above), so
    /// the swept surface shows through both and the zebra reads; selection and
    /// the active row keep their own meaning colours.
    pub fn fill(self, style: &BResolvedUiStyle) -> Option<BColor> {
        let band = |alpha: f32| {
            let (h, s, l, _) = style.colors.row_alt;
            (h, s, l, alpha)
        };
        match self {
            Self::Flat => Some(band(ROW_FLAT_ALPHA)),
            Self::Alt => Some(band(ROW_BAND_ALPHA)),
            Self::Selected => Some(style.colors.row_selected),
            Self::Active => Some(style.colors.row_active),
        }
    }

    /// The other zebra band: the colour a tree row's hierarchy lines take, so
    /// a line on a flat row is the alternate row's shade and on an alternate
    /// row the flat row's — subtle against its own, and never a third colour.
    /// A selected or active row takes the flat band's.
    pub fn opposite_band(self) -> Self {
        match self {
            Self::Flat => Self::Alt,
            Self::Alt | Self::Selected | Self::Active => Self::Flat,
        }
    }

    /// What it paints under the pointer. A row that already carries a fill
    /// lightens by one step off *that* fill, so the pointer reads on a
    /// selected row as clearly as on an empty one; a row with no fill of its
    /// own takes the shared item hover.
    ///
    /// Controls inside a header strip are a separate, deliberate family and
    /// hover to an accent wash instead (`chrome::paint`), the way Blender
    /// highlights a menu entry in blue and an outliner row in grey.
    pub fn hover(self, style: &BResolvedUiStyle, host: BHostType) -> BColor {
        match self.fill(style) {
            Some(resting) if matches!(self, Self::Selected | Self::Active) => {
                style.selected_hover(resting)
            }
            _ => style.host_item_hover(host),
        }
    }
}

impl BView {
    pub fn row(list: Vec<BView>) -> Self {
        Self::Row { list }
    }
    pub fn list_row(list: Vec<BView>, shade: BRowShade) -> Self {
        Self::ListRow { list, shade }
    }
    pub fn column(list: Vec<BView>) -> Self {
        Self::Column { list }
    }
    /// Flexible space that grows to fill a stacking axis, pushing the views
    /// after it toward the far end (used for the activity bar's three-region
    /// layout so the bottom group pins to the bottom).
    pub fn spacer() -> Self {
        Self::Spacer
    }
    /// Fixed-width blank of `level` tree grid columns.
    pub fn indent(level: u16) -> Self {
        Self::Indent { level }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::BState;
    use crate::v2::theme::{BPaletteName, BThemeMode, resolve_ui_style, ui_style_defaults_in};

    #[test]
    fn both_resting_rows_are_tinted_at_two_opacities() {
        let style = resolve_ui_style(
            ui_style_defaults_in(BPaletteName::Sphere, BThemeMode::Dark),
            &BState::new(),
        );
        let flat = BRowShade::Flat.fill(&style).expect("a flat row is tinted now");
        let alt = BRowShade::Alt.fill(&style).expect("an alternate row is tinted");
        // Same stripe hue/saturation/lightness — the two bands differ only in
        // opacity, the band the stronger of the two.
        assert_eq!((flat.0, flat.1, flat.2), (alt.0, alt.1, alt.2), "both bands are the one stripe tint");
        assert!(flat.3 < alt.3, "the band is the stronger opacity: {} vs {}", flat.3, alt.3);
        assert!(flat.3 > 0.0 && alt.3 < 1.0, "both stay translucent so the swept surface reads through");
    }
}
