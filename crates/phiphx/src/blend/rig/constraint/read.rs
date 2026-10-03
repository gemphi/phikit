//! Reading a pose channel's constraints: `bConstraint` and the data struct
//! its type names, through the file's own DNA.

use super::*;
use crate::blend::file::types::*;
use crate::blend::rig::read::BFields;
use std::collections::HashMap;

/// What a constraint's targets resolve against: its owner's object (a rig
/// or not) and the rig's bones by name.
pub struct BTargetNames<'a> {
    pub object: u64,
    pub bones: &'a HashMap<String, usize>,
}

impl BTargetNames<'_> {
    /// A `tar` pointer and `subtarget` name as a target: the owner's own
    /// rig, another object, or a bone (or vertex group) of one.
    pub fn resolve(&self, tar: u64, sub: &str) -> BTarget {
        match tar {
            0 => BTarget::None,
            t if t == self.object && sub.is_empty() => BTarget::Rig(None),
            t if t == self.object => self.bones.get(sub).map_or(BTarget::None, |&b| BTarget::Rig(Some(b))),
            t if sub.is_empty() => BTarget::Object(t),
            t => BTarget::Bone(t, sub.to_string()),
        }
    }
}

impl BlendFile {
    /// Every constraint on the pose channel `pc`, in order.
    pub(crate) fn rig_constraints(&self, pc: &BlendBlock, dna: &DnaType, names: &BTargetNames) -> Vec<BConstraint> {
        let first = self.ptr(pc, self.off("bPoseChannel", "constraints", dna));
        self.list(first, self.off("bConstraint", "*next", dna)).into_iter().map(|con| self.rig_constraint(con, dna, names)).collect()
    }

    /// Every constraint on object `ob` (`Object.constraints`), in order.
    pub(crate) fn object_constraints(&self, ob: &BlendBlock, dna: &DnaType) -> Vec<BConstraint> {
        let first = self.ptr(ob, self.off("Object", "constraints", dna));
        let bones = HashMap::new();
        let names = BTargetNames { object: ob.old_address, bones: &bones };
        self.list(first, self.off("bConstraint", "*next", dna)).into_iter().map(|con| self.rig_constraint(con, dna, &names)).collect()
    }

    fn rig_constraint(&self, con: &BlendBlock, dna: &DnaType, names: &BTargetNames) -> BConstraint {
        let c = BFields::new(self, dna, con, "bConstraint");
        let ty = c.int("type") as u16;
        let data = self.block_at(c.ptr("*data"));
        let kind = data.map_or(BConstraintKind::Other(ty), |d| self.constraint_kind(ty, d, dna, names));
        BConstraint {
            name: c.name("name[64]"),
            kind,
            flag: c.int("flag"),
            ownspace: c.int("ownspace") as u8,
            tarspace: c.int("tarspace") as u8,
            influence: c.f32("enforce", 1.0),
            headtail: c.f32("headtail", 0.0),
            space: names.resolve(c.ptr("*space_object"), &c.name("space_subtarget[64]")),
            expanded: if c.has("ui_expand_flag") { c.int("ui_expand_flag") & 1 != 0 } else { c.int("flag") & 1 != 0 },
        }
    }

