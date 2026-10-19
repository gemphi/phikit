//! How a menu entry is written down.
//!
//! Every entry is built through one of these rather than by filling in the
//! struct, so an entry that opens a modal cannot be spelt two ways and the
//! set files stay readable as menus rather than as records.

use super::{BMenuItem, BMenuKind, BMenuSpec};
use crate::v2::state::val::V_MODAL_FILE_BROWSER;

pub const fn op(label: &'static str, shortcut: &'static str) -> BMenuItem {
    BMenuItem {
        label,
        shortcut,
        kind: BMenuKind::Op,
        modal: "",
        purpose: "",
        url: "",
    }
}
/// An operator that opens the file browser for a job (`purpose` is a
/// `BFilePurpose` slug). Blender: `wm.open_mainfile`, `wm.save_as_mainfile`,
/// `wm.link`, `wm.append`, every `*.import_*` / `*.export_*` operator.
pub const fn op_file(label: &'static str, shortcut: &'static str, purpose: &'static str) -> BMenuItem {
    BMenuItem {
        label,
        shortcut,
        kind: BMenuKind::Op,
        modal: V_MODAL_FILE_BROWSER,
        purpose,
        url: "",
    }
}
/// An operator that opens a web page (Blender `wm.url_open` / `wm.url_open_preset`).
pub const fn op_url(label: &'static str, url: &'static str) -> BMenuItem {
    BMenuItem {
        label,
        shortcut: "",
        kind: BMenuKind::Op,
        modal: "",
        purpose: "",
        url,
    }
}
/// An operator that opens a modal. `modal` is the modal slug
/// (`BModalType::slug`), carried declaratively so `leaf_action` can route it
/// without a per-label match.
pub const fn op_modal(
    label: &'static str,
    shortcut: &'static str,
    modal: &'static str,
) -> BMenuItem {
    BMenuItem {
        label,
        shortcut,
        kind: BMenuKind::Op,
        modal,
        purpose: "",
        url: "",
    }
}
/// An entry that fires `act` when picked, and closes the menu.
pub const fn act(label: &'static str, shortcut: &'static str, act: fn() -> crate::v2::view::prim::BAction) -> BMenuItem {
    BMenuItem {
        label,
        shortcut,
        kind: BMenuKind::Act(act),
        modal: "",
        purpose: "",
        url: "",
    }
}
/// A checkable entry flipping `key`, checked while it is `true`.
pub const fn tog(label: &'static str, key: fn() -> crate::v2::state::BPath) -> BMenuItem {
    BMenuItem {
        label,
        shortcut: "",
        kind: BMenuKind::Toggle(key, false),
        modal: "",
        purpose: "",
        url: "",
    }
}
/// A checkable entry flipping `key` that is also a key binding — View ▸
/// Sidebar (N): checked while the key is `true`.
pub const fn tog_keyed(label: &'static str, shortcut: &'static str, key: fn() -> crate::v2::state::BPath) -> BMenuItem {
    BMenuItem {
        label,
        shortcut,
        kind: BMenuKind::Toggle(key, false),
        modal: "",
        purpose: "",
        url: "",
    }
}

/// A checkable key binding whose key is the showing editor's — View ▸
/// Sidebar (N) in a table several editors share: `key` is given the host
/// when the row is drawn and when it is picked, so each editor flips its
/// own region. Checked while that key is `true`.
pub const fn tog_host(
    label: &'static str,
    shortcut: &'static str,
    key: fn(crate::v2::host::BHostType) -> crate::v2::state::BPath,
) -> BMenuItem {
    BMenuItem {
        label,
        shortcut,
        kind: BMenuKind::HostToggle(key, false),
        modal: "",
        purpose: "",
        url: "",
    }
}
/// `tog_host`, on by default: checked unless the showing editor's key is
/// `false` — View ▸ Toolbar (T), which Blender shows until you hide it.
pub const fn tog_host_on(
    label: &'static str,
    shortcut: &'static str,
    key: fn(crate::v2::host::BHostType) -> crate::v2::state::BPath,
) -> BMenuItem {
    BMenuItem {
        label,
        shortcut,
        kind: BMenuKind::HostToggle(key, true),
        modal: "",
        purpose: "",
        url: "",
    }
}
/// A checkable entry flipping `key`, on by default: checked unless the key
/// is `false` — a zone's "Show …".
pub const fn tog_shown(label: &'static str, key: fn() -> crate::v2::state::BPath) -> BMenuItem {
    BMenuItem {
        label,
        shortcut: "",
        kind: BMenuKind::Toggle(key, true),
        modal: "",
        purpose: "",
        url: "",
    }
}

#[path = "set_more.rs"]
mod more;
pub use more::*;
