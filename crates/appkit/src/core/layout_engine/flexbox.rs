//! # Flexbox Layout Resolver
//!
//! Takes a `ViewNode` tree and an available `NSRect`, then computes
//! absolute frames for every node. Implements a simplified flexbox model:
//!
//! - VStack: distributes children vertically with spacing
//! - HStack: distributes children horizontally with spacing
//! - ZStack: overlays children in the same frame
//! - Spacer: fills remaining space in a stack
//! - Fixed sizes are honored exactly
//! - Flexible sizes share remaining space equally
//! - Fraction sizes take a proportion of the parent
//! - Padding (Edges) is applied before laying out children
//!
//! Coordinate system: top-left origin (y=0 at top, increases downward).

use super::constraints::{Edges, LayoutResult, Size, SizeDimension, ViewId};
use super::view_node::ViewNode;
use crate::graphics::NSRect;

/// The flexbox layout engine.
#[derive(Debug, Clone)]
pub struct FlexboxEngine;

impl FlexboxEngine {
    pub fn new() -> Self {
        Self
    }

    /// Resolve a ViewNode tree into absolute frames within the given rect.
    pub fn layout(&self, root: &ViewNode, available: NSRect) -> LayoutResult {
        let mut result = LayoutResult::new();
        self.resolve(root, available, &mut result);
        result
    }

    /// Recursive resolver — computes the frame for `node` within `available`,
    /// pushes it to `result`, then recurses into children.
    fn resolve(&self, node: &ViewNode, available: NSRect, result: &mut LayoutResult) {
        match node {
            ViewNode::VStack {
                id,
                children,
                spacing,
                padding,
                size,
            } => {
                let frame = self.compute_frame(*size, available);
                result.push(*id, frame);
                let inner = self.apply_padding(frame, *padding);
                self.layout_vstack(children, *spacing, inner, result);
            }
            ViewNode::HStack {
                id,
                children,
                spacing,
                padding,
                size,
            } => {
                let frame = self.compute_frame(*size, available);
                result.push(*id, frame);
                let inner = self.apply_padding(frame, *padding);
                self.layout_hstack(children, *spacing, inner, result);
            }
            ViewNode::ZStack {
                id,
                children,
                padding,
                size,
            } => {
                let frame = self.compute_frame(*size, available);
                result.push(*id, frame);
                let inner = self.apply_padding(frame, *padding);
                self.layout_zstack(children, inner, result);
            }
            ViewNode::Text { id, size, .. } => {
                let frame = self.compute_frame(*size, available);
                result.push(*id, frame);
            }
            ViewNode::Button { id, size, .. } => {
                let frame = self.compute_frame(*size, available);
                result.push(*id, frame);
            }
            ViewNode::Image { id, size, .. } => {
                let frame = self.compute_frame(*size, available);
                result.push(*id, frame);
            }
            ViewNode::Spacer { id, .. } => {
                // Spacers get their frame assigned by the parent stack.
                // If resolved standalone, they take the full available rect.
                result.push(*id, available);
            }
            ViewNode::Panel {
                id,
                child,
                padding,
                size,
                ..
            } => {
                let frame = self.compute_frame(*size, available);
                result.push(*id, frame);
                let inner = self.apply_padding(frame, *padding);
                self.resolve(child, inner, result);
            }
            ViewNode::Custom { id, size, .. } => {
                let frame = self.compute_frame(*size, available);
                result.push(*id, frame);
            }
            ViewNode::ForEach {
                id,
                children,
                spacing,
            } => {
                // ForEach behaves like a VStack for layout purposes.
                result.push(*id, available);
                self.layout_vstack(children, *spacing, available, result);
            }
        }
    }

    /// Lay out children vertically within `inner` rect.
    fn layout_vstack(
        &self,
        children: &[ViewNode],
        spacing: f32,
        inner: NSRect,
        result: &mut LayoutResult,
    ) {
        if children.is_empty() {
            return;
        }

        let total_spacing = spacing * (children.len() as f32 - 1.0).max(0.0);
        let available_height = (inner.height() - total_spacing).max(0.0);

        // First pass: measure fixed children, count flexible ones.
        let mut fixed_total = 0.0f32;
        let mut flexible_count = 0u32;

        for child in children {
            match self.child_height(child, inner.height()) {
                ChildMeasure::Fixed(h) => fixed_total += h,
                ChildMeasure::Flexible => flexible_count += 1,
            }
        }

        let remaining = (available_height - fixed_total).max(0.0);
        let flex_height = if flexible_count > 0 {
            remaining / flexible_count as f32
        } else {
            0.0
        };

        // Second pass: assign frames.
        let mut y = inner.y();
        for child in children {
            let child_h = match self.child_height(child, inner.height()) {
                ChildMeasure::Fixed(h) => h,
                ChildMeasure::Flexible => flex_height,
            };

            let child_rect = NSRect::new(inner.x(), y, inner.width(), child_h);
            self.resolve(child, child_rect, result);
            y += child_h + spacing;
        }
    }

