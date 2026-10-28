//! World-matrix resolution and view-layer flag application — the parts of
//! `BKE_object_where_is_calc` / `BKE_layer_collection_sync` Phiber needs.

use crate::blend::file::types::*;
use crate::blend::object::rotation::compose_world;
use crate::blend::object::vertex_parent::{is_vertex_parent, parent_matrix};
use glam::Mat4;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
enum BSolve {
    Unsolved,
    Solving,
    Done,
}

impl BlendFile {
    /// Rebuild every object's world matrix from its parent chain, the way
    /// `BKE_object_where_is_calc` does (`world = parentmat · parentinv ·
    /// local`, `solve_parenting`). Objects whose parent is not in the scene
    /// (or a cycle) fall back to their local matrix.
    ///
    /// `PAROBJECT` takes the parent's world; a vertex parent (`PARVERT1`,
    /// `PARVERT3`) the frame on the parent's vertices
    /// (`object/vertex_parent.rs`), taken here from the mesh as **saved** —
    /// the modifiers have not run yet when this is first called. Once they
    /// have, [`Self::reparent_to_vertices`] moves those children to the
    /// evaluated vertices. Bone parenting uses the parent object's matrix
    /// here; `anim/follow.rs` solves it at the frame.
    pub(crate) fn resolve_world_matrices(objects: &mut [BlendSceneObject]) {
        let world = solve_worlds(objects, &|p| objects[p].vertices.as_slice());
        for (object, w) in objects.iter_mut().zip(world) {
            object.matrix_world = w;
        }
    }

    /// Re-solve the vertex-parented children (and everything under them)
    /// against their parents' **evaluated** meshes, `evaluated` keyed by
    /// object index as `evaluate_modifiers` keys it. A parent without an
    /// entry keeps its saved mesh. Every other object keeps the world it
    /// has, so this may run after the frame's follow solve without undoing
    /// it.
    pub(crate) fn reparent_to_vertices(objects: &mut [BlendSceneObject], evaluated: &HashMap<usize, crate::blend::modifier::BlendEvalMesh>) {
        if !objects.iter().any(|o| is_vertex_parent(o.partype)) {
            return;
        }
        let verts = |p: usize| evaluated.get(&p).map_or(objects[p].vertices.as_slice(), |m| m.vertices.as_slice());
        let moved = vertex_children_worlds(objects, &verts);
        for (i, w) in moved {
            objects[i].matrix_world = w;
        }
    }
}

/// `(object index, world)` for every object whose world depends on a vertex
/// parent — the vertex-parented children and their descendants — solved
/// from the parents' current worlds and `verts_of` (their evaluated or
/// saved vertices).
pub(crate) fn vertex_children_worlds<'a>(objects: &'a [BlendSceneObject], verts_of: &dyn Fn(usize) -> &'a [[f32; 3]]) -> Vec<(usize, [[f32; 4]; 4])> {
    let parent = parent_index(objects);
    let under: Vec<bool> = (0..objects.len())
        .map(|i| {
            let (mut at, mut hops) = (Some(i), 0);
            while let Some(k) = at.filter(|_| hops < objects.len()) {
                if is_vertex_parent(objects[k].partype) && parent(k).is_some() {
                    return true;
                }
                at = parent(k);
                hops += 1;
            }
            false
        })
        .collect();
    let mut world: Vec<[[f32; 4]; 4]> = objects.iter().map(|o| o.matrix_world).collect();
    let mut state: Vec<BSolve> = under.iter().map(|&u| match u {
        true => BSolve::Unsolved,
        false => BSolve::Done,
    }).collect();
    for i in 0..objects.len() {
        solve(i, objects, &parent, verts_of, &mut world, &mut state);
    }
    (0..objects.len()).filter(|&i| under[i]).map(|i| (i, world[i])).collect()
}

/// Every object's world from scratch: `parentmat · parentinv · local`.
fn solve_worlds<'a>(objects: &'a [BlendSceneObject], verts_of: &dyn Fn(usize) -> &'a [[f32; 3]]) -> Vec<[[f32; 4]; 4]> {
    let parent = parent_index(objects);
    let mut world: Vec<[[f32; 4]; 4]> = objects.iter().map(|o| o.matrix_local).collect();
    let mut state = vec![BSolve::Unsolved; objects.len()];
    for i in 0..objects.len() {
        solve(i, objects, &parent, verts_of, &mut world, &mut state);
    }
    world
}

/// The index of each object's parent, when it is in the scene and not the
/// object itself.
fn parent_index(objects: &[BlendSceneObject]) -> impl Fn(usize) -> Option<usize> + '_ {
    let by_addr: HashMap<u64, usize> = objects.iter().enumerate().filter(|(_, o)| o.block_address != 0).map(|(i, o)| (o.block_address, i)).collect();
    move |i| by_addr.get(&objects[i].parent_address).copied().filter(|&p| p != i)
}

/// `solve_parenting` for object `i`, its parent first (a cycle leaves the
/// local matrix, as `solve_world` in `anim/motion.rs` does).
fn solve<'a>(
    i: usize,
    objects: &'a [BlendSceneObject],
    parent: &impl Fn(usize) -> Option<usize>,
    verts_of: &dyn Fn(usize) -> &'a [[f32; 3]],
    world: &mut [[[f32; 4]; 4]],
    state: &mut [BSolve],
) -> [[f32; 4]; 4] {
    match state[i] {
        BSolve::Done => return world[i],
        BSolve::Solving => return objects[i].matrix_local,
        BSolve::Unsolved => state[i] = BSolve::Solving,
    }
    if let Some(p) = parent(i) {
        let pw = solve(p, objects, parent, verts_of, world, state);
        let o = &objects[i];
        let parentmat = match is_vertex_parent(o.partype) {
            true => parent_matrix(o.partype, o.parent_verts, Mat4::from_cols_array_2d(&pw), verts_of(p)).to_cols_array_2d(),
            false => pw,
        };
        world[i] = compose_world(&parentmat, &o.parentinv, &o.matrix_local);
    }
    state[i] = BSolve::Done;
    world[i]
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
