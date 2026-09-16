//! Row / column / list-row / spacer / indent — the linear layout
//! primitives and their shading tag.

use crate::v2::view::BView;

/// Row background shade for list-style rows (file browser): flat rows,
/// lightened alternating rows, and the selected row (`bg_active`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BRowShade {
    Flat,
    Alt,
    Selected,
}

impl BView {
    pub fn row(children: Vec<BView>) -> Self {
        Self::Row { children }
    }
    pub fn list_row(children: Vec<BView>, shade: BRowShade) -> Self {
        Self::ListRow { children, shade }
    }
    pub fn column(children: Vec<BView>) -> Self {
        Self::Column { children }
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
