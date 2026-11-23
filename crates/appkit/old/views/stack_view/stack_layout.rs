//! Stack view layout logic — performs horizontal and vertical arrangement of subviews.

use crate::graphics::{NSAutoresizingMask, NSEdgeInsets, NSRect, NSUserInterfaceLayoutOrientation};
use crate::view::NSView;
use crate::NSLayoutAttribute;
use std::sync::Arc;

use super::stack_rendering::NSStackView;

/// Delegate that performs automatic layout for the stack view's arranged subviews.
pub struct NSStackViewDelegate {
    pub orientation: NSUserInterfaceLayoutOrientation,
    pub spacing: f32,
    pub edge_insets: NSEdgeInsets,
    pub alignment: NSLayoutAttribute,
    pub arranged_subviews: Arc<std::sync::Mutex<Vec<NSView>>>,
}

impl crate::view::NSViewDelegate for NSStackViewDelegate {
    fn layout(&mut self, view: &mut NSView) {
        let mut arranged = self.arranged_subviews.lock().unwrap();
        NSStackView::perform_layout(
            view,
            &mut arranged,
            self.orientation,
            self.spacing,
            self.edge_insets,
            self.alignment,
        );
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Performs layout of arranged subviews within the given view frame.
///
/// Distributes subviews along the primary axis (horizontal or vertical),
/// respecting spacing, edge insets, alignment, and flex sizing.
pub(crate) fn perform_layout(
    view: &mut NSView,
    arranged_subviews: &mut Vec<NSView>,
    orientation: NSUserInterfaceLayoutOrientation,
    spacing: f32,
    edge_insets: NSEdgeInsets,
    alignment: NSLayoutAttribute,
) {
    let frame = view.frame();
    let available_width = frame.width() - edge_insets.left - edge_insets.right;
    let available_height = frame.height() - edge_insets.top - edge_insets.bottom;

    if arranged_subviews.is_empty() {
        return;
    }

    match orientation {
        NSUserInterfaceLayoutOrientation::Horizontal => {
            layout_horizontal(arranged_subviews, spacing, edge_insets, alignment, available_width, available_height);
        }
        NSUserInterfaceLayoutOrientation::Vertical => {
            layout_vertical(arranged_subviews, spacing, edge_insets, alignment, available_width, available_height);
        }
    }

    view.remove_all_subviews();
    for sub in arranged_subviews {
        view.add_subview(sub.clone());
    }
}

fn layout_horizontal(
    arranged_subviews: &mut Vec<NSView>,
    spacing: f32,
    edge_insets: NSEdgeInsets,
    alignment: NSLayoutAttribute,
    available_width: f32,
    available_height: f32,
) {
    let total_spacing = spacing * (arranged_subviews.len().saturating_sub(1) as f32);
    let available_for_views = (available_width - total_spacing).max(0.0);
    let flex_count = arranged_subviews
        .iter()
        .filter(|v| v.autoresizing_mask().contains(NSAutoresizingMask::WIDTH_SIZABLE))
        .count();
    let flex_width = if flex_count > 0 {
        let total_static: f32 = arranged_subviews
            .iter()
            .filter(|v| !v.autoresizing_mask().contains(NSAutoresizingMask::WIDTH_SIZABLE))
            .map(|v| v.frame().width())
            .sum();
        ((available_for_views - total_static).max(0.0) / flex_count as f32).max(10.0)
    } else {
        0.0
    };

    let mut x = edge_insets.left;
    for i in 0..arranged_subviews.len() {
        let is_flex = arranged_subviews[i]
            .autoresizing_mask()
            .contains(NSAutoresizingMask::WIDTH_SIZABLE);
        let v_width = if is_flex { flex_width } else { arranged_subviews[i].frame().width() };
        let v_height = match alignment {
            NSLayoutAttribute::Fill => available_height,
            _ => arranged_subviews[i].frame().height().min(available_height),
        };
        let y_offset = match alignment {
            NSLayoutAttribute::CenterY => (available_height - v_height) / 2.0,
            NSLayoutAttribute::Bottom => available_height - v_height,
            _ => 0.0,
        };
        arranged_subviews[i].set_frame(NSRect::new(x, edge_insets.top + y_offset, v_width, v_height));
        x += v_width + spacing;
    }
}

fn layout_vertical(
    arranged_subviews: &mut Vec<NSView>,
    spacing: f32,
    edge_insets: NSEdgeInsets,
    alignment: NSLayoutAttribute,
    available_width: f32,
    available_height: f32,
) {
    let total_spacing = spacing * (arranged_subviews.len().saturating_sub(1) as f32);
    let available_for_views = (available_height - total_spacing).max(0.0);
    let flex_count = arranged_subviews
        .iter()
        .filter(|v| v.autoresizing_mask().contains(NSAutoresizingMask::HEIGHT_SIZABLE))
        .count();
    let flex_height = if flex_count > 0 {
        let total_static: f32 = arranged_subviews
            .iter()
            .filter(|v| !v.autoresizing_mask().contains(NSAutoresizingMask::HEIGHT_SIZABLE))
            .map(|v| v.frame().height())
            .sum();
        ((available_for_views - total_static).max(0.0) / flex_count as f32).max(10.0)
    } else {
        0.0
    };

    let total_content_height: f32 = arranged_subviews.iter().map(|v| {
        let is_flex = v.autoresizing_mask().contains(NSAutoresizingMask::HEIGHT_SIZABLE);
        if is_flex { flex_height } else { v.frame().height() }
    }).sum::<f32>() + spacing * (arranged_subviews.len().saturating_sub(1) as f32);

    let y_start_offset = if total_content_height < available_height {
        (available_height - total_content_height) / 2.0
    } else {
        0.0
    };
    let mut y = edge_insets.top + y_start_offset;
    for i in 0..arranged_subviews.len() {
        let is_flex = arranged_subviews[i]
            .autoresizing_mask()
            .contains(NSAutoresizingMask::HEIGHT_SIZABLE);
        let v_height = if is_flex { flex_height } else { arranged_subviews[i].frame().height() };
        let v_width = match alignment {
            NSLayoutAttribute::Fill => available_width,
            _ => arranged_subviews[i].frame().width().min(available_width),
        };
        let x_offset = match alignment {
            NSLayoutAttribute::CenterX => (available_width - v_width) / 2.0,
            NSLayoutAttribute::Trailing => available_width - v_width,
            _ => 0.0,
        };
        arranged_subviews[i].set_frame(NSRect::new(edge_insets.left + x_offset, y, v_width, v_height));
        y += v_height + spacing;
    }
}
