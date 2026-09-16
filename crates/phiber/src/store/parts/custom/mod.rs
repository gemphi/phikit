//! Custom part spec - delegates to host-specific renderer.
//!
//! The Custom part is the viewport, node graph, text editor, outliner tree,
//! timeline ruler, etc. Each host type provides its own render key that
//! the view layer uses to dispatch to the correct renderer.

use crate::store::parts::{BPart, BPartSpec, BPartType};

pub mod def;
pub mod set;

/// Spec for a Custom part.
#[derive(Clone, Copy, Debug)]
pub struct CustomPartSpec {
    pub idx: BPartType,
    pub items: &'static [BPart],
}

impl CustomPartSpec {
    pub const fn new() -> Self {
        Self {
            idx: BPartType::Custom,
            items: &[],
        }
    }
}

impl BPartSpec for CustomPartSpec {
    fn idx(&self) -> BPartType {
        self.idx
    }
    fn items(&self) -> &'static [BPart] {
        self.items
    }
}
