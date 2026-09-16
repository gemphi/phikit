//! Menu primitives — menu items, separators, popover anchors and popovers.

use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl};

/// Popover placement relative to its anchor. `Below` for menubar drops,
/// `BelowEnd` for right-aligned host-switch drops, and `Right` for nested
/// submenus (Blender's `TOPBAR_MT_file_recover` etc).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPopoverSide {
    Below,
    BelowEnd,
    Right,
}

impl BView {
    pub fn menu_item(
        icon: impl Into<String>,
        label: impl Into<String>,
        shortcut: impl Into<String>,
        action: BAction,
    ) -> Self {
        Self::MenuItem(
            BControl::new()
                .with_icon(icon)
                .with_label(label)
                .with_shortcut(shortcut)
                .with_action(action),
        )
    }
    pub fn menu_item_hover(
        icon: impl Into<String>,
        label: impl Into<String>,
        shortcut: impl Into<String>,
        action: BAction,
        hover: BAction,
    ) -> Self {
        Self::MenuItem(
            BControl::new()
                .with_icon(icon)
                .with_label(label)
                .with_shortcut(shortcut)
                .with_action(action)
                .with_hover(hover),
        )
    }
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
