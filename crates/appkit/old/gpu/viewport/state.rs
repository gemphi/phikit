//! Viewport state — struct definition, construction, and public API.

use crate::ecs::{Entity, Scene};
use crate::gpu::ViewportCameraController;
use crate::{NSPoint, NSRect, NSView};
use std::sync::{Arc, Mutex};

use super::camera_integration::NS3DViewportGPUDelegate;

/// Free-function wrapper for viewport cache invalidation.
pub fn invalidate_viewport_cache() {
    NS3DViewportGPU::invalidate_viewport_cache();
}

/// GPU-accelerated 3D viewport
pub struct NS3DViewportGPU {
    view: NSView,
    pub camera: Arc<Mutex<ViewportCameraController>>,
    pub scene: Arc<Mutex<Scene>>,
    pub on_selection_changed: Option<Arc<dyn Fn(Option<Entity>) + Send + Sync>>,
    pub on_file_dropped: Option<Arc<dyn Fn(std::path::PathBuf) + Send + Sync>>,
    pub selected_entity: Option<u64>,
}

impl NS3DViewportGPU {
    pub fn new(frame: NSRect, scene: Arc<Mutex<Scene>>) -> Self {
        let camera = Arc::new(Mutex::new(ViewportCameraController::new()));

        let mut view = NSView::new(frame);
        view.set_delegate(NS3DViewportGPUDelegate::new(
            camera.clone(),
            scene.clone(),
        ));

        Self {
            view,
            camera,
            scene,
            on_selection_changed: None,
            on_file_dropped: None,
            selected_entity: None,
        }
    }

    pub fn set_on_file_dropped<F>(&mut self, callback: F)
    where
        F: Fn(std::path::PathBuf) + Send + Sync + 'static,
    {
        let callback_arc = Arc::new(callback);
        self.on_file_dropped = Some(callback_arc.clone());
        if let Some(delegate_arc) = self.view.delegate() {
            if let Ok(mut delegate) = delegate_arc.lock() {
                if let Some(d) = delegate
                    .as_any_mut()
                    .downcast_mut::<NS3DViewportGPUDelegate>()
                {
                    d.on_file_dropped = Some(callback_arc);
                }
            }
        }
    }

    pub fn set_selected_entity(&mut self, id: Option<u64>) {
        self.selected_entity = id;
        if let Some(delegate_arc) = self.view.delegate() {
            if let Ok(mut delegate) = delegate_arc.lock() {
                if let Some(d) = delegate
                    .as_any_mut()
                    .downcast_mut::<NS3DViewportGPUDelegate>()
                {
                    d.selected_entity = id;
                }
            }
        }
    }

    pub fn set_on_selection_changed<F>(&mut self, handler: F)
    where
        F: Fn(Option<Entity>) + Send + Sync + 'static,
    {
        let handler_arc = Arc::new(handler);
        self.on_selection_changed = Some(handler_arc.clone());

        if let Some(delegate_arc) = self.view.delegate() {
            if let Ok(mut delegate) = delegate_arc.lock() {
                if let Some(d) = delegate
                    .as_any_mut()
                    .downcast_mut::<NS3DViewportGPUDelegate>()
                {
                    d.on_selection_changed = Some(handler_arc);
                }
            }
        }
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }

    pub fn view_mut(&mut self) -> &mut NSView {
        &mut self.view
    }

    /// Invalidate the viewport pixel cache, forcing a full re-render on the next draw call.
    pub fn invalidate_viewport_cache() {
        use std::sync::atomic::Ordering;
        super::rendering::VIEWPORT_LAST_RENDER_MS.store(0, Ordering::Release);
        super::rendering::VIEWPORT_CACHE_INVALIDATED.store(true, Ordering::Release);
    }
}
