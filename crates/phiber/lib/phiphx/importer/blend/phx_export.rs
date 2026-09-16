//! Emit a portable `.phx` scene alongside every imported `.blend`.
//!
//! Parsing a 98 MB blend is slow; the `.phx` sidecar captures the converted
//! scene (per-object geometry + material colours + collection hierarchy) in
//! the engine's own format so it re-imports instantly and can be shared for
//! on-the-fly collaboration (open the `.phx`, no Blender or original blend
//! required). Written best-effort and skipped when already fresh.

use super::types::{BlendCollection, BlendScene};
use crate::phx::exporters::scene::{
    PhxCollection, PhxMaterial, PhxMesh, PhxScene, export_phx_scene_to_bytes,
};
use std::path::Path;

/// Convert a parsed `BlendScene` into the engine's `PhxScene` format.
pub fn blend_scene_to_phx(scene: &BlendScene) -> PhxScene {
    let mut phx = PhxScene::new(&scene.name);
    for obj in &scene.objects {
        if obj.vertices.is_empty() {
            continue;
        }
        let mut indices = Vec::new();
        for face in &obj.faces {
            if face.len() < 3 {
                continue;
            }
            for i in 1..(face.len() - 1) {
                indices.push([face[0], face[i], face[i + 1]]);
            }
        }
        phx.add_mesh(PhxMesh {
            name: obj.name.clone(),
            vertices: obj.vertices.clone(),
            normals: Vec::new(),
            uvs: Vec::new(),
            // Per-vertex material colour so re-import preserves the look.
            colors: vec![obj.base_color; obj.vertices.len()],
            indices,
            materials: vec![format!("{}_mat", obj.name)],
            matrix_world: obj.matrix_world,
            visible: true,
        });
        phx.add_material(PhxMaterial {
            name: format!("{}_mat", obj.name),
            base_color: obj.base_color,
            metallic: obj.metallic,
            roughness: obj.roughness,
            emission: [0.0, 0.0, 0.0],
            emission_strength: 0.0,
            alpha: obj.base_color[3],
            use_nodes: true,
            base_color_texture: None,
            normal_texture: None,
            roughness_texture: None,
        });
    }
    for c in &scene.collections {
        add_collection(&mut phx, c);
    }
    phx
}

fn add_collection(phx: &mut PhxScene, c: &BlendCollection) {
    phx.add_collection(PhxCollection {
        name: c.name.clone(),
        objects: c.object_names.clone(),
        children: c.children.iter().map(|ch| ch.name.clone()).collect(),
        instance_offset: [0.0, 0.0, 0.0],
        is_instance: c.is_instance,
    });
    for child in &c.children {
        add_collection(phx, child);
    }
}

/// Write `<blend>.phx` next to the source blend (best-effort, freshness-gated).
pub fn write_phx_sidecar(blend_path: &Path, scene: &BlendScene) {
    let phx_path = blend_path.with_extension("phx");
    if is_fresh(&phx_path, blend_path) {
        println!("[Blend Native] .phx sidecar up to date: {:?}", phx_path);
        return;
    }
    let phx = blend_scene_to_phx(scene);
    match export_phx_scene_to_bytes(&phx) {
        Ok(bytes) => match std::fs::write(&phx_path, &bytes) {
            Ok(_) => println!(
                "[Blend Native] wrote .phx sidecar ({} meshes, {} bytes): {:?}",
                phx.meshes.len(),
                bytes.len(),
                phx_path
            ),
            Err(e) => eprintln!("[Blend Native] failed to write .phx: {}", e),
        },
        Err(e) => eprintln!("[Blend Native] failed to serialize .phx: {:?}", e),
    }
}

/// True when `phx_path` exists and is at least as new as `blend_path`.
fn is_fresh(phx_path: &Path, blend_path: &Path) -> bool {
    let (Ok(p), Ok(b)) = (std::fs::metadata(phx_path), std::fs::metadata(blend_path)) else {
        return false;
    };
    match (p.modified(), b.modified()) {
        (Ok(pm), Ok(bm)) => pm >= bm,
        _ => false,
    }
}
