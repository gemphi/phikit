//! BPane specs - typed pane positions within a region.
//!
//! A pane ONLY owns its parts. Parts produce BView nodes.
//! The hierarchy walks itself - no cross-level lookups.

pub mod def;
pub mod set;
pub use def::BPaneType;

use crate::store::parts::BPart;
use crate::store::spec::{BSpec, BSpecType};

impl BPaneType {
    pub fn label(self) -> &'static str {
        set::LABELS[self as usize]
    }

    pub fn all() -> &'static [BPaneType] {
        set::ALL
    }

    /// Get the full spec for this pane, including its part set.
    pub fn items(self) -> &'static BPaneSpec {
        &set::ALL_SPECS[self as usize]
    }

    pub fn with_set(self) -> &'static BPaneSpec {
        &set::ALL_SPECS[self as usize]
    }

    /// Typed panel role for the BUI layer.
    pub fn panel_role(self) -> crate::store::BPanelRole {
        match self {
            BPaneType::Left => crate::store::BPanelRole::PaneLeft,
            BPaneType::Center => crate::store::BPanelRole::PaneCenter,
            BPaneType::Right => crate::store::BPanelRole::PaneRight,
        }
    }
}

/// Static spec for a pane within a region.
/// A pane ONLY owns its parts. Each BPart produces a BView node.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPaneSpec {
    pub idx: BPaneType,
    /// Parts within this pane. Each part produces a BView node.
    pub items: &'static [BPart],
}

impl BPaneSpec {
    pub const fn new(idx: BPaneType) -> Self {
        Self { idx, items: &[] }
    }

    pub const fn with_items(idx: BPaneType, items: &'static [BPart]) -> Self {
        Self { idx, items }
    }
}

impl BSpec for BPaneSpec {
    fn idx(&self) -> BSpecType {
        BSpecType::BPane(self.idx)
    }
    fn items(&self) -> Vec<&dyn BSpec> {
        self.items.iter().map(|p| p as &dyn BSpec).collect()
    }
}

impl BPaneSpec {
    /// Return the child part set for simple hierarchy loops.
    pub fn items(&self) -> &'static [BPart] {
        self.items
    }
}
