//! Menu state types - open paths and checked items.

use std::collections::HashSet;

/// Menu state tracks the currently open dropdown path and checked menu items.
///
/// The open path is a sequence of keys separated by `::` (e.g. `"File::Area"`).
/// Any key that is a prefix of the open path is considered open, so a top-level
/// menu stays visible while its nested submenus are also open.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MenuState {
    pub open: Option<Vec<String>>,
    pub checked: HashSet<String>,
}
