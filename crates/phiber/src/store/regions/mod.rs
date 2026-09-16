//! BRegion specs - full Blender 5.2 17 region types.
//!
//! A region owns its panes. Panes own parts.
//!
//! Ref: Blender Source - makesdna/DNA_screen_types.h (ARegion, eRegion_Type, eRegion_Alignment)

pub mod def;
pub mod set;
pub use def::{BRegionAlignment, BRegionType};

use crate::store::panes::BPaneSpec;
use crate::store::spec::{BSpec, BSpecType};

impl BRegionType {
    pub fn label(self) -> &'static str {
        match self {
            BRegionType::Header => def::HEADER_LABEL,
            BRegionType::Center | BRegionType::Window => def::CENTER_LABEL,
            BRegionType::Footer => def::FOOTER_LABEL,
            BRegionType::Channels => def::CHANNELS_LABEL,
            BRegionType::Temporary => def::TEMPORARY_LABEL,
            BRegionType::UI => def::UI_LABEL,
            BRegionType::Tools => def::TOOLS_LABEL,
            BRegionType::ToolProps => def::TOOL_PROPS_LABEL,
            BRegionType::Preview => def::PREVIEW_LABEL,
            BRegionType::HUD => def::HUD_LABEL,
            BRegionType::NavBar => def::NAV_BAR_LABEL,
            BRegionType::Execute => def::EXECUTE_LABEL,
            BRegionType::ToolHeader => def::TOOL_HEADER_LABEL,
            BRegionType::XR => def::XR_LABEL,
            BRegionType::AssetShelf => def::ASSET_SHELF_LABEL,
            BRegionType::AssetShelfHeader => def::ASSET_SHELF_HEADER_LABEL,
            BRegionType::Scrubbing => def::SCRUBBING_LABEL,
        }
    }

    pub fn all() -> &'static [BRegionType] {
        set::ALL
    }

    pub fn items(self) -> &'static BRegionSpec {
        let idx = match self {
            BRegionType::Header => 0,
            BRegionType::Center | BRegionType::Window => 1,
            BRegionType::Footer => 2,
            BRegionType::Channels => 3,
            BRegionType::Temporary => 4,
            BRegionType::UI => 5,
            BRegionType::Tools => 6,
            BRegionType::ToolProps => 7,
            BRegionType::Preview => 8,
            BRegionType::HUD => 9,
            BRegionType::NavBar => 10,
            BRegionType::Execute => 11,
            BRegionType::ToolHeader => 12,
            BRegionType::XR => 13,
            BRegionType::AssetShelf => 14,
            BRegionType::AssetShelfHeader => 15,
            BRegionType::Scrubbing => 16,
        };
        &set::ALL_SPECS[idx]
    }

    pub fn with_set(self) -> &'static BRegionSpec {
        self.items()
    }
}

/// Static spec for a region.
/// A region owns its panes. Each BPaneSpec owns its own parts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BRegionSpec {
    pub idx: BRegionType,
    pub alignment: BRegionAlignment,
    /// Panes within this region. Each pane owns its parts.
    pub items: &'static [BPaneSpec],
}

impl BRegionSpec {
    pub const fn new(idx: BRegionType) -> Self {
        let alignment = match idx {
            BRegionType::Header | BRegionType::ToolHeader | BRegionType::AssetShelfHeader => {
                BRegionAlignment::Top
            }
            BRegionType::Footer => BRegionAlignment::Bottom,
            BRegionType::Tools => BRegionAlignment::Left,
            BRegionType::UI | BRegionType::ToolProps => BRegionAlignment::Right,
            BRegionType::HUD => BRegionAlignment::Float,
            _ => BRegionAlignment::None,
        };
        Self {
            idx,
            alignment,
            items: &[],
        }
    }

    pub const fn with_items(idx: BRegionType, items: &'static [BPaneSpec]) -> Self {
        let alignment = match idx {
            BRegionType::Header | BRegionType::ToolHeader | BRegionType::AssetShelfHeader => {
                BRegionAlignment::Top
            }
            BRegionType::Footer => BRegionAlignment::Bottom,
            BRegionType::Tools => BRegionAlignment::Left,
            BRegionType::UI | BRegionType::ToolProps => BRegionAlignment::Right,
            BRegionType::HUD => BRegionAlignment::Float,
            _ => BRegionAlignment::None,
        };
        Self {
            idx,
            alignment,
            items,
        }
    }

    pub const fn with_alignment_and_items(
        idx: BRegionType,
        alignment: BRegionAlignment,
        items: &'static [BPaneSpec],
    ) -> Self {
        Self {
            idx,
            alignment,
            items,
        }
    }
}

impl BSpec for BRegionSpec {
    fn idx(&self) -> BSpecType {
        BSpecType::BRegion(self.idx)
    }
    fn items(&self) -> Vec<&dyn BSpec> {
        self.items.iter().map(|p| p as &dyn BSpec).collect()
    }
}

impl BRegionSpec {
    /// Return the child pane set for simple hierarchy loops.
    pub fn items(&self) -> &'static [BPaneSpec] {
        self.items
    }
}
