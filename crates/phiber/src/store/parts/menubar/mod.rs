//! Menubar part spec - menu strip in the header region.
//!
//! Ref: Blender's header menubar (File, Edit, Render, Window, Help)

use crate::store::parts::{BPart, BPartSpec, BPartType};

pub mod def;
pub mod set;

/// Spec for a menubar part.
#[derive(Clone, Copy, Debug)]
pub struct MenubarPartSpec {
    pub idx: BPartType,
    pub items: &'static [BPart],
}

impl MenubarPartSpec {
    pub const fn new() -> Self {
        Self {
            idx: BPartType::Menubar,
            items: &[],
        }
    }
}

impl BPartSpec for MenubarPartSpec {
    fn idx(&self) -> BPartType {
        self.idx
    }
    fn items(&self) -> &'static [BPart] {
        self.items
    }
}
