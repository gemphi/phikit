//! Divider types, state, and hit-testing for `NSSplitView`.

use crate::{NSPoint, NSSize};
use std::sync::Arc;

/// Width of the divider hit region in pixels. A click within ±4px of a
/// divider center is considered a hit (total hit width = 8px).
pub(crate) const DIVIDER_HIT_WIDTH: f32 = 8.0;

/// Tracks the state of an in-progress divider drag operation.
///
/// Created when the user presses the mouse button within the hit region of a
/// divider, and cleared when the mouse button is released.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DividerDrag {
    /// Zero-based index of the divider being dragged.
    pub index: usize,
    /// Mouse position (in window coordinates) at the start of the drag.
    pub start_pos: NSPoint,
    /// Divider position (in split-view local coordinates) at the start of the drag.
    pub start_divider_pos: f32,
}

/// Visual style for the divider line between split-view panes.
///
/// Corresponds to [`NSSplitView.DividerStyle`](https://developer.apple.com/documentation/appkit/nssplitview/dividerstyle).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSSplitViewDividerStyle {
    Thick,
    Thin,
    PaneSplitter,
    None,
}

/// Callback type for divider change notifications.
/// Receives the current divider positions and the total width of the split view.
pub type DividerChangedCallback = Arc<dyn Fn(Vec<f32>, f32) + Send + Sync>;

/// Shared, thread-safe state that describes the split view's configuration.
///
/// Protected by a `Mutex` so that both the delegate's draw/layout calls and
/// the editor's action system can read/write divider positions safely.
pub struct NSSplitViewState {
    /// Visual style used when drawing divider lines.
    pub divider_style: NSSplitViewDividerStyle,
    /// `true` = children arranged left-to-right; `false` = top-to-bottom.
    /// Matches Swift's `NSSplitView.isVertical`.
    pub vertical: bool,
    /// Pixel positions of each divider along the split axis.
    /// `divider_positions[i]` marks the boundary between child `i` and `i+1`.
    pub divider_positions: Vec<f32>,
    /// Index of the divider currently being dragged, if any.
    pub dragging_divider: Option<usize>,
    /// Size of the view during the last layout pass.
    pub last_layout_size: Option<NSSize>,
    /// Optional callback invoked when divider positions change after a drag.
    /// Receives the current divider positions and the total width of the split view.
    pub on_divider_changed: Option<DividerChangedCallback>,
}

impl std::fmt::Debug for NSSplitViewState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NSSplitViewState")
            .field("divider_style", &self.divider_style)
            .field("vertical", &self.vertical)
            .field("divider_positions", &self.divider_positions)
            .field("dragging_divider", &self.dragging_divider)
            .field("last_layout_size", &self.last_layout_size)
            .field("on_divider_changed", &self.on_divider_changed.as_ref().map(|_| "..."))
            .finish()
    }
}
