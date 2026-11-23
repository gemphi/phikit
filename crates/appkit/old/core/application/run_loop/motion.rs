//! Cursor movement and device motion handlers.
use super::RunLoopState;
use super::super::NSApplication;
use crate::event::{NSEvent, NSEventType};
use crate::graphics::NSPoint;

impl NSApplication {
    /// Handles winit cursor motion updates and translates them into mouse events.
    pub(in crate::application) fn handle_cursor_moved(
        state: &mut RunLoopState,
        position: winit::dpi::PhysicalPosition<f64>,
    ) {
        let scale_factor = state
            .winit_window
            .as_ref()
            .map(|w| w.scale_factor())
            .unwrap_or(1.0);
        let logical_pos = position.to_logical::<f32>(scale_factor);
        let pos = NSPoint::new(logical_pos.x, logical_pos.y);

        let (left_drag, right_drag, prev_pos, mut content, mut toolbar, toolbar_h, drag_session_info) = {
            let shared = Self::shared();
            let mut app = shared.lock().unwrap();
            let prev = app.last_mouse_pos;
            let left = app.is_left_mouse_down;
            let right = app.is_right_mouse_down;
            app.last_mouse_pos = pos;

            if let Some(ref mut ds) = app.active_drag_session {
                ds.dragging_location = pos;
            }

            let drag_info = app.active_drag_session.clone();
            let tb_h = app.windows.get(0).map(|w| w.toolbar_height).unwrap_or(0.0);
            let content = app.windows.get_mut(0).and_then(|w| w.content_view.take());
            let toolbar = app.windows.get_mut(0).and_then(|w| w.toolbar_view.take());
            (left, right, prev, content, toolbar, tb_h, drag_info)
        };

        let in_toolbar = pos.y < toolbar_h;
        if in_toolbar {
            if let Some(ref mut tb) = toolbar {
                let ns_event = NSEvent {
                    event_type: NSEventType::MouseMoved,
                    location_in_window: pos,
                    timestamp: 0.0,
                    delta_x: pos.x - prev_pos.x,
                    delta_y: pos.y - prev_pos.y,
                    modifier_flags: 0,
                    characters: None,
                    file_path: None,
                };
                Self::dispatch_event_to_view(tb, &ns_event);
            }
        }

        if let Some(mut view) = content {
            let shared = Self::shared();
            if let Some(drag_info) = drag_session_info {
                Self::dispatch_drag_update(&mut view, &drag_info, pos);
            } else {
                let event_type = if left_drag {
                    NSEventType::LeftMouseDragged
                } else if right_drag {
                    NSEventType::RightMouseDragged
                } else {
                    NSEventType::MouseMoved
                };
                let ns_event = NSEvent {
                    event_type,
                    location_in_window: pos,
                    timestamp: 0.0,
                    delta_x: pos.x - prev_pos.x,
                    delta_y: pos.y - prev_pos.y,
                    modifier_flags: Self::modifier_flags(&shared.lock().unwrap()),
                    characters: None,
                    file_path: None,
                };
                Self::dispatch_event_to_view(&mut view, &ns_event);
            }

            if left_drag || right_drag {
                let dx = pos.x - prev_pos.x;
                let dy = pos.y - prev_pos.y;
                if let Some(on_scroll) = {
                    let app = shared.lock().unwrap();
                    app.on_scroll_event.clone()
                } {
                    let cam_event = NSEvent {
                        event_type: if right_drag {
                            NSEventType::RightMouseDragged
                        } else {
                            NSEventType::LeftMouseDragged
                        },
                        location_in_window: pos,
                        timestamp: 0.0,
                        delta_x: dx,
                        delta_y: dy,
                        modifier_flags: Self::modifier_flags(&shared.lock().unwrap()),
                        characters: None,
                        file_path: None,
                    };
                    on_scroll(&cam_event);
                }
            }

            let mut app = shared.lock().unwrap();
            if !app.windows.is_empty() {
                let current_hovered = app.windows[0].last_hovered_view_id;
                if current_hovered != state.last_hovered_view_id || left_drag || right_drag || in_toolbar {
                    state.last_hovered_view_id = current_hovered;
                    state.ui_dirty = true;
                }
                app.windows[0].content_view = Some(view);
                if let Some(tb) = toolbar {
                    app.windows[0].toolbar_view = Some(tb);
                }
            }
        } else {
            let shared = Self::shared();
            let mut app = shared.lock().unwrap();
            if !app.windows.is_empty() {
                if let Some(tb) = toolbar {
                    app.windows[0].toolbar_view = Some(tb);
                }
            }
        }

        if state.ui_dirty {
            if let Some(ref w) = state.winit_window {
                w.request_redraw();
            }
        }
    }

    /// Handles continuous mouse device motion updates during active dragging.
    pub(in crate::application) fn handle_device_motion(state: &mut RunLoopState, delta: (f64, f64)) {
        let shared = Self::shared();
        let (is_dragging, last_pos, content) = {
            let mut app = shared.lock().unwrap();
            let left = app.is_left_mouse_down;
            let right = app.is_right_mouse_down;
            let dragging = left || right;
            let pos = app.last_mouse_pos;
            let content = app.windows.get_mut(0).and_then(|w| w.content_view.take());
            (dragging, pos, content)
        };

        if is_dragging {
            if let Some(mut view) = content {
                let scale_factor = state
                    .winit_window
                    .as_ref()
                    .map(|w| w.scale_factor())
                    .unwrap_or(1.0);
                let new_pos = NSPoint::new(
                    last_pos.x + (delta.0 as f32 / scale_factor as f32),
                    last_pos.y + (delta.1 as f32 / scale_factor as f32),
                );

                {
                    let mut app = shared.lock().unwrap();
                    app.last_mouse_pos = new_pos;
                }

                let left = shared.lock().unwrap().is_left_mouse_down;
                let event_type = if left {
                    NSEventType::LeftMouseDragged
                } else {
                    NSEventType::RightMouseDragged
                };

                let ns_event = NSEvent {
                    event_type,
                    location_in_window: new_pos,
                    timestamp: 0.0,
                    delta_x: delta.0 as f32,
                    delta_y: delta.1 as f32,
                    modifier_flags: Self::modifier_flags(&shared.lock().unwrap()),
                    characters: None,
                    file_path: None,
                };
                Self::dispatch_event_to_view(&mut view, &ns_event);

                let mut app = shared.lock().unwrap();
                if !app.windows.is_empty() {
                    app.windows[0].content_view = Some(view);
                }
            }
        } else if let Some(view) = content {
            let mut app = shared.lock().unwrap();
            if !app.windows.is_empty() {
                app.windows[0].content_view = Some(view);
            }
        }
    }
}
