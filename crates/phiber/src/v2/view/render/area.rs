//! Area-level layout: each area wraps its single active host. Sizing is
//! driven by the area's `BAreaType` and read from `theme::METRICS`.
//!
//! Blender ref: `ScrArea` allocates a rectangle in the screen `AreaMap`.
//! Top/Bottom get fixed heights, Left/Right get fixed widths, Center flexes.
//! Left/Right areas scroll when their host content overflows.

use super::{BAreaLayout, BAreaSize};
use crate::v2::area::BAreaType;
use crate::v2::theme::METRICS;

impl BAreaLayout for BAreaType {
    fn size(&self) -> BAreaSize {
        match self {
            BAreaType::Top => BAreaSize {
                width: None,
                height: Some(METRICS.h_header),
            },
            BAreaType::Bottom => BAreaSize {
                width: None,
                height: Some(METRICS.h_bottom_strip),
            },
            // Left/Right are content- AND zone-sized: the host's side regions
            // (UI panel = METRICS.w_panel, strips = w_toolbar / w_navbar …)
            // dictate the width, and the zone seam may override it at drag
            // time. Pinning `w_area_left/right` here pinned the zone NARROWER
            // than the panel metric (196 < 240) and clipped the Buts panel
            // after side regions started stretching (`min_w + w_full`).
            BAreaType::Left | BAreaType::Right => BAreaSize {
                width: None,
                height: None,
            },
            BAreaType::Center => BAreaSize {
                width: None,
                height: None,
            },
        }
    }
    fn is_scrollable(&self) -> bool {
        matches!(self, BAreaType::Left | BAreaType::Right)
    }
}
