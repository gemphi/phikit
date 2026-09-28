//! Profile modal — user identity + session actions (Phiber-native).

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BProfileField {
    pub key: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
}

impl BSpec for BProfileField {
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

const fn f(key: &'static str, label: &'static str, icon: &'static str) -> BProfileField {
    BProfileField { key, label, icon }
}

pub static FIELDS: &[BProfileField] = &[
    f("display_name", "Display Name", "user"),
    f("handle", "Handle", "link"),
    f("email", "Email", "chat"),
    f("avatar", "Avatar", "image"),
    f("theme", "Theme", "preferences"),
    f("status", "Status", "info"),
];

pub static ACTIONS: &[BProfileField] = &[
    f("sign_out", "Sign Out", "x"),
    f("delete", "Delete Account", "trash"),
];
