//! `BModalSpec` — static descriptor for one modal.

use super::BModalType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BModalSpec {
    pub kind: BModalType,
    pub label: &'static str,
    pub icon: &'static str,
    pub width: f32,
    pub height: f32,
}

impl BSpec for BModalSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(crate::v2::part::BPartType::Custom) // modals rendered as overlay parts
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}