    fn constraint_kind(&self, ty: u16, d: &BlendBlock, dna: &DnaType, names: &BTargetNames) -> BConstraintKind {
        let st = match ty {
            TYPE_TRANSLIKE => "bTransLikeConstraint",
            TYPE_LOCLIKE => "bLocateLikeConstraint",
            TYPE_ROTLIKE => "bRotateLikeConstraint",
            TYPE_SIZELIKE => "bSizeLikeConstraint",
            TYPE_DAMPTRACK => "bDampTrackConstraint",
            TYPE_STRETCHTO => "bStretchToConstraint",
            TYPE_ROTLIMIT => "bRotLimitConstraint",
            TYPE_DISTLIMIT => "bDistLimitConstraint",
            TYPE_ARMATURE => "bArmatureConstraint",
            TYPE_KINEMATIC => "bKinematicConstraint",
            TYPE_CHILDOF => "bChildOfConstraint",
            TYPE_TRACKTO => "bTrackToConstraint",
            TYPE_LOCKTRACK => "bLockTrackConstraint",
            TYPE_LOCLIMIT => "bLocLimitConstraint",
            TYPE_SIZELIMIT => "bSizeLimitConstraint",
            TYPE_SAMEVOL => "bSameVolumeConstraint",
            TYPE_TRANSFORM => "bTransformConstraint",
            TYPE_MINMAX => "bMinMaxConstraint",
            TYPE_ACTION => "bActionConstraint",
            TYPE_PIVOT => "bPivotConstraint",
            TYPE_FOLLOWPATH => "bFollowPathConstraint",
            TYPE_CLAMPTO => "bClampToConstraint",
            _ => return BConstraintKind::Other(ty),
        };
        let f = BFields::new(self, dna, d, st);
        let target = || names.resolve(f.ptr("*tar"), &f.name("subtarget[64]"));
        match ty {
            TYPE_TRANSLIKE => BConstraintKind::CopyTransforms { target: target(), mix: f.int("mix_mode") as u8, flag: f.int("flag") },
            TYPE_LOCLIKE => BConstraintKind::CopyLocation { target: target(), flag: f.int("flag") },
            TYPE_ROTLIKE => BConstraintKind::CopyRotation { target: target(), flag: f.int("flag"), order: f.int("euler_order") as u8, mix: f.int("mix_mode") as u8 },
            TYPE_SIZELIKE => BConstraintKind::CopyScale { target: target(), flag: f.int("flag"), power: f.f32("power", 1.0) },
            TYPE_DAMPTRACK => BConstraintKind::DampedTrack { target: target(), track: f.int("trackflag") },
            TYPE_STRETCHTO => BConstraintKind::StretchTo {
                target: target(),
                volume: f.int("volmode"),
                plane: f.int("plane"),
                rest: f.f32("orglength", 0.0),
                bulge: f.f32("bulge", 1.0),
                bulge_min: f.f32("bulge_min", 1.0),
                bulge_max: f.f32("bulge_max", 1.0),
                bulge_smooth: f.f32("bulge_smooth", 0.0),
                flag: f.int("flag"),
            },
            TYPE_ROTLIMIT => BConstraintKind::LimitRotation {
                min: Vec3::new(f.f32("xmin", 0.0), f.f32("ymin", 0.0), f.f32("zmin", 0.0)),
                max: Vec3::new(f.f32("xmax", 0.0), f.f32("ymax", 0.0), f.f32("zmax", 0.0)),
                flag: f.int("flag"),
                order: f.int("euler_order") as u8,
            },
            TYPE_DISTLIMIT => BConstraintKind::LimitDistance { target: target(), dist: f.f32("dist", 0.0), soft: f.f32("soft", 0.0), flag: f.int("flag"), mode: f.int("mode") },
            TYPE_ARMATURE => {
                let first = f.ptr("targets");
                let targets = self
                    .list(first, self.off("bConstraintTarget", "*next", dna))
                    .into_iter()
                    .map(|t| {
                        let t = BFields::new(self, dna, t, "bConstraintTarget");
                        (names.resolve(t.ptr("*tar"), &t.name("subtarget[64]")), t.f32("weight", 0.0))
                    })
                    .collect();
                BConstraintKind::Armature { flag: f.int("flag"), targets }
            }
            TYPE_CHILDOF => BConstraintKind::ChildOf { target: target(), flag: f.int("flag"), invmat: f.mat4("invmat[4][4]") },
            TYPE_TRACKTO => BConstraintKind::TrackTo { target: target(), track: f.int("reserved1"), up: f.int("reserved2"), flags: f.int("flags") },
            TYPE_LOCKTRACK => BConstraintKind::LockedTrack { target: target(), track: f.int("trackflag"), lock: f.int("lockflag") },
            TYPE_LOCLIMIT | TYPE_SIZELIMIT => {
                let (min, max) = (f.vec3_of(["xmin", "ymin", "zmin"]), f.vec3_of(["xmax", "ymax", "zmax"]));
                match ty {
                    TYPE_LOCLIMIT => BConstraintKind::LimitLocation { min, max, flag: f.int("flag") },
                    _ => BConstraintKind::LimitScale { min, max, flag: f.int("flag") },
                }
            }
            // The file's DNA keeps the axis under its old name, `flag`
            // (`DNA_STRUCT_RENAME_ELEM(bSameVolumeConstraint, flag, free_axis)`).
            TYPE_SAMEVOL => BConstraintKind::MaintainVolume {
                axis: if f.has("free_axis") { f.int("free_axis") } else { f.int("flag") },
                mode: f.int("mode"),
                volume: f.f32("volume", 1.0),
            },
            TYPE_TRANSFORM => BConstraintKind::Transformation { target: target(), data: Box::new(self.transform_data(&f)) },
            TYPE_MINMAX => BConstraintKind::Floor { target: target(), axis: f.int("minmaxflag"), offset: f.f32("offset", 0.0), flag: f.int("flag") },
            TYPE_ACTION => BConstraintKind::Action { target: target(), data: Box::new(self.action_data(&f, dna)) },
            TYPE_FOLLOWPATH => BConstraintKind::FollowPath {
                target: target(),
                flag: f.int("followflag"),
                offset: f.f32("offset", 0.0),
                offset_fac: f.f32("offset_fac", 0.0),
                track: f.int("trackflag") as i16,
                up: f.int("upflag") as i16,
            },
            TYPE_CLAMPTO => BConstraintKind::ClampTo { target: target(), axis: f.int("flag"), cyclic: f.int("flag2") & super::path::CLAMPTO_CYCLIC != 0 },
            TYPE_PIVOT => BConstraintKind::Pivot { target: target(), offset: f.vec3("offset[3]", Vec3::ZERO), axis: f.int("rotAxis") as i8 as i32, flag: f.int("flag") },
            _ => BConstraintKind::Ik(BIkData {
                target: target(),
                pole: names.resolve(f.ptr("*poletar"), &f.name("polesubtarget[64]")),
                iterations: f.int("iterations") as i32,
                flag: f.int("flag"),
                rootbone: f.int("rootbone") as i32,
                poleangle: f.f32("poleangle", 0.0),
                weight: f.f32("weight", 1.0),
                orientweight: f.f32("orientweight", 1.0),
                kind: f.int("type") as i32,
            }),
        }
    }

