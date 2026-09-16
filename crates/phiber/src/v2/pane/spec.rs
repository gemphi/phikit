//! `BPaneSpec` — one pane within a region. Owns its parts inline.

use super::BPaneType;
use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPaneSpec {
    pub kind: BPaneType,
    pub parts: &'static [BPartType],
}

impl BSpec for BPaneSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Pane(self.kind)
    }
    fn label(&self) -> &'static str {
        self.kind.label()
    }
}
