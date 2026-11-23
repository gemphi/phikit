//! Buffer synchronization — bridges ECS scene state to GPU buffers.
//!
//! Orchestrates entity iteration, material updates, and mesh creation.

use super::super::renderer::GpuRenderer;
use std::sync::atomic::{AtomicU64, Ordering};

// Track scene version to avoid redundant mesh syncs
static LAST_SYNCED_SCENE_VERSION: AtomicU64 = AtomicU64::new(0);

impl GpuRenderer {
    /// Ensure GPU mesh buffers exist for all scene entities with ModelComponents.
    /// ✅ OPTIMIZED: Only syncs when scene has changed (dirty tracking)
    pub fn sync_scene_meshes(&self, scene: &crate::ecs::Scene, selected_id: Option<u64>) {
        // Check if scene has changed since last sync
        let current_version = scene.version();
        let last_version = LAST_SYNCED_SCENE_VERSION.load(Ordering::Relaxed);
        
        // Skip sync if scene hasn't changed (major performance win)
        if current_version == last_version && current_version > 0 {
            // Still need to update selection state if it changed
            if let Some(id) = selected_id {
                self.update_selection_only(id);
            }
            return;
        }
        
        let (device, queue) = if let (Some(d), Some(q)) = (&self.device, &self.queue) {
            (d, q)
        } else {
            return;
        };
        let mut buffers = self.mesh_buffers.lock().unwrap();

        // Collect ALL entity IDs (roots + children via meta) for mesh sync
        let all_entity_ids: Vec<u64> = {
            let mut ids: Vec<u64> = scene.root_entities.iter().map(|e| e.id()).collect();
            for &id in scene.meta.keys() {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            }
            ids
        };

        let total_entities = all_entity_ids.len();
        let mut new_loads = 0u32;
        let mut updates = 0u32;
        let mut fallbacks = 0u32;

        for entity_id in all_entity_ids {
            let is_selected = selected_id == Some(entity_id);
            let selection_f = if is_selected { 1.0 } else { 0.0 };

            // Check for ModelComponent
            if let Some(model) = scene
                .world
                .get::<crate::ecs::ModelComponent>(crate::ecs::Entity::from_id(entity_id))
            {
                let transform_matrix = if let Some(t) = scene
                    .world
                    .get::<crate::ecs::Transform>(crate::ecs::Entity::from_id(entity_id))
                {
                    let scale = glam::Mat4::from_scale(t.scale);
                    let rot = glam::Mat4::from_quat(t.rotation);
                    let trans = glam::Mat4::from_translation(t.translation);
                    (trans * rot * scale).to_cols_array_2d()
                } else {
                    glam::Mat4::IDENTITY.to_cols_array_2d()
                };

                // Check if we already have a buffer for this entity
                if let Some(existing) = buffers.iter_mut().find(|m| m.entity_id == entity_id) {
                    // Update material if anything changed
                    if existing.material_data.roughness != model.roughness
                        || existing.material_data.metallic != model.metallic
                        || existing.material_data.is_selected != selection_f
                        || existing.material_data.base_color != model.base_color
                        || existing.material_data.model_matrix != transform_matrix
                        || existing.material_data.env_intensity != model.env_intensity
                        || existing.material_data.anisotropy != model.anisotropy
                    {
                        existing.material_data.roughness = model.roughness;
                        existing.material_data.metallic = model.metallic;
                        existing.material_data.is_selected = selection_f;
                        existing.material_data.base_color = model.base_color;
                        existing.material_data.model_matrix = transform_matrix;
                        existing.material_data.env_intensity = model.env_intensity;
                        existing.material_data.anisotropy = model.anisotropy;

                        // Update GPU buffer
                        queue.write_buffer(
                            &existing.material_buffer,
                            0,
                            bytemuck::cast_slice(&[existing.material_data]),
                        );
                        updates += 1;
                    }
                    continue;
                }

                // Create new buffer — try builtin primitives first, then load from mesh_path
                if model.mesh_path.starts_with("builtin:") {
                    let mesh = self.create_builtin_mesh(
                        device,
                        queue,
                        &model,
                        entity_id,
                        selection_f,
                        transform_matrix,
                    );
                    buffers.push(mesh);
                    new_loads += 1;
                    continue;
                }

                if !model.mesh_path.is_empty() {
                    if let Some(mesh) = self.load_mesh_from_file(
                        device,
                        queue,
                        &model,
                        entity_id,
                        selection_f,
                        transform_matrix,
                    ) {
                        buffers.push(mesh);
                        new_loads += 1;
                        continue;
                    }
                }

                // Fallback to default cube
                let cube = self.create_fallback_cube(
                    device,
                    queue,
                    &model,
                    entity_id,
                    selection_f,
                    transform_matrix,
                );
                buffers.push(cube);
                fallbacks += 1;
            }
        }

        if new_loads > 0 || fallbacks > 0 {
            println!(
                "[GpuRenderer] sync_scene_meshes: entities={} new_loads={} updates={} fallbacks={} total_buffers={}",
                total_entities, new_loads, updates, fallbacks, buffers.len()
            );
        }

        // Update last synced version
        LAST_SYNCED_SCENE_VERSION.store(current_version, Ordering::Relaxed);

        // --- Gizmo Update ---
        drop(buffers); // Release the lock before calling gizmo methods
        if let Some(id) = selected_id {
            self.update_gizmo_vertices(device, scene, id);
        } else {
            self.clear_gizmo();
        }
    }

    /// Fast path: Update only selection state without full mesh sync
    fn update_selection_only(&self, selected_id: u64) {
        let queue = if let Some(q) = &self.queue {
            q
        } else {
            return;
        };
        
        if let Ok(mut buffers) = self.mesh_buffers.try_lock() {
            for buf in buffers.iter_mut() {
                let new_selection = if buf.entity_id == selected_id { 1.0 } else { 0.0 };
                if buf.material_data.is_selected != new_selection {
                    buf.material_data.is_selected = new_selection;
                    queue.write_buffer(
                        &buf.material_buffer,
                        0,
                        bytemuck::cast_slice(&[buf.material_data]),
                    );
                }
            }
        }
    }
}
