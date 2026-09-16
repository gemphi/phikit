//! Part set — one `BPartSpec` per `BPartType`, order matches the enum.

use super::{BPartSpec, BPartType};

pub static SET: &[BPartSpec] = &[
    BPartSpec {
        kind: BPartType::Menubar,
        label: "Menu Bar",
    },
    BPartSpec {
        kind: BPartType::Modebar,
        label: "Mode Bar",
    },
    BPartSpec {
        kind: BPartType::Switcher,
        label: "Switcher",
    },
    BPartSpec {
        kind: BPartType::Spacebar,
        label: "Space Bar",
    },
    BPartSpec {
        kind: BPartType::Toolbar,
        label: "Toolbar",
    },
    BPartSpec {
        kind: BPartType::Sidebar,
        label: "Sidebar",
    },
    BPartSpec {
        kind: BPartType::Panel,
        label: "Panel",
    },
    BPartSpec {
        kind: BPartType::Channels,
        label: "Channels",
    },
    BPartSpec {
        kind: BPartType::Statusbar,
        label: "Status Bar",
    },
    BPartSpec {
        kind: BPartType::Zonebar,
        label: "Zone Visibility",
    },
    BPartSpec {
        kind: BPartType::NavBar,
        label: "Nav Bar",
    },
    BPartSpec {
        kind: BPartType::HUD,
        label: "HUD",
    },
    BPartSpec {
        kind: BPartType::Execute,
        label: "Execute",
    },
    BPartSpec {
        kind: BPartType::AssetShelf,
        label: "Asset Shelf",
    },
    BPartSpec {
        kind: BPartType::ToolProps,
        label: "Tool Properties",
    },
    BPartSpec {
        kind: BPartType::AdjustLastOp,
        label: "Adjust Last Op",
    },
    BPartSpec {
        kind: BPartType::Scrubbing,
        label: "Scrubbing",
    },
    BPartSpec {
        kind: BPartType::Activity,
        label: "Activity",
    },
    BPartSpec {
        kind: BPartType::Custom,
        label: "Custom",
    },
];
