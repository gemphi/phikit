use super::TimelineState;
use crate::{NSColor, NSPoint, NSRect, NSView, NSViewDelegate};
use std::sync::{Arc, Mutex};

pub(crate) struct TimelineDelegate {
    pub(crate) state: Arc<Mutex<TimelineState>>,
}

impl NSViewDelegate for TimelineDelegate {
    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        let state = self.state.lock().unwrap();
        let bounds = view.bounds();

        // Background
        let bg_color = crate::theme::color(crate::theme::keys::CLR_BG_TIMELINE);
        context.set_color(bg_color);
        context.fill_rect(bounds);

        // Ruler
        let ruler_height = 30.0;
        let ruler_bg = crate::theme::color(crate::theme::keys::CLR_BG_RULER);
        context.set_color(ruler_bg);
        context.fill_rect(NSRect::sized(bounds.width(), ruler_height));

        // Playhead
        context.set_color(NSColor::system_accent());
        context.fill_rect(NSRect::new(
            state.playhead_pos - 1.0,
            0.0,
            2.0,
            bounds.height(),
        ));

        // Track lanes
        let track_h = 28.0;
        let track_y_start = ruler_height + 8.0;
        let track_count = state.tracks.len().max(3); // Show at least 3 empty lanes
        for i in 0..track_count {
            let y = track_y_start + i as f32 * (track_h + 4.0);
            if y + track_h > bounds.height() {
                break;
            }
            let lane_rect = NSRect::new(10.0, y, bounds.width() - 20.0, track_h);
            let is_even = i % 2 == 0;
            let lane_color = if is_even {
                NSColor::new(0.14, 0.14, 0.16, 1.0)
            } else {
                NSColor::new(0.18, 0.18, 0.20, 1.0)
            };
            context.set_color(lane_color);
            context.fill_rounded_rect(lane_rect, 4.0);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
