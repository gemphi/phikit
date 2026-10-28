//! Shared geometry: a collection instance draws its member's mesh, and the
//! member is converted once however many instances show it. Blender keeps
//! one `Mesh` under every dupli of an object (`make_duplis_collection`,
//! `object_dupli.cc`), and Cycles one `Geometry` under its many `Object`s
//! (`scene/object.cpp`); here every instance's `SceneObject` holds the
//! same `Arc<SceneMesh>` (and the same material slots and UVs) as its
//! source, so a renderer that keys on the `Arc` uploads the triangles once
//! and places them per instance (phiray, roadmap §4.4, Step 1).

use super::SceneObject;
use crate::blend::BlendSceneObject;
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
mod tests {
    use super::*;
    use std::sync::Arc;

    fn object(name: &str, block: u64, source: Option<u64>, verts: usize) -> BlendSceneObject {
        BlendSceneObject {
            name: name.into(),
            block_address: block,
            instance_source: source.map(|m| (m, crate::scene::IDENTITY_M4)),
            is_instance: source.is_some(),
            vertices: vec![[0.0; 3]; verts],
            ..BlendSceneObject::default()
        }
    }

    /// Two instances of a member in the scene share the member; two of a
    /// member outside it share the first of them.
    #[test]
    fn an_instance_shares_its_member_or_the_first_instance_of_it() {
        let raw = [
            object("Car", 10, None, 3),
            object("Lot/Car", 0, Some(10), 3),
            object("Lot.001/Car", 0, Some(10), 3),
            object("Lot/Tree", 0, Some(20), 3),
            object("Lot.001/Tree", 0, Some(20), 3),
            object("Lot/Empty", 0, Some(30), 0),
        ];
        let wanted: HashSet<usize> = (0..raw.len()).collect();
        assert_eq!(sources(&raw, &wanted), vec![None, Some(0), Some(0), None, Some(3), None]);
        // A member not wanted (a camera, say) is nobody's source.
        let wanted: HashSet<usize> = (1..raw.len()).collect();
        assert_eq!(sources(&raw, &wanted)[..3], [None, None, Some(1)]);
    }

    #[test]
    fn a_shared_object_holds_the_sources_arcs_under_its_own_placement() {
        let mesh = crate::scene::SceneMesh { vertices: vec![[1.0; 3]], indices: vec![0, 0, 0], normals: crate::scene::SceneNormals::PerFace(Vec::new()), corners: Default::default() };
        let mut from = SceneObject::identity("Car", mesh);
        from.uvs = Arc::new(vec![[0.5, 0.5]]);
        let mut src = object("Lot/Car", 0, Some(10), 3);
        src.matrix_world[3] = [4.0, 5.0, 6.0, 1.0];
        let got = shared_object(&src, &from);
        assert!(Arc::ptr_eq(&got.mesh, &from.mesh) && Arc::ptr_eq(&got.uvs, &from.uvs) && Arc::ptr_eq(&got.material_indices, &from.material_indices));
        assert_eq!((got.name.as_str(), got.matrix_world[3]), ("Lot/Car", [4.0, 5.0, 6.0, 1.0]));
        let mut objects = vec![from, SceneObject::identity("Other", crate::scene::SceneMesh { vertices: Vec::new(), indices: Vec::new(), normals: crate::scene::SceneNormals::PerFace(Vec::new()), corners: Default::default() })];
        alias(&mut objects, 0, 1);
        assert!(Arc::ptr_eq(&objects[0].mesh, &objects[1].mesh));
        alias(&mut objects, 0, 7);
    }
}
