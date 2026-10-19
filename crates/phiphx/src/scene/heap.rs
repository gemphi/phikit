//! What a loaded scene costs in memory — Blender's `MEM_get_memory_in_use`
//! for the buffers we actually keep.

use super::{SceneModel, SceneNormals};

impl SceneModel {
    /// Approximate heap bytes held by this scene's mesh buffers.
    ///
    /// The figure the open report shows (`report.memory_bytes`). It counts
    /// the buffers, not the allocator's overhead or the block table the
    /// parser has already dropped — so it is the number a user can act on
    /// ("this file costs 400 MB of geometry"), not a total process figure.
    pub fn heap_bytes(&self) -> u64 {
        self.objects.iter().map(Self::object_bytes).sum()
    }

    fn object_bytes(obj: &super::SceneObject) -> u64 {
        let m = &obj.mesh;
        let normals = match &m.normals {
            SceneNormals::PerVertex(v) | SceneNormals::PerFace(v) => v.len(),
        };
        let mut bytes = 0u64;
        bytes += (m.vertices.len() * size_of::<[f32; 3]>()) as u64;
        bytes += (m.indices.len() * size_of::<u32>()) as u64;
        bytes += (normals * size_of::<[f32; 3]>()) as u64;
        bytes += (m.corners.ids.len() * size_of::<u32>()) as u64;
        bytes += (m.corners.normals.len() * size_of::<[f32; 3]>()) as u64;
        bytes += (m.corners.colors.len() * size_of::<[f32; 3]>()) as u64;
        // UVs and material indices live on SceneObject, not SceneMesh.
        bytes += (obj.uvs.len() * size_of::<[f32; 2]>()) as u64;
        bytes += (obj.material_indices.len() * size_of::<u16>()) as u64;
        bytes
    }
}
