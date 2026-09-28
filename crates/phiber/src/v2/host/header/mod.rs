//! Shared header shapes — declared once, used by every editor host.
//!
//! Blender gives every editor the same header order: the editor-type
//! switcher, then the mode selector where that editor has modes, then the
//! editor's own menus, all in the left group
//! (`ED_region_header` + `layout.template_header()`); the centre carries the
//! editor's search and the right end its filters and view controls.
//!
//! Twenty-four host files each carried their own copy of a header that said
//! something different from this — menus left, mode centre, switcher right —
//! and `region::complete_header` quietly rewrote it on every build. The
//! declaration now says what is actually rendered, in one place.

use crate::v2::pane::{BPaneSpec, BPaneType};
use crate::v2::part::BPartType;

/// The header every editor host shares. The Centre and Right panes are
/// declared empty on purpose: they are the flex slot that pushes the right
/// group to the far end, and `complete_header` fills them with the host's
/// search and filters when it has them.
pub static EDITOR: &[BPaneSpec] = &[
    BPaneSpec {
        kind: BPaneType::Left,
        parts: &[BPartType::Switcher, BPartType::Modebar, BPartType::Menubar],
    },
    BPaneSpec {
        kind: BPaneType::Center,
        parts: &[],
    },
    BPaneSpec {
        kind: BPaneType::Right,
        parts: &[],
    },
];

/// The tool header. Blender's `RGN_TYPE_TOOL_HEADER` carries the active
/// tool's settings, left-aligned — not a second copy of the mode dropdown,
/// which is what Image, UV and the Sequencer were each declaring.
pub static TOOL: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Left,
    parts: &[BPartType::ToolProps],
}];
