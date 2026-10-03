//! What the viewport draws of a rig in Object mode (`overlay_armature.cc`):
//! the bones it shows (`animrig::bone_is_visible`) and, for each, where its
//! shape stands — its display matrix (`draw_bone_update_disp_matrix_default`
//! / `_custom_shape`) — and the shape's edges.
//!
//! A bone shows when it is not hidden (`PCHAN_DRAW_HIDDEN`) and one of its bone
//! collections is visible with all its parents — or, while a collection is
//! soloed, soloed — or it is in none (and nothing is soloed). A file from
//! before bone collections (3.x) has layers and bone groups instead, which
//! Blender turns into collections as it opens it (`versioning_400.cc`): one
//! per used layer, visible when the armature shows the layer, and one per
//! bone group, hidden.

use super::read::BFields;
use super::math::BEulerOrder;
use super::BRig;
use crate::blend::file::types::*;
use glam::{Mat4, Vec3};
use std::collections::HashMap;
use std::sync::Arc;

/// `eArmature_Drawtype`.
pub const ARM_DRAW_OCTA: i8 = 0;
pub const ARM_DRAW_STICK: i8 = 1;
pub const ARM_DRAW_BBONE: i8 = 2;
pub const ARM_DRAW_ENVELOPE: i8 = 3;
pub const ARM_DRAW_WIRE: i8 = 4;

const ARM_BCOLL_SOLO_ACTIVE: u32 = 1 << 6;
const ARM_NO_CUSTOM: u32 = 1 << 10;
const BONE_HIDDEN_P: u32 = 1 << 6;
const BONE_DRAWWIRE: u32 = 1 << 17;
const BONE_COLLECTION_VISIBLE: u32 = 1 << 0;
const BONE_COLLECTION_SOLO: u32 = 1 << 4;
const PCHAN_DRAW_NO_CUSTOM_BONE_SIZE: u32 = 1 << 0;
/// `PCHAN_DRAW_HIDDEN`: a pose bone hidden — from 5.0.54 on; before, the
/// bone's `BONE_HIDDEN_P`, which Blender copies across as it opens the file.
const PCHAN_DRAW_HIDDEN: u32 = 1 << 1;
const PCHAN_HIDE_VERSION: (u32, u32) = (500, 54);
/// `Object.dt` (`OB_WIRE`) and `Object.dtx` (`OB_DRAW_IN_FRONT`).
const OB_WIRE: u32 = 2;
const OB_DRAW_IN_FRONT: u32 = 1 << 6;
const OB_MESH: u32 = 1;

/// `bone_octahedral_verts` and the twelve edges between them.
const OCTAHEDRON: [[f32; 3]; 6] = [[0.0, 0.0, 0.0], [0.1, 0.1, 0.1], [0.1, 0.1, -0.1], [-0.1, 0.1, -0.1], [-0.1, 0.1, 0.1], [0.0, 1.0, 0.0]];
const OCTAHEDRON_EDGES: [(usize, usize); 12] = [(0, 1), (0, 2), (0, 3), (0, 4), (5, 1), (5, 2), (5, 3), (5, 4), (1, 2), (2, 3), (3, 4), (4, 1)];

/// How one bone draws.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BBoneLook {
    pub shown: bool,
    /// `Bone.drawtype`: its own display type, `None` for the armature's.
    pub drawtype: Option<i8>,
    /// `bPoseChannel.custom`: the custom shape object.
    pub shape: Option<u64>,
    /// `custom_tx`: the bone whose pose places the shape.
    pub shape_at: Option<usize>,
    pub shape_offset: Vec3,
    pub shape_turn: Vec3,
    pub shape_scale: Vec3,
    /// Scaled by the bone's length (`PCHAN_DRAW_NO_CUSTOM_BONE_SIZE` off).
    pub bone_size: bool,
    /// `BONE_DRAWWIRE`: drawn as wire whatever the object.
    pub wire: bool,
}

