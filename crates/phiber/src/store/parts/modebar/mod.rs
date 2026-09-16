//! Modebar part spec - mode selector tabs in the header region.
//!
//! Ref: Blender's mode dropdown (Object Mode, Edit Mode, Sculpt, etc.)

use crate::store::parts::{BPart, BPartSpec, BPartType};

pub mod def;
pub mod set;

/// A single mode tab in the mode bar.
#[derive(Clone, Copy, Debug)]
pub struct ModeTab {
    pub label: &'static str,
    pub icon: &'static str,
    pub active: bool,
}

/// Spec for a Modebar part.
#[derive(Clone, Copy, Debug)]
pub struct ModebarPartSpec {
    pub idx: BPartType,
    pub items: &'static [BPart],
}

impl ModebarPartSpec {
    pub const fn new() -> Self {
        Self {
            idx: BPartType::Modebar,
            items: &[],
        }
    }
}

impl BPartSpec for ModebarPartSpec {
    fn idx(&self) -> BPartType {
        self.idx
    }
    fn items(&self) -> &'static [BPart] {
        self.items
    }
}
