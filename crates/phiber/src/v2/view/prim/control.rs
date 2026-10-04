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
    /// The app's own mark: drawn larger and in the accent, because a brand is
    /// not one more icon in the row.
    pub brand: bool,
    /// How many icon-button slots the control spans in its row — Blender's
    /// `scale_x` on the pause button that stands in for both play buttons.
    /// `0` and `1` are one slot.
    pub span: u8,
    /// Drawn quiet: dimmed ink, no fill, still clickable.
    ///
    /// Two reasons, one ink, and they are named here because the audit found
    /// `text_dim` carrying four meanings with nobody saying so:
    ///
    /// - not in force — Blender's `layout.active = False` (the X-ray button
    ///   outside Solid and Wireframe);
    /// - off — a toggle whose state is false, where Blender dims the icon
    ///   rather than hiding it (the Outliner's restriction column).
    ///
    /// Both read as "this is here but it is not doing anything", which is why
    /// they share an ink. `active` is the opposite pole and they do not stack:
    /// a control that is on but not in force does not take the on fill, since
    /// that fill is exactly the claim being denied (`view/draw/button.rs`).
    pub muted: bool,
    /// A colour this control's face shows instead of an icon — a colour
    /// control's swatch, which is a button because clicking it opens the picker.
    pub swatch: Option<crate::v2::theme::BColor>,
    /// This row opens a menu of its own, so a menu renderer ends it with an
    /// arrow column.
    ///
    /// The arrow used to be the string `"▶"` handed in as the `shortcut`,
    /// which spent the one column a row has for a key binding and drew the
    /// arrow in the shortcut's quieter caption type. A row can now say that
    /// it leads somewhere AND say what key opens it.
    pub submenu: bool,
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
    pub fn as_brand(mut self) -> Self {
        self.brand = true;
        self
    }
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
    pub fn with_span(mut self, span: u8) -> Self {
        self.span = span;
        self
    }
    /// The colour this control's face shows (`swatch`).
    pub fn with_swatch(mut self, color: crate::v2::theme::BColor) -> Self {
        self.swatch = Some(color);
        self
    }

    pub fn with_muted(mut self, muted: bool) -> Self {
        self.muted = muted;
        self
    }
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
    pub fn with_submenu(mut self, submenu: bool) -> Self {
        self.submenu = submenu;
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
