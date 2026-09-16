//! Statusbar part spec - status items in the Footer region.
//!
//! Ref: Blender's status bar (scene stats, memory, progress)

use crate::store::parts::{BPart, BPartSpec, BPartType};

pub mod def;
pub mod set;

/// A single status bar item.
#[derive(Clone, Copy, Debug)]
pub struct StatusItem {
    pub idx: &'static str,
    pub label: &'static str,
}

/// Spec for a Statusbar part.
#[derive(Clone, Copy, Debug)]
pub struct StatusbarPartSpec {
    pub idx: BPartType,
    /// Child items in the BSpec hierarchy (always empty for a leaf part).
    pub items: &'static [BPart],
}

impl StatusbarPartSpec {
    pub const fn new() -> Self {
        Self {
            idx: BPartType::Statusbar,
            items: &[],
        }
    }
}

impl BPartSpec for StatusbarPartSpec {
    fn idx(&self) -> BPartType {
        self.idx
    }
    fn items(&self) -> &'static [BPart] {
        self.items
    }
}
