//! A mesh an Armature modifier deforms, kept so its rig can pose it at any frame.
use crate::device::blend::modifier::*;

/// A mesh an Armature modifier deforms, kept so its rig can pose it at any
/// frame (`scene/rig.rs`): the mesh the Armature modifier receives (the
/// modifiers before it applied, weights loaded), its vertex groups, the rig
/// object, and the modifiers after it.
pub struct BlendSkinSource {
    /// Index into the scene's objects.
    pub object: usize,
    /// The object's saved address — what a modifier driver set is keyed by
    /// (`modifier/driver.rs`), so a frame's driven settings can find this
    /// mesh's own. The index above is a position in one scene's list; the
    /// address is the identity, as everywhere else.
    pub object_address: u64,
    /// The rig object's block address.
    pub armature: u64,
    pub base: BlendEvalMesh,
    pub groups: Vec<String>,
    pub(crate) tail: Vec<Box<dyn BModifier>>,
}

impl std::fmt::Debug for BlendSkinSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlendSkinSource").field("object", &self.object).field("armature", &self.armature).field("tail", &self.tail.len()).finish()
    }
}

impl BlendSkinSource {
    /// A mesh deformed by the armature at `armature` alone — no modifier
    /// after the Armature: what a mesh built in code (a garment cut from a
    /// body, a hairstyle) is skinned with (`scene/build.rs`).
    pub fn bare(object: usize, armature: u64, base: BlendEvalMesh, groups: Vec<String>) -> Self {
        Self { object, object_address: 0, armature, base, groups, tail: Vec::new() }
    }

    /// The modifiers after the Armature over `deformed` (the base's
    /// vertices, posed) — the mesh the viewport draws at that pose.
    pub fn evaluate(&self, deformed: Vec<[f32; 3]>, world: Mat4) -> BlendEvalMesh {
        self.evaluate_driven(deformed, world, &[])
    }

    /// As [`Self::evaluate`], with this frame's driven settings over the
    /// tail's own — the per-frame path.
    ///
    /// The tail is **cloned** before the settings land on it, which is what
    /// Blender does: the depsgraph evaluates a copy of the object and the
    /// drivers write into that copy, never into the file's stack
    /// (`BKE_modifier_copydata_generic`). Two frames evaluated at once
    /// therefore cannot fight over one `strength` field, and this source
    /// stays shareable behind its `Arc`.
    ///
    /// `driven` empty is the common case and clones nothing.
    pub fn evaluate_driven(
        &self,
        deformed: Vec<[f32; 3]>,
        world: Mat4,
        driven: &[(driver::BModWrite, driver::BModValue)],
    ) -> BlendEvalMesh {
        if driven.is_empty() {
            return self.run(&self.tail, deformed, world);
        }
        let mut tail: Vec<Box<dyn BModifier>> = self.tail.iter().map(|m| m.clone_box()).collect();
        for m in tail.iter_mut() {
            for (write, value) in driven {
                m.set_driven(&write.prop, write.component, *value);
            }
        }
        // `is_disabled` again, on the settings this frame's drivers left —
        // the same reason it is asked after `set_driven` at open.
        tail.retain(|m| !m.disabled());
        self.run(&tail, deformed, world)
    }

    fn run(&self, tail: &[Box<dyn BModifier>], deformed: Vec<[f32; 3]>, world: Mat4) -> BlendEvalMesh {
        if tail.is_empty() {
            return BlendEvalMesh { vertices: deformed, ..Default::default() };
        }
        // Positions only: the UVs, materials and weights the modifiers would
        // carry along are the ones read with the mesh.
        let mut mesh = BlendEvalMesh {
            vertices: deformed,
            faces: self.base.faces.clone(),
            edge_creases: self.base.edge_creases.clone(),
            loose_edges: self.base.loose_edges.clone(),
            vert_creases: self.base.vert_creases.clone(),
            sharp_edges: self.base.sharp_edges.clone(),
            ..Default::default()
        };
        let worlds = HashMap::new();
        let ctx = EvalCtx { world, worlds: &worlds };
        for m in tail {
            m.apply(&mut mesh, &ctx);
        }
        mesh
    }
}
