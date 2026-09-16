//! Zone set - static spec arrays with hardcoded toggle data.
//! Each zone spec carries its own icon, tooltip, toggleable flag, and fraction.

use super::def::*;
use super::{BZoneSpec, BZoneType};

pub static ALL_ZONES: &[BZoneType] = &[
    BZoneType::Full,
    BZoneType::Top,
    BZoneType::Left,
    BZoneType::Center,
    BZoneType::Right,
    BZoneType::Bottom,
];

/// Hardcoded zone specs — toggle data lives here, not in view functions.
pub static ALL_SPECS: &[BZoneSpec] = &[
    BZoneSpec {
        idx: BZoneType::Full,
        icon: "",
        tooltip: "",
        toggleable: false,
        fraction: 1.0,
    },
    BZoneSpec {
        idx: BZoneType::Top,
        icon: "",
        tooltip: "",
        toggleable: false,
        fraction: 0.04,
    },
    BZoneSpec {
        idx: BZoneType::Left,
        icon: "zone_left",
        tooltip: "Toggle Left Panel",
        toggleable: true,
        fraction: 0.18,
    },
    BZoneSpec {
        idx: BZoneType::Center,
        icon: "",
        tooltip: "",
        toggleable: false,
        fraction: 0.64,
    },
    BZoneSpec {
        idx: BZoneType::Right,
        icon: "zone_right",
        tooltip: "Toggle Right Panel",
        toggleable: true,
        fraction: 0.18,
    },
    BZoneSpec {
        idx: BZoneType::Bottom,
        icon: "zone_bottom",
        tooltip: "Toggle Bottom Panel",
        toggleable: true,
        fraction: 0.28,
    },
];

/// Zones that can be toggled on/off in the space view.
pub static TOGGLEABLE: &[BZoneType] = &[BZoneType::Left, BZoneType::Right, BZoneType::Bottom];

pub static LABELS: &[&str] = &[
    FULL_LABEL,
    TOP_LABEL,
    LEFT_LABEL,
    RIGHT_LABEL,
    CENTER_LABEL,
    BOTTOM_LABEL,
];
