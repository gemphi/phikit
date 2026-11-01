//! # ViewHost
//!
//! Manages the lifecycle of a declarative view: holds the ViewBuilder,
//! watches for state changes, and rebuilds the NSView tree when dirty.
//!
//! ## Usage
//!
//! ```ignore
//! use rust_appkit::layout_engine::*;
//!
//! let screen = WelcomeScreen::default();
//! let mut host = ViewHost::new(Box::new(screen), available_rect);
//!
//! // On each frame:
//! if host.needs_rebuild() {
//!     let new_view = host.rebuild();
//!     // swap into the window's content view
//! }
//! ```

use super::renderer::render;
use super::state::StateGroup;
use super::view_builder::ViewBuilder;
use super::view_node::ViewNode;
use crate::graphics::NSRect;
use crate::view::NSView;

/// Hosts a declarative view, managing rebuild lifecycle.
pub struct ViewHost {
    /// The declarative view builder (implements body()).
    builder: Box<dyn ViewBuilder + Send + Sync>,
    /// The available rect for layout.
    available: NSRect,
    /// The last-rendered view tree (cached).
    cached_view: Option<NSView>,
    /// The last-computed ViewNode tree.
    cached_tree: Option<ViewNode>,
    /// State tracking group — monitors all observed state.
    pub state_group: StateGroup,
    /// Force rebuild on next call.
    force_rebuild: bool,
}

impl ViewHost {
    /// Create a new ViewHost with a builder and available rect.
    pub fn new(builder: Box<dyn ViewBuilder + Send + Sync>, available: NSRect) -> Self {
        Self {
            builder,
            available,
            cached_view: None,
            cached_tree: None,
            state_group: StateGroup::new(),
            force_rebuild: true, // first build is always needed
        }
    }

    /// Check if the view needs to be rebuilt (state changed or never built).
    pub fn needs_rebuild(&self) -> bool {
        self.force_rebuild || self.state_group.any_dirty()
    }

    /// Rebuild the view tree and return the new NSView.
    /// Cleans all dirty state flags after rebuild.
    pub fn rebuild(&mut self) -> NSView {
        let tree = self.builder.body();
        let view = render(&tree, self.available);
        self.cached_tree = Some(tree);
        self.cached_view = Some(view.clone());
        self.state_group.clean_all();
        self.force_rebuild = false;
        view
    }

    /// Get the cached view (last render), or rebuild if needed.
    pub fn view(&mut self) -> NSView {
        if self.needs_rebuild() {
            self.rebuild()
        } else {
            self.cached_view.clone().unwrap_or_else(|| self.rebuild())
        }
    }

    /// Update the available rect (e.g. on window resize) and force rebuild.
    pub fn set_available(&mut self, rect: NSRect) {
        if self.available != rect {
            self.available = rect;
            self.force_rebuild = true;
        }
    }

    /// Force a rebuild on the next frame.
    pub fn invalidate(&mut self) {
        self.force_rebuild = true;
    }

    /// Replace the builder (e.g. navigating to a different screen).
    pub fn set_builder(&mut self, builder: Box<dyn ViewBuilder + Send + Sync>) {
        self.builder = builder;
        self.force_rebuild = true;
        self.cached_view = None;
        self.cached_tree = None;
    }
}