    /// Lay out children horizontally within `inner` rect.
    fn layout_hstack(
        &self,
        children: &[ViewNode],
        spacing: f32,
        inner: NSRect,
        result: &mut LayoutResult,
    ) {
        if children.is_empty() {
            return;
        }

        let total_spacing = spacing * (children.len() as f32 - 1.0).max(0.0);
        let available_width = (inner.width() - total_spacing).max(0.0);

        // First pass: measure fixed children, count flexible ones.
        let mut fixed_total = 0.0f32;
        let mut flexible_count = 0u32;

        for child in children {
            match self.child_width(child, inner.width()) {
                ChildMeasure::Fixed(w) => fixed_total += w,
                ChildMeasure::Flexible => flexible_count += 1,
            }
        }

        let remaining = (available_width - fixed_total).max(0.0);
        let flex_width = if flexible_count > 0 {
            remaining / flexible_count as f32
        } else {
            0.0
        };

        // Second pass: assign frames.
        let mut x = inner.x();
        for child in children {
            let child_w = match self.child_width(child, inner.width()) {
                ChildMeasure::Fixed(w) => w,
                ChildMeasure::Flexible => flex_width,
            };

            let child_rect = NSRect::new(x, inner.y(), child_w, inner.height());
            self.resolve(child, child_rect, result);
            x += child_w + spacing;
        }
    }

    /// Lay out children overlaid in the same frame.
    fn layout_zstack(&self, children: &[ViewNode], inner: NSRect, result: &mut LayoutResult) {
        for child in children {
            self.resolve(child, inner, result);
        }
    }

    /// Compute the frame for a node given its Size spec and available rect.
    fn compute_frame(&self, size: Size, available: NSRect) -> NSRect {
        let w = self.resolve_dimension(size.width, available.width());
        let h = self.resolve_dimension(size.height, available.height());
        NSRect::new(available.x(), available.y(), w, h)
    }

    /// Resolve a single SizeDimension to a concrete pixel value.
    fn resolve_dimension(&self, dim: SizeDimension, available: f32) -> f32 {
        match dim {
            SizeDimension::Fixed(v) => v,
            SizeDimension::Flexible => available,
            SizeDimension::Fraction(f) => available * f.clamp(0.0, 1.0),
            SizeDimension::Min(min) => available.max(min),
        }
    }

    /// Inset a rect by the given edges.
    fn apply_padding(&self, rect: NSRect, padding: Edges) -> NSRect {
        NSRect::new(
            rect.x() + padding.left,
            rect.y() + padding.top,
            (rect.width() - padding.horizontal()).max(0.0),
            (rect.height() - padding.vertical()).max(0.0),
        )
    }

    /// Determine the height contribution of a child in a VStack.
    fn child_height(&self, node: &ViewNode, parent_height: f32) -> ChildMeasure {
        match node {
            ViewNode::Spacer { min_length, .. } => {
                if *min_length > 0.0 {
                    // Spacer with min_length acts as flexible but with a minimum.
                    // For simplicity, treat as flexible (the flex distribution
                    // will give it at least its share).
                    ChildMeasure::Flexible
                } else {
                    ChildMeasure::Flexible
                }
            }
            _ => {
                let size = node.size();
                match size.height {
                    SizeDimension::Fixed(h) => ChildMeasure::Fixed(h),
                    SizeDimension::Fraction(f) => {
                        ChildMeasure::Fixed(parent_height * f.clamp(0.0, 1.0))
                    }
                    SizeDimension::Min(_) | SizeDimension::Flexible => ChildMeasure::Flexible,
                }
            }
        }
    }

    /// Determine the width contribution of a child in an HStack.
    fn child_width(&self, node: &ViewNode, parent_width: f32) -> ChildMeasure {
        match node {
            ViewNode::Spacer { min_length, .. } => {
                if *min_length > 0.0 {
                    ChildMeasure::Flexible
                } else {
                    ChildMeasure::Flexible
                }
            }
            _ => {
                let size = node.size();
                match size.width {
                    SizeDimension::Fixed(w) => ChildMeasure::Fixed(w),
                    SizeDimension::Fraction(f) => {
                        ChildMeasure::Fixed(parent_width * f.clamp(0.0, 1.0))
                    }
                    SizeDimension::Min(_) | SizeDimension::Flexible => ChildMeasure::Flexible,
                }
            }
        }
    }
}

impl Default for FlexboxEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal measurement result for a child in a stack.
enum ChildMeasure {
    Fixed(f32),
    Flexible,
}
