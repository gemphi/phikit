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
    /// Whether a channel is unfolded: `ctx.dopesheet.expanded.<id>`.
    ///
    /// Blender's `ACHANNEL_SETTING_EXPAND` (`ED_anim_api.hh:670`), which it
    /// stores per data-block — `ADT_UI_EXPANDED` on an object's AnimData,
    /// `ACT_COLLAPSED` on an action. One key here, because the channel list is
    /// one tree and a row is identified by its path through it.
    ///
    /// `id` is interned (`state::intern`), the same way the Outliner's tree
    /// rows are, since a channel is named after an object or an F-curve and
    /// neither is known at compile time.
    Expanded(&'static str),
}

impl BKey for DopeKey {
    fn path(&self) -> BPath {
        match self {
            Self::Search => BPath::owned(vec!["dopesheet", "filter_text"]),
            Self::FilterOpen => BPath::owned(vec!["dopesheet", "filter_open"]),
            Self::Filter(rna) => BPath::owned(vec!["dopesheet", rna]),
            Self::Expanded(id) => BPath::owned(vec!["dopesheet", "expanded", id]),
        }
    }
}
