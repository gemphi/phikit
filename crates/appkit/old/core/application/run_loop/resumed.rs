//! Application launch and window creation when winit resumes.
use super::RunLoopState;
use super::super::NSApplication;
use crate::graphics::NSRect;
use softbuffer::{Context, Surface};
use std::sync::{Arc, Mutex};
use winit::window::{Window, WindowBuilder};

impl NSApplication {
    /// Handles application window creation and initialization when winit resumes.
    pub(in crate::application) fn handle_resumed(
        state: &mut RunLoopState,
        elwt: &winit::event_loop::EventLoopWindowTarget<()>,
    ) {
        if state.winit_window.is_some() {
            return;
        }

        let window_title = NSApplication::shared()
            .try_lock()
            .ok()
            .and_then(|app| app.windows.first().map(|w| w.title.clone()))
            .unwrap_or_else(|| "Rust Kit Pro".to_string());

        let builder = WindowBuilder::new()
            .with_title(&window_title)
            .with_maximized(true);

        let w = match builder.build(elwt) {
            Ok(w) => {
                // Center the window on the primary monitor by default.
                if let Some(monitor) = w.current_monitor() {
                    let monitor_size = monitor.size();
                    let monitor_pos = monitor.position();
                    let window_size = w.outer_size();
                    let x = monitor_pos.x
                        + (monitor_size.width as i32 - window_size.width as i32) / 2;
                    let y = monitor_pos.y
                        + (monitor_size.height as i32 - window_size.height as i32) / 2;
                    w.set_outer_position(winit::dpi::PhysicalPosition::new(x, y));
                }
                Arc::new(w)
            }
            Err(e) => {
                eprintln!("[NSApplication] Failed to create window: {e}");
                return;
            }
        };

        let ctx =
            Context::new(w.clone()).expect("[NSApplication] Failed to create softbuffer context");
        let mut surf =
            Surface::new(&ctx, w.clone()).expect("[NSApplication] Failed to create surface");
        let inner = w.inner_size();
        surf.resize(
            std::num::NonZeroU32::new(inner.width.max(1)).unwrap(),
            std::num::NonZeroU32::new(inner.height.max(1)).unwrap(),
        )
        .ok();

        {
            let shared = Self::shared();
            let mut app = shared.lock().unwrap();
            app.softbuffer_context = Some(Arc::new(ctx));

            let mut gpu = crate::gpu::GpuRenderer::new();
            pollster::block_on(gpu.initialize(w.clone()));
            let gpu_arc = Arc::new(Mutex::new(gpu));
            crate::gpu::set_global_renderer(gpu_arc.clone());
            app.gpu_renderer = Some(gpu_arc);

            app.is_running = true;
            app.is_active = true;
        }

        state.winit_window = Some(w.clone());
        state.surface = Some(surf);

        if !state.launched {
            state.launched = true;
            Self::fire_application_launched(&w);
        }
    }

    /// Notifies the delegate of successful launch and prepares the primary window.
    pub(in crate::application) fn fire_application_launched(w: &Arc<Window>) {
        let delegate_opt = {
            let shared = Self::shared();
            let app = shared.lock().unwrap();
            app.delegate.clone()
        };

        if let Some(delegate_arc) = delegate_opt {
            if let Ok(mut d) = delegate_arc.lock() {
                d.application_did_finish_launching();
            }
        }

        let sz = w.inner_size();
        let scale = w.scale_factor();
        let logical = sz.to_logical::<f32>(scale);
        let actual_w = logical.width;
        let actual_h = logical.height;

        crate::graphics::image::set_display_scale_factor(scale as f32);

        {
            let shared = Self::shared();
            let mut app = shared.lock().unwrap();
            if let Some(win) = app.windows.get_mut(0) {
                win.winit_window = Some(w.clone());
                win.is_visible = true;
                win.is_key = true;
                win.frame = NSRect::sized(actual_w, actual_h);

                if let Some(ref mut content) = win.content_view {
                    content.set_frame(NSRect::sized(actual_w, actual_h));
                    content.remove_all_subviews();
                    content.set_gradient_background(
                        crate::theme::color(crate::theme::keys::CLR_BG_EDITOR_TOP),
                        crate::theme::color(crate::theme::keys::CLR_BG_EDITOR_BOTTOM),
                    );
                }
                let w_size = actual_w;
                let h_size = actual_h;
                let rebuild_fn = app.on_rebuild_layout.clone();
                app.pending_actions.push(Box::new(move |root_view| {
                    if let Some(rebuild) = rebuild_fn {
                        rebuild(root_view, w_size, h_size);
                    }
                }));
            }
        }

        w.request_redraw();
    }
}
