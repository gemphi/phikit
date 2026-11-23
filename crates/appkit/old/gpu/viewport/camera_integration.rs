//! Camera integration — viewport delegate with input handling and rendering dispatch.
use crate::ecs::{Entity, Scene};
use crate::event::{NSEvent, NSEventType};
use crate::gpu::{GpuRenderer, ViewportCameraController};
use crate::{NSPoint, NSView, NSViewDelegate};
use std::sync::{Arc, Mutex};

use super::overlay_drawing::{self, GizmoDragAxis};
use super::rendering;

/// Delegate handling all viewport input events and rendering dispatch.
pub(super) struct NS3DViewportGPUDelegate {
    pub camera: Arc<Mutex<ViewportCameraController>>,
    pub scene: Arc<Mutex<Scene>>,
    last_mouse: NSPoint,
    pub on_selection_changed: Option<Arc<dyn Fn(Option<Entity>) + Send + Sync>>,
    pub on_file_dropped: Option<Arc<dyn Fn(std::path::PathBuf) + Send + Sync>>,
    pub selected_entity: Option<u64>,
    gizmo_drag: GizmoDragAxis,
    gizmo_drag_origin: NSPoint,
}

impl NS3DViewportGPUDelegate {
    pub fn new(
        camera: Arc<Mutex<ViewportCameraController>>,
        scene: Arc<Mutex<Scene>>,
    ) -> Self {
        Self {
            camera,
            scene,
            last_mouse: NSPoint::new(0.0, 0.0),
            on_selection_changed: None,
            on_file_dropped: None,
            selected_entity: None,
            gizmo_drag: GizmoDragAxis::None,
            gizmo_drag_origin: NSPoint::new(0.0, 0.0),
        }
    }
}

impl NSViewDelegate for NS3DViewportGPUDelegate {
    fn mouse_down(&mut self, view: &mut NSView, event: &NSEvent) {
        let pos = event.location_in_window();
        self.last_mouse = pos;
        self.gizmo_drag = GizmoDragAxis::None;

        if event.event_type() == NSEventType::LeftMouseDown {
            let bounds = view.bounds();
            let nx = (pos.x / bounds.width()) * 2.0 - 1.0;
            let ny = 1.0 - (pos.y / bounds.height()) * 2.0;

            use crate::prelude::{Entity as _, Vec3};
            let cam = self.camera.lock().unwrap();
            let view_proj = cam.view_projection_matrix().to_glam();
            let inv_view_proj = view_proj.inverse();

            let near_pt = inv_view_proj.project_point3(Vec3::new(nx, ny, 0.0));
            let far_pt = inv_view_proj.project_point3(Vec3::new(nx, ny, 1.0));
            let ray_dir = (far_pt - near_pt).normalize();
            drop(cam);

            // Check gizmo hit first (if entity is selected)
            if let Some(sel_id) = self.selected_entity {
                let scene = self.scene.lock().unwrap();
                if let Some(t) = scene
                    .world
                    .get::<crate::ecs::Transform>(crate::ecs::Entity::from_id(sel_id))
                {
                    let origin = t.translation;
                    let hit_axis = overlay_drawing::test_gizmo_hit(near_pt, ray_dir, origin);
                    if hit_axis != GizmoDragAxis::None {
                        self.gizmo_drag = hit_axis;
                        self.gizmo_drag_origin = pos;
                        return;
                    }
                }
            }

            // No gizmo hit — do normal selection ray cast
            let scene = self.scene.lock().unwrap();
            overlay_drawing::perform_selection_ray_cast(
                &scene,
                near_pt,
                ray_dir,
                &self.on_selection_changed,
            );
        }
    }

    fn mouse_up(&mut self, _view: &mut NSView, _event: &NSEvent) {
        self.gizmo_drag = GizmoDragAxis::None;
    }
    fn mouse_dragged(&mut self, _view: &mut NSView, event: &NSEvent) {
        let pos = event.location_in_window();
        let delta_x = pos.x - self.last_mouse.x;
        let delta_y = pos.y - self.last_mouse.y;
        self.last_mouse = pos;

        // If dragging a gizmo axis, translate the selected entity
        if self.gizmo_drag != GizmoDragAxis::None {
            if let Some(sel_id) = self.selected_entity {
                let cam = self.camera.lock().unwrap();
                let move_scale = cam.distance * 0.003;
                drop(cam);

                let mut scene = self.scene.lock().unwrap();
                if let Some(t) = scene
                    .world
                    .get_mut::<crate::ecs::Transform>(crate::ecs::Entity::from_id(sel_id))
                {
                    match self.gizmo_drag {
                        GizmoDragAxis::X => t.translation.x += delta_x * move_scale,
                        GizmoDragAxis::Y => t.translation.y -= delta_y * move_scale,
                        GizmoDragAxis::Z => t.translation.z += delta_x * move_scale,
                        GizmoDragAxis::None => {}
                    }
                }
            }
            return;
        }

        let mut cam = self.camera.lock().unwrap();
        if event.event_type() == NSEventType::RightMouseDragged || event.is_option_key_down() {
            cam.handle_zoom(-delta_y * 0.01, 1.0);
        } else if event.is_shift_key_down() {
            cam.handle_pan(delta_x, delta_y, 0.01);
        } else {
            cam.handle_drag(delta_x * 0.005, -delta_y * 0.005, 1.0);
        }
    }

    fn scroll_wheel(&mut self, _view: &mut NSView, event: &NSEvent) {
        let (_, dy, _) = event.scrolling_delta();
        let mut cam = self.camera.lock().unwrap();
        cam.handle_zoom(-dy as f32 * 0.05, 1.0);
    }
    fn key_down(&mut self, _view: &mut NSView, event: &NSEvent) {
        if let Some(chars) = event.characters() {
            match chars.to_lowercase().as_str() {
                "f" => {
                    if let Some(sel_id) = self.selected_entity {
                        let scene = self.scene.lock().unwrap();
                        if let Some(t) = scene
                            .world
                            .get::<crate::ecs::Transform>(crate::ecs::Entity::from_id(sel_id))
                        {
                            let mut cam = self.camera.lock().unwrap();
                            cam.focus_on(crate::math::SCNVector3::new(
                                t.translation.x,
                                t.translation.y,
                                t.translation.z,
                            ));
                        }
                    } else {
                        println!("[Viewport] Focus requested (no selection)");
                    }
                }
                _ => {}
            }
        }
    }

    fn gpu_draw(
        &self, _view: &NSView, _renderer: &GpuRenderer,
        encoder: &mut wgpu::CommandEncoder, target_view: &wgpu::TextureView,
    ) {
        rendering::perform_gpu_draw(
            _view,
            _renderer,
            encoder,
            target_view,
            &self.camera,
            &self.scene,
            self.selected_entity,
        );
    }

    fn draw(&self, view: &NSView, context: &mut crate::render::RenderContext) {
        rendering::draw_with_cache(
            view,
            context,
            &self.camera,
            &self.scene,
            self.selected_entity,
        );
    }

    fn dropped_file(&mut self, _view: &mut NSView, event: &crate::event::NSEvent) {
        if let Some(path) = &event.file_path {
            println!("[Viewport] File dropped: {:?}", path);
            if let Some(cb) = &self.on_file_dropped { cb(path.clone()); }
        }
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
