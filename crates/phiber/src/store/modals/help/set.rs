//! Help modal set - static spec arrays and label/icon/doc mappings.

use super::BHelpSpec;
use super::def::*;

pub static ALL: &[BHelpType] = &[
    BHelpType::Documentation,
    BHelpType::Tutorials,
    BHelpType::KeyboardShortcuts,
    BHelpType::ApiReference,
    BHelpType::CommunityForum,
    BHelpType::ReportBug,
];

pub static ALL_SPECS: &[BHelpSpec] = &[
    BHelpSpec::new(BHelpType::Documentation),
    BHelpSpec::new(BHelpType::Tutorials),
    BHelpSpec::new(BHelpType::KeyboardShortcuts),
    BHelpSpec::new(BHelpType::ApiReference),
    BHelpSpec::new(BHelpType::CommunityForum),
    BHelpSpec::new(BHelpType::ReportBug),
];

pub static LABELS: &[&str] = &[
    HELP_DOCUMENTATION_LABEL,
    HELP_TUTORIALS_LABEL,
    HELP_KEYBOARD_SHORTCUTS_LABEL,
    HELP_API_REFERENCE_LABEL,
    HELP_COMMUNITY_FORUM_LABEL,
    HELP_REPORT_BUG_LABEL,
];
pub static ICONS: &[&str] = &[
    HELP_DOCUMENTATION_ICON,
    HELP_TUTORIALS_ICON,
    HELP_KEYBOARD_SHORTCUTS_ICON,
    HELP_API_REFERENCE_ICON,
    HELP_COMMUNITY_FORUM_ICON,
    HELP_REPORT_BUG_ICON,
];
pub static DOCS: &[&str] = &[
    HELP_DOCUMENTATION_DOC,
    HELP_TUTORIALS_DOC,
    HELP_KEYBOARD_SHORTCUTS_DOC,
    HELP_API_REFERENCE_DOC,
    HELP_COMMUNITY_FORUM_DOC,
    HELP_REPORT_BUG_DOC,
];
