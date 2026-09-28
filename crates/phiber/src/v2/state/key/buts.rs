//! The Properties editor's keys — every RNA property a panel edits, under
//! `ctx.buts.*`. The property's own key (`BPropSpec::key`, e.g. `engine`,
//! `cycles.samples`) is the last segment, so the store reads like Blender's
//! RNA paths.

use super::BKey;
use crate::v2::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ButsKey {
    /// A property's value: `ctx.buts.<prop key>`.
    Value(&'static str),
    /// One axis of a vector property: `ctx.buts.<prop key>.<axis>`.
    Axis(&'static str, &'static str),
    /// Whether an enum property's dropdown is open: `ctx.buts.dropdown.<prop key>`.
    Dropdown(&'static str),
    /// The active context tab of the navbar (Render, Output, Object, …).
    NavbarTab,
    /// Whether the navbar shows labels beside its icons (`"true"`) or icons only.
    NavbarExpanded,
}

impl BKey for ButsKey {
    fn path(&self) -> BPath {
        match self {
            Self::Value(key) => BPath::owned(vec!["buts", key]),
            Self::Axis(key, axis) => BPath::owned(vec!["buts", key, axis]),
            Self::Dropdown(key) => BPath::owned(vec!["buts", "dropdown", key]),
            Self::NavbarTab => BPath::owned(vec!["buts", "navbar", "tab"]),
            Self::NavbarExpanded => BPath::owned(vec!["buts", "navbar", "expanded"]),
        }
    }
}
