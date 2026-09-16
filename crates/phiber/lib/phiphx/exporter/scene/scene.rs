//! `PhxScene` construction and conversion methods.

use super::types::{
    PhxCamera, PhxCollection, PhxInstance, PhxLight, PhxMaterial, PhxMesh, PhxScene,
};
use crate::phx::{PointCloud, Vertex};

impl PhxScene {
    /// Create a new empty scene
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            frame: 1,
            resolution: [1920, 1080],
            format: "phx-scene-v1".to_string(),
            camera: None,
            lights: Vec::new(),
            meshes: Vec::new(),
            instances: Vec::new(),
            materials: Vec::new(),
            collections: Vec::new(),
        }
    }

    /// Add a camera to the scene
    pub fn with_camera(mut self, camera: PhxCamera) -> Self {
        self.camera = Some(camera);
        self
    }

    /// Add a light to the scene
    pub fn add_light(&mut self, light: PhxLight) {
        self.lights.push(light);
    }

    /// Add a mesh to the scene
    pub fn add_mesh(&mut self, mesh: PhxMesh) {
        self.meshes.push(mesh);
    }

    /// Add an instance to the scene
    pub fn add_instance(&mut self, instance: PhxInstance) {
        self.instances.push(instance);
    }

    /// Add a material to the scene
    pub fn add_material(&mut self, material: PhxMaterial) {
        self.materials.push(material);
    }

    /// Add a collection to the scene
    pub fn add_collection(&mut self, collection: PhxCollection) {
        self.collections.push(collection);
    }

    /// Get total vertex count across all meshes
    pub fn total_vertices(&self) -> usize {
        self.meshes.iter().map(|m| m.vertices.len()).sum()
    }

    /// Get total triangle count across all meshes
    pub fn total_triangles(&self) -> usize {
        self.meshes.iter().map(|m| m.indices.len()).sum()
    }

    /// Convert mesh vertices to a point cloud
    pub fn to_point_cloud(&self) -> PointCloud {
        let mut cloud = PointCloud::new(&self.name);

        for mesh in &self.meshes {
            for (i, vertex) in mesh.vertices.iter().enumerate() {
                let mut v = Vertex::new(vertex[0], vertex[1], vertex[2]);

                if i < mesh.normals.len() {
                    let n = mesh.normals[i];
                    v = v.with_normal(n[0], n[1], n[2]);
                }

                if i < mesh.uvs.len() {
                    let uv = mesh.uvs[i];
                    v = v.with_uv(uv[0], uv[1]);
                }

                cloud.add_vertex(v);
            }
        }

        cloud
    }
}
