//! A non-rigged mesh whose modifier stack is driven — kept past open so
//! a frame that changes the driven settings re-evaluates the geometry.
//!
//! `BlendSkinSource` is the rigged half: an Armature splits its stack and
//! re-poses + re-evaluates every frame. This is the other half: an object
//! with no Armature in its stack whose modifiers carry drivers (a driven
//! Displace strength, a driven Subdivision level). Blender's depsgraph
//! evaluates a copy per frame; this does the same by cloning the stack,
//! applying the driven settings, retaining non-disabled modifiers and
//! running the result.
//!
//! **Topology changes per frame.** A driven Subdivision level can go from
//! 0 to 2 between frames, which changes the vertex and triangle counts,
//! so the triangles cannot be kept from the open — each frame builds its
//! own `STriangles` and `SceneMesh`.

use super::driver::{BModValue, BModWrite};
use super::{BModifier, BlendEvalMesh, EvalCtx};
use crate::blend::file::types::*;
use glam::Mat4;
use std::collections::HashMap;

/// A non-rigged mesh retained for per-frame driven re-evaluation.
pub struct BlendDrivenSource {
    /// Index into the scene's objects.
    pub object: usize,
    /// The object's saved address — keyed the same way as `BlendSkinSource`.
    pub object_address: u64,
    /// The rest mesh, before any modifier.
    pub base: BlendEvalMesh,
    /// The full modifier stack, in file order.
    pub(crate) stack: Vec<Box<dyn BModifier>>,
}

impl std::fmt::Debug for BlendDrivenSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlendDrivenSource")
            .field("object", &self.object)
            .field("stack", &self.stack.len())
            .finish()
    }
}

impl BlendDrivenSource {
    /// The evaluated mesh at one frame, with this frame's driven settings
    /// over the stack's own — the per-frame path.
    ///
    /// The stack is **cloned** before the settings land, which is what
    /// Blender does: the depsgraph evaluates a copy of the object and the
    /// drivers write into that copy, never into the file's stack
    /// (`BKE_modifier_copydata_generic`).
    pub fn evaluate_at(
        &self,
        world: Mat4,
        driven: &[(BModWrite, BModValue)],
    ) -> BlendEvalMesh {
        let mut stack: Vec<Box<dyn BModifier>> = self
            .stack
            .iter()
            .map(|m| m.clone_box())
            .collect();
        for m in stack.iter_mut() {
            for (write, value) in driven {
                m.set_driven(&write.prop, write.component, *value);
            }
        }
        // `is_disabled` on the settings this frame's drivers left.
        stack.retain(|m| !m.disabled());
        if stack.is_empty() {
            return self.base.clone();
        }
        let mut mesh = self.base.clone();
        let worlds = HashMap::new();
        let ctx = EvalCtx { world, worlds: &worlds };
        for m in &stack {
            m.apply(&mut mesh, &ctx);
        }
        mesh
    }
}

impl BlendFile {
    /// Every mesh object that carries modifier drivers but has no Armature
    /// modifier — the objects whose geometry must be re-evaluated per frame
    /// when a driver changes a modifier's setting.
    ///
    /// Objects *with* an Armature are already retained by `skin_sources`;
    /// this collects the rest. An object whose modifier drivers are all on
    /// modifiers the stack does not model (an unsupported type) is left
    /// out — its geometry does not change.
    pub(crate) fn driven_sources(
        &self,
        objects: &[BlendSceneObject],
        dna: &DnaType,
    ) -> Vec<BlendDrivenSource> {
        let driven_addrs: std::collections::HashSet<u64> = {
            let held = self
                .driven_modifiers
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            held.keys().copied().collect()
        };
        if driven_addrs.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::new();
        for (i, o) in objects.iter().enumerate() {
            if o.object_type != "MESH" || o.vertices.is_empty() {
                continue;
            }
            if !driven_addrs.contains(&o.block_address) {
                continue;
            }
            let Some(ob) = self.block_at_code(o.block_address, "OB") else {
                continue;
            };
            let stack = self.modifier_stack(ob, dna);
            // Skip objects with an Armature — they are `skin_sources`.
            if stack.iter().any(|m| m.armature_source().is_some()) {
                continue;
            }
            if stack.is_empty() {
                continue;
            }
            let me = self.mesh_block_of(ob, dna);
            let dverts_loaded = stack.iter().any(|m| m.wants_weights())
                && me.is_some();
            let dverts = match (dverts_loaded, me) {
                (true, Some(me)) => self.deform_verts(me, dna, o.vertices.len()),
                _ => Vec::new(),
            };
            let creases = me
                .map(|me| self.mesh_creases(me, dna, o.vertices.len()))
                .unwrap_or_default();
            let loose = me
                .map(|me| self.mesh_loose_edges(me, dna, &o.faces, o.vertices.len()))
                .unwrap_or_default();
            let base = BlendEvalMesh {
                vertices: o.vertices.clone(),
                faces: o.faces.clone(),
                loop_uvs: o.loop_uvs.clone(),
                loop_colors: o.loop_colors.clone(),
                face_materials: o.face_materials.clone(),
                face_smooth: o.face_smooth.clone(),
                dverts,
                edge_creases: creases.edge,
                vert_creases: creases.vert,
                loose_edges: loose,
                sharp_edges: me.map(|me| self.mesh_sharp_edges(me, dna, o.vertices.len())).unwrap_or_default(),
            };
            out.push(BlendDrivenSource {
                object: i,
                object_address: o.block_address,
                base,
                stack,
            });
        }
        out
    }
}
