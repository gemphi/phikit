//! Splash / Welcome set — the static columns of the welcome window.
//!
//! Blender ref: `WM_MT_splash` (`scripts/startup/bl_operators/wm.py:3431`):
//!   split 1 — col1 "New File" (templates)        │ col2 "Recent Files"
//!             (5 rows + "More…") or, with no recents, "Getting Started" links
//!   split 2 — col1 "Open…", "Recover Last Session" │ col2 "What's New", donate
//! Branding is Phiber's; structure and labels are Blender's.

use crate::v2::state::{BKey as _, AppKey, ModalKey};
use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

/// One clickable row in the welcome window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BSplashEntry {
    pub icon: &'static str,
    pub label: &'static str,
    /// Target: a space slug for a template, `doc:<slug>` for a document the
    /// reader opens, a URL for a link that leaves the app.
    pub target: &'static str,
}

impl BSplashEntry {
    /// The action a link entry takes: a document opens in the reader, a URL
    /// goes to the browser.
    pub fn link_action(&self) -> crate::v2::view::prim::BAction {
        link_action(self.target)
    }
}

/// The action for a link target: `doc:<slug>` opens the reader, anything
/// else is handed to the browser.
pub fn link_action(target: &str) -> crate::v2::view::prim::BAction {
        use crate::v2::view::prim::BAction;
    match crate::v2::modal::reader::doc_target(target) {
        Some(slug) => BAction::Set(ModalKey::ReaderDoc.path(), slug.to_string()),
        None => BAction::Set(AppKey::Link.path(), target.to_string()),
    }
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

/// Product name shown in the banner and window title.
pub const APP_NAME: &str = "Phiber";

/// "New File" column — each entry switches to the workspace it names.
/// Blender lists its app templates here (General, 2D Animation, Sculpting,
/// VFX, Video Editing); Phiber offers the workspaces it actually ships.
pub static TEMPLATES: &[BSplashEntry] = &[
    e("layout", "General", "Layout"),
    e("shader", "Shading", "Shading"),
    e("uv", "UV Lights", "UvEditing"),
    e("keyframe", "Animation", "Animation"),
    e("render", "Rendering", "Rendering"),
    // Third field is the TARGET — the space's slug, written to `ctx.space`.
    // It is not the display label: renaming the space to "ECS Wand" changed
    // the label only, so the target stays "Compositing".
    e("compositor", "ECS Wand", "Compositing"),
    e("geonode", "Geometry Nodes", "GeometryNodes"),
    e("script", "Scripting", "Scripting"),
];

/// "Getting Started" — shown in place of the recent list when it is empty
/// (`WM_MT_splash`, L3462-3468). Labels are Blender's.
/// Every entry but the Phiber website reads in the app (`doc:` targets
/// open the reader); only Phiber's own site leaves it.
pub static GETTING_STARTED: &[BSplashEntry] = &[
    e("manual", "Manual", "doc:manual"),
    e("support", "Support", "doc:support"),
    e("community", "User Communities", "doc:community"),
    e("involve", "Get Involved", "doc:get_involved"),
    e("url", "Phiber Website", "https://phiace.dev"),
];

/// Second split, right column (`WM_MT_splash`, L3485-3486).
pub static FOOT_LINKS: &[BSplashEntry] = &[
    e("release", "What's New", "doc:whats_new"),
    e("heart", "Support Phiber", "https://phiace.dev/support"),
];

/// Kept for callers that still import the old name.
pub static LINKS: &[BSplashEntry] = GETTING_STARTED;

/// How many recent files the welcome window lists — `template_recent_files(rows=5)`.
pub const RECENT_ROWS: usize = 5;
/// How many recent files are remembered. Blender's `FSMENU_RECENT_MAX`.
pub const RECENT_MAX: usize = 10;

pub const EMPTY_RECENT: &str = "No recent files";
