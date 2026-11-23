//! # NS3DRenderView — 3D Rendering Surface Component
//!
//! A reusable view component that provides GPU-accelerated 3D rendering capabilities.
//! Wraps the NS3DViewportGPU to provide a clean, composable interface for panels
//! that need 3D rendering functionality.
//!
//! ## Apple Documentation
//! - [MTKView](https://developer.apple.com/documentation/metalkit/mtkview) - Metal rendering view
//! - [NSView](https://developer.apple.com/documentation/appkit/nsview)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use rust_appkit::prelude::*;
//! use rust_appkit::render_view::NS3DRenderView;
//!
//! // Create a 3D render view with a scene
//! let scene = Arc::new(Mutex::new(Scene::new()));
//! let mut render_view = NS3DRenderView::new(frame, scene);
//!
//! // Configure selection callback
//! render_view.set_on_selection_changed(|entity| {
//!     println!("Selected entity: {:?}", entity);
//! });
//!
//! // Configure file drop callback
//! render_view.set_on_file_dropped(|path| {
//!     println!("File dropped: {:?}", path);
//! });
//!
//! // Add to parent view
//! parent_view.add_subview(render_view.view().clone());
//! ```
//!
//! ## Delegate Pattern
//!
//! The render view supports callbacks for user interactions:
//! - `set_on_selection_changed`: Called when an entity is selected in the 3D view
//! - `set_on_file_dropped`: Called when a file is dropped onto the view
//!
//! ## See Also
//! - [NS3DViewportGPU](crate::gpu::NS3DViewportGPU) - Underlying GPU rendering implementation
//! - [ViewportCameraController](crate::gpu::ViewportCameraController) - Camera control

use crate::ecs::{Entity, Scene};
use crate::gpu::NS3DViewportGPU;
use crate::prelude::*;
use std::sync::{Arc, Mutex};

/// A reusable 3D rendering view component that wraps NS3DViewportGPU.
///
/// This component provides a clean interface for embedding 3D rendering
/// capabilities into panels without manual GPU setup.
pub struct NS3DRenderView {
    viewport: NS3DViewportGPU,
}

impl NS3DRenderView {
    /// Creates a new 3D render view with the specified frame and scene.
    ///
    /// # Arguments
    ///
    /// * `frame` - The initial frame rectangle for the view
    /// * `scene` - Shared reference to the ECS scene to render
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let scene = Arc::new(Mutex::new(Scene::new()));
    /// let render_view = NS3DRenderView::new(
    ///     NSRect::new(0.0, 0.0, 800.0, 600.0),
    ///     scene
    /// );
    /// ```
    pub fn new(frame: NSRect, scene: Arc<Mutex<Scene>>) -> Self {
        let viewport = NS3DViewportGPU::new(frame, scene);
        Self { viewport }
    }

    /// Returns a reference to the underlying NSView.
    pub fn view(&self) -> &NSView {
        self.viewport.view()
    }

    /// Returns a mutable reference to the underlying NSView.
    pub fn view_mut(&mut self) -> &mut NSView {
        self.viewport.view_mut()
    }

    /// Returns a reference to the scene being rendered.
    pub fn scene(&self) -> &Arc<Mutex<Scene>> {
        &self.viewport.scene
    }

    /// Zoom the camera by a delta amount.
    /// 
    /// # Arguments
    /// 
    /// * `delta` - Zoom delta (positive to zoom in, negative to zoom out)
    /// * `sensitivity` - Zoom sensitivity multiplier
    pub fn zoom_camera(&mut self, delta: f32, sensitivity: f32) {
        if let Ok(mut cam) = self.viewport.camera.lock() {
            cam.handle_zoom(delta, sensitivity);
        }
    }

    /// Reset the camera to its default position.
    pub fn reset_camera(&mut self) {
        if let Ok(mut cam) = self.viewport.camera.lock() {
            cam.reset();
        }
    }

    /// Sets the callback to be invoked when an entity is selected in the 3D view.
    ///
    /// # Arguments
    ///
    /// * `callback` - Function to call with the selected entity (or None for deselection)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// render_view.set_on_selection_changed(|entity| {
    ///     if let Some(e) = entity {
    ///         println!("Selected entity ID: {}", e.id());
    ///     } else {
    ///         println!("Deselected");
    ///     }
    /// });
    /// ```
    pub fn set_on_selection_changed<F>(&mut self, callback: F)
    where
        F: Fn(Option<Entity>) + Send + Sync + 'static,
    {
        self.viewport.set_on_selection_changed(callback);
    }

    /// Sets the callback to be invoked when a file is dropped onto the view.
    ///
    /// # Arguments
    ///
    /// * `callback` - Function to call with the path of the dropped file
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// render_view.set_on_file_dropped(|path| {
    ///     println!("Importing file: {:?}", path);
    ///     // Handle file import...
    /// });
    /// ```
    pub fn set_on_file_dropped<F>(&mut self, callback: F)
    where
        F: Fn(std::path::PathBuf) + Send + Sync + 'static,
    {
        self.viewport.set_on_file_dropped(callback);
    }

    /// Sets the currently selected entity for visual highlighting.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - The ID of the entity to select, or None to clear selection
    pub fn set_selected_entity(&mut self, entity_id: Option<u64>) {
        self.viewport.set_selected_entity(entity_id);
    }

    /// Invalidates the viewport cache, forcing a full re-render on the next draw call.
    pub fn invalidate_cache(&self) {
        crate::gpu::invalidate_viewport_cache();
    }

    /// Returns a reference to the camera controller for direct manipulation.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// if let Ok(mut cam) = render_view.camera().lock() {
    ///     cam.yaw = 0.0;
    ///     cam.pitch = 0.0;
    ///     cam.update_from_arcball();
    /// }
    /// ```
    pub fn camera(&self) -> &Arc<Mutex<crate::gpu::ViewportCameraController>> {
        &self.viewport.camera
    }
}
