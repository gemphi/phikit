//! Help modal — docs browser categories + entries.
//! Blender ref: `getting_started/help.rst` and the Help menu.

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BHelpEntry {
    pub icon: &'static str,
    pub label: &'static str,
    pub url: &'static str,
}

impl BSpec for BHelpEntry {
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

const fn h(icon: &'static str, label: &'static str, url: &'static str) -> BHelpEntry {
    BHelpEntry { icon, label, url }
}

pub static WEB_LINKS: &[BHelpEntry] = &[
    h("manual", "Manual", "https://docs.blender.org/manual/"),
    h(
        "tutorials",
        "Tutorials",
        "https://www.blender.org/support/tutorials/",
    ),
    h("support", "Support", "https://www.blender.org/support/"),
    h(
        "community",
        "User Communities",
        "https://www.blender.org/community/",
    ),
    h(
        "involve",
        "Get Involved",
        "https://www.blender.org/get-involved/",
    ),
    h(
        "release",
        "Release Notes",
        "https://www.blender.org/download/releases/",
    ),
    h(
        "bug",
        "Report a Bug",
        "https://projects.blender.org/blender/blender/issues",
    ),
];

pub static ACTIONS: &[BHelpEntry] = &[h("info", "Save System Info", "operator:wm.sysinfo")];
