//! Zone-level layout: `BZoneType` decodes into orientation + fixed height.
//!
//! - `Top` / `Bottom` — horizontal strips whose areas stack vertically.
//! - `Left` / `Center` / `Right` — full-height columns of stacked editors.
//! - `Full` — horizontal row of side-by-side areas.
//!
//! Sizes read from `theme::METRICS` (no inline literals).

use super::{BAxis, BZoneLayout};
use crate::v2::theme::METRICS;
use crate::v2::zone::BZoneType;

impl BZoneLayout for BZoneType {
    fn axis(&self) -> BAxis {
        match self {
            BZoneType::Top | BZoneType::Bottom => BAxis::Column,
            BZoneType::Left | BZoneType::Center | BZoneType::Right => BAxis::Column,
            BZoneType::Full => BAxis::Row,
        }
    }
    fn fixed_height(&self) -> Option<f32> {
        match self {
            BZoneType::Top => Some(METRICS.h_header), // Blender HEADERY
            BZoneType::Bottom => Some(METRICS.h_bottom_strip), // Timeline + Statusbar
            _ => None,
        }
    }
}
