//! NSTableView rendering delegate.

use super::{NSTableColumn, NSTableViewDataSource};
use crate::theme::{self, keys};
use crate::view::NSViewDelegate;
use crate::{NSColor, NSPoint, NSRect, NSView};
use std::sync::{Arc, Mutex};

/// Internal delegate that renders the table content.
pub struct NSTableViewRenderDelegate {
    pub columns: Arc<Mutex<Vec<NSTableColumn>>>,
    pub selected_row: Arc<Mutex<Option<usize>>>,
    pub data_source: Arc<dyn NSTableViewDataSource>,
}

impl NSViewDelegate for NSTableViewRenderDelegate {
    fn draw(&self, view: &NSView, ctx: &mut crate::render::RenderContext) {
        let bounds = view.bounds();
        let columns = self.columns.lock().unwrap();
        let selected = *self.selected_row.lock().unwrap();
        let row_count = self.data_source.number_of_rows();

        let row_h = theme::f32_or(keys::SIZE_CONTROL_HEIGHT, 24.0);
        let header_h = theme::f32_or(keys::SIZE_TAB_HEIGHT, 28.0);
        let font_size = theme::f32_or(keys::SIZE_FONT_UI, 12.0);
        let padding_x = theme::f32_or(keys::SPACING_MD, 8.0);

        // Background
        ctx.set_color(theme::color(keys::CLR_BG_CONTENT));
        ctx.fill_rect(bounds);

        // Header row
        ctx.set_color(theme::color(keys::CLR_BG_PANEL));
        ctx.fill_rect(NSRect::sized(bounds.width(), header_h));

        let mut col_x = 0.0_f32;
        for col in columns.iter() {
            if col.is_hidden {
                continue;
            }
            ctx.set_color(theme::color(keys::CLR_TEXT_SECONDARY));
            ctx.draw_text(
                &col.title,
                NSPoint::new(col_x + padding_x, (header_h - font_size) / 2.0),
                font_size,
            );
            col_x += col.width;
        }

        // Separator under header
        ctx.set_color(theme::color(keys::CLR_SEPARATOR));
        ctx.fill_rect(NSRect::new(0.0, header_h, bounds.width(), 1.0));

        // Data rows
        for row in 0..row_count {
            let row_y = header_h + 1.0 + row as f32 * row_h;
            if row_y + row_h > bounds.height() {
                break;
            }

            // Selection highlight
            if selected == Some(row) {
                ctx.set_color(theme::color(keys::CLR_BG_SELECTED));
                ctx.fill_rect(NSRect::new(0.0, row_y, bounds.width(), row_h));
            }

            // Alternating row background
            if selected != Some(row) && row % 2 == 1 {
                ctx.set_color(theme::color(keys::CLR_BG_SIDEBAR));
                ctx.fill_rect(NSRect::new(0.0, row_y, bounds.width(), row_h));
            }

            // Cell values
            let mut cx = 0.0_f32;
            for col in columns.iter() {
                if col.is_hidden {
                    continue;
                }
                let value = self.data_source.value_for(row, &col.identifier);
                ctx.set_color(theme::color(keys::CLR_TEXT_LABEL));
                ctx.draw_text(
                    &value,
                    NSPoint::new(cx + padding_x, row_y + (row_h - font_size) / 2.0),
                    font_size,
                );
                cx += col.width;
            }
        }
    }

    fn mouse_down(&mut self, view: &mut NSView, event: &crate::event::NSEvent) {
        let pos = event.location_in_window();
        let header_h = theme::f32_or(keys::SIZE_TAB_HEIGHT, 28.0);
        let row_h = theme::f32_or(keys::SIZE_CONTROL_HEIGHT, 24.0);

        if pos.y > header_h {
            let row = ((pos.y - header_h - 1.0) / row_h) as usize;
            let row_count = self.data_source.number_of_rows();
            if row < row_count {
                *self.selected_row.lock().unwrap() = Some(row);
            }
        }
        view.set_needs_display();
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
