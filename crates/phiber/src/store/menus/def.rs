//! Menu specs - shared enums, structs, constants, and trait.
//!
//! This file only declares types, constants, and enums. Runtime sets
//! (static arrays of `MenuItem`) live in `set.rs` for each menu group.
//! The root `mod.rs` wires `BMenu` indices to the concrete specs.

// ── Top-level menu constants ──

pub const MENU_FILE: &str = "File";
pub const MENU_EDIT: &str = "Edit";
pub const MENU_RENDER: &str = "Render";
pub const MENU_WINDOW: &str = "Window";
pub const MENU_HELP: &str = "Help";

pub const MENU_FILE_ICON: &str = "file";
pub const MENU_EDIT_ICON: &str = "edit";
pub const MENU_RENDER_ICON: &str = "render";
pub const MENU_WINDOW_ICON: &str = "app-window";
pub const MENU_HELP_ICON: &str = "help";

// ── Enums ──

#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BMenu {
    File,
    Edit,
    Render,
    Window,
    Help,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MenuItemKind {
    Label,
    Check,
    Radio,
    Separator,
}

impl BMenu {
    pub fn idx(self) -> usize {
        self as usize
    }

    pub fn from_idx(idx: usize) -> Option<Self> {
        const ALL: [BMenu; 5] = [
            BMenu::File,
            BMenu::Edit,
            BMenu::Render,
            BMenu::Window,
            BMenu::Help,
        ];
        ALL.get(idx).copied()
    }

    pub fn label(self) -> String {
        match self {
            BMenu::File => MENU_FILE.to_string(),
            BMenu::Edit => MENU_EDIT.to_string(),
            BMenu::Render => MENU_RENDER.to_string(),
            BMenu::Window => MENU_WINDOW.to_string(),
            BMenu::Help => MENU_HELP.to_string(),
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            BMenu::File => MENU_FILE_ICON,
            BMenu::Edit => MENU_EDIT_ICON,
            BMenu::Render => MENU_RENDER_ICON,
            BMenu::Window => MENU_WINDOW_ICON,
            BMenu::Help => MENU_HELP_ICON,
        }
    }
}

// ── Spec structs ──

/// A single menu item: label + icon + optional shortcut + optional submenu.
/// `None` label = separator. The label IS the key - no separate index enum.
/// Not a BSpec; it is data owned by a MenuSpec.
#[derive(Clone, Copy, Debug)]
pub struct MenuItem {
    pub kind: MenuItemKind,
    pub label: Option<&'static str>,
    pub icon: Option<&'static str>,
    pub shortcut: Option<&'static str>,
    pub items: Option<&'static [MenuItem]>,
    pub checked: bool,
}

impl MenuItem {
    pub const fn new() -> Self {
        Self {
            kind: MenuItemKind::Label,
            label: None,
            icon: None,
            shortcut: None,
            items: None,
            checked: false,
        }
    }

    pub const fn with_label(self, label: &'static str) -> Self {
        Self {
            label: Some(label),
            ..self
        }
    }

    pub const fn with_icon(self, icon: &'static str) -> Self {
        Self {
            icon: Some(icon),
            ..self
        }
    }

    pub const fn with_shortcut(self, shortcut: &'static str) -> Self {
        Self {
            shortcut: Some(shortcut),
            ..self
        }
    }

    pub const fn with_items(self, items: &'static [MenuItem]) -> Self {
        Self {
            items: Some(items),
            ..self
        }
    }

    pub const fn with_kind(self, kind: MenuItemKind) -> Self {
        Self { kind, ..self }
    }

    pub const fn with_checked(self, checked: bool) -> Self {
        Self { checked, ..self }
    }

    pub const fn node(label: &'static str, icon: &'static str) -> Self {
        Self::new().with_label(label).with_icon(icon)
    }

    pub const fn check(label: &'static str, icon: &'static str, checked: bool) -> Self {
        Self::new()
            .with_kind(MenuItemKind::Check)
            .with_label(label)
            .with_icon(icon)
            .with_checked(checked)
    }

    pub const fn radio(label: &'static str, icon: &'static str, checked: bool) -> Self {
        Self::new()
            .with_kind(MenuItemKind::Radio)
            .with_label(label)
            .with_icon(icon)
            .with_checked(checked)
    }

    pub const fn separator() -> Self {
        Self::new().with_kind(MenuItemKind::Separator)
    }

    /// Recursively iterate over this item and all nested items.
    pub fn for_each<F: FnMut(&MenuItem)>(&self, f: &mut F) {
        f(self);
        if let Some(items) = self.items {
            for item in items {
                item.for_each(f);
            }
        }
    }

    /// Recursively find the first item matching a predicate.
    pub fn find<F: FnMut(&MenuItem) -> bool>(&self, f: &mut F) -> Option<&MenuItem> {
        if f(self) {
            return Some(self);
        }
        if let Some(items) = self.items {
            for item in items {
                if let Some(found) = item.find(f) {
                    return Some(found);
                }
            }
        }
        None
    }
}

/// A full menu: idx + ordered items.
#[derive(Clone, Debug)]
pub struct MenuSpec {
    pub idx: BMenu,
    pub items: &'static [MenuItem],
}

impl MenuSpec {
    pub fn label(&self) -> String {
        self.idx.label()
    }

    /// Iterate over all items in this spec, including nested submenus.
    pub fn for_each<F: FnMut(&MenuItem)>(&self, f: &mut F) {
        for item in self.items {
            item.for_each(f);
        }
    }

    /// Find the first item in this spec matching a predicate, recursively.
    pub fn find<F: FnMut(&MenuItem) -> bool>(&self, f: &mut F) -> Option<&MenuItem> {
        for item in self.items {
            if let Some(found) = item.find(f) {
                return Some(found);
            }
        }
        None
    }
}

/// Trait: each menu struct provides its `MenuSpec`.
pub trait MenuSpecProvider {
    fn idx() -> BMenu;
    fn spec() -> MenuSpec;
}
