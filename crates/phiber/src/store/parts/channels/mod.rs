//! Channels part spec - animation channel list in the Center region's left pane.
//!
//! Ref: Blender's animation channels (DopeSheet, Graph Editor, NLA)

use crate::store::parts::{BPart, BPartSpec, BPartType};

pub mod def;
pub mod set;

/// A single channel entry.
#[derive(Clone, Copy, Debug)]
pub struct ChannelEntry {
    pub idx: &'static str,
    pub icon: &'static str,
    pub selected: bool,
    pub locked: bool,
}

/// Spec for a Channels part.
#[derive(Clone, Copy, Debug)]
pub struct ChannelsPartSpec {
    pub idx: BPartType,
    pub items: &'static [BPart],
}

impl ChannelsPartSpec {
    pub const fn new() -> Self {
        Self {
            idx: BPartType::Channels,
            items: &[],
        }
    }
}

impl BPartSpec for ChannelsPartSpec {
    fn idx(&self) -> BPartType {
        self.idx
    }
    fn items(&self) -> &'static [BPart] {
        self.items
    }
}
