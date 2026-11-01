//! Stack view struct, constructors, property accessors, and view management.

use crate::graphics::{NSEdgeInsets, NSRect, NSUserInterfaceLayoutOrientation};
use crate::view::NSView;
use crate::NSLayoutAttribute;
use std::sync::Arc;

use super::stack_layout::{self, NSStackViewDelegate};

/// Stack view distribution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NSStackViewDistribution {
    Fill,
    FillEqually,
    FillProportionally,
    EqualSpacing,
}

#[derive(Clone)]
pub struct NSStackView {
    view: NSView,
    orientation: NSUserInterfaceLayoutOrientation,
    distribution: NSStackViewDistribution,
    spacing: f32,
    edge_insets: NSEdgeInsets,
    alignment: NSLayoutAttribute,
    arranged_subviews: Arc<std::sync::Mutex<Vec<NSView>>>,
}

impl NSStackView {
    pub fn new(frame: NSRect) -> Self {
        let arranged_subviews = Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut view = NSView::new(frame);
        view.set_delegate(NSStackViewDelegate {
            orientation: NSUserInterfaceLayoutOrientation::Horizontal,
            spacing: 8.0,
            edge_insets: NSEdgeInsets::new(0.0, 0.0, 0.0, 0.0),
            alignment: NSLayoutAttribute::Leading,
            arranged_subviews: arranged_subviews.clone(),
        });
        Self {
            view,
            orientation: NSUserInterfaceLayoutOrientation::Horizontal,
            distribution: NSStackViewDistribution::Fill,
            spacing: 8.0,
            edge_insets: NSEdgeInsets::new(0.0, 0.0, 0.0, 0.0),
            alignment: NSLayoutAttribute::Leading,
            arranged_subviews,
        }
    }

    /// Create a stack view controller from an existing view handle
    pub fn from_view(view: NSView) -> Self {
        let arranged_subviews = Arc::new(std::sync::Mutex::new(Vec::new()));
        Self {
            view,
            orientation: NSUserInterfaceLayoutOrientation::Horizontal,
            distribution: NSStackViewDistribution::Fill,
            spacing: 8.0,
            edge_insets: NSEdgeInsets::new(0.0, 0.0, 0.0, 0.0),
            alignment: NSLayoutAttribute::Leading,
            arranged_subviews,
        }
    }

    /// Create a horizontal stack view (convenience constructor)
    pub fn horizontal() -> Self {
        Self::new(NSRect::zero())
    }

    /// Create a horizontal stack view with spacing
    pub fn horizontal_with_spacing(spacing: f32) -> Self {
        let mut stack = Self::horizontal();
        stack.set_spacing(spacing);
        stack
    }

    /// Create a vertical stack view (convenience constructor)
    pub fn vertical() -> Self {
        let mut stack = Self::new(NSRect::zero());
        stack.set_orientation(NSUserInterfaceLayoutOrientation::Vertical);
        stack
    }

    /// Create a vertical stack view with spacing
    pub fn vertical_with_spacing(spacing: f32) -> Self {
        let mut stack = Self::vertical();
        stack.set_spacing(spacing);
        stack
    }

    pub fn orientation(&self) -> NSUserInterfaceLayoutOrientation { self.orientation }
    pub fn set_orientation(&mut self, orientation: NSUserInterfaceLayoutOrientation) {
        self.orientation = orientation;
        self.synchronize_delegate();
    }

    pub fn distribution(&self) -> NSStackViewDistribution { self.distribution }
    pub fn set_distribution(&mut self, dist: NSStackViewDistribution) {
        self.distribution = dist;
        self.synchronize_delegate();
    }

    pub fn spacing(&self) -> f32 { self.spacing }
    pub fn set_spacing(&mut self, spacing: f32) {
        self.spacing = spacing;
        self.synchronize_delegate();
    }

    pub fn edge_insets(&self) -> NSEdgeInsets { self.edge_insets }
    pub fn set_edge_insets(&mut self, insets: NSEdgeInsets) {
        self.edge_insets = insets;
        self.synchronize_delegate();
    }

