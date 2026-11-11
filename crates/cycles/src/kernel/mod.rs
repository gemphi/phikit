// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! What runs for every path: `intern/cycles/kernel`. So far the names of
//! the kernels a path moves between, ported from the `DeviceKernel` enum in
//! `kernel/types.h:1744-1763`, and the work tile paths start from.

mod constant;
mod data;
mod record;
mod tile;

pub use constant::{constant, flag};
pub use data::{DataStruct, Field, FieldKind, KernelData};
pub use record::{RecordArray, RecordField, RecordLayout};
pub use tile::KernelWorkTile;

/// `DeviceKernel`, the integrator part: each kernel does one thing to a
/// path and names the kernel the path goes to next.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum DeviceKernel {
    IntegratorInitFromCamera = 0,
    IntegratorInitFromBake,
    IntegratorIntersectClosest,
    IntegratorIntersectShadow,
    IntegratorIntersectSubsurface,
    IntegratorIntersectVolumeStack,
    IntegratorIntersectDedicatedLight,
    IntegratorIntersectMnee,
    IntegratorShadeBackground,
    IntegratorShadeLightNee,
    IntegratorShadeLightForward,
    IntegratorShadeSurface,
    IntegratorShadeSurfaceRaytrace,
    IntegratorShadeVolume,
    IntegratorShadeVolumeRayMarching,
    IntegratorShadeShadow,
    IntegratorShadeDedicatedLight,
    IntegratorShadowPathMneePending,
    /// The CPU's one kernel: a switch over all of the above.
    IntegratorMegakernel,
}

impl DeviceKernel {
    /// The kernels a path can be queued for, in the enum's order.
    pub const INTEGRATOR: [DeviceKernel; 18] = [
        Self::IntegratorInitFromCamera,
        Self::IntegratorInitFromBake,
        Self::IntegratorIntersectClosest,
        Self::IntegratorIntersectShadow,
        Self::IntegratorIntersectSubsurface,
        Self::IntegratorIntersectVolumeStack,
        Self::IntegratorIntersectDedicatedLight,
        Self::IntegratorIntersectMnee,
        Self::IntegratorShadeBackground,
        Self::IntegratorShadeLightNee,
        Self::IntegratorShadeLightForward,
        Self::IntegratorShadeSurface,
        Self::IntegratorShadeSurfaceRaytrace,
        Self::IntegratorShadeVolume,
        Self::IntegratorShadeVolumeRayMarching,
        Self::IntegratorShadeShadow,
        Self::IntegratorShadeDedicatedLight,
        Self::IntegratorShadowPathMneePending,
    ];
}