/// A custom shape: its object's name, and its mesh's edges in its own
/// space (a shape ignores its object's transform).
#[derive(Clone, Debug, PartialEq)]
pub struct BBoneShape {
    pub name: String,
    pub edges: Vec<[Vec3; 2]>,
}

/// How an armature object draws.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BRigLook {
    /// `bArmature.drawtype` (`ARM_DRAW_*`).
    pub drawtype: i8,
    /// Custom shapes shown (`ARM_NO_CUSTOM` off).
    pub custom: bool,
    /// `OB_DRAW_IN_FRONT`: drawn over the meshes.
    pub in_front: bool,
    /// The object shown as wire: no filled bones.
    pub wire: bool,
    /// By bone, in the rig's order.
    pub bones: Vec<BBoneLook>,
    pub shapes: HashMap<u64, Arc<BBoneShape>>,
}

/// What a shown bone draws, in its display space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BBoneDraw {
    /// A custom shape, by object address.
    Shape(u64),
    Octahedral,
    /// A line from head to tail (Stick and Wire).
    Stick,
}

/// A shown bone: its display matrix (armature space) and what it draws.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BBoneShown {
    pub bone: usize,
    pub disp: Mat4,
    pub draw: BBoneDraw,
}

impl BRigLook {
    /// Every shown bone at `pose` (each bone's `pose_mat`).
    pub fn shown(&self, rig: &BRig, pose: &[Mat4]) -> Vec<BBoneShown> {
        self.bones
            .iter()
            .enumerate()
            .filter(|(i, b)| b.shown && *i < pose.len() && *i < rig.bones.len())
            .map(|(i, b)| {
                let length = rig.bones[i].length;
                match b.shape.filter(|s| self.custom && self.shapes.contains_key(s)) {
                    Some(shape) => {
                        let at = pose[b.shape_at.filter(|&j| j < pose.len()).unwrap_or(i)];
                        let size = b.shape_scale * if b.bone_size { length } else { 1.0 };
                        let turn = Mat4::from_mat3(BEulerOrder(1).to_mat3(b.shape_turn));
                        let disp = at * Mat4::from_translation(b.shape_offset) * turn * Mat4::from_scale(size);
                        BBoneShown { bone: i, disp, draw: BBoneDraw::Shape(shape) }
                    }
                    None => {
                        // B-Bones and envelopes draw as octahedra here.
                        let draw = match b.drawtype.filter(|&t| t >= 0).unwrap_or(self.drawtype) {
                            ARM_DRAW_STICK | ARM_DRAW_WIRE => BBoneDraw::Stick,
                            _ => BBoneDraw::Octahedral,
                        };
                        BBoneShown { bone: i, disp: pose[i] * Mat4::from_scale(Vec3::splat(length)), draw }
                    }
                }
            })
            .collect()
    }

    /// Every shown bone's edges at `pose`, the armature object at `world`.
    pub fn lines(&self, rig: &BRig, pose: &[Mat4], world: Mat4) -> Vec<[Vec3; 2]> {
        let mut out = Vec::new();
        for shown in self.shown(rig, pose) {
            let m = world * shown.disp;
            match shown.draw {
                BBoneDraw::Shape(s) => out.extend(self.shapes[&s].edges.iter().map(|[a, b]| [m.transform_point3(*a), m.transform_point3(*b)])),
                BBoneDraw::Octahedral => {
                    out.extend(OCTAHEDRON_EDGES.iter().map(|&(a, b)| [m.transform_point3(OCTAHEDRON[a].into()), m.transform_point3(OCTAHEDRON[b].into())]))
                }
                BBoneDraw::Stick => out.push([m.transform_point3(Vec3::ZERO), m.transform_point3(Vec3::Y)]),
            }
        }
        out
    }
}

/// An armature's bone collections as the file keeps them, or as Blender
/// makes them from layers and bone groups.
struct BCollections {
    solo: bool,
    /// Each bone's collections (by `Bone` address): (visible with its
    /// parents, soloed).
    of_bone: HashMap<u64, Vec<(bool, bool)>>,
}

