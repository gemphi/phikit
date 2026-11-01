//! Activity Bar view controller — configures NSActivityBar component.
//!
//! ## Apple Pattern
//! Follows Apple's view controller pattern where controllers are thin wrappers
//! that configure view components. Similar to:
//! - [`NSViewController`](https://developer.apple.com/documentation/appkit/nsviewcontroller)
//! - [`NSWindowController`](https://developer.apple.com/documentation/appkit/nswindowcontroller)
//!
//! ## Architecture
//! ```
//! NSActivityBarViewController (thin controller)
//!   └── NSActivityBar (view component - handles all UI)
//! ```
//!
//! ## See Also
//! - [NSViewController](https://developer.apple.com/documentation/appkit/nsviewcontroller)
//! - [View Controller Programming Guide](https://developer.apple.com/library/archive/featuredarticles/ViewControllerPGforiPhoneOS/)

use super::model::*;
use crate::views::activity_bar::{NSActivityBar, NSActivityBarItem, ActivityBarGroup};
use crate::prelude::*;

/// Activity bar view controller.
///
/// Thin wrapper around NSActivityBar component that configures it
/// with the editor's navigation items.
pub struct NSActivityBarViewController {
    pub activity_bar: NSActivityBar,
    pub state: NSActivityBarState,
}

impl NSActivityBarViewController {
    /// Create the activity bar for the given frame.
    pub fn new(frame: NSRect) -> Self {
        let state = NSActivityBarState::new();
        let activity_bar = Self::build_activity_bar(frame, &state);
        Self { activity_bar, state }
    }

    /// Rebuild the activity bar (e.g. after state change).
    pub fn rebuild(&mut self) {
        let frame = self.activity_bar.view().frame();
        self.activity_bar = Self::build_activity_bar(frame, &self.state);
    }

    /// Set the active item by id.
    pub fn set_active(&mut self, id: &'static str) {
        self.state.active_id = id;
        self.activity_bar.set_selected(id);
    }

    fn build_activity_bar(frame: NSRect, state: &NSActivityBarState) -> NSActivityBar {
        let mut bar = NSActivityBar::new(frame);
        
        // Add items from state
        for item in &state.items {
            let bar_item = NSActivityBarItem::new(item.id, item.icon_id.clone())
                .with_label(&item.label)
                .with_group(match item.group {
                    super::model::ActivityBarGroup::Top => ActivityBarGroup::Top,
                    super::model::ActivityBarGroup::Middle => ActivityBarGroup::Middle,
                    super::model::ActivityBarGroup::Bottom => ActivityBarGroup::Bottom,
                });
            bar.add_item(bar_item);
        }
        
        // Set active item
        bar.set_selected(state.active_id);
        
        bar
    }
}
