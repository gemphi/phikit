//! Shared geometry: a collection instance draws its member's mesh, and the
//! member is converted once however many instances show it. Blender keeps
//! one `Mesh` under every dupli of an object (`make_duplis_collection`,
//! `object_dupli.cc`), and Cycles one `Geometry` under its many `Object`s
//! (`scene/object.cpp`); here every instance's `SceneObject` holds the
//! same `Arc<SceneMesh>` (and the same material slots and UVs) as its
//! source, so a renderer that keys on the `Arc` uploads the triangles once
//! and places them per instance (phiray, roadmap §4.4, Step 1).

use crate::scene::SceneObject;
use crate::device::blend::BlendSceneObject;
use std::collections::{HashMap, HashSet};

/// The object each instance shares its geometry with: blend index → the
/// blend index of its source — the member itself when the scene has it,
/// else the first instance of that member — or `None` for an object that
/// is its own source. The rule is `modifier_jobs`'s `by_block`, so an
/// instance's source is the object whose modifier stack was evaluated.
pub(super) fn sources(raw: &[BlendSceneObject], wanted: &HashSet<usize>) -> Vec<Option<usize>> {
    let mut first: HashMap<u64, usize> = HashMap::new();
    raw.iter()
        .enumerate()
        .map(|(i, o)| {
            if !wanted.contains(&i) || o.vertices.is_empty() {
                return None;
            }
            let key = match o.instance_source {
                Some((member, _)) => member,
                None => o.block_address,
            };
            match first.get(&key) {
                Some(&source) => Some(source),
                None => {
                    first.insert(key, i);
                    None
                }
            }
        })
        .collect()
}

/// `src` drawn with `from`'s geometry: the member's mesh, material slots
/// and UVs under the instance's own name, placement, colour and shadow
/// (`instance_of` gave it those; the geometry it copied is dropped).
pub(super) fn shared_object(src: &BlendSceneObject, from: &SceneObject) -> SceneObject {
    SceneObject {
        name: src.name.clone(),
        mesh: from.mesh.clone(),
        location: src.location,
        scale: src.scale,
        matrix_world: src.matrix_world,
        materials: from.materials.clone(),
        material_indices: from.material_indices.clone(),
        uvs: from.uvs.clone(),
        color: src.color,
        shadow: src.shadow,
    }
}

/// `objects[to]` takes `objects[from]`'s geometry (the `Arc`s, not a copy).
pub(super) fn alias(objects: &mut [SceneObject], from: usize, to: usize) {
    if from == to || from >= objects.len() || to >= objects.len() {
        return;
    }
    let (mesh, slots, uvs) = (objects[from].mesh.clone(), objects[from].material_indices.clone(), objects[from].uvs.clone());
    objects[to].mesh = mesh;
    objects[to].material_indices = slots;
    objects[to].uvs = uvs;
}

#[cfg(test)]
#[path = "../../../test/read/scene/model/share.rs"]
mod share_test;