    /// `bTransformConstraint`'s mapping and its three pairs of ranges.
    fn transform_data(&self, f: &BFields) -> BTransformData {
        let map = f.bytes("map[3]", 3);
        let range = |min: &str, max: &str| [f.vec3(min, Vec3::ZERO), f.vec3(max, Vec3::ZERO)];
        BTransformData {
            from: f.int("from") as u8,
            to: f.int("to") as u8,
            map: [map[0], map[1], map[2]],
            expo: f.int("expo") != 0,
            from_rotation_mode: f.int("from_rotation_mode") as u8,
            to_euler_order: f.int("to_euler_order") as u8,
            mix_loc: f.int("mix_mode_loc") as u8,
            mix_rot: f.int("mix_mode_rot") as u8,
            mix_scale: f.int("mix_mode_scale") as u8,
            from_range: [
                range("from_min[3]", "from_max[3]"),
                range("from_min_rot[3]", "from_max_rot[3]"),
                range("from_min_scale[3]", "from_max_scale[3]"),
            ],
            to_range: [range("to_min[3]", "to_max[3]"), range("to_min_rot[3]", "to_max_rot[3]"), range("to_min_scale[3]", "to_max_scale[3]")],
        }
    }

    /// `bActionConstraint`, with its action's pose-bone curves for the slot
    /// it names (the only slot when it names none).
    fn action_data(&self, f: &BFields, dna: &DnaType) -> BActionData {
        let act = f.ptr("*act");
        let curves = match act {
            0 => Vec::new(),
            a => self.actions(dna).get(&a).map_or_else(Vec::new, |action| {
                action.curves(f.int("action_slot_handle") as i32).iter().filter(|c| c.path.starts_with("pose.bones[")).cloned().collect()
            }),
        };
        BActionData {
            channel: f.int("type") as i16,
            start: f.int("start") as i32 as f32,
            end: f.int("end") as i32 as f32,
            min: f.f32("min", 0.0),
            max: f.f32("max", 0.0),
            flag: f.int("flag"),
            mix: f.int("mix_mode"),
            eval_time: f.f32("eval_time", 0.0),
            has_action: act != 0,
            curves,
        }
    }
}
