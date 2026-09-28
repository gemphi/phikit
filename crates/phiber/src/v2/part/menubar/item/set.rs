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
    BMenuSpec { label, items, mark: None, panel: None }
}

/// A menu that shows a mark instead of its label.
pub const fn marked(label: &'static str, icon: &'static str, items: &'static [BMenuItem]) -> BMenuSpec {
    BMenuSpec { label, items, mark: Some(icon), panel: None }
}

/// A panel popover in the menubar's row (`layout.popover(panel=…)`).
pub const fn panel(label: &'static str, body: fn(&crate::v2::state::BState) -> Vec<crate::v2::view::BView>) -> BMenuSpec {
    BMenuSpec { label, items: &[], mark: None, panel: Some(body) }
}