impl BCollections {
    fn shows(&self, bone: u64) -> bool {
        match self.of_bone.get(&bone).filter(|c| !c.is_empty()) {
            None => !self.solo,
            Some(list) => list.iter().any(|&(visible, solo)| if self.solo { solo } else { visible }),
        }
    }
}

impl BlendFile {
    /// How armature object `ob` draws `rig` (its bones read in the same
    /// order).
    pub(crate) fn rig_look(&self, ob: &BlendBlock, rig: &BRig, dna: &DnaType) -> Option<BRigLook> {
        self.scoped(ob, || self.rig_look_scoped(ob, rig, dna))
    }

    fn rig_look_scoped(&self, ob: &BlendBlock, rig: &BRig, dna: &DnaType) -> Option<BRigLook> {
        let o = BFields::new(self, dna, ob, "Object");
        let arm = self.block_at_code(o.ptr("*data"), "AR")?;
        let a = BFields::new(self, dna, arm, "bArmature");
        let pose = self.block_at(o.ptr("*pose"));
        let collections = self.scoped(arm, || self.bone_collections(&a, pose, rig, dna));
        let (version, sub) = PCHAN_HIDE_VERSION;
        let file = self.header.version_number;
        let pose_hide = file > version || (file == version && self.subversion(dna) >= sub);
        let bones: Vec<BBoneLook> = rig.bones.iter().map(|b| self.bone_look(b.address, rig, &collections, pose_hide, dna)).collect();
        let shapes = bones.iter().filter_map(|b| b.shape).filter_map(|s| self.bone_shape(s, dna).map(|shape| (s, Arc::new(shape)))).collect();
        Some(BRigLook {
            drawtype: a.int("drawtype") as i32 as i8,
            custom: a.int("flag") & ARM_NO_CUSTOM == 0,
            in_front: o.int("dtx") & OB_DRAW_IN_FRONT != 0,
            wire: o.int("dt") <= OB_WIRE,
            bones,
            shapes,
        })
    }

    fn bone_look(&self, chan: u64, rig: &BRig, collections: &BCollections, pose_hide: bool, dna: &DnaType) -> BBoneLook {
        let Some(pc) = self.block_at(chan) else { return BBoneLook::default() };
        let c = BFields::new(self, dna, pc, "bPoseChannel");
        let bone = self.block_at(c.ptr("*bone"));
        let b = bone.map(|b| BFields::new(self, dna, b, "Bone"));
        let flag = b.as_ref().map_or(0, |b| b.int("flag"));
        let drawtype = b.as_ref().filter(|b| b.has("drawtype")).map(|b| b.int("drawtype") as u8 as i8).filter(|&t| t >= 0);
        let hidden = if pose_hide { c.int("drawflag") & PCHAN_DRAW_HIDDEN != 0 } else { flag & BONE_HIDDEN_P != 0 };
        let scale = if c.has("custom_scale_xyz[3]") { c.vec3("custom_scale_xyz[3]", Vec3::ONE) } else { Vec3::splat(c.f32("custom_scale", 1.0)) };
        BBoneLook {
            shown: !hidden && collections.shows(bone.map_or(0, |b| b.old_address)),
            drawtype,
            shape: Some(c.ptr("*custom")).filter(|&s| s != 0),
            shape_at: rig.bones.iter().position(|b| b.address == c.ptr("*custom_tx") && b.address != 0),
            shape_offset: c.vec3("custom_translation[3]", Vec3::ZERO),
            shape_turn: c.vec3("custom_rotation_euler[3]", Vec3::ZERO),
            shape_scale: scale,
            bone_size: c.int("drawflag") & PCHAN_DRAW_NO_CUSTOM_BONE_SIZE == 0,
            wire: flag & BONE_DRAWWIRE != 0,
        }
    }

