//! Shared test fixtures for the scene traversal tests (`tests_traversal`,
//! `tests_material`).  Both files `use super::tests_fixture::*`.

use crate::blend::{BlendScene, BlendSceneObject};

/// Minimal mesh object carrying only the fields the traversal reads
/// (`name`, `vertices`, `faces`, `loop_normals`); everything else inert.
pub fn blend_obj(
    name: &str,
    vertices: Vec<[f32; 3]>,
    faces: Vec<Vec<u32>>,
    loop_normals: Vec<[f32; 3]>,
) -> BlendSceneObject {
    BlendSceneObject {
        name: name.to_string(),
        object_type: "MESH".to_string(),
        location: [0.0; 3],
        rotation: [0.0; 3],
        quaternion: [1.0, 0.0, 0.0, 0.0],
        scale: [1.0; 3],
        matrix_world: crate::scene::IDENTITY_M4,
        matrix_local: crate::scene::IDENTITY_M4,
        block_address: 0,
        parent_address: 0,
        parentinv: crate::scene::IDENTITY_M4,
        partype: 0,
        visibility: Default::default(),
        color: [1.0; 4],
        shadow: Default::default(),
        vertices,
        faces: faces.into(),
        loop_normals,
        face_materials: Vec::new(),
        face_smooth: Vec::new(),
        data: Default::default(),
        loose_edges: Vec::new(),
        loop_uvs: Vec::new(),
        loop_colors: Vec::new(),
        modifiers: Vec::new(),
        skipped_modifiers: Vec::new(),
        instance_targets: Vec::new(),
        is_instance: false,
        base_color: [0.8, 0.8, 0.8, 1.0],
        roughness: 0.5,
        metallic: 0.0,
        materials: Vec::new(),
        texture_paths: Vec::new(),
    }
}

pub fn blend_scene(objects: Vec<BlendSceneObject>) -> BlendScene {
    BlendScene {
        name: "test".to_string(),
        collections: Vec::new(),
        objects,
        active_object: None,
        selected: Vec::new(),
        hidden: Vec::new(),
        props: Default::default(),
        deferred: None,
    }
}
