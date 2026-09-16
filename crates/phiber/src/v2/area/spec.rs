//! `BAreaSpec` — one area within a space. Static; owned by its space.

use super::BAreaType;
use crate::v2::host::BHostType;
use crate::v2::spec::{BSpec, BSpecKind};
use crate::v2::zone::BZoneType;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BAreaSpec {
    pub kind: BAreaType,
    pub zone: BZoneType,
    pub default_host: BHostType,
    /// Hosts this area can switch between.
    pub hosts: &'static [BHostType],
}

impl BSpec for BAreaSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Area(self.kind)
    }
    fn label(&self) -> &'static str {
        self.kind.label()
    }
}
