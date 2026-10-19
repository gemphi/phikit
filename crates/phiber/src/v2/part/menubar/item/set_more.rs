//! `item/set.rs`'s second half — fire / fire_host / op_export / RECENT /
//! HISTORY / live / sub / SEP / m / polled. Split from `set.rs` so each file
//! sits under the 240-line read cap (`tests/level_structure.rs`).

use super::{BMenuItem, BMenuKind, BMenuSpec};
use crate::v2::state::val::V_MODAL_FILE_BROWSER;

/// An operator fired as one intent (`Set(ctx.op.<key>, value)`).
pub const fn fire(label: &'static str, shortcut: &'static str, key: crate::v2::state::OpKey, value: &'static str) -> BMenuItem {
    BMenuItem {
        label,
        shortcut,
        kind: BMenuKind::Fire(key, value),
        modal: "",
        purpose: "",
        url: "",
    }
}
/// An operator fired as one intent whose value names the showing editor —
/// the host's slug plus `suffix`. In a table several editors share
/// (`EDITOR_VIEW` for the node editors), Frame All (Home) fires
/// `node_view` with `<host>:all`, so each editor frames its own graph.
pub const fn fire_host(label: &'static str, shortcut: &'static str, key: crate::v2::state::OpKey, suffix: &'static str) -> BMenuItem {
    BMenuItem {
        label,
        shortcut,
        kind: BMenuKind::HostFire(key, suffix),
        modal: "",
        purpose: "",
        url: "",
    }
}
/// An operator that opens the file browser for a job and, once a path is
/// picked, exports in `format` (`gltf` / `usd` / `ply`).
pub const fn op_export(label: &'static str, format: &'static str) -> BMenuItem {
    BMenuItem {
        label,
        shortcut: "",
        kind: BMenuKind::Fire(crate::v2::state::OpKey::ExportFormat, format),
        modal: V_MODAL_FILE_BROWSER,
        purpose: "export",
        url: "",
    }
}
/// The recent files, listed live.
pub const RECENT: BMenuItem = BMenuItem {
    label: "Open Recent",
    shortcut: "",
    kind: BMenuKind::Recent,
    modal: "",
    purpose: "",
    url: "",
};
/// The undo steps, listed live.
pub const HISTORY: BMenuItem = BMenuItem {
    label: "Undo History",
    shortcut: "",
    kind: BMenuKind::History,
    modal: "",
    purpose: "",
    url: "",
};
/// A submenu whose entries `list` builds from the state when it opens.
pub const fn live(label: &'static str, list: fn(&crate::v2::state::BState) -> Vec<crate::v2::view::BView>) -> BMenuItem {
    BMenuItem {
        label,
        shortcut: "",
        kind: BMenuKind::Live(list),
        modal: "",
        purpose: "",
        url: "",
    }
}
pub const fn sub(label: &'static str, items: &'static [BMenuItem]) -> BMenuItem {
    BMenuItem {
        label,
        shortcut: "",
        kind: BMenuKind::Sub(items),
        modal: "",
        purpose: "",
        url: "",
    }
}
pub const SEP: BMenuItem = BMenuItem {
    label: "",
    shortcut: "",
    kind: BMenuKind::Sep,
    modal: "",
    purpose: "",
    url: "",
};
pub const fn m(label: &'static str, items: &'static [BMenuItem]) -> BMenuSpec {
    BMenuSpec { label, items, mark: None, panel: None, shown: None }
}

/// A menu the header draws only while `poll` holds.
pub const fn polled(label: &'static str, items: &'static [BMenuItem], poll: fn(&crate::v2::state::BState) -> bool) -> BMenuSpec {
    BMenuSpec { label, items, mark: None, panel: None, shown: Some(poll) }
}

/// A menu that shows a mark instead of its label.
pub const fn marked(label: &'static str, icon: &'static str, items: &'static [BMenuItem]) -> BMenuSpec {
    BMenuSpec { label, items, mark: Some(icon), panel: None, shown: None }
}

/// A panel popover in the menubar's row (`layout.popover(panel=…)`).
pub const fn panel(label: &'static str, body: fn(&crate::v2::state::BState) -> Vec<crate::v2::view::BView>) -> BMenuSpec {
    BMenuSpec { label, items: &[], mark: None, panel: Some(body), shown: None }
}
