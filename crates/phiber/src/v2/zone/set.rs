//! Zone static set — indexed by `BZoneType` discriminant order.

use super::{BZoneSpec, BZoneType};

pub static SET: &[BZoneSpec] = &[
    BZoneSpec {
        kind: BZoneType::Top,
        label: "Top",
        icon: "",
        tooltip: "",
        toggleable: false,
    },
    BZoneSpec {
        kind: BZoneType::Left,
        label: "Left",
        icon: "zone_left",
        tooltip: "Toggle Left Panel",
        toggleable: true,
    },
    BZoneSpec {
        kind: BZoneType::Center,
        label: "Center",
        icon: "",
        tooltip: "",
        toggleable: false,
    },
    BZoneSpec {
        kind: BZoneType::Right,
        label: "Right",
        icon: "zone_right",
        tooltip: "Toggle Right Panel",
        toggleable: true,
    },
    BZoneSpec {
        kind: BZoneType::Bottom,
        label: "Bottom",
        icon: "zone_bottom",
        tooltip: "Toggle Bottom Panel",
        toggleable: true,
    },
    BZoneSpec {
        kind: BZoneType::Full,
        label: "Full",
        icon: "",
        tooltip: "",
        toggleable: false,
    },
    BZoneSpec {
        kind: BZoneType::Rail,
        label: "Rail",
        icon: "",
        tooltip: "",
        toggleable: false,
    },
];

/// One-word zone slug (used as a path segment).
pub fn slug(z: BZoneType) -> &'static str {
    match z {
        BZoneType::Top => "top",
        BZoneType::Left => "left",
        BZoneType::Center => "center",
        BZoneType::Right => "right",
        BZoneType::Bottom => "bottom",
        BZoneType::Full => "full",
        BZoneType::Rail => "rail",
    }
}
