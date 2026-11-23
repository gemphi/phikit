//! Window resize handler.
use super::RunLoopState;
use super::super::NSApplication;
use crate::graphics::NSRect;

impl NSApplication {
    /// Handles window resizing events by scaling layout properties proportionally.
    pub(in crate::application) fn handle_resize(
        state: &mut RunLoopState,
        physical_size: winit::dpi::PhysicalSize<u32>,
    ) {
        state.ui_dirty = true;
        let scale_factor = state
            .winit_window
            .as_ref()
            .map(|w| w.scale_factor())
            .unwrap_or(1.0);
        let logical_size = physical_size.to_logical::<f32>(scale_factor);

        let shared = Self::shared();
        {
            let mut app = shared.lock().unwrap();
            let rebuild_fn = app.on_rebuild_layout.clone();
            if let Some(win) = app.windows.get_mut(0) {
                win.frame = NSRect::sized(logical_size.width, logical_size.height);
                win.layout_chrome();
                if let Some(ref mut content) = win.content_view {
                    if let Some(rebuild) = rebuild_fn {
                        rebuild(content, logical_size.width, logical_size.height);
                    } else {
                        let old_w = content.frame().size.width;
                        let old_h = content.frame().size.height;
                        Self::layout_subviews(
                            content, old_w, old_h, logical_size.width, logical_size.height,
                        );
                        content.layout_subviews();
                    }
                }
            }

            if let Some(gpu_mutex) = &app.gpu_renderer {
                if let Ok(mut gpu) = gpu_mutex.lock() {
                    gpu.resize(physical_size.width, physical_size.height);
                }
            }
        }

        if let Some(ref mut surf) = state.surface {
            surf.resize(
                std::num::NonZeroU32::new(physical_size.width.max(1)).unwrap(),
                std::num::NonZeroU32::new(physical_size.height.max(1)).unwrap(),
            )
            .ok();
        }

        if let Some(ref w) = state.winit_window {
            w.request_redraw();
        }
    }
}
