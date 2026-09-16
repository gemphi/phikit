//! Help modal constants.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BHelpType {
    Documentation,
    Tutorials,
    KeyboardShortcuts,
    ApiReference,
    CommunityForum,
    ReportBug,
}

pub const HELP_DOCUMENTATION_LABEL: &str = "Documentation";
pub const HELP_TUTORIALS_LABEL: &str = "Tutorials";
pub const HELP_KEYBOARD_SHORTCUTS_LABEL: &str = "Keyboard Shortcuts";
pub const HELP_API_REFERENCE_LABEL: &str = "API Reference";
pub const HELP_COMMUNITY_FORUM_LABEL: &str = "Community Forum";
pub const HELP_REPORT_BUG_LABEL: &str = "Report a Bug";

pub const HELP_DOCUMENTATION_ICON: &str = "book";
pub const HELP_TUTORIALS_ICON: &str = "play";
pub const HELP_KEYBOARD_SHORTCUTS_ICON: &str = "keyboard";
pub const HELP_API_REFERENCE_ICON: &str = "code";
pub const HELP_COMMUNITY_FORUM_ICON: &str = "users";
pub const HELP_REPORT_BUG_ICON: &str = "bug";

pub const HELP_DOCUMENTATION_DOC: &str = "Phiber is a next-generation 3D editor built on top of GPUI, Rust, and a data-driven BSpec hierarchy.\n\nThe main window is split into spaces, zones, areas, hosts, regions, panes, and parts. Each level is defined in store/ and rendered by the state/view pipeline.\n\nUse the menu bar to switch spaces, hosts, and modes.";
pub const HELP_TUTORIALS_DOC: &str = "Tutorials walk through common Phiber workflows.\n\n1. Open the View3D space to inspect the default scene.\n2. Use the Outliner to select objects.\n3. Switch hosts (3D View, Outliner, Properties) from the area header.\n\nMore tutorials will ship with v0.2.0.";
pub const HELP_KEYBOARD_SHORTCUTS_DOC: &str = "Keyboard shortcuts follow Blender-style bindings when possible.\n\n- Space: play/pause playback\n- G: grab/move selected\n- R: rotate\n- S: scale\n- Tab: switch object/edit mode\n- F1: open this Help modal";
pub const HELP_API_REFERENCE_DOC: &str = "The Phiber API is split into store, state, and view layers.\n\nStore crates define static specs. State crates track runtime values. The view layer (bui/) maps specs and state to GPUI views.\n\nSee the BSpec documentation for the full hierarchy.";
pub const HELP_COMMUNITY_FORUM_DOC: &str = "Join the Phiber community to share scenes, add-ons, and feedback.\n\n- Discord: coming soon\n- Forum: placeholder\n- GitHub: private beta";
pub const HELP_REPORT_BUG_DOC: &str = "Found a bug? Please include the following in your report:\n\n- Phiber version\n- Steps to reproduce\n- Expected and actual behavior\n- Screenshots if relevant";
