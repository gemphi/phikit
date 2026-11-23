//! Keyboard, scroll wheel, and file drop handlers for NSApplication run loop.
use super::RunLoopState;
use super::super::NSApplication;
use crate::event::{NSEvent, NSEventType};

impl NSApplication {
    /// Handles mouse wheel scroll events and routes them.
    pub(in crate::application) fn handle_mouse_wheel(
        state: &mut RunLoopState,
        delta: winit::event::MouseScrollDelta,
    ) {
        const PIXELS_PER_LINE: f32 = 20.0;

        let (dx, dy) = match delta {
            winit::event::MouseScrollDelta::LineDelta(x, y) => {
                (x * PIXELS_PER_LINE, y * PIXELS_PER_LINE)
            }
            winit::event::MouseScrollDelta::PixelDelta(p) => {
                let scale_factor = state
                    .winit_window
                    .as_ref()
                    .map(|w| w.scale_factor())
                    .unwrap_or(1.0);
                let lp = p.to_logical::<f32>(scale_factor);
                (lp.x, lp.y)
            }
        };

        let shared = Self::shared();
        let (pos, content, modifier_flags) = {
            let mut app = shared.lock().unwrap();
            let pos = app.last_mouse_pos;
            let flags = Self::modifier_flags(&app);
            let content = app.windows.get_mut(0).and_then(|w| w.content_view.take());
            (pos, content, flags)
        };

        let ns_event = NSEvent {
            event_type: NSEventType::ScrollWheel,
            location_in_window: pos,
            timestamp: 0.0,
            delta_x: dx,
            delta_y: dy,
            modifier_flags,
            characters: None,
            file_path: None,
        };

        if let Some(mut view) = content {
            Self::dispatch_event_to_view(&mut view, &ns_event);
            let mut app = shared.lock().unwrap();
            if !app.windows.is_empty() {
                app.windows[0].content_view = Some(view);
            }
        }

        if let Some(on_scroll) = {
            let app = shared.lock().unwrap();
            app.on_scroll_event.clone()
        } {
            on_scroll(&ns_event);
        }

        if let Some(ref w) = state.winit_window {
            w.request_redraw();
        }
    }

    /// Handles local dropped files.
    pub(in crate::application) fn handle_dropped_file(state: &mut RunLoopState, path: std::path::PathBuf) {
        let shared = Self::shared();
        let (pos, content) = {
            let mut app = shared.lock().unwrap();
            let pos = app.last_mouse_pos;
            let content = app.windows.get_mut(0).and_then(|w| w.content_view.take());
            (pos, content)
        };

        if let Some(mut view) = content {
            let ns_event = NSEvent {
                event_type: NSEventType::DroppedFile,
                location_in_window: pos,
                timestamp: 0.0,
                delta_x: 0.0,
                delta_y: 0.0,
                modifier_flags: Self::modifier_flags(&shared.lock().unwrap()),
                characters: None,
                file_path: Some(path),
            };
            Self::dispatch_event_to_view(&mut view, &ns_event);

            let mut app = shared.lock().unwrap();
            if !app.windows.is_empty() {
                app.windows[0].content_view = Some(view);
            }
        }
    }

    /// Handles keyboard key press events.
    pub(in crate::application) fn handle_key_press(state: &mut RunLoopState, characters: String) {
        let shared = Self::shared();
        let (content, modifier_flags) = {
            let mut app = shared.lock().unwrap();
            let content = app.windows.get_mut(0).and_then(|w| w.content_view.take());
            let flags = Self::modifier_flags(&app);
            (content, flags)
        };

        let ns_event = NSEvent {
            event_type: NSEventType::KeyDown,
            location_in_window: crate::graphics::NSPoint::new(0.0, 0.0),
            timestamp: 0.0,
            delta_x: 0.0,
            delta_y: 0.0,
            modifier_flags,
            characters: Some(characters),
            file_path: None,
        };

        if let Some(mut view) = content {
            Self::dispatch_event_to_view(&mut view, &ns_event);
            let mut app = shared.lock().unwrap();
            if !app.windows.is_empty() {
                app.windows[0].content_view = Some(view);
            }
        }

        if let Some(on_key) = {
            let app = shared.lock().unwrap();
            app.on_key_event.clone()
        } {
            on_key(&ns_event);
        }

        if let Some(ref w) = state.winit_window {
            w.request_redraw();
        }
    }
}
