//! Command Palette modal — searchable list of operators.
//! Blender ref: `bpy.ops.wm.search_menu` and `bpy.ops.wm.search_operator`.

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BCommandKind {
    Menu,
    Operator,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BCommandField {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: BCommandKind,
}

impl BSpec for BCommandField {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.label
    }
}

const fn k(key: &'static str, label: &'static str, kind: BCommandKind) -> BCommandField {
    BCommandField { key, label, kind }
}

/// The three sources the palette can search: menu items, operators, addons.
pub static SOURCES: &[BCommandField] = &[
    k("wm.search_menu", "Menu Search", BCommandKind::Menu),
    k(
        "wm.search_operator",
        "Operator Search",
        BCommandKind::Operator,
    ),
];
