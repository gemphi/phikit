//! What a menu and its entries are.
//!
//! Structure only: the shapes. Every menu's *content* lives in a set —
//! `topbar/set.rs` for the app menus, `editor/set.rs` for the per-editor
//! ones — and `menubar/set.rs` says which host gets which.

pub mod set;

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

// No `PartialEq`: `Toggle` and `Act` hold `fn` pointers, and comparing those
// is not meaningful — two distinct functions can share an address once the
// linker merges them, and one function can have two. Nothing compares menu
// items (they are matched, which needs no equality), so the derive was a
// wrong answer waiting for a caller.
#[derive(Clone, Copy, Debug)]
pub enum BMenuKind {
    /// An operator the set names nothing for: it opens what `modal`,
    /// `purpose` or `url` say, or — with none — reports itself as not
    /// available yet (`menubar/op.rs`).
    Op,
    /// A checkable entry: the key it flips, and whether it is on by default
    /// — checked unless the key is `false` (a zone's "Show …"), rather than
    /// only while it is `true`.
    Toggle(fn() -> crate::v2::state::BPath, bool),
    /// A checkable entry whose key belongs to the editor showing the menu:
    /// the key is asked for with the host at hand, when drawn and when
    /// picked. For a table several editors share (`image::VIEW` serves the
    /// Image and UV editors; `NODE` the node editors), View ▸ Sidebar flips
    /// *that* editor's `<host>.show.sidebar`, not a fixed one's.
    HostToggle(fn(crate::v2::host::BHostType) -> crate::v2::state::BPath, bool),
    Sub(&'static [BMenuItem]),
    Sep,
    /// An entry that writes state when picked — `image.view_zoom_ratio`,
    /// `image.view_all` — through the action the set names.
    Act(fn() -> crate::v2::view::prim::BAction),
    /// An operator fired as one intent: `Set(ctx.op.<key>, value)`, consumed
    /// by the rule (or the app) that owns it.
    Fire(crate::v2::state::OpKey, &'static str),
    /// An operator whose value names the editor showing the menu — the host's
    /// slug followed by the suffix: `HostFire(NodeView, ":all")` picked in the
    /// Compositor fires `Set(ctx.op.node_view, "compositor:all")`, in the
    /// Shader editor `"shader:all"`. One shared table (`EDITOR_VIEW` serves the
    /// node editors) thus frames *that* editor's graph, the way `HostToggle`
    /// flips that editor's region. A host the slug does not name (the palette
    /// walking the table) reports the entry instead.
    HostFire(crate::v2::state::OpKey, &'static str),
    /// The recent files, one entry each, from the list the splash keeps.
    Recent,
    /// The undo steps, listed live from `app::undo::listing()`.
    History,
    /// A submenu listed live from the state (Add ▸ Action: the file's
    /// actions).
    Live(fn(&crate::v2::state::BState) -> Vec<crate::v2::view::BView>),
}

/// No `PartialEq`: its `kind` holds `fn` pointers (see [`BMenuKind`]).
#[derive(Clone, Copy, Debug)]
pub struct BMenuItem {
    pub label: &'static str,
    pub shortcut: &'static str,
    pub kind: BMenuKind,
    /// Declarative modal target: the slug of the modal this item opens when
    /// clicked (e.g. `"settings"`, `"file_browser"`). Empty for ordinary
    /// operators that just close the menu. This keeps the item's action
    /// intent in the set itself so the renderer never needs a hardcoded
    /// `(menu, label)` routing table.
    pub modal: &'static str,
    /// File-browser purpose slug (`BFilePurpose::slug`) for items that open
    /// the file browser for a job: open / save_as / import / export / link /
    /// append. Empty otherwise.
    pub purpose: &'static str,
    /// External URL opened in the system browser (Blender `wm.url_open`).
    pub url: &'static str,
}

/// No `PartialEq`: `panel` is a `fn` pointer (see [`BMenuKind`]).
#[derive(Clone, Copy, Debug)]
pub struct BMenuSpec {
    pub label: &'static str,
    pub items: &'static [BMenuItem],
    /// An icon shown in place of the label. Only the app menu uses one:
    /// Blender puts its logo at the head of the top bar, not the word for it.
    pub mark: Option<&'static str>,
    /// A panel popover rather than a menu (`layout.popover(panel=…)`): the
    /// body is built from the state, and `items` is unused.
    pub panel: Option<fn(&crate::v2::state::BState) -> Vec<crate::v2::view::BView>>,
    /// The menu's poll (`if … : layout.menu(…)` in the header's draw):
    /// shown only while it holds. `None`: always.
    pub shown: Option<fn(&crate::v2::state::BState) -> bool>,
}

impl BMenuSpec {
    /// Does the header show it now?
    pub fn shows(&self, s: &crate::v2::state::BState) -> bool {
        self.shown.is_none_or(|poll| poll(s))
    }
}

impl BSpec for BMenuSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Menubar)
    }
    fn label(&self) -> &'static str {
        self.label
    }
}


