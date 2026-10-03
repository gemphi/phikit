//! The outliner's keys — `SpaceOutliner` and the per-object restriction
//! flags, all under `ctx.outliner.*`.

use super::BKey;
use crate::v2::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OutlinerKey {
    /// Whether a tree row is unfolded: `ctx.outliner.expanded.<row id>`.
    Expanded(&'static str),
    /// The header's filter text (`filter_text`).
    Search,
    /// `display_mode` (View Layer, Scenes, Blender File, …).
    DisplayMode,
    /// `SO_SKIP_SORT_ALPHA`, inverted: alphabetical sorting on.
    SortAlpha,
    /// A restriction flag of one object: `ctx.outliner.<object>.<field>`
    /// (`hide_get`, `hide_viewport`, `hide_render`, `hide_select`).
    Restrict(&'static str, &'static str),
    /// Whether a restriction column is shown: `ctx.outliner.col.<field>`
    /// (Blender's `show_restrict_column_*`).
    Column(&'static str),
    /// Open/closed state of the filter popover (`OUTLINER_PT_filter`).
    FilterOpen,
    /// One of the Filter popover's properties: `ctx.outliner.filter.<rna>`
    /// (`use_filter_object_mesh`, `filter_state`, …).
    Filter(&'static str),
    /// How many objects have been picked: the list shows the active row
    /// when it changes (`part::outliner::reveal`).
    Reveal,
}

impl BKey for OutlinerKey {
    fn path(&self) -> BPath {
        match self {
            Self::Expanded(id) => BPath::owned(vec!["outliner", "expanded", id]),
            Self::Search => BPath::owned(vec!["outliner", "search"]),
            Self::DisplayMode => BPath::owned(vec!["outliner", "display_mode"]),
            Self::SortAlpha => BPath::owned(vec!["outliner", "sort_alpha"]),
            Self::Restrict(object, field) => BPath::owned(vec!["outliner", object, field]),
            Self::Column(field) => BPath::owned(vec!["outliner", "col", field]),
            Self::FilterOpen => BPath::owned(vec!["outliner", "filter_open"]),
            Self::Filter(rna) => BPath::owned(vec!["outliner", "filter", rna]),
            Self::Reveal => BPath::owned(vec!["outliner", "reveal"]),
        }
    }
}
