//! Tree primitives — the outliner row geometry (Blender ref:
//! `outliner_draw_tree_element`).

use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BRowShade};

impl BView {
    /// A tree row at `depth`: hierarchy guides, a disclosure slot, the
    /// data-type icon, the flexible `children` (name), then the fixed
    /// right-aligned `trail` (restriction toggles).
    #[allow(clippy::too_many_arguments)]
    pub fn tree_row(
        depth: u16,
        expanded: Option<bool>,
        toggle: Option<BAction>,
        icon: impl Into<String>,
        children: Vec<BView>,
        trail: Vec<BView>,
        shade: BRowShade,
    ) -> Self {
        Self::TreeRow {
            depth,
            expanded,
            toggle,
            icon: icon.into(),
            children,
            trail,
            shade,
        }
    }
}
