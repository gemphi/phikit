//! Toolbar part spec - left tool rail in the Center region.
//!
//! Ref: Blender's toolbar (Move, Rotate, Scale, Select, Annotate, etc.)

use crate::store::parts::{BPart, BPartSpec, BPartType};

pub mod def;
pub mod set;

/// A single tool entry.
#[derive(Clone, Copy, Debug)]
pub struct ToolEntry {
    pub idx: &'static str,
    pub label: &'static str,
    pub icon: &'static str,
    pub active: bool,
}

/// Spec for a Toolbar part.
#[derive(Clone, Copy, Debug)]
pub struct ToolbarPartSpec {
    pub idx: BPartType,
    pub items: &'static [BPart],
}

impl ToolbarPartSpec {
    pub const fn new() -> Self {
        Self {
            idx: BPartType::Toolbar,
            items: &[],
        }
    }
}

impl BPartSpec for ToolbarPartSpec {
    fn idx(&self) -> BPartType {
        self.idx
    }
    fn items(&self) -> &'static [BPart] {
        self.items
    }
}