    /// The armature's bone collections (4.0+), or the ones Blender makes of
    /// a 3.x file's layers and bone groups.
    fn bone_collections(&self, a: &BFields, pose: Option<&BlendBlock>, rig: &BRig, dna: &DnaType) -> BCollections {
        let mut of_bone: HashMap<u64, Vec<(bool, bool)>> = HashMap::new();
        // The file keeps the array as the `collections` list, in its order
        // (`armature_blend_write`); Blender builds the array again on read.
        if a.has("**collection_array") {
            let list = self.list(a.ptr("collections"), self.off("BoneCollection", "*next", dna));
            let fields: Vec<BFields> = list.iter().map(|b| BFields::new(self, dna, b, "BoneCollection")).collect();
            // A collection is visible with its parents: roots first, each
            // collection's children in `child_index .. + child_count`.
            let mut visible: Vec<bool> = fields.iter().map(|f| f.int("flags") & BONE_COLLECTION_VISIBLE != 0).collect();
            for (i, f) in fields.iter().enumerate() {
                let (first, count) = (f.int("child_index") as usize, f.int("child_count") as usize);
                for child in first..(first + count).min(fields.len()) {
                    if child > i {
                        visible[child] = visible[child] && visible[i];
                    }
                }
            }
            for (i, f) in fields.iter().enumerate() {
                let solo = f.int("flags") & BONE_COLLECTION_SOLO != 0;
                let next = self.off("BoneCollectionMember", "*next", dna);
                for member in self.list(f.ptr("bones"), next) {
                    let bone = BFields::new(self, dna, member, "BoneCollectionMember").ptr("*bone");
                    of_bone.entry(bone).or_default().push((visible[i], solo));
                }
            }
            return BCollections { solo: a.int("flag") & ARM_BCOLL_SOLO_ACTIVE != 0, of_bone };
        }
        // Layers: a collection per used layer, visible when shown; bone
        // groups: a hidden collection each.
        let (used, shown) = (a.int("layer_used"), a.int("layer"));
        let groups = pose.map_or(0, |p| {
            let first = BFields::new(self, dna, p, "bPose").ptr("agroups");
            self.list(first, self.off("bActionGroup", "*next", dna)).len()
        });
        for b in &rig.bones {
            let Some(pc) = self.block_at(b.address) else { continue };
            let c = BFields::new(self, dna, pc, "bPoseChannel");
            let Some(bone) = self.block_at(c.ptr("*bone")) else { continue };
            let layer = BFields::new(self, dna, bone, "Bone").int("layer") & used;
            let mut list: Vec<(bool, bool)> = (0..32).filter(|l| layer & (1 << l) != 0).map(|l| (shown & (1 << l) != 0, false)).collect();
            let group = c.int("agrp_index") as u16 as i16;
            if group > 0 && group as usize <= groups {
                list.push((false, false));
            }
            of_bone.insert(bone.old_address, list);
        }
        BCollections { solo: false, of_bone }
    }

    /// A custom shape object's mesh as edges, in its own space. An empty or
    /// another kind of object as a shape draws nothing here.
    fn bone_shape(&self, address: u64, dna: &DnaType) -> Option<BBoneShape> {
        let ob = self.block_at_code(address, "OB")?;
        let o = BFields::new(self, dna, ob, "Object");
        if o.int("type") != OB_MESH {
            return None;
        }
        // Its data read in its own scope, as every ID's is (`file/scope.rs`).
        let me = self.scoped(ob, || self.block_at_code(o.ptr("*data"), "ME"))?;
        let ((verts, faces), mut pairs) = self.scoped(me, || (self.extract_mesh_from_block(me, dna), self.mesh_edges(me, dna)));
        if pairs.is_empty() {
            for f in &faces {
                pairs.extend((0..f.len()).map(|k| (f[k], f[(k + 1) % f.len()])));
            }
            pairs = pairs.into_iter().map(|(a, b)| (a.min(b), a.max(b))).collect();
            pairs.sort_unstable();
            pairs.dedup();
        }
        let at = |i: u32| verts.get(i as usize).map(|v| Vec3::from_array(*v));
        let edges = pairs.into_iter().filter_map(|(a, b)| Some([at(a)?, at(b)?])).collect();
        Some(BBoneShape { name: self.id_name(ob, dna), edges })
    }
}
