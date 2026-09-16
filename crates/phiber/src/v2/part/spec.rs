//! `BPartSpec` — one part descriptor (identity + label).

use super::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPartSpec {
    pub kind: BPartType,
    pub label: &'static str,
}

impl BSpec for BPartSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(self.kind)
    }
    fn label(&self) -> &'static str {
        self.label
    }
}
