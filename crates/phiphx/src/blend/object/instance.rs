//! Collection instances (`OB_DUPLICOLLECTION`, `make_duplis_collection` in
//! `object_dupli.cc`): an object — an empty, as a rule — that shows a
//! collection where it stands. Every object of the collection, and of its
//! child collections, is drawn again at
//! `instancer.world · translate(−collection.instance_offset) · member.world`;
//! the collection itself need not be in the scene, and its objects are seen
//! only through the instances.
//!
//! Each instance joins the scene's objects as one of its own: the member's
//! geometry and materials, that matrix, no parent and no file block (it has
//! none — `block_address` 0), named `instancer/member`, as the outliner
//! never lists it. Its transform is its own rest transform (`motion.rs`
//! falls back to the object's loc/rot/scale with no block). Not yet: an
//! instance of an instance, and modifiers on the members (they are drawn
//! as their meshes stand).

use super::transform::IDENTITY4;
use crate::blend::file::types::*;
use glam::{Mat4, Vec3};
use std::collections::HashMap;

/// `Object.transflag`: instance a collection.
pub const OB_DUPLICOLLECTION: u32 = 1 << 8;

impl BlendFile {
    /// The objects `objects` instance, appended to them.
    pub(crate) fn expand_collection_instances(&self, objects: &mut Vec<BlendSceneObject>, dna: &DnaType) {
        let mut members: HashMap<u64, Vec<BlendSceneObject>> = HashMap::new();
        let mut out = Vec::new();
        for instancer in objects.iter().filter(|o| o.block_address != 0) {
            let Some(block) = self.block_at(instancer.block_address) else { continue };
            if self.int_field(block, "Object", "transflag", dna) & OB_DUPLICOLLECTION == 0 {
                continue;
            }
            // DNA keeps the old names: `dup_group` is RNA's
            // `instance_collection`, `dupli_ofs` its `instance_offset`.
            let collection = self.ptr(block, self.off("Object", "*dup_group", dna));
            let Some(collection_block) = self.block_at(collection) else { continue };
            let offset = self.collection_offset(collection_block, dna);
            let shown = members.entry(collection).or_insert_with(|| self.collection_members(collection, dna));
            let base = Mat4::from_cols_array_2d(&instancer.matrix_world) * Mat4::from_translation(-offset);
            for member in shown.iter() {
                out.push(instance_of(member, &instancer.name, base * Mat4::from_cols_array_2d(&member.matrix_world)));
            }
        }
        objects.extend(out);
    }

    /// `Collection.instance_offset` (`dupli_ofs` in DNA).
    fn collection_offset(&self, block: &BlendBlock, dna: &DnaType) -> Vec3 {
        let Some(o) = self.off("Collection", "dupli_ofs[3]", dna) else { return Vec3::ZERO };
        let at = |i: usize| block.data.get(o + 4 * i..o + 4 * i + 4).map_or(0.0, |b| self.read_f32(b));
        Vec3::new(at(0), at(1), at(2))
    }

    /// Every object of the collection at `addr` and its children, once
    /// each, with its world matrix among them (a member parented to another
    /// member keeps that parent).
    fn collection_members(&self, addr: u64, dna: &DnaType) -> Vec<BlendSceneObject> {
        let mut out: Vec<BlendSceneObject> = Vec::new();
        let mut stack = vec![(addr, 0u32)];
        while let Some((at, depth)) = stack.pop() {
            let Some(block) = self.block_at(at).filter(|_| depth <= 64) else { continue };
            for link in self.list(self.ptr(block, self.off("Collection", "gobject", dna)), self.off("CollectionObject", "*next", dna)) {
                let ob = self.ptr(link, self.off("CollectionObject", "*ob", dna));
                let Some(ob_block) = self.block_at(ob) else { continue };
                if out.iter().any(|o| o.block_address == ob) {
                    continue;
                }
                if let Ok(Some(object)) = self.extract_object_simple(ob_block) {
                    out.push(object);
                }
            }
            for link in self.list(self.ptr(block, self.off("Collection", "children", dna)), self.off("CollectionChild", "*next", dna)) {
                stack.push((self.ptr(link, self.off("CollectionChild", "*collection", dna)), depth + 1));
            }
        }
        Self::resolve_world_matrices(&mut out);
        out
    }
}

/// `member` drawn by `instancer` at `world`.
fn instance_of(member: &BlendSceneObject, instancer: &str, world: Mat4) -> BlendSceneObject {
    let (scale, rotation, location) = world.to_scale_rotation_translation();
    let matrix = world.to_cols_array_2d();
    BlendSceneObject {
        name: format!("{instancer}/{}", member.name),
        location: location.into(),
        rotation: rotation.to_euler(glam::EulerRot::XYZ).into(),
        quaternion: [rotation.w, rotation.x, rotation.y, rotation.z],
        scale: scale.into(),
        matrix_world: matrix,
        matrix_local: matrix,
        block_address: 0,
        parent_address: 0,
        parentinv: IDENTITY4,
        is_instance: true,
        ..member.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_instance_stands_where_its_instancer_puts_the_member() {
        let mut member = BlendSceneObject::default();
        member.name = "Fan".into();
        member.vertices = vec![[0.0, 0.0, 1.0]];
        let world = Mat4::from_translation(Vec3::new(5.0, 2.0, 0.0)) * Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let copy = instance_of(&member, "Seat.004", world);
        assert_eq!(copy.name, "Seat.004/Fan");
        assert_eq!((copy.block_address, copy.parent_address), (0, 0));
        assert!(copy.is_instance);
        assert_eq!(copy.vertices, member.vertices, "the member's own geometry");
        assert!((Vec3::from(copy.location) - Vec3::new(5.0, 2.0, 0.0)).length() < 1e-6);
        assert!((copy.quaternion[0] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6, "turned a quarter about Z");
    }
}
