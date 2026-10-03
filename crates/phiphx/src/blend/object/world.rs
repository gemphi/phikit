//! World-matrix resolution and view-layer flag application — the parts of
//! `BKE_object_where_is_calc` / `BKE_layer_collection_sync` Phiber needs.

use crate::blend::file::types::*;

impl BlendFile {
    /// Rebuild every object's world matrix from its parent chain, the way
    /// `BKE_object_where_is_calc` does (`world = parent.world · parentinv ·
    /// local`). Objects whose parent is not in the scene (or a cycle) fall
    /// back to their local matrix. Only `PAROBJECT` parenting is modelled;
    /// bone/vertex parenting uses the parent object's matrix as an
    /// approximation.
    pub(crate) fn resolve_world_matrices(objects: &mut [BlendSceneObject]) {
        use std::collections::HashMap;
        let by_addr: HashMap<u64, usize> = objects
            .iter()
            .enumerate()
            .filter(|(_, o)| o.block_address != 0)
            .map(|(i, o)| (o.block_address, i))
            .collect();
        let local: Vec<[[f32; 4]; 4]> = objects.iter().map(|o| o.matrix_local).collect();
        let world = crate::blend::anim::motion::solve_world(
            &local,
            |i| by_addr.get(&objects[i].parent_address).copied(),
            |i| &objects[i].parentinv,
        );
        for (object, w) in objects.iter_mut().zip(world) {
            object.matrix_world = w;
        }
    }
}

/// Copy `LayerCollection` exclude / hide flags onto the collection tree.
pub(crate) fn apply_layer_flags(c: &mut BlendCollection, flags: &std::collections::HashMap<u64, u32>) {
    if let Some(f) = flags.get(&c.block_address) {
        c.exclude = f & crate::blend::object::layer::LAYER_COLLECTION_EXCLUDE != 0;
        c.hide_viewport = f & crate::blend::object::layer::LAYER_COLLECTION_HIDE != 0;
    }
    for child in &mut c.children {
        apply_layer_flags(child, flags);
    }
}
