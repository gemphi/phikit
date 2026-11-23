//! Run loop — winit event loop integration and per-frame handlers.
//!
//! Submodules handle specific event categories:
//! - `input` — mouse click dispatch
//! - `keyboard` — keyboard, scroll wheel, file drop
//! - `motion` — cursor movement, resize, device motion
//! - `redraw` — frame rendering and surface presentation
//! - `resumed` — window creation and app launch

mod input;
mod keyboard;
mod motion;
mod redraw;
mod resize;
mod resumed;

use super::NSApplication;
use crate::{NSError, NSResult};
use softbuffer::Surface;
use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

/// Private state structure for tracking run loop lifecycle resources.
pub(in crate::application) struct RunLoopState {
    pub(in crate::application) winit_window: Option<Arc<Window>>,
    pub(in crate::application) surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    pub(in crate::application) launched: bool,
    /// Cached UI pixel buffer — reused across frames when nothing changed.
    pub(in crate::application) ui_cache: Vec<u32>,
    /// Whether the UI needs a full re-render this frame.
    pub(in crate::application) ui_dirty: bool,
    /// Last hovered view ID — redraw only when hover target changes.
    pub(in crate::application) last_hovered_view_id: Option<u64>,
}

impl NSApplication {
    /// Start the standard winit event processing loop.
    pub fn run_app() -> NSResult<()> {
        let event_loop = EventLoop::new().map_err(|e| NSError::WinitError(e.to_string()))?;
        event_loop.set_control_flow(ControlFlow::Wait);

        let mut state = RunLoopState {
            winit_window: None,
            surface: None,
            launched: false,
            ui_cache: Vec::new(),
            ui_dirty: true,
            last_hovered_view_id: None,
        };

        event_loop
            .run(move |event, elwt| {
                match event {
                    Event::Resumed => {
                        Self::handle_resumed(&mut state, elwt);
                    }
                    Event::WindowEvent {
                        event: WindowEvent::RedrawRequested,
                        ..
                    } => {
                        Self::handle_redraw(&mut state);
                        if let Some(ref w) = state.winit_window {
                            w.request_redraw();
                        }
                    }
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::Resized(size) => Self::handle_resize(&mut state, size),
                        WindowEvent::CloseRequested => elwt.exit(),
                        WindowEvent::CursorMoved { position, .. } => {
                            Self::handle_cursor_moved(&mut state, position);
                        }
                        WindowEvent::MouseInput {
                            state: btn_state,
                            button,
                            ..
                        } => {
                            Self::handle_mouse_input(&mut state, btn_state, button);
                        }
                        WindowEvent::MouseWheel { delta, .. } => {
                            Self::handle_mouse_wheel(&mut state, delta);
                        }
                        WindowEvent::ModifiersChanged(modifiers) => {
                            let shared = Self::shared();
                            let mut app = shared.lock().unwrap();
                            let s = modifiers.state();
                            app.shift_pressed = s.shift_key();
                            app.ctrl_pressed = s.control_key();
                            app.opt_pressed = s.alt_key();
                            app.cmd_pressed = s.super_key();
                        }
                        WindowEvent::DroppedFile(path) => {
                            Self::handle_dropped_file(&mut state, path);
                        }
                        WindowEvent::KeyboardInput {
                            event: key_event, ..
                        } => {
                            if key_event.state == winit::event::ElementState::Pressed {
                                if let Some(text) = &key_event.text {
                                    Self::handle_key_press(&mut state, text.to_string());
                                }
                            }
                        }
                        _ => {}
                    },
                    Event::DeviceEvent {
                        event: winit::event::DeviceEvent::MouseMotion { delta },
                        ..
                    } => {
                        Self::handle_device_motion(&mut state, delta);
                    }
                    _ => {}
                }
            })
            .map_err(|e| NSError::WinitError(e.to_string()))
    }
}
