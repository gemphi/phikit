//! Menu primitives — separators and popovers. Menu rows are `BView::MenuItem(BControl)`
//! built at call sites from the shared control record.

use crate::v2::view::BView;
use crate::v2::view::prim::BAction;

/// Popover placement relative to its anchor, and with it how wide the panel
/// starts: a side is the one thing a declaring site says about where its
/// popover goes (`draw::popover::place`, `draw::popover::BPopoverWidth`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPopoverSide {
    /// Below the trigger, leading edges flush: a menubar drop, a property
    /// dropdown, an enum picker.
    Below,
    /// Below the trigger, TRAILING edges flush: a header popover opened from
    /// an icon at the header's right end, so the panel grows back across the
    /// header instead of off the edge of the host. One side zone wide, which
    /// is also the top of Blender's own header-popover band, so in a side
    /// zone its leading edge lands on the header's first icon.
    BelowEnd,
    /// The host switcher's picker: below the trigger, leading edges flush,
    /// and up to two panels wide with its categories and hosts side by side.
    Switcher,
    /// Beside its row: a nested submenu (Blender's `TOPBAR_MT_file_recover`
    /// etc).
    Right,
}

impl BView {
    pub fn menu_sep() -> Self {
        Self::MenuSep
    }
    /// The ONE popover constructor — placement is an explicit `side` value,
    /// not a constructor-per-placement family.
    pub fn popover(
        anchor: BView,
        body: Vec<BView>,
        open: bool,
        close: BAction,
        side: BPopoverSide,
    ) -> Self {
        Self::Popover {
            anchor: Box::new(anchor),
            body,
            open,
            close,
            side,
        }
    }
}

/// A header dropdown — `layout.prop(…, text="")` on an enum, or a
/// `template_ID`'s browse button: the anchor shows the choice (or only its
/// icon), and opens under it the list with the one in use ticked. Whether
/// it is open is `open_key` (a `MenuKey`), so opening another closes it.
#[derive(Clone, Debug)]
pub struct BPick {
    open_key: crate::v2::state::BPath,
    anchor: crate::v2::view::prim::BControl,
    items: Vec<BView>,
}

impl BPick {
    pub fn new(open_key: crate::v2::state::BPath, anchor: crate::v2::view::prim::BControl) -> Self {
        Self { open_key, anchor, items: Vec::new() }
    }

    /// One row: `label`, ticked when it is the one in use, doing `action`.
    ///
    /// Ticked AND lit. This is the one shared builder for a picker's rows, and
    /// it had picked the weaker of the two answers — a glyph and no fill — so
    /// every picker built on it said "on" more quietly than the same dropdown
    /// built through `BCtrlKind::Select`, which does both (`on_state_budget`).
    pub fn item(mut self, label: &str, ticked: bool, action: BAction) -> Self {
        let row = crate::v2::view::prim::BControl::new().with_icon(if ticked { "check" } else { "" }).with_label(label).with_active(ticked).with_action(action);
        self.items.push(BView::MenuItem(row));
        self
    }

    pub fn view(self, s: &crate::v2::state::BState) -> BView {
        let open = s.get(&self.open_key) == Some(crate::v2::state::val::V_TRUE);
        let anchor = BView::MenuAnchor(self.anchor.with_active(open).with_action(BAction::Toggle(self.open_key.clone())));
        BView::popover(anchor, self.items, open, BAction::Set(self.open_key, crate::v2::state::val::V_EMPTY.to_string()), BPopoverSide::Below)
    }
}
