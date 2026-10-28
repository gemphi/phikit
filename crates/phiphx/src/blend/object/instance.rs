//! Collection instances (`OB_DUPLICOLLECTION`, `make_duplis_collection` in
//! `object_dupli.cc`): an object — an empty, as a rule — that shows a
//! collection where it stands. Every object of the collection, and of its
//! child collections, is drawn again at
//! `instancer.world · translate(−collection.instance_offset) · member.world`;
//! the collection itself need not be in the scene, and its objects are seen
//! only through the instances. Which instancers count and which members
//! show is `instance_members.rs`.
//!
//! A member that instances a collection itself instances it again inside
//! the instance (`make_recursive_duplis`): its copies stand at
//! `space · member.world · translate(−offset) · inner.world`, where `space`
//! is everything above it, down to `MAX_DUPLI_RECUR` levels and never
//! through an object already on the way down (`instance_stack`).
//!
//! Each instance joins the scene's objects as one of its own: the member's
//! geometry and materials, that matrix, no parent and no file block (it has
//! none — `block_address` 0), named by its instancers and itself
//! (`top/inner/member`), as the outliner never lists it. Its transform is
//! its own rest transform (`motion.rs` falls back to the object's
//! loc/rot/scale with no block). It keeps the member's block and world in
//! `instance_source`, so the modifier pass draws it with the member's
//! evaluated geometry — Mirror, Subdivision and the rest — evaluated once
//! per member (`modifier/mod.rs`). Not yet: an Armature-deformed member's
//! pose, Geometry Nodes instances, the background (`Scene.set`) scene.

use super::instance_members::{collection_generator, DupliMember};
use super::transform::IDENTITY4;
use crate::blend::file::types::*;
use glam::{Mat4, Vec3};
use std::collections::HashMap;

/// `MAX_DUPLI_RECUR` (`BKE_duplilist.hh`): `copy_dupli_context` refuses the
/// level one short of it, so generators run at levels 0 to 6.
const MAX_DUPLI_RECUR: u32 = 8;

/// What one `object_duplilist` call works from: the members of every
/// collection met so far, read once each.
struct Duplis<'a> {
    file: &'a BlendFile,
    dna: &'a DnaType,
    scene: HashMap<u64, &'a BlendSceneObject>,
    members: HashMap<u64, Vec<DupliMember>>,
    out: Vec<BlendSceneObject>,
}

impl BlendFile {
    /// The objects `objects` instance, appended to them: `object_duplilist`
    /// for every object the depsgraph's iterator would hand it.
    pub(crate) fn expand_collection_instances(&self, objects: &mut Vec<BlendSceneObject>, dna: &DnaType) {
        let bases = self.viewport_bases(dna);
        let field = self.object_visibility_field(dna);
        let mut duplis = Duplis {
            file: self,
            dna,
            scene: objects.iter().filter(|o| o.block_address != 0).map(|o| (o.block_address, o)).collect(),
            members: HashMap::new(),
            out: Vec::new(),
        };
        for instancer in objects.iter().filter(|o| o.block_address != 0) {
            if bases.as_ref().is_some_and(|b| !b.contains(&instancer.block_address)) {
                continue;
            }
            let Some(block) = self.block_at(instancer.block_address) else { continue };
            if !collection_generator(self.int_field(block, "Object", "transflag", dna), self.int_field(block, "Object", field, dna)) {
                continue;
            }
            // DNA keeps the old names: `dup_group` is RNA's
            // `instance_collection`, `dupli_ofs` its `instance_offset`.
            let collection = self.ptr(block, self.off("Object", "*dup_group", dna));
            let world = Mat4::from_cols_array_2d(&instancer.matrix_world);
            let mut stack = vec![instancer.block_address];
            duplis.collection(instancer.block_address, collection, world, Mat4::IDENTITY, 0, &instancer.name, &mut stack);
        }
        let out = duplis.out;
        objects.extend(out);
    }

    /// `Collection.instance_offset` (`dupli_ofs` in DNA).
    fn collection_offset(&self, block: &BlendBlock, dna: &DnaType) -> Vec3 {
        let Some(o) = self.off("Collection", "dupli_ofs[3]", dna) else { return Vec3::ZERO };
        let at = |i: usize| block.data.get(o + 4 * i..o + 4 * i + 4).map_or(0.0, |b| self.read_f32(b));
        Vec3::new(at(0), at(1), at(2))
    }
}

impl Duplis<'_> {
    /// `make_duplis_collection` for the instancer at `ob` standing at
    /// `world`, inside the dupli space `space` at recursion `level`, with
    /// `make_recursive_duplis` for each member that instances in turn.
    #[allow(clippy::too_many_arguments)]
    fn collection(&mut self, ob: u64, collection: u64, world: Mat4, space: Mat4, level: u32, name: &str, stack: &mut Vec<u64>) {
        let Some(block) = self.file.block_at(collection) else { return };
        let collection_mat = world * Mat4::from_translation(-self.file.collection_offset(block, self.dna));
        if !self.members.contains_key(&collection) {
            let read = self.read_members(collection);
            self.members.insert(collection, read);
        }
        // The members are read once; the recursion below needs them while
        // it pushes copies, so it walks this one's (address, world, what it
        // instances) rather than holding the map.
        let mut nested = Vec::new();
        for member in &self.members[&collection] {
            if member.object.block_address == ob {
                continue;
            }
            let at = Mat4::from_cols_array_2d(&member.object.matrix_world);
            self.out.push(instance_of(&member.object, name, space * collection_mat * at));
            let recurses = !stack.contains(&member.object.block_address) && level + 1 < MAX_DUPLI_RECUR - 1;
            if recurses && collection_generator(member.transflag, 0) && member.collection != 0 {
                nested.push((member.object.block_address, member.collection, at, format!("{name}/{}", member.object.name)));
            }
        }
        for (inner, inner_collection, at, inner_name) in nested {
            stack.push(inner);
            self.collection(inner, inner_collection, at, space * collection_mat, level + 1, &inner_name, stack);
            stack.pop();
        }
    }

    /// The members `collection_cache` lets through, each with its world
    /// matrix: a scene object's own, else worked out among the collection's
    /// objects (a member parented to another member keeps that parent).
    fn read_members(&self, collection: u64) -> Vec<DupliMember> {
        let (file, dna) = (self.file, self.dna);
        let cache = file.collection_cache(collection, dna);
        let mut objects: Vec<BlendSceneObject> = cache
            .iter()
            .filter_map(|(ob, _)| match self.scene.get(ob) {
                Some(o) => Some((*o).clone()),
                None => file.block_at(*ob).and_then(|b| file.extract_object_simple(b).ok().flatten()),
            })
            .collect();
        BlendFile::resolve_world_matrices(&mut objects);
        objects
            .into_iter()
            .map(|mut object| {
                if let Some(o) = self.scene.get(&object.block_address) {
                    object.matrix_world = o.matrix_world;
                }
                let transflag = cache.iter().find(|(at, _)| *at == object.block_address).map_or(0, |c| c.1);
                let collection = file.block_at(object.block_address).map_or(0, |b| file.ptr(b, file.off("Object", "*dup_group", dna)));
                DupliMember { object, transflag, collection }
            })
            .collect()
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
        instance_source: Some((member.block_address, member.matrix_world)),
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
