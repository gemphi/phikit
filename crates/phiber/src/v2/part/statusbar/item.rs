//! `BStatusbarItem` — one segment. Reads its runtime value from state via the
//! spec's key; falls back to the spec's label when the key is absent. No
//! inline strings live here.

use super::set::{BStatusSegment, BStatusSegmentSpec};
use crate::v2::state::{BPath, BState};
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BStatusbarItem {
    pub spec: BStatusSegmentSpec,
}

impl BStatusbarItem {
    pub fn new(spec: BStatusSegmentSpec) -> Self {
        Self { spec }
    }

    fn state_key(kind: BStatusSegment) -> &'static str {
        match kind {
            BStatusSegment::Scene => "scene",
            BStatusSegment::Frame => "frame",
            BStatusSegment::Stats => "stats",
            BStatusSegment::Memory => "memory",
            BStatusSegment::Version => "version",
        }
    }
}

impl BViewItem for BStatusbarItem {
    fn build(&self, s: &BState) -> BView {
        let path = BPath::owned(vec![Self::state_key(self.spec.kind)]);
        let val = s.get(&path).unwrap_or(self.spec.label);
        BView::label(val)
    }
}
