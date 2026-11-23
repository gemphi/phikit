//! Redraw and compositing handlers for NSApplication run loop.
use super::RunLoopState;
use super::super::NSApplication;
use crate::render::RenderContext;
use std::time::Instant;

impl NSApplication {
    /// Primary redraw logic using software buffers and hardware composition.
    pub(in crate::application) fn handle_redraw(state: &mut RunLoopState) {
        let Some(ref w) = state.winit_window else {
            return;
        };
        let Some(ref mut surf) = state.surface else {
            return;
        };

        let inner = w.inner_size();
        let scale_factor = w.scale_factor() as f32;
        let width = inner.width as usize;
        let height = inner.height as usize;
        if width == 0 || height == 0 {
            return;
        }

        let (mut content_view, mut toolbar_view) = {
            let shared = Self::shared();
            let mut app = shared.lock().unwrap();
            let content = app.windows.get_mut(0).and_then(|w| w.content_view.take());
            let toolbar = app.windows.get_mut(0).and_then(|w| w.toolbar_view.take());
            (content, toolbar)
        };

        if let Some(mut content) = content_view {
            if Self::process_pending_actions_on_view(&mut content) {
                state.ui_dirty = true;
            }

            if state.ui_dirty {
                let frame_start = Instant::now();

                thread_local! {
                    static UI_BUF: std::cell::RefCell<Vec<u32>> = std::cell::RefCell::new(Vec::new());
                }
                UI_BUF.with(|buf| {
                    let mut ui_pixels = buf.borrow_mut();
                    let needed = width * height;
                    if ui_pixels.len() != needed {
                        ui_pixels.resize(needed, 0);
                    }

                    ui_pixels.fill(0);
                    let mut ctx = RenderContext::new(width, height, &mut ui_pixels);
                    ctx.set_scale_factor(scale_factor);

                    if let Some(ref mut toolbar) = toolbar_view {
                        Self::render_view(toolbar, &mut ctx);
                    }

                    Self::render_view(&mut content, &mut ctx);
                    state.ui_dirty = false;

                    Self::present_frame(state, &mut content, width, height, &ui_pixels);
                });

                let frame_ms = frame_start.elapsed().as_secs_f64() * 1000.0;
                if frame_ms > 16.0 {
                    println!("[PERF] Frame took {:.1}ms (>{:.0}ms budget @ 60fps)", frame_ms, 16.67);
                }
            }

            let shared = Self::shared();
            let mut app = shared.lock().unwrap();
            if !app.windows.is_empty() {
                app.windows[0].content_view = Some(content);
                if let Some(toolbar) = toolbar_view {
                    app.windows[0].toolbar_view = Some(toolbar);
                }
            }
        }
    }

    /// Process queue of pending actions on the root view. Returns true if any were processed.
    pub(in crate::application) fn process_pending_actions_on_view(content: &mut crate::NSView) -> bool {
        let actions = {
            let shared = Self::shared();
            let mut app = shared.lock().unwrap();
            std::mem::take(&mut app.pending_actions)
        };
        let had_actions = !actions.is_empty();
        for action in actions {
            action(content);
        }
        had_actions
    }

    /// Draw GPU views and present the blended frame output.
    pub(in crate::application) fn present_frame(
        state: &mut RunLoopState,
        content: &mut crate::NSView,
        width: usize,
        height: usize,
        ui_pixels: &[u32],
    ) {
        let gpu_renderer_opt = {
            let shared = Self::shared();
            let app = shared.lock().unwrap();
            app.gpu_renderer.clone()
        };

        if let Some(gpu_mutex) = gpu_renderer_opt {
            let mut gpu = gpu_mutex.lock().unwrap();
            if let (Some(device), Some(queue), Some(surface)) =
                (gpu.device.clone(), gpu.queue.clone(), gpu.surface.as_ref())
            {
                if let Ok(output) = surface.get_current_texture() {
                    let view = output
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Main Encoder"),
                        });

                    Self::render_view_gpu(content, &gpu, &mut encoder, &view);
                    gpu.update_ui_overlay(width as u32, height as u32, ui_pixels);
                    gpu.render_overlay(&view, &mut encoder);

                    queue.submit(std::iter::once(encoder.finish()));
                    output.present();
                }
            }
        } else if let Some(ref mut surf) = state.surface {
            surf.resize(
                std::num::NonZeroU32::new(width as u32).unwrap(),
                std::num::NonZeroU32::new(height as u32).unwrap(),
            )
            .ok();
            if let Ok(mut buf) = surf.buffer_mut() {
                buf.copy_from_slice(ui_pixels);
                buf.present().ok();
            }
        }
    }
}
