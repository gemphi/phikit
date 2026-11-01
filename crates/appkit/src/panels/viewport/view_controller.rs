//! # NSViewportViewController — 3D Rendering Surface Panel
//!
//! Implements the assembly and management logic for the 3D rendering surface.
//! Mirrors Apple's view controller pattern for modular GPU canvases.
//!
//! This controller is now a thin configuration layer that instantiates and
//! configures the NS3DRenderView component, following the panel refactoring
//! architecture where panels use view components instead of manual UI construction.
//!
//! ## Apple Documentation
//! - [NSViewController](https://developer.apple.com/documentation/appkit/nsviewcontroller)
//! - [NSView](https://developer.apple.com/documentation/appkit/nsview)
//! - [MTKView](https://developer.apple.com/documentation/metalkit/mtkview)
//!
//! ## See Also
//! - [NS3DRenderView](crate::render_view::NS3DRenderView) - 3D rendering view component
//! - [NS3DViewportGPU](crate::gpu::NS3DViewportGPU) - Underlying GPU implementation

use super::model::*;
use crate::ecs::{Entity, EntityMeta, Scene};
use crate::prelude::*;
use crate::render_view::NS3DRenderView;
use crate::theme::{self, keys};
use crate::layout::{spacing, hierarchy};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct NSViewportViewController {
    pub view: NSView,
    pub render_view: NS3DRenderView,
    pub state: NSViewportState,
    pub selected_entity: Arc<Mutex<Option<u64>>>,
}

impl NSViewportViewController {
    pub fn new(
        rect: NSRect,
        entities: Vec<(u64, String, Option<([f32; 3], [f32; 4], [f32; 3])>)>,
        _mesh_paths: &std::collections::HashMap<u64, String>,
        materials: &std::collections::HashMap<u64, ([f32; 4], f32, f32)>,
    ) -> Self {
        let dummy_scene = Arc::new(Mutex::new(Scene::new()));

        // Populate ECS
        if let Ok(mut ecs) = dummy_scene.lock() {
            for (id, name, transform) in entities {
                let entity = ecs.world.spawn_empty();
                let eid = entity.id();
                if let Some((p, r, s)) = transform {
                    ecs.world.insert(
                        Entity::from_id(eid),
                        Transform {
                            translation: glam::Vec3::from_array(p),
                            rotation: glam::Quat::from_array(r),
                            scale: glam::Vec3::from_array(s),
                        },
                    );
                }

                // Add ModelComponent if mesh path is provided for this entity
                if let Some(mesh_path) = _mesh_paths.get(&id) {
                    // Use base_color, roughness, metallic from materials map if available
                    let (base_color, roughness, metallic) = materials
                        .get(&id)
                        .copied()
                        .unwrap_or(([1.0, 1.0, 1.0, 1.0], 0.5, 0.0));
                    ecs.world.insert(
                        Entity::from_id(eid),
                        crate::ecs::ModelComponent {
                            mesh_path: mesh_path.clone(),
                            material_override: None,
                            visible: true,
                            roughness,
                            metallic,
                            base_color,
                            env_intensity: 0.5,
                            anisotropy: 0.0,
                        },
                    );
                }

                ecs.root_entities.push(Entity::from_id(eid));
                ecs.meta.insert(
                    eid,
                    EntityMeta {
                        name,
                        children: Vec::new(),
                    },
                );
            }
        }

        // Create the render view component
        let mut render_view = NS3DRenderView::new(rect, dummy_scene);
        
        // Configure the render view
        render_view.view_mut().set_hidden(false);
        render_view.view_mut().set_frame(NSRect::new(
            0.0,
            0.0,
            rect.width(),
            rect.height(),
        ));
        render_view.view_mut().set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
        );

        // Create container view
        let mut view = NSView::new(rect).with_name("ViewportPanel");
        view.set_background_color(Some(hierarchy::bg_color("viewport")));
        view.set_autoresizing_mask(
            NSAutoresizingMask::WIDTH_SIZABLE | NSAutoresizingMask::HEIGHT_SIZABLE,
        );
        view.set_corner_radius(crate::theme::ThemeToken::RADIUS_PANEL());
        view.set_clips_to_bounds(true);

        // Add render view to container
        view.add_subview(render_view.view().clone());

        Self {
            view,
            render_view,
            state: NSViewportState::new(),
            selected_entity: Arc::new(Mutex::new(None)),
        }
    }

    pub fn on_selection_changed(&mut self, entity_id: Option<u64>) {
        self.render_view.set_selected_entity(entity_id);
        if let Ok(mut s) = self.state.selected_entity.lock() {
            *s = entity_id;
        }
    }
}

impl NSViewportViewController {
    pub fn with_mesh_paths(
        rect: NSRect,
        entities: Vec<(u64, String, Option<([f32; 3], [f32; 4], [f32; 3])>)>,
        mesh_paths: &std::collections::HashMap<u64, String>,
    ) -> Self {
        Self::new(
            rect,
            entities,
            mesh_paths,
            &std::collections::HashMap::new(),
        )
    }

    pub fn render_view(&self) -> &NS3DRenderView {
        &self.render_view
    }
    
    pub fn render_view_mut(&mut self) -> &mut NS3DRenderView {
        &mut self.render_view
    }
    
    pub fn frame_selection(&mut self) { /* Focus camera on selection */
    }

    pub fn set_on_file_dropped(
        &mut self,
        callback: Arc<dyn Fn(std::path::PathBuf) + Send + Sync>,
    ) {
        // Wire file drop callback to render view
        self.render_view.set_on_file_dropped(move |path| {
            callback(path);
        });
    }
}
