//! Region-level layout: `BRegionType` decodes into orientation, vertical
//! placement, and fixed size. All sizes come from `theme::METRICS`.
//!
//! Blender ref: `ARegion` alignment flags (`RGN_ALIGN_TOP` / `BOTTOM` /
//! `LEFT` / `RIGHT`) plus header/footer heights.

use super::{BAxis, BPlacement, BRegionLayout};
use crate::v2::region::BRegionType;
use crate::v2::theme::METRICS;

impl BRegionLayout for BRegionType {
    fn axis(&self) -> BAxis {
        match self {
            // NavBar / Tools are vertical strips — icons stack top-to-bottom.
            BRegionType::NavBar | BRegionType::Tools => BAxis::Column,
            // Center / Flex regions must not force items_center so that a
            // Canvas child can fill the full height of the region.
            _ => BAxis::Row,
        }
    }

    fn placement(&self) -> BPlacement {
        match self {
            BRegionType::Header
            | BRegionType::ToolHeader
            | BRegionType::AssetShelfHeader
            | BRegionType::HUD
            | BRegionType::Scrubbing => BPlacement::Top,
            BRegionType::Footer | BRegionType::Execute | BRegionType::AssetShelf => {
                BPlacement::Bottom
            }
            BRegionType::Tools | BRegionType::NavBar => BPlacement::Left,
            BRegionType::UI | BRegionType::ToolProps | BRegionType::Channels => BPlacement::Right,
            _ => BPlacement::Flex,
        }
    }

    fn fixed_size(&self) -> Option<f32> {
        match self {
            BRegionType::Header | BRegionType::ToolHeader | BRegionType::AssetShelfHeader => {
                Some(METRICS.h_header)
            }
            BRegionType::Footer | BRegionType::Execute => Some(METRICS.h_footer),
            BRegionType::Scrubbing => Some(METRICS.h_scrubbing),
            BRegionType::AssetShelf => Some(METRICS.h_asset_shelf),
            BRegionType::Tools => Some(METRICS.w_toolbar),
            BRegionType::NavBar => Some(METRICS.w_navbar),
            BRegionType::UI => Some(METRICS.w_panel),
            BRegionType::Channels => Some(METRICS.w_channels),
            BRegionType::ToolProps => Some(METRICS.w_tool_props),
            _ => None,
        }
    }
}
