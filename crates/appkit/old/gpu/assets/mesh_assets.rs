//! PHX, PHX, and Blend mesh loading with raw buffer support.
//!
//! Contains loaders for PHX point clouds, PHX binary scenes, Blender files,
//! PHX scene mesh conversion, and programmatic raw buffer creation.

use crate::gpu::{MeshBuffer, Vertex};
use std::path::Path;
use super::asset_loading::AssetLoader;

impl AssetLoader {
    /// Load a PHX point cloud/mesh using native phx importer
    pub fn load_phx_point_cloud(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        path: &Path,
        entity_id: u64,
    ) -> Result<MeshBuffer, String> {
        println!("[AssetLoader] Loading PHX: {:?}", path);
        let config = phx::importers::MeshToCloudConfig::default();
        let path_str = path.to_string_lossy().to_string();
        let cloud = phx::importers::usd::UsdImporter::import_file_with_config(&path_str, &config)
            .or_else(|_| {
                let data = std::fs::read(path)
                    .map_err(|e| phx::PointCloudError::ImportError(format!("Read error: {}", e)))?;
                let text = String::from_utf8_lossy(&data);
                phx::PointCloud::import(
                    &text,
                    &phx::PointCloudFormat::Ply {
                        binary: false,
                        include_normals: true,
                        include_colors: true,
                    },
                )
            })
            .map_err(|e| format!("PHX import failed: {:?}", e))?;

        let mut all_vertices = Vec::new();
        for v in cloud.vertices() {
            let normal = v.normal.unwrap_or([0.0, 1.0, 0.0]);
            let color = v
                .color
                .map(|c| {
                    [
                        c[0] as f32 / 255.0,
                        c[1] as f32 / 255.0,
                        c[2] as f32 / 255.0,
                        c[3] as f32 / 255.0,
                    ]
                })
                .unwrap_or([0.8, 0.8, 0.8, 1.0]);
            all_vertices.push(Vertex { position: v.position, normal, color });
        }
        let all_indices: Vec<u32> = (0..all_vertices.len() as u32).collect();
        println!("[AssetLoader] PHX loaded: {} points", all_vertices.len());
        Ok(MeshBuffer::new(
            device, layout, &all_vertices, Some(&all_indices),
            crate::gpu::mesh::MaterialUniform::default(),
            path.to_string_lossy().to_string(), entity_id,
        ))
    }

    /// Load a PHX binary scene (picks first valid mesh)
    pub fn load_phx(
        device: &wgpu::Device, layout: &wgpu::BindGroupLayout,
        path: &Path, entity_id: u64,
    ) -> Result<MeshBuffer, String> {
        println!("[AssetLoader] Loading PHX: {:?}", path);
        let data = std::fs::read(path).map_err(|e| format!("Failed to read PHX: {}", e))?;
        let phx_scene = phx::exporters::phx_scene::import_phx_scene_from_bytes(&data)
            .map_err(|e| format!("PHX parse failed: {:?}", e))?;
        Self::load_from_phx_meshes(device, layout, &phx_scene.meshes, path, entity_id)
    }

    /// Load a .blend file via native SDNA parser
    pub fn load_blend(
        device: &wgpu::Device, layout: &wgpu::BindGroupLayout,
        path: &Path, entity_id: u64,
    ) -> Result<MeshBuffer, String> {
        println!("[AssetLoader] Loading Blend: {:?}", path);
        let blend_scene = phx::importers::blend_native::import_blend_native(path)
            .map_err(|e| format!("Blend import failed: {:?}", e))?;
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        for obj in &blend_scene.objects {
            if obj.vertices.is_empty() { continue; }
            let vertex_offset = all_vertices.len() as u32;
            for pos in &obj.vertices {
                all_vertices.push(Vertex {
                    position: *pos, normal: [0.0, 1.0, 0.0], color: [0.8, 0.8, 0.8, 1.0],
                });
            }
            for face in &obj.faces {
                if face.len() < 3 { continue; }
                for i in 1..(face.len() - 1) {
                    all_indices.push(vertex_offset + face[0]);
                    all_indices.push(vertex_offset + face[i]);
                    all_indices.push(vertex_offset + face[i + 1]);
                }
            }
        }
        if all_vertices.is_empty() {
            return Err("Blend file contains no mesh data".to_string());
        }
        Self::compute_flat_normals(&mut all_vertices, &all_indices);
        println!(
            "[AssetLoader] Blend loaded: {} vertices, {} indices from {} objects",
            all_vertices.len(), all_indices.len(), blend_scene.objects.len()
        );
        Ok(MeshBuffer::new(
            device, layout, &all_vertices, Some(&all_indices),
            crate::gpu::mesh::MaterialUniform::default(),
            path.to_string_lossy().to_string(), entity_id,
        ))
    }

    /// Create MeshBuffer from raw vertex/index arrays (for programmatic mesh creation)
    pub fn load_from_raw_buffers(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        positions: &[[f32; 3]],
        normals: Option<&[[f32; 3]]>,
        indices: &[u32],
        entity_id: u64,
        label: &str,
    ) -> Result<MeshBuffer, String> {
        if positions.is_empty() {
            return Err("Empty vertex buffer".to_string());
        }
        let mut vertices: Vec<Vertex> = positions
            .iter()
            .enumerate()
            .map(|(i, pos)| {
                let normal = normals
                    .and_then(|n| n.get(i))
                    .copied()
                    .unwrap_or([0.0, 1.0, 0.0]);
                Vertex { position: *pos, normal, color: [0.8, 0.8, 0.8, 1.0] }
            })
            .collect();
        if normals.is_none() || normals.map(|n| n.is_empty()).unwrap_or(true) {
            Self::compute_flat_normals(&mut vertices, indices);
        }
        Ok(MeshBuffer::new(
            device, layout, &vertices, Some(indices),
            crate::gpu::mesh::MaterialUniform::default(),
            label.to_string(), entity_id,
        ))
    }

    /// Helper: convert PHX scene meshes to a MeshBuffer
    pub(crate) fn load_from_phx_meshes(
        device: &wgpu::Device, layout: &wgpu::BindGroupLayout,
        meshes: &[phx::exporters::phx_scene::PhxMesh],
        path: &Path, entity_id: u64,
    ) -> Result<MeshBuffer, String> {
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        for mesh in meshes {
            if mesh.vertices.is_empty() { continue; }
            let vertex_offset = all_vertices.len() as u32;
            for (i, pos) in mesh.vertices.iter().enumerate() {
                let normal = mesh.normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]);
                let color = mesh.colors.get(i).copied().unwrap_or([0.8, 0.8, 0.8, 1.0]);
                all_vertices.push(Vertex { position: *pos, normal, color });
            }
            for tri in &mesh.indices {
                all_indices.push(vertex_offset + tri[0]);
                all_indices.push(vertex_offset + tri[1]);
                all_indices.push(vertex_offset + tri[2]);
            }
        }
        if all_vertices.is_empty() {
            return Err("PHX scene contains no mesh data".to_string());
        }
        Self::compute_flat_normals(&mut all_vertices, &all_indices);
        println!(
            "[AssetLoader] PHX loaded: {} vertices, {} indices from {} meshes",
            all_vertices.len(), all_indices.len(), meshes.len()
        );
        Ok(MeshBuffer::new(
            device, layout, &all_vertices, Some(&all_indices),
            crate::gpu::mesh::MaterialUniform::default(),
            path.to_string_lossy().to_string(), entity_id,
        ))
    }
}
