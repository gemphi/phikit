//! Tree primitives — the outliner row geometry (Blender ref:
//! `outliner_draw_tree_element`).

use crate::v2::state::BPath;
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BRowShade};

/// A tree row's drag and drop: `payload` names what the row is (`ob:Cube`,
/// `col:Scene Collection/Props`); a row dragged onto another writes
/// `<dragged>><target>` to `drop` — the Outliner's `outliner.*_drop` —
/// led by `&` when Ctrl is held (link, not move). `label` is what the
/// ghost under the pointer says.
#[derive(Clone, Debug, PartialEq)]
pub struct BTreeDrag {
    pub payload: String,
    pub drop: BPath,
    pub label: String,
}

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
        list: Vec<BView>,
        trail: Vec<BView>,
        shade: BRowShade,
    ) -> Self {
        Self::TreeRow {
            depth,
            expanded,
            toggle,
            lead: Vec::new(),
            icon: icon.into(),
            list,
            trail,
            shade,
            drag: None,
        }
    }

    /// A tree row that can be dragged and dropped on.
    pub fn with_drag(mut self, drag: BTreeDrag) -> Self {
        if let Self::TreeRow { drag: slot, .. } = &mut self {
            *slot = Some(drag);
        }
        self
    }

    /// A tree row with `lead` between its disclosure and its icon.
    pub fn with_lead(mut self, lead: Vec<BView>) -> Self {
        if let Self::TreeRow { lead: slot, .. } = &mut self {
            *slot = lead;
        }
        self
    }
}
