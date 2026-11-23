//! # NSOutlineView — Hierarchical Tree View
//!
//! A view that displays hierarchical data in an expandable tree structure.
//!
//! ## Apple Documentation
//! - [NSOutlineView](https://developer.apple.com/documentation/appkit/nsoutlineview)
//! - [NSOutlineViewDataSource](https://developer.apple.com/documentation/appkit/nsoutlineviewdatasource)
//! - [NSOutlineViewDelegate](https://developer.apple.com/documentation/appkit/nsoutlineviewdelegate)
//!
//! ## Usage
//!
//! ```rust,ignore
//! let mut outline = NSOutlineView::new(frame);
//! outline.set_data_source(MyDataSource);
//! // Data source provides hierarchical structure via number_of_children_of_item,
//! // child_of_item, and is_item_expandable methods
//! ```
//!
//! ## See Also
//! - [NSTableView](https://developer.apple.com/documentation/appkit/nstableview)
//! - [NSBrowser](https://developer.apple.com/documentation/appkit/nsbrowser)

// pub mod builder; (Replaced by NSOutlineViewDataSource/Delegate pattern)

use crate::graphics::{NSColor, NSPoint, NSRect};
use crate::view::NSView;
use std::sync::Arc;

pub trait NSOutlineViewDataSource: Send + Sync {
    fn number_of_children_of_item(&self, item: Option<&String>) -> usize;
    fn child_of_item(&self, item: Option<&String>, index: usize) -> String;
    fn is_item_expandable(&self, item: &String) -> bool;
    fn item_entity_id(&self, item: &String) -> Option<u64>;

    /// Returns an icon identifier for the given item (used for hierarchy type icons).
    fn icon_for_item(&self, _item: &String) -> Option<crate::svg_icons::IconId> {
        None
    }

    /// Called when an item is selected in the outline view
    fn outline_view_did_select_item(&mut self, _item: Option<&String>) {}

    /// Called when an item is expanded
    fn outline_view_item_did_expand(&mut self, _item: &String) {}

    /// Called when an item is collapsed
    fn outline_view_item_did_collapse(&mut self, _item: &String) {}
}

pub trait NSOutlineViewDelegate: Send + Sync {
    /// Called when selection changes
    fn outline_view_selection_did_change(&mut self, selected_item: Option<&String>);

    /// Called when an item is double-clicked
    fn outline_view_item_did_double_click(&mut self, item: &String);

    /// Called when an item will be expanded
    fn outline_view_item_will_expand(&mut self, _item: &String) -> bool {
        true
    }

    /// Called when an item will be collapsed
    fn outline_view_item_will_collapse(&mut self, _item: &String) -> bool {
        true
    }
}

pub struct NSOutlineView {
    view: NSView,
    data_source: Option<Arc<dyn NSOutlineViewDataSource>>,
}

impl NSOutlineView {
    pub fn new(frame: NSRect) -> Self {
        let view = NSView::new(frame).with_name("NSOutlineView");
        Self {
            view,
            data_source: None,
        }
    }

    pub fn set_data_source(&mut self, ds: impl NSOutlineViewDataSource + 'static) {
        let ds_shared = Arc::new(ds);
        self.data_source = Some(ds_shared.clone());

        let delegate = OutlineViewDelegate {
            data_source: ds_shared,
        };
        self.view.set_delegate(delegate);
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }
    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }
}

struct OutlineViewDelegate {
    data_source: Arc<dyn NSOutlineViewDataSource>,
}

impl crate::view::NSViewDelegate for OutlineViewDelegate {
    fn draw(&self, view: &NSView, ctx: &mut crate::render::RenderContext) {
        let bounds = view.bounds();
        let bg = crate::theme::color(crate::theme::keys::CLR_BG_CONTENT);
        ctx.set_color(bg);
        ctx.fill_rect(bounds);

        let mut y = bounds.min_y() + 8.0;
        let row_height = 24.0;
        let indent_step = 16.0;

        self.draw_recursive(
            None,
            0,
            &mut y,
            row_height,
            indent_step,
            ctx,
            bounds.width(),
        );
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl OutlineViewDelegate {
    fn draw_recursive(
        &self,
        parent: Option<&String>,
        depth: usize,
        y: &mut f32,
        row_height: f32,
        indent: f32,
        ctx: &mut crate::render::RenderContext,
        width: f32,
    ) {
        let count = self.data_source.number_of_children_of_item(parent);
        let line_color = crate::theme::color(crate::theme::keys::CLR_SEPARATOR);

        for i in 0..count {
            let child = self.data_source.child_of_item(parent, i);
            let x = 8.0 + depth as f32 * indent;
            let is_last = i == count - 1;

            // Draw tree connector lines (vertical + horizontal branch)
            if depth > 0 {
                ctx.set_color(line_color);
                let line_x = x - indent / 2.0;
                // Vertical line from parent down to this item
                ctx.fill_rect(NSRect::new(line_x, *y, 1.0, row_height / 2.0));
                // Horizontal branch to the icon
                ctx.fill_rect(NSRect::new(line_x, *y + row_height / 2.0 - 0.5, indent / 2.0 - 2.0, 1.0));

                // Continue vertical line down if not last sibling
                if !is_last {
                    ctx.fill_rect(NSRect::new(line_x, *y + row_height / 2.0, 1.0, row_height / 2.0));
                }
            }

            // Draw entity-type icon if the data source provides one
            let icon_size: usize = 14;
            if let Some(icon_id) = self.data_source.icon_for_item(&child) {
                // Use the RenderContext offset to position icons relative to the view
                let icon_x = (x + ctx.offset.x) as usize;
                let icon_y = (*y + (row_height - icon_size as f32) / 2.0 + ctx.offset.y) as usize;
                let tint_color = crate::theme::color(crate::theme::keys::CLR_TEXT_SECONDARY);
                let tint = (((tint_color.r * 255.0) as u32) << 16)
                    | (((tint_color.g * 255.0) as u32) << 8)
                    | ((tint_color.b * 255.0) as u32);
                let buf_width = ctx.width;
                let buffer = ctx.buffer_mut();
                crate::svg_icons::render_icon_to_buffer(
                    &icon_id, buffer, buf_width, icon_x, icon_y, icon_size, tint,
                );
            }

            // Draw item label after the icon
            ctx.set_color(NSColor::system_label());
            ctx.draw_text(&child, NSPoint::new(x + 18.0, *y + 4.0), 13.0);

            *y += row_height;

            if self.data_source.is_item_expandable(&child) {
                // Draw continuing vertical line for children
                let child_start_y = *y;
                let child_count = self.data_source.number_of_children_of_item(Some(&child));
                self.draw_recursive(Some(&child), depth + 1, y, row_height, indent, ctx, width);

                // Draw the vertical connector line spanning all children
                if child_count > 0 {
                    let line_x = 8.0 + (depth + 1) as f32 * indent - indent / 2.0;
                    let line_height = *y - child_start_y - row_height / 2.0;
                    if line_height > 0.0 {
                        ctx.set_color(line_color);
                        ctx.fill_rect(NSRect::new(line_x, child_start_y, 1.0, line_height));
                    }
                }
            }
        }
    }
}
