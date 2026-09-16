//! `BHostSpec` — one editor host with its regions inline.

use super::BHostType;
use crate::v2::region::BRegionSpec;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BHostSpec {
    pub kind: BHostType,
    pub label: &'static str,
    pub icon: &'static str,
    pub regions: &'static [BRegionSpec],
}

impl BSpec for BHostSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Host(self.kind)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}
