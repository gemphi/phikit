//! Profile modal — user identity + session actions (Phiber-native).

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BProfileField {
    pub key: &'static str,
    pub label: &'static str,
}

impl BSpec for BProfileField {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.label
    }
}

const fn f(key: &'static str, label: &'static str) -> BProfileField {
    BProfileField { key, label }
}

pub static FIELDS: &[BProfileField] = &[
    f("display_name", "Display Name"),
    f("handle", "Handle"),
    f("email", "Email"),
    f("avatar", "Avatar"),
    f("theme", "Theme"),
    f("status", "Status"),
];

pub static ACTIONS: &[BProfileField] = &[f("sign_out", "Sign Out"), f("delete", "Delete Account")];
