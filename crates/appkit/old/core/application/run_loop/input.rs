//! Mouse click handlers for NSApplication run loop.
use super::RunLoopState;
use super::super::NSApplication;
use crate::event::{NSEvent, NSEventType};
use crate::graphics::NSPoint;
use winit::event::{ElementState, MouseButton as WinitMouseButton};

impl NSApplication {
    /// Handles mouse button press and release inputs.
    pub(in crate::application) fn handle_mouse_input(
        state: &mut RunLoopState,
        element_state: ElementState,
        button: WinitMouseButton,
    ) {
        let click_start = std::time::Instant::now();
        state.ui_dirty = true;
        let shared = Self::shared();
        let (pos, content, toolbar, toolbar_height, drag_session_info) = {
            let mut app = shared.lock().unwrap();
            let pos = app.last_mouse_pos;
            let drag_info =
                if element_state == ElementState::Released && button == WinitMouseButton::Left {
                    app.active_drag_session.take()
                } else {
                    None
                };
            let toolbar_h = app.windows.get(0).map(|w| w.toolbar_height).unwrap_or(0.0);
            let content = app.windows.get_mut(0).and_then(|w| w.content_view.take());
            let toolbar = app.windows.get_mut(0).and_then(|w| w.toolbar_view.take());
            (pos, content, toolbar, toolbar_h, drag_info)
        };

        let in_toolbar = toolbar.is_some() && pos.y < toolbar_height;

        if in_toolbar {
            if let Some(mut toolbar_view) = toolbar {
                let (event_type, is_down) = match (button, element_state) {
                    (WinitMouseButton::Left, ElementState::Pressed) => {
                        (NSEventType::LeftMouseDown, true)
                    }
                    (WinitMouseButton::Left, ElementState::Released) => {
                        (NSEventType::LeftMouseUp, false)
                    }
                    (WinitMouseButton::Right, ElementState::Pressed) => {
                        (NSEventType::RightMouseDown, true)
                    }
                    (WinitMouseButton::Right, ElementState::Released) => {
                        (NSEventType::RightMouseUp, false)
                    }
                    _ => {
                        let mut app = shared.lock().unwrap();
                        if !app.windows.is_empty() {
                            app.windows[0].toolbar_view = Some(toolbar_view);
                            if let Some(c) = content {
                                app.windows[0].content_view = Some(c);
                            }
                        }
                        return;
                    }
                };

                {
                    let mut app = shared.lock().unwrap();
                    if button == WinitMouseButton::Left {
                        app.is_left_mouse_down = is_down;
                    } else if button == WinitMouseButton::Right {
                        app.is_right_mouse_down = is_down;
                    }
                }

                let ns_event = NSEvent {
                    event_type,
                    location_in_window: pos,
                    timestamp: 0.0,
                    delta_x: 0.0,
                    delta_y: 0.0,
                    modifier_flags: Self::modifier_flags(&shared.lock().unwrap()),
                    characters: None,
                    file_path: None,
                };
                Self::dispatch_event_to_view(&mut toolbar_view, &ns_event);

                if let Some(mut content_view) = content {
                    Self::process_pending_actions_on_view(&mut content_view);
                    let mut app = shared.lock().unwrap();
                    if !app.windows.is_empty() {
                        app.windows[0].toolbar_view = Some(toolbar_view);
                        app.windows[0].content_view = Some(content_view);
                    }
                } else {
                    Self::process_pending_actions_on_view(&mut toolbar_view);
                    let mut app = shared.lock().unwrap();
                    if !app.windows.is_empty() {
                        app.windows[0].toolbar_view = Some(toolbar_view);
                    }
                }
            }
        } else if let Some(mut view) = content {
            if let Some(tb) = toolbar {
                let mut app = shared.lock().unwrap();
                if !app.windows.is_empty() {
                    app.windows[0].toolbar_view = Some(tb);
                }
            }

            if let Some(drag_info) = drag_session_info {
                Self::dispatch_drop(&mut view, &drag_info, pos);
                let mut app = shared.lock().unwrap();
                app.drag_hovered_view_id = None;
                app.is_left_mouse_down = false;
                if !app.windows.is_empty() {
                    app.windows[0].content_view = Some(view);
                }
                if let Some(ref w) = state.winit_window {
                    w.request_redraw();
                }
                return;
            }

            let (event_type, is_down) = match (button, element_state) {
                (WinitMouseButton::Left, ElementState::Pressed) => {
                    (NSEventType::LeftMouseDown, true)
                }
                (WinitMouseButton::Left, ElementState::Released) => {
                    (NSEventType::LeftMouseUp, false)
                }
                (WinitMouseButton::Right, ElementState::Pressed) => {
                    (NSEventType::RightMouseDown, true)
                }
                (WinitMouseButton::Right, ElementState::Released) => {
                    (NSEventType::RightMouseUp, false)
                }
                _ => {
                    shared.lock().unwrap().windows[0].content_view = Some(view);
                    return;
                }
            };

            {
                let mut app = shared.lock().unwrap();
                if button == WinitMouseButton::Left {
                    app.is_left_mouse_down = is_down;
                } else if button == WinitMouseButton::Right {
                    app.is_right_mouse_down = is_down;
                }
            }

            let ns_event = NSEvent {
                event_type,
                location_in_window: pos,
                timestamp: 0.0,
                delta_x: 0.0,
                delta_y: 0.0,
                modifier_flags: Self::modifier_flags(&shared.lock().unwrap()),
                characters: None,
                file_path: None,
            };
            Self::dispatch_event_to_view(&mut view, &ns_event);
            Self::process_pending_actions_on_view(&mut view);

            let mut app = shared.lock().unwrap();
            if !app.windows.is_empty() {
                app.windows[0].content_view = Some(view);
            }
        }
        if let Some(ref w) = state.winit_window {
            w.request_redraw();
        }
        let click_ms = click_start.elapsed().as_secs_f64() * 1000.0;
        if click_ms > 5.0 {
            println!("[PERF] Click dispatch took {:.1}ms", click_ms);
        }
    }
}
