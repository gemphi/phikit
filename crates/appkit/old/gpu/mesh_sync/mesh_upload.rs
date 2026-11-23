//! Mesh buffer creation — builtin primitives, file loading, and fallback cubes.

use super::super::renderer::GpuRenderer;

impl GpuRenderer {
    /// Create a mesh buffer from a builtin primitive type (sphere, cylinder, cube).
    ///
    /// Returns the created `MeshBuffer` with material data applied.
    pub(crate) fn create_builtin_mesh(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        model: &crate::ecs::ModelComponent,
        entity_id: u64,
        selection_f: f32,
        transform_matrix: [[f32; 4]; 4],
    ) -> crate::gpu::MeshBuffer {
        let layout = self.material_bind_group_layout.as_ref().unwrap();
        let color = model.base_color;
        let mut mesh = match model.mesh_path.as_str() {
            "builtin:sphere" => {
                crate::gpu::MeshBuffer::sphere(device, layout, 0.4, color, entity_id)
            }
            "builtin:cylinder" => {
                crate::gpu::MeshBuffer::cylinder(device, layout, 0.3, 0.8, color, entity_id)
            }
            _ => crate::gpu::MeshBuffer::cube(device, layout, 0.4, color, entity_id),
        };
        mesh.material_data.roughness = model.roughness;
        mesh.material_data.metallic = model.metallic;
        mesh.material_data.is_selected = selection_f;
        mesh.material_data.model_matrix = transform_matrix;
        mesh.material_data.base_color = model.base_color;
        mesh.material_data.env_intensity = model.env_intensity;
        mesh.material_data.anisotropy = model.anisotropy;
        queue.write_buffer(
            &mesh.material_buffer,
            0,
            bytemuck::cast_slice(&[mesh.material_data]),
        );
        println!(
            "[GpuRenderer] sync_scene_meshes: builtin primitive entity={} type={}",
            entity_id, model.mesh_path
        );
        mesh
    }

    /// Load a mesh from a file path and apply material data.
    ///
    /// Returns `Some(MeshBuffer)` on success, `None` if loading fails or path doesn't exist.
    pub(crate) fn load_mesh_from_file(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        model: &crate::ecs::ModelComponent,
        entity_id: u64,
        selection_f: f32,
        transform_matrix: [[f32; 4]; 4],
    ) -> Option<crate::gpu::MeshBuffer> {
        let path = std::path::Path::new(&model.mesh_path);
        if !path.exists() {
            println!(
                "[GpuRenderer] sync_scene_meshes: path not found entity={} path={:?}",
                entity_id, path
            );
            return None;
        }

        println!(
            "[GpuRenderer] sync_scene_meshes: loading entity={} path={:?}",
            entity_id, path
        );
        match crate::gpu::AssetLoader::load_mesh(
            device,
            self.material_bind_group_layout.as_ref().unwrap(),
            path,
            entity_id,
            model.base_color,
        ) {
            Ok(mesh) => {
                let mut mesh = mesh;
                mesh.material_data.roughness = model.roughness;
                mesh.material_data.metallic = model.metallic;
                mesh.material_data.is_selected = selection_f;
                mesh.material_data.model_matrix = transform_matrix;
                mesh.material_data.env_intensity = model.env_intensity;
                mesh.material_data.anisotropy = model.anisotropy;

                // Use base_color from ModelComponent (set by importer or scene JSON)
                // Guard: if model.base_color is all zeros, use white to prevent black rendering
                let bc = model.base_color;
                if bc[0] + bc[1] + bc[2] < 0.01 {
                    mesh.material_data.base_color = [1.0, 1.0, 1.0, 1.0];
                    println!(
                        "[GpuRenderer] WARNING: entity={} has zero base_color, using white",
                        entity_id
                    );
                } else {
                    mesh.material_data.base_color = bc;
                }
                println!(
                    "[GpuRenderer] entity={} base_color=[{:.2},{:.2},{:.2},{:.2}]",
                    entity_id,
                    mesh.material_data.base_color[0],
                    mesh.material_data.base_color[1],
                    mesh.material_data.base_color[2],
                    mesh.material_data.base_color[3]
                );

                // AABB Debug Logging
                if let Some((min, max)) = mesh.bounds {
                    println!(
                        "[GpuRenderer] Mesh AABB: entity={} bounds=[{:.3},{:.3},{:.3}] -> [{:.3},{:.3},{:.3}]",
                        entity_id, min[0], min[1], min[2], max[0], max[1], max[2]
                    );
                    if min == [0.0, 0.0, 0.0] && max == [0.0, 0.0, 0.0] {
                        println!("[GpuRenderer] WARNING: Mesh AABB is empty! Vertex extraction might be failing.");
                    }
                }

                queue.write_buffer(
                    &mesh.material_buffer,
                    0,
                    bytemuck::cast_slice(&[mesh.material_data]),
                );
                println!(
                    "[GpuRenderer] sync_scene_meshes: loaded OK entity={} verts={} indices={}",
                    entity_id, mesh.vertex_count, mesh.index_count
                );
                Some(mesh)
            }
            Err(e) => {
                println!(
                    "[GpuRenderer] sync_scene_meshes: load FAILED entity={} path={:?} err={}",
                    entity_id, path, e
                );
                None
            }
        }
    }

    /// Create a fallback cube mesh when no other mesh source is available.
    ///
    /// Uses white base_color to avoid black rendering when model.base_color is unset.
    pub(crate) fn create_fallback_cube(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        model: &crate::ecs::ModelComponent,
        entity_id: u64,
        selection_f: f32,
        transform_matrix: [[f32; 4]; 4],
    ) -> crate::gpu::MeshBuffer {
        let color: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        let cube_size = 0.4;
        let mut cube = crate::gpu::MeshBuffer::cube(
            device,
            self.material_bind_group_layout.as_ref().unwrap(),
            cube_size,
            color,
            entity_id,
        );
        cube.material_data.roughness = model.roughness;
        cube.material_data.metallic = model.metallic;
        cube.material_data.is_selected = selection_f;
        cube.material_data.model_matrix = transform_matrix;
        cube.material_data.env_intensity = model.env_intensity;
        cube.material_data.anisotropy = model.anisotropy;
        queue.write_buffer(
            &cube.material_buffer,
            0,
            bytemuck::cast_slice(&[cube.material_data]),
        );
        cube
    }
}
