//! The animation editors' shared filter keys — Blender's `DopeSheet`, all
//! under `ctx.dopesheet.*`.
//!
//! One `DopeSheet` per space in Blender, but the settings are the same
//! struct and the same panel in every one of them
//! (`bl_ui/space_dopesheet.py::dopesheet_filter`, drawn by
//! `DOPESHEET_PT_filters`, `GRAPH_PT_filters` and `NLA_PT_filters`). They
//! share one scope here for the same reason: filtering the Dope Sheet to the
//! selected objects and then opening the Graph editor should not show
//! everything again.

use super::BKey;
use crate::v2::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DopeKey {
    /// `filter_text`: the channels are narrowed to those whose name matches.
    Search,
    /// Whether the filter popover is open.
    FilterOpen,
    /// One of the filter's properties: `ctx.dopesheet.<rna>`
    /// (`show_only_selected`, `show_transforms`, …).
    Filter(&'static str),
}

impl BKey for DopeKey {
    fn path(&self) -> BPath {
        match self {
            Self::Search => BPath::owned(vec!["dopesheet", "filter_text"]),
            Self::FilterOpen => BPath::owned(vec!["dopesheet", "filter_open"]),
            Self::Filter(rna) => BPath::owned(vec!["dopesheet", rna]),
        }
    }
}
