//! Settings modal — sidebar categories + section groups per category.
//! Blender ref: `bl_ui/space_userpref.py` UserPreferences classes.

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BSettingsCategory {
    Interface,
    Themes,
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

const fn c(
    kind: BSettingsCategory,
    icon: &'static str,
    label: &'static str,
) -> BSettingsCategorySpec {
    BSettingsCategorySpec { kind, icon, label }
}

pub static CATEGORIES: &[BSettingsCategorySpec] = &[
    c(BSettingsCategory::Interface, "interface", "Interface"),
    c(BSettingsCategory::Themes, "theme", "Themes"),
    c(BSettingsCategory::Viewport, "viewport", "Viewport"),
    c(BSettingsCategory::Lights, "light", "Lights"),
    c(BSettingsCategory::Editing, "edit", "Editing"),
    c(BSettingsCategory::Animation, "anim", "Animation"),
    c(BSettingsCategory::AddOns, "addon", "Add-ons"),
    c(BSettingsCategory::Input, "input", "Input"),
    c(BSettingsCategory::Navigation, "nav", "Navigation"),
    c(BSettingsCategory::Keymap, "keymap", "Keymap"),
    c(BSettingsCategory::System, "system", "System"),
    c(BSettingsCategory::SaveLoad, "save", "Save & Load"),
    c(BSettingsCategory::FilePaths, "path", "File Paths"),
    c(
        BSettingsCategory::Experimental,
        "experiment",
        "Experimental",
    ),
];
