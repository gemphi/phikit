//! Constraints on bones and objects: what each is (read from `bConstraint`
//! and its data, [`read`]), and `BKE_constraints_solve` over an owner's
//! stack ([`stack`]) — a bone of a rig being posed ([`bone`]) or an object
//! ([`object`]).

mod action;
mod aim;
mod armature;
mod bone;
mod copy;
mod limit;
pub(crate) mod object;
mod parent;
mod path;
pub(crate) mod read;
mod stack;
#[cfg(test)]
mod tests;
mod track;
mod transform;

pub use action::BActionData;
pub use object::BObjectWork;
pub use stack::{BConstraintHost, BConstraintStack};
pub use transform::BTransformData;

use glam::{Mat4, Vec3};

/// `CONSTRAINT_TYPE_*` this solver evaluates.
pub const TYPE_CHILDOF: u16 = 1;
pub const TYPE_FOLLOWPATH: u16 = 4;
pub const TYPE_TRACKTO: u16 = 2;
pub const TYPE_LOCLIMIT: u16 = 6;
pub const TYPE_SIZELIMIT: u16 = 7;
pub const TYPE_ACTION: u16 = 12;
pub const TYPE_LOCKTRACK: u16 = 13;
pub const TYPE_MINMAX: u16 = 16;
pub const TYPE_TRANSFORM: u16 = 19;
pub const TYPE_SAMEVOL: u16 = 24;
pub const TYPE_PIVOT: u16 = 25;
pub const TYPE_ROTLIMIT: u16 = 5;
pub const TYPE_ROTLIKE: u16 = 8;
pub const TYPE_LOCLIKE: u16 = 9;
pub const TYPE_SIZELIKE: u16 = 10;
pub const TYPE_KINEMATIC: u16 = 3;
pub const TYPE_DISTLIMIT: u16 = 14;
pub const TYPE_STRETCHTO: u16 = 15;
pub const TYPE_DAMPTRACK: u16 = 21;
pub const TYPE_TRANSLIKE: u16 = 23;
pub const TYPE_ARMATURE: u16 = 30;
pub const TYPE_CLAMPTO: u16 = 18;

/// `bConstraint.flag`.
pub const CONSTRAINT_DISABLE: u32 = 1 << 2;
pub const CONSTRAINT_SPACEONCE: u32 = 1 << 6;
pub const CONSTRAINT_OFF: u32 = 1 << 9;

/// What a constraint points at.
#[derive(Clone, Debug, PartialEq)]
pub enum BTarget {
    /// No target (or one this rig cannot see).
    None,
    /// This rig: one of its bones, or the rig object itself.
    Rig(Option<usize>),
    /// Another object, by block address.
    Object(u64),
    /// A bone of another rig, by its object's block address and its name —
    /// the object itself when it has no such bone.
    Bone(u64, String),
}

/// The Inverse Kinematics constraint's settings (`bKinematicConstraint`).
#[derive(Clone, Debug, PartialEq)]
pub struct BIkData {
    pub target: BTarget,
    pub pole: BTarget,
    pub iterations: i32,
    /// `CONSTRAINT_IK_*`.
    pub flag: u32,
    /// Chain length (0: to the root).
    pub rootbone: i32,
    pub poleangle: f32,
    pub weight: f32,
    pub orientweight: f32,
    /// `CONSTRAINT_IK_COPYPOSE` (0) or `_DISTANCE`.
    pub kind: i32,
}

