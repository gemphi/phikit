//! The mesh a Shrinkwrap wraps to: the target object's **evaluated** mesh,
//! `BKE_modifier_get_evaluated_mesh_from_evaluated_object(ob_target)` in
//! `shrinkwrapModifier_deform` — its own modifier stack applied, in its own
//! object space — and not the mesh saved in the file.
//!
//! A half modelled under Mirror, Solidify and Subdivision (the CW-POSES
//! `console_middle_geo`) is a different surface from its saved half: wrapping
//! to the saved one put `panel_underbelly_geo`'s vertices up to 1.5 cm from
//! where Blender puts them, and its seam vertices off the mirror plane, so
//! the Mirror after it merged fewer of them than Blender's does.
//!
//! The target's stack runs without the scene's world matrices, which only a
//! Mirror about another object, an Armature or a Shrinkwrap in that stack
//! would read; those see no object and act as they do with none. A target
//! whose stack leads back to an object being read (a cycle the depsgraph
//! would break) is wrapped to its saved mesh.

use super::super::{BlendEvalMesh, EvalCtx};
use super::tree::Target;
use super::OB_MESH;
use crate::blend::file::types::*;
use glam::Mat4;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

thread_local! {
    /// Targets whose stacks are being read on this thread, innermost last.
    static READING: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
}

/// A target object's evaluated mesh as a tree, or `None` when it is no mesh.
pub(super) fn read_target(f: &BlendFile, addr: u64, ob: &BlendBlock, dna: &DnaType, vertex_tree: bool) -> Option<Arc<Target>> {
    let mesh = evaluated_mesh_of(f, addr, ob, dna)?;
    let faces: Vec<Vec<u32>> = mesh.faces.iter().map(|c| c.to_vec()).collect();
    Target::new(mesh.vertices, &faces, &mesh.face_smooth, vertex_tree).map(Arc::new)
}

/// The mesh object at `addr`, evaluated in its own space
/// (`BKE_modifier_get_evaluated_mesh_from_evaluated_object`), or `None`
/// when it is no mesh or is `ob` itself. Shared with the Surface Deform,
/// whose target is read the same way.
pub(in super::super) fn evaluated_mesh_of(f: &BlendFile, addr: u64, ob: &BlendBlock, dna: &DnaType) -> Option<BlendEvalMesh> {
    let t_ob = f.block_at_code(addr, "OB")?;
    if t_ob.old_address == ob.old_address || f.int_field(t_ob, "Object", "type", dna) != OB_MESH {
        return None;
    }
    let (positions, faces) = f.scoped(t_ob, || f.extract_mesh_data(t_ob, dna));
    let smooth = f.scoped(t_ob, || f.extract_face_smooth(t_ob, dna, faces.len()));
    let mesh = BlendEvalMesh { vertices: positions, faces: faces.into(), face_smooth: smooth, ..Default::default() };
    Some(evaluated(f, t_ob, dna, mesh))
}

/// `t_ob`'s modifier stack over its saved `mesh`, with the weights, creases
/// and loose edges the stack reads loaded as `modifier_jobs` loads them.
fn evaluated(f: &BlendFile, t_ob: &BlendBlock, dna: &DnaType, mut mesh: BlendEvalMesh) -> BlendEvalMesh {
    let addr = t_ob.old_address;
    if READING.with(|r| r.borrow().contains(&addr)) {
        return mesh;
    }
    READING.with(|r| r.borrow_mut().push(addr));
    let stack = f.modifier_stack(t_ob, dna);
    READING.with(|r| r.borrow_mut().pop());
    let Some(me) = f.mesh_block_of(t_ob, dna).filter(|_| !stack.is_empty()) else { return mesh };
    let n = mesh.vertices.len();
    if stack.iter().any(|m| m.wants_weights()) {
        mesh.dverts = f.deform_verts(me, dna, n);
    }
    let creases = f.mesh_creases(me, dna, n);
    mesh.edge_creases = creases.edge;
    mesh.vert_creases = creases.vert;
    mesh.loose_edges = f.mesh_loose_edges(me, dna, &mesh.faces, n);
    let worlds = HashMap::new();
    let ctx = EvalCtx { world: Mat4::IDENTITY, worlds: &worlds };
    for m in &stack {
        m.apply(&mut mesh, &ctx);
    }
    mesh
}
