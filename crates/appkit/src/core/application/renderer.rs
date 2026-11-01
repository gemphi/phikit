//! Rendering pipeline helpers for NSApplication.
use super::NSApplication;
use crate::gpu::GpuRenderer;
use crate::render::RenderContext;
use crate::view::NSView;
use std::sync::Mutex;

impl NSApplication {
    /// Recursively render a view and its subviews via software rendering.
    pub fn render_view(view: &mut NSView, ctx: &mut RenderContext) {
        view.draw(ctx);
    }

    /// Recursively call gpu_draw for all GPU-backed views in the view hierarchy.
    pub fn render_view_gpu(
        view: &mut NSView,
        gpu: &GpuRenderer,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &wgpu::TextureView,
    ) {
        if view.is_hidden() {
            return;
        }

        if let Some(delegate_arc) = view.delegate() {
            if let Ok(delegate) = delegate_arc.lock() {
                delegate.gpu_draw(view, gpu, encoder, target_view);
            }
        }

        for sub in view.subviews_mut() {
            Self::render_view_gpu(sub, gpu, encoder, target_view);
        }
    }
}
