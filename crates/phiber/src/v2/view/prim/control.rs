//! Control — the ONE props record every interactive primitive shares.
//!
//! `Button`, `MenuItem`, `MenuAnchor`, `Tab` and `BRailEntry` differ only in
//! WHICH of these props their renderer reads — in a flex-based theme a
//! control is a control. Optional props default to `None` / `false` and the
//! `with_*` builders fill the ones that apply, so one record plus one
//! vocabulary replaces the 2^n constructor-permutation matrix
//! (`button` / `button_action` / `button_hover`, `tab` / `tab_labeled` /
//! `tab_with_tooltip`, `menu_item` / `menu_item_hover`, …). Adding a prop is
//! one field + one builder, not a new family member per combination.

use crate::v2::view::prim::BAction;

#[derive(Clone, Debug, Default)]
pub struct BControl {
    pub icon: Option<String>,
    pub label: Option<String>,
    /// Right-aligned menu hint (e.g. "Ctrl S", "▶").
    pub shortcut: Option<String>,
    /// Hover tooltip; rails show it beside the strip when collapsed.
    pub tooltip: Option<String>,
    pub active: bool,
    pub action: Option<BAction>,
    /// Fired on pointer-enter — menus use it for hover-switching.
    pub hover: Option<BAction>,
}

impl BControl {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
    pub fn with_action(mut self, action: BAction) -> Self {
        self.action = Some(action);
        self
    }
    /// Accepts `BAction` or `Option<BAction>` so optional hover wiring
    /// (menubar hover-switching) chains without leaving the builder.
    pub fn with_hover(mut self, hover: impl Into<Option<BAction>>) -> Self {
        self.hover = hover.into();
        self
    }
}