    pub fn alignment(&self) -> NSLayoutAttribute { self.alignment }
    pub fn set_alignment(&mut self, alignment: NSLayoutAttribute) {
        self.alignment = alignment;
        self.synchronize_delegate();
    }

    fn synchronize_delegate(&mut self) {
        let delegate = NSStackViewDelegate {
            orientation: self.orientation,
            spacing: self.spacing,
            edge_insets: self.edge_insets,
            alignment: self.alignment,
            arranged_subviews: self.arranged_subviews.clone(),
        };
        self.view.set_delegate(delegate);
        self.update_layout();
    }

    pub fn add_arranged_subview(&mut self, view: NSView) {
        self.arranged_subviews.lock().unwrap().push(view.clone());
        self.view.add_subview(view);
        self.update_layout();
    }

    pub fn remove_all_arranged_subviews(&mut self) {
        self.arranged_subviews.lock().unwrap().clear();
        self.view.remove_all_subviews();
        self.update_layout();
    }
    pub fn view(&self) -> &NSView { &self.view }
    pub fn view_mut(&mut self) -> &mut NSView { &mut self.view }
    pub fn view_clone(&self) -> NSView { self.view.clone() }

    /// Set the frame and re-layout children.
    pub fn with_frame(mut self, frame: NSRect) -> Self {
        self.view.set_frame(frame);
        self.update_layout();
        self
    }

    pub fn update_layout(&mut self) {
        let mut view_clone = self.view.clone();
        let mut arranged = self.arranged_subviews.lock().unwrap();
        Self::perform_layout(
            &mut view_clone,
            &mut arranged,
            self.orientation,
            self.spacing,
            self.edge_insets,
            self.alignment,
        );
        self.view = view_clone;
    }

    pub(crate) fn perform_layout(
        view: &mut NSView,
        arranged_subviews: &mut Vec<NSView>,
        orientation: NSUserInterfaceLayoutOrientation,
        spacing: f32,
        edge_insets: NSEdgeInsets,
        alignment: NSLayoutAttribute,
    ) {
        stack_layout::perform_layout(view, arranged_subviews, orientation, spacing, edge_insets, alignment);
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // Builder Pattern Methods
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Set the orientation using builder pattern
    pub fn with_orientation(mut self, orientation: NSUserInterfaceLayoutOrientation) -> Self {
        self.set_orientation(orientation);
        self
    }
    
    /// Set the distribution using builder pattern
    pub fn with_distribution(mut self, distribution: NSStackViewDistribution) -> Self {
        self.set_distribution(distribution);
        self
    }
    
    /// Set the spacing using builder pattern
    pub fn with_spacing(mut self, spacing: f32) -> Self {
        self.set_spacing(spacing);
        self
    }
    
    /// Set the edge insets using builder pattern
    pub fn with_edge_insets(mut self, insets: NSEdgeInsets) -> Self {
        self.set_edge_insets(insets);
        self
    }
    
    /// Set uniform padding on all sides using builder pattern
    pub fn with_padding(mut self, padding: f32) -> Self {
        self.set_edge_insets(NSEdgeInsets::new(padding, padding, padding, padding));
        self
    }
    
    /// Set the alignment using builder pattern
    pub fn with_alignment(mut self, alignment: NSLayoutAttribute) -> Self {
        self.set_alignment(alignment);
        self
    }
    
    /// Set the name using builder pattern
    pub fn with_name(mut self, name: &str) -> Self {
        self.view.set_name(name);
        self
    }
    
    /// Set the background color using builder pattern
    pub fn with_background_color(mut self, color: Option<crate::graphics::NSColor>) -> Self {
        self.view.set_background_color(color);
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PStack — Phi Stack (Configurable Custom Stack)
// ═══════════════════════════════════════════════════════════════════════

/// Create a vertical PStack (Phi Stack) - convenience constructor
pub fn pstack_vertical() -> NSStackView {
    NSStackView::vertical()
}

/// Create a horizontal PStack (Phi Stack) - convenience constructor  
pub fn pstack_horizontal() -> NSStackView {
    NSStackView::horizontal()
}

/// Create a PStack with custom configuration
pub fn pstack() -> NSStackView {
    NSStackView::new(NSRect::zero())
}

