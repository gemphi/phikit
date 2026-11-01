//! NSSplitViewItem — wraps a view with split behavior metadata.

use crate::NSView;

/// How a split view item behaves during layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSSplitViewItemBehavior {
    /// Left sidebar — fixed preferred width, collapsible.
    Sidebar,
    /// Main content — fills remaining space, not collapsible.
    Content,
    /// Right inspector — fixed preferred width, collapsible.
    Inspector,
}

/// A single item in an NSSplitViewController.
#[derive(Clone)]
pub struct NSSplitViewItem {
    pub view: NSView,
    pub behavior: NSSplitViewItemBehavior,
    pub is_collapsed: bool,
    pub preferred_thickness: f32,
    pub minimum_thickness: f32,
    pub maximum_thickness: f32,
    pub can_collapse: bool,
}

impl NSSplitViewItem {
    /// Create a generic item with default content behavior.
    pub fn new(view: NSView) -> Self {
        Self {
            view,
            behavior: NSSplitViewItemBehavior::Content,
            is_collapsed: false,
            preferred_thickness: 0.0,
            minimum_thickness: 100.0,
            maximum_thickness: f32::INFINITY,
            can_collapse: false,
        }
    }

    /// Create a sidebar item (left panel, collapsible).
    pub fn sidebar(view: NSView) -> Self {
        Self {
            view,
            behavior: NSSplitViewItemBehavior::Sidebar,
            is_collapsed: false,
            preferred_thickness: 260.0,
            minimum_thickness: 200.0,
            maximum_thickness: 400.0,
            can_collapse: true,
        }
    }

    /// Create a content item (center, fills space, not collapsible).
    pub fn content(view: NSView) -> Self {
        Self {
            view,
            behavior: NSSplitViewItemBehavior::Content,
            is_collapsed: false,
            preferred_thickness: 0.0,
            minimum_thickness: 300.0,
            maximum_thickness: f32::INFINITY,
            can_collapse: false,
        }
    }

    /// Create an inspector item (right panel, collapsible).
    pub fn inspector(view: NSView) -> Self {
        Self {
            view,
            behavior: NSSplitViewItemBehavior::Inspector,
            is_collapsed: false,
            preferred_thickness: 260.0,
            minimum_thickness: 200.0,
            maximum_thickness: 400.0,
            can_collapse: true,
        }
    }

    /// Set preferred thickness.
    pub fn with_preferred_thickness(mut self, thickness: f32) -> Self {
        self.preferred_thickness = thickness;
        self
    }

    /// Set minimum thickness.
    pub fn with_minimum_thickness(mut self, thickness: f32) -> Self {
        self.minimum_thickness = thickness;
        self
    }

    /// Set collapsed state.
    pub fn collapsed(mut self, collapsed: bool) -> Self {
        self.is_collapsed = collapsed;
        self
    }
}
