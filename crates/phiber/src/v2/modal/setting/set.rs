//! Settings categories — the Preferences sidebar.
//! Blender ref: `bl_ui/space_userpref.py` UserPreferences classes.
//!
//! One table, two projections. The category list and the nav-bar tabs that
//! render it come out of the same macro, so a category can never exist in one
//! and not the other. Each category also names the style sections it owns:
//! the theme values the spaces read (`UI_STYLE_DESCRIPTORS`) are already
//! grouped into Colors / Radii / Typography / …, and this is where those
//! groups are handed to a screen.

use crate::v2::part::navbar::set::BNavTabSpec;
use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BSettingsCategory {
    Interface,
    Appearance,
    Viewport,
    Lights,
    Editing,
    Animation,
    AddOns,
    Input,
    Navigation,
    Keymap,
    System,
    SaveLoad,
    FilePaths,
    Experimental,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BSettingsCategorySpec {
    pub kind: BSettingsCategory,
    pub icon: &'static str,
    pub label: &'static str,
    /// Style sections this category shows, by their descriptor section title.
    pub sections: &'static [&'static str],
}

impl BSpec for BSettingsCategorySpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

macro_rules! categories {
    ($(($kind:ident, $icon:literal, $label:literal, $sections:expr)),* $(,)?) => {
        pub static CATEGORIES: &[BSettingsCategorySpec] = &[$(
            BSettingsCategorySpec {
                kind: BSettingsCategory::$kind,
                icon: $icon,
                label: $label,
                sections: $sections,
            },
        )*];

        /// The same categories as nav-bar tabs, for the Preferences rail.
        pub static NAV_TABS: &[BNavTabSpec] = &[$(
            BNavTabSpec { icon: $icon, label: $label },
        )*];
    };
}

categories![
    (Interface, "interface", "Interface", &["Dimensions", "Spacing", "Typography"]),
    // Blender's "Themes"; named Appearance here, as the system settings it
    // sits beside call it.
    (Appearance, "theme", "Appearance", &["Colors", "Radii", "Strokes", "Effects"]),
    (Viewport, "viewport", "Viewport", &["Effects"]),
    (Lights, "light", "Lights", &[]),
    (Editing, "edit", "Editing", &["Resize", "Strokes"]),
    (Animation, "anim", "Animation", &[]),
    (AddOns, "addon", "Add-ons", &[]),
    (Input, "input", "Input", &[]),
    (Navigation, "nav", "Navigation", &[]),
    (Keymap, "keymap", "Keymap", &[]),
    (System, "system", "System", &[]),
    (SaveLoad, "save", "Save & Load", &[]),
    (FilePaths, "path", "File Paths", &[]),
    (Experimental, "experiment", "Experimental", &[]),
];

/// The category a label names, or the first one.
pub fn category_for(label: &str) -> &'static BSettingsCategorySpec {
    CATEGORIES
        .iter()
        .find(|c| c.label == label)
        .unwrap_or(&CATEGORIES[0])
}
