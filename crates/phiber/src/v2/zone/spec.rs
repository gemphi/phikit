//! `BZoneSpec` — static per-zone data.

use super::BZoneType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BZoneSpec {
    pub kind: BZoneType,
    pub label: &'static str,
    pub icon: &'static str,
    pub tooltip: &'static str,
    pub toggleable: bool,
}

impl BSpec for BZoneSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Zone(self.kind)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}
