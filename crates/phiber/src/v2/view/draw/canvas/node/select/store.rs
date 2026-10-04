//! How a host's selection is spelled in one state value, and read back.
//!
//! One key for the whole set, with the active node first. A key per selected
//! node would make selecting five nodes five store writes and five undo steps,
//! and would leave the active mark somewhere it could disagree with the
//! membership. Blender keeps both in `bNode.flag` and commits a selection
//! change in one go (`node_select_single` walks every node, then notifies
//! once), so one value is the closer shape as well as the cheaper one.

use super::BNodeSelection;

/// `active;also;also` — the active node first.
const NEXT: char = ';';

pub fn encode(selection: &BNodeSelection) -> String {
    selection.names.join(&NEXT.to_string())
}

pub fn decode(text: &str) -> BNodeSelection {
    BNodeSelection {
        names: text.split(NEXT).filter(|n| !n.is_empty()).map(str::to_string).collect(),
    }
}

/// This host's selection, out of the store.
pub fn selection_of(
    state: &crate::v2::state::BState,
    host: crate::v2::host::BHostType,
) -> BNodeSelection {
    use crate::v2::state::{BKey as _, NodeKey};
    decode(state.get(&NodeKey::Selected(host).path()).unwrap_or(""))
}

