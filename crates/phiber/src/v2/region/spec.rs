//! `BRegionSpec` — one region within a host. Owns its panes inline.

use super::BRegionType;
use crate::v2::pane::BPaneSpec;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BRegionSpec {
    pub kind: BRegionType,
    pub panes: &'static [BPaneSpec],
}

impl BSpec for BRegionSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Region(self.kind)
    }
    fn label(&self) -> &'static str {
        self.kind.label()
    }
}
