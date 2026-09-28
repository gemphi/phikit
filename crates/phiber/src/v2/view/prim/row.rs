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

impl BRowShade {
    /// What a row of this shade paints at rest — `None` to let its surface
    /// through.
    ///
    /// A resting fill is opaque, so a row that has nothing of its own to say
    /// paints nothing rather than restating its panel's colour flat: a flat
    /// fill over a swept surface would cancel the sweep on every row.
    pub fn fill(self, style: &BResolvedUiStyle) -> Option<BColor> {
        match self {
            Self::Flat => None,
            Self::Alt => Some(style.colors.row_alt),
            Self::Selected => Some(style.colors.row_selected),
            Self::Active => Some(style.colors.row_active),
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
