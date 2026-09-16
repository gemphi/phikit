//! Modal set — one `BModalSpec` per `BModalType`.

use super::{BModalSpec, BModalType};

// Order MUST match `BModalType` variant order.
pub static SET: &[BModalSpec] = &[
    BModalSpec {
        kind: BModalType::Settings,
        label: "Preferences",
        icon: "settings",
        width: 960.0,
        height: 640.0,
    },
    BModalSpec {
        kind: BModalType::Help,
        label: "Help",
        icon: "help",
        width: 900.0,
        height: 640.0,
    },
    BModalSpec {
        kind: BModalType::Profile,
        label: "Profile",
        icon: "profile",
        width: 480.0,
        height: 520.0,
    },
    BModalSpec {
        kind: BModalType::Command,
        label: "Command Palette",
        icon: "command",
        width: 640.0,
        height: 420.0,
    },
    BModalSpec {
        kind: BModalType::Splash,
        label: "Welcome to Phiber",
        icon: "phiber",
        width: 880.0,
        height: 560.0,
    },
    BModalSpec {
        kind: BModalType::About,
        label: "About Blender",
        icon: "about",
        width: 640.0,
        height: 480.0,
    },
    BModalSpec {
        kind: BModalType::FileBrowser,
        label: "Blender File View",
        icon: "folder",
        width: 1024.0,
        height: 640.0,
    },
];
