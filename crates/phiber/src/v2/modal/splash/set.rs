//! Splash / Welcome set — the static columns of the welcome window.
//! Blender ref: `WM_OT_splash` (`wm_splash_screen.cc`) — a New File column,
//! a Recent Files column and a row of links along the bottom.

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

/// One clickable row in the welcome window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BSplashEntry {
    pub icon: &'static str,
    pub label: &'static str,
    /// Target: a space slug for a template, a URL for a link.
    pub target: &'static str,
}

impl BSpec for BSplashEntry {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Custom)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

const fn e(icon: &'static str, label: &'static str, target: &'static str) -> BSplashEntry {
    BSplashEntry {
        icon,
        label,
        target,
    }
}

/// "New File" column — each entry switches to the workspace it names.
/// Only workspaces that are actually offered as tabs appear here.
pub static TEMPLATES: &[BSplashEntry] = &[
    e("layout", "General", "Layout"),
    e("shader", "Shading", "Shading"),
    e("uv", "UV Editing", "UvEditing"),
    e("keyframe", "Animation", "Animation"),
    e("render", "Rendering", "Rendering"),
    e("compositor", "Compositing", "Compositing"),
    e("geonode", "Geometry Nodes", "GeometryNodes"),
    e("script", "Scripting", "Scripting"),
];

/// Bottom link row.
pub static LINKS: &[BSplashEntry] = &[
    e("manual", "Manual", "https://docs.blender.org/manual/"),
    e(
        "tutorial",
        "Tutorials",
        "https://www.blender.org/support/tutorials/",
    ),
    e("life_buoy", "Support", "https://www.blender.org/support/"),
    e("users", "Community", "https://www.blender.org/community/"),
];

/// How many recent projects the welcome window lists. Blender's
/// `FSMENU_RECENT_MAX`.
pub const RECENT_MAX: usize = 10;

pub const EMPTY_RECENT: &str = "No recent projects yet — open one from File ▸ Open…";
