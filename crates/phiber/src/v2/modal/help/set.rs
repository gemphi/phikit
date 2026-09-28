//! Help modal — docs browser categories + entries.
//! Blender ref: `getting_started/help.rst` and the Help menu.

use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BHelpEntry {
    pub icon: &'static str,
    pub label: &'static str,
    /// `doc:<slug>` for a document the reader shows, a URL for a link that
    /// leaves the app, `operator:` for an action.
    pub target: &'static str,
}

impl BSpec for BHelpEntry {
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

const fn h(icon: &'static str, label: &'static str, url: &'static str) -> BHelpEntry {
    BHelpEntry { icon, label, target: url }
}

/// Blender's Help menu, read in the app: every entry opens its page in the
/// reader (`modal::reader::store`), which carries the manual's text.
pub static WEB_LINKS: &[BHelpEntry] = &[
    h("manual", "Manual", "doc:manual"),
    h("tutorials", "Tutorials", "doc:tutorials"),
    h("support", "Support", "doc:support"),
    h("community", "User Communities", "doc:community"),
    h("involve", "Get Involved", "doc:get_involved"),
    h("release", "What's New", "doc:whats_new"),
    h("release", "Release Notes", "doc:release_notes"),
    h("bug", "Report a Bug", "doc:report_bug"),
];

pub static ACTIONS: &[BHelpEntry] = &[h("info", "Save System Info", "operator:wm.sysinfo")];
