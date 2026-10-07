//! The space a driver target is read in — `DTAR_FLAG_LOCALSPACE` and
//! `DTAR_FLAG_LOCAL_CONSTS` (`fcurve_driver.cc`) — over a snapshot of every
//! object's channels.
//!
//! The three the enum offers are the three the Driver panel offers
//! (`prop_local_space_items`, `rna_fcurve.cc`), and they are *not* two of
//! one thing and a special case:
//!
//! * **World Space** (no flag) — `object_to_world`: the local matrix
//!   composed up the parent chain, which is what `solve_world` already
//!   builds for the player.
//! * **Transform Space** (`DTAR_FLAG_LOCALSPACE`) — `BKE_object_to_mat4`:
//!   the object's own channels and nothing above them.
//! * **Local Space** (`DTAR_FLAG_LOCALSPACE | DTAR_FLAG_LOCAL_CONSTS`) —
//!   `BKE_constraint_mat_convertspace(WORLD → LOCAL)`. For a parent-less
//!   object that is **not** the same as Transform Space and not the same as
//!   World Space either: Blender subtracts only the object's own *rotation*
//!   (`BKE_object_to_mat4`, normalised, translation zeroed, inverted) and
//!   leaves everything else, so the matrix comes out in the object's own
//!   rotated axes. For a parented one it subtracts
//!   `parent.world · parentinv` instead.
//!
//! **Stated rather than hidden:** Local Space is meant to include what the
//! object's constraints add. The pass here runs before constraints are
//! solved (`BlendMotionList::solve` applies them after the drivers), so a
//! Local Space target with constraints is read without them. A target with
//! none — which is what a Local Space driver target almost always is — is
//! exact.

use crate::blend::object::transform::{IDENTITY4, ObjectTransformRaw, object_to_mat4};
use std::collections::HashMap;

/// Which matrix a driver target reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BDriverSpace {
    /// No flag: `object_to_world`.
    #[default]
    World,
    /// `DTAR_FLAG_LOCALSPACE`: the object's own channels.
    Transform,
    /// `DTAR_FLAG_LOCALSPACE | DTAR_FLAG_LOCAL_CONSTS`: the world matrix
    /// with the parent's effect — or, parent-less, its own rotation — taken
    /// off.
    Local,
}

impl BDriverSpace {
    /// `DTAR_FLAG_LOCALSPACE`.
    pub const LOCALSPACE: u32 = 1 << 2;
    /// `DTAR_FLAG_LOCAL_CONSTS`.
    pub const LOCAL_CONSTS: u32 = 1 << 3;

    /// The space `DriverTarget.flag` names.
    pub fn of_flag(flag: u32) -> Self {
        match (flag & Self::LOCALSPACE != 0, flag & Self::LOCAL_CONSTS != 0) {
            (true, true) => Self::Local,
            (true, false) => Self::Transform,
            (false, _) => Self::World,
        }
    }
}

/// Every object's matrices as a driver pass reads them.
pub trait BDriverWorld {
    /// `world = parent.world · parentinv · local` over this snapshot, in
    /// list order. A parent outside the list leaves the object on its local.
    fn driver_worlds(&self, index: &HashMap<u64, usize>) -> Vec<[[f32; 4]; 4]>;
}

impl BDriverWorld for [ObjectTransformRaw] {
    fn driver_worlds(&self, index: &HashMap<u64, usize>) -> Vec<[[f32; 4]; 4]> {
        let local: Vec<[[f32; 4]; 4]> = self.iter().map(object_to_mat4).collect();
        let parent = |i: usize| self.get(i).and_then(|r| index.get(&r.parent_address)).copied();
        let identity = &IDENTITY4;
        let parentinv = |i: usize| self.get(i).map_or(identity, |r| &r.parentinv);
        super::super::motion::solve_world(&local, parent, parentinv)
    }
}
