//! Blender's physics settings, read from the file: the Rigid Body World
//! (`Scene.rigidbody_world`), each object's rigid body and rigid body
//! constraint (`Object.rigidbody_object`, `Object.rigidbody_constraint`),
//! the Cloth modifier's settings (`ClothModifierData.sim_parms`,
//! `coll_parms`), the Collision modifier's and a force field's
//! (`Object.pd`, one `PartDeflect` serves both) and the Soft Body's
//! (`Object.soft`). DNA: `DNA_rigidbody_types.h`, `DNA_cloth_types.h`,
//! `DNA_object_force_types.h`.
//!
//! Settings only. Nothing here simulates; `phiray/docs/ROADMAP.md` §2 says
//! where the solver goes and how these numbers feed it. What the viewport
//! *shows* for a Cloth at the file's frame is a different thing and lives
//! in `modifier/cloth.rs`, which reads the point cache rather than
//! stepping a solver. Every field is read by its DNA name, so a renamed
//! member reads as the default rather than as a wrong neighbour — with the
//! flag **bits** taken from the DNA enums themselves, since
//! `eRigidBodyOb_Flag`, `eRigidBodyCon_Flag` and `ePField_Flag` leave gaps
//! (`NEEDS_VALIDATE`, `NEEDS_RESHAPE`) that an off-by-one shift reads as
//! the neighbouring setting.

mod body;
mod cloth;
mod deflect;
mod joint;
mod soft;
mod world;

pub use body::{BRigidBody, BRigidKind, BRigidShape, BMeshSource};
pub use cloth::{BCloth, BClothCollision};
pub use deflect::{BCollider, BFieldKind, BFieldShape, BForceField};
pub use joint::{BJointKind, BRigidJoint, BSpringKind};
pub use soft::{BSoftBody, BSoftCollision};
pub use world::{BEffectorWeights, BPointCache, BRigidWorld};

use crate::blend::file::types::*;
use crate::blend::BlendSceneObject;

/// `eModifierType` of the modifiers whose presence turns a setting on.
const MODIFIER_SOFT_BODY: u32 = 10;
const MODIFIER_CLOTH: u32 = 22;
const MODIFIER_COLLISION: u32 = 23;

/// Everything the file's Physics tabs hold.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlendPhysics {
    /// The scene's Rigid Body World; `None` when the scene has none.
    pub world: Option<BRigidWorld>,
    /// `Scene.physics_settings.gravity`, the Scene tab's Gravity.
    pub gravity: [f32; 3],
    pub bodies: Vec<BRigidBody>,
    pub joints: Vec<BRigidJoint>,
    pub cloths: Vec<BCloth>,
    pub colliders: Vec<BCollider>,
    pub fields: Vec<BForceField>,
    pub softs: Vec<BSoftBody>,
}

impl BlendPhysics {
    /// Whether the file carries any physics at all.
    pub fn is_empty(&self) -> bool {
        self.world.is_none()
            && self.bodies.is_empty()
            && self.joints.is_empty()
            && self.cloths.is_empty()
            && self.colliders.is_empty()
            && self.fields.is_empty()
            && self.softs.is_empty()
    }
}

/// A NUL-terminated string of at most `len` bytes at `off`.
pub(crate) fn string_at(d: &[u8], off: Option<usize>, len: usize) -> String {
    let Some(o) = off else { return String::new() };
    let s = d.get(o..(o + len).min(d.len())).unwrap_or(&[]);
    let n = s.iter().position(|&c| c == 0).unwrap_or(s.len());
    String::from_utf8_lossy(&s[..n]).to_string()
}

impl BlendFile {
    /// The physics of the scene `sc`, for the objects the read found.
    pub(crate) fn extract_physics(&self, sc: &BlendBlock, objects: &[BlendSceneObject], dna: &DnaType) -> BlendPhysics {
        let mut out = BlendPhysics {
            world: self.scoped(sc, || self.rigid_world(sc, dna)),
            gravity: self.scene_gravity(sc, dna),
            ..Default::default()
        };
        for obj in objects.iter().filter(|o| !o.is_instance) {
            let Some(ob) = self.block_at_code(obj.block_address, "OB") else { continue };
            self.scoped(ob, || self.object_physics(ob, &obj.name, dna, &mut out));
        }
        out
    }

    /// `Scene.physics_settings.gravity`: an embedded `PhysicsSettings`.
    fn scene_gravity(&self, sc: &BlendBlock, dna: &DnaType) -> [f32; 3] {
        let ps = self.header.pointer_size as usize;
        let at = Self::struct_field_offset_exact("Scene", "physics_settings", dna, ps)
            .zip(Self::struct_field_offset_exact("PhysicsSettings", "gravity[3]", dna, ps))
            .map(|(a, b)| a + b);
        match at {
            Some(o) if o + 12 <= sc.data.len() => std::array::from_fn(|k| self.read_f32(&sc.data[o + 4 * k..o + 4 * k + 4])),
            _ => [0.0, 0.0, -9.81],
        }
    }

    /// One object's physics: its rigid body, its constraint, and what its
    /// modifier stack switches on.
    fn object_physics(&self, ob: &BlendBlock, name: &str, dna: &DnaType, out: &mut BlendPhysics) {
        if let Some(rb) = self.block_at(self.ptr(ob, self.off("Object", "*rigidbody_object", dna))) {
            out.bodies.push(self.rigid_body(rb, name, dna));
        }
        if let Some(con) = self.block_at(self.ptr(ob, self.off("Object", "*rigidbody_constraint", dna))) {
            out.joints.push(self.rigid_joint(con, name, dna));
        }
        let pd = self.block_at(self.ptr(ob, self.off("Object", "*pd", dna)));
        if let Some(pd) = pd.filter(|pd| self.int_field(pd, "PartDeflect", "forcefield", dna) != 0) {
            out.fields.push(self.force_field(pd, name, dna));
        }
        let first = self.ptr(ob, self.off("Object", "modifiers", dna));
        let next = self.off("ModifierData", "*next", dna);
        for md in self.list(first, next) {
            let kind = self.int_field(md, "ModifierData", "type", dna);
            match kind {
                MODIFIER_CLOTH => out.cloths.push(self.cloth(md, ob, name, dna)),
                MODIFIER_COLLISION => {
                    if let Some(pd) = pd {
                        out.colliders.push(self.collider(pd, name, dna));
                    }
                }
                MODIFIER_SOFT_BODY => {
                    if let Some(sb) = self.block_at(self.ptr(ob, self.off("Object", "*soft", dna))) {
                        out.softs.push(self.soft_body(sb, ob, name, dna));
                    }
                }
                _ => {}
            }
        }
    }

    /// `ID.name` of the object at `addr`, less its `OB` prefix.
    pub(crate) fn object_name_at(&self, addr: u64) -> String {
        self.block_at_code(addr, "OB")
            .and_then(|ob| self.extract_name_from_block(ob, "name").ok())
            .unwrap_or_default()
    }

    /// The names of `ob`'s vertex groups, by their 1-based DNA index.
    pub(crate) fn vertex_group_name(&self, ob: &BlendBlock, index: u32, dna: &DnaType) -> String {
        if index == 0 {
            return String::new();
        }
        let me = self.block_at_code(self.ptr(ob, self.off("Object", "*data", dna)), "ME");
        let names = match me {
            Some(me) => self.vertex_group_names(ob, me, dna),
            None => Vec::new(),
        };
        names.get(index as usize - 1).cloned().unwrap_or_default()
    }
}
