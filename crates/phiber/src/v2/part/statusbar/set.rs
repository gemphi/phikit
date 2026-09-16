//! Statusbar set — the ordered segment schema. Runtime values come from state.
//! Blender ref: `bl_ui/space_statusbar.py` `STATUSBAR_HT_header`.

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BStatusSegment {
    Scene,
    Frame,
    Stats,
    Memory,
    Version,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BStatusSegmentSpec {
    pub kind: BStatusSegment,
    pub label: &'static str,
}

impl BSpec for BStatusSegmentSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Statusbar)
    }
    fn label(&self) -> &'static str {
        self.label
    }
}

pub static SEGMENTS: &[BStatusSegmentSpec] = &[
    BStatusSegmentSpec {
        kind: BStatusSegment::Scene,
        label: "Scene",
    },
    BStatusSegmentSpec {
        kind: BStatusSegment::Frame,
        label: "Frame",
    },
    BStatusSegmentSpec {
        kind: BStatusSegment::Stats,
        label: "Stats",
    },
    BStatusSegmentSpec {
        kind: BStatusSegment::Memory,
        label: "Memory",
    },
    BStatusSegmentSpec {
        kind: BStatusSegment::Version,
        label: "Version",
    },
];