/// One constraint's own settings.
#[derive(Clone, Debug, PartialEq)]
pub enum BConstraintKind {
    CopyTransforms { target: BTarget, mix: u8, flag: u32 },
    CopyLocation { target: BTarget, flag: u32 },
    CopyRotation { target: BTarget, flag: u32, order: u8, mix: u8 },
    CopyScale { target: BTarget, flag: u32, power: f32 },
    DampedTrack { target: BTarget, track: u32 },
    StretchTo { target: BTarget, volume: u32, plane: u32, rest: f32, bulge: f32, bulge_min: f32, bulge_max: f32, bulge_smooth: f32, flag: u32 },
    LimitRotation { min: Vec3, max: Vec3, flag: u32, order: u8 },
    LimitDistance { target: BTarget, dist: f32, soft: f32, flag: u32, mode: u32 },
    /// Weighted bones (`bConstraintTarget`s with a weight).
    Armature { flag: u32, targets: Vec<(BTarget, f32)> },
    Ik(BIkData),
    /// `bChildOfConstraint`: `flag` the `CHILDOF_*` channels, `invmat` the
    /// Set Inverse matrix.
    ChildOf { target: BTarget, flag: u32, invmat: Mat4 },
    /// `bTrackToConstraint`: `track` `TRACK_*`, `up` `UP_*`, `flags`
    /// `TARGET_Z_UP`.
    TrackTo { target: BTarget, track: u32, up: u32, flags: u32 },
    /// `bLockTrackConstraint`: `track` `TRACK_*`, `lock` `LOCK_*`.
    LockedTrack { target: BTarget, track: u32, lock: u32 },
    /// `bLocLimitConstraint` / `bSizeLimitConstraint`: `LIMIT_[XYZ]MIN/MAX`.
    LimitLocation { min: Vec3, max: Vec3, flag: u32 },
    LimitScale { min: Vec3, max: Vec3, flag: u32 },
    /// `bSameVolumeConstraint`: `SAMEVOL_*` axis and mode.
    MaintainVolume { axis: u32, mode: u32, volume: f32 },
    Transformation { target: BTarget, data: Box<BTransformData> },
    /// `bMinMaxConstraint` (Floor): `axis` `TRACK_*`, `flag` `MINMAX_USEROT`.
    Floor { target: BTarget, axis: u32, offset: f32, flag: u32 },
    Action { target: BTarget, data: Box<BActionData> },
    /// `bPivotConstraint`: `axis` `PIVOTCON_AXIS_*`, `flag` `PIVOTCON_FLAG_*`.
    Pivot { target: BTarget, offset: Vec3, axis: i32, flag: u32 },
    /// `bFollowPathConstraint`: `flag` `FOLLOWPATH_*`, `offset` in frames,
    /// `offset_fac` the fixed position, `track` / `up` the axes that follow.
    FollowPath { target: BTarget, flag: u32, offset: f32, offset_fac: f32, track: i16, up: i16 },
    /// `bClampToConstraint`: `axis` its `flag` (`CLAMPTO_AUTO`, X, Y, Z),
    /// `cyclic` its `flag2` `CLAMPTO_CYCLIC`.
    ClampTo { target: BTarget, axis: u32, cyclic: bool },
    /// A type this solver does not evaluate (`CONSTRAINT_TYPE_*`).
    Other(u16),
}

/// One `bConstraint`.
#[derive(Clone, Debug, PartialEq)]
pub struct BConstraint {
    pub name: String,
    pub kind: BConstraintKind,
    pub flag: u32,
    pub ownspace: u8,
    pub tarspace: u8,
    /// `enforce`, as saved (drivers override it per frame).
    pub influence: f32,
    pub headtail: f32,
    /// The custom space's object and bone.
    pub space: BTarget,
    /// Its panel is open (`ui_expand_flag`, before 2.90 `flag`'s
    /// `CONSTRAINT_EXPAND`): only then does the viewport draw its line to its
    /// target (`overlay_relation.hh`).
    pub expanded: bool,
}

/// `bConstraintOb`: the owner as constraints see it, in world space.
pub struct BCob {
    /// The owning bone of the rig being posed; `None` for an object.
    pub bone: Option<usize>,
    pub matrix: Mat4,
    /// `rotOrder`: the owner's Euler order (XYZ for quaternions).
    pub rot_order: i16,
    pub custom: Mat4,
}

impl BConstraint {
    /// The other objects this constraint reads (its custom space too), and
    /// whether it reads a bone of one — what must be solved before it.
    pub fn reads(&self) -> impl Iterator<Item = (u64, bool)> + '_ {
        std::iter::once(&self.space).chain(self.kind.targets()).filter_map(|t| match t {
            BTarget::Object(a) => Some((*a, false)),
            BTarget::Bone(a, _) => Some((*a, true)),
            BTarget::None | BTarget::Rig(_) => None,
        })
    }
}

impl BConstraintKind {
    /// Every target this constraint reads (its custom space aside).
    pub fn targets(&self) -> Vec<&BTarget> {
        match self {
            Self::CopyTransforms { target, .. }
            | Self::CopyLocation { target, .. }
            | Self::CopyRotation { target, .. }
            | Self::CopyScale { target, .. }
            | Self::DampedTrack { target, .. }
            | Self::StretchTo { target, .. }
            | Self::LimitDistance { target, .. }
            | Self::ChildOf { target, .. }
            | Self::TrackTo { target, .. }
            | Self::LockedTrack { target, .. }
            | Self::Transformation { target, .. }
            | Self::Floor { target, .. }
            | Self::Action { target, .. }
            | Self::Pivot { target, .. }
            | Self::FollowPath { target, .. }
            | Self::ClampTo { target, .. } => vec![target],
            Self::Armature { targets, .. } => targets.iter().map(|t| &t.0).collect(),
            Self::Ik(ik) => vec![&ik.target, &ik.pole],
            Self::LimitRotation { .. } | Self::LimitLocation { .. } | Self::LimitScale { .. } | Self::MaintainVolume { .. } | Self::Other(_) => Vec::new(),
        }
    }
}

