//! Controls set - per-host header control clusters (static data).

use super::{CtrlCluster, CtrlEntry, EMPTY_CLUSTER, def};
use crate::store::hosts::BHostType;

const fn e(c: (&'static str, &'static str, &'static str), active: bool) -> CtrlEntry {
    CtrlEntry {
        idx: c.0,
        label: c.1,
        icon: c.2,
        active,
    }
}

pub static VIEW3D_ORIENTATION: &[CtrlEntry] = &[
    e(def::ORIENT_GLOBAL, true),
    e(def::ORIENT_LOCAL, false),
    e(def::ORIENT_NORMAL, false),
    e(def::ORIENT_GIMBAL, false),
    e(def::ORIENT_VIEW, false),
];

pub static VIEW3D_SNAP: &[CtrlEntry] = &[
    e(def::SNAP_INCREMENT, true),
    e(def::SNAP_VERTEX, false),
    e(def::SNAP_EDGE, false),
    e(def::SNAP_FACE, false),
    e(def::SNAP_VOLUME, false),
];

pub static VIEW3D_TOGGLES: &[CtrlEntry] = &[
    e(def::TOGGLE_SNAP, false),
    e(def::TOGGLE_PROPORTIONAL, false),
];

pub static VIEW3D_SHADING: &[CtrlEntry] = &[
    e(def::SHADE_WIREFRAME, false),
    e(def::SHADE_SOLID, true),
    e(def::SHADE_MATERIAL, false),
    e(def::SHADE_RENDERED, false),
];

pub static VIEW3D_VIEW_TOGGLES: &[CtrlEntry] =
    &[e(def::VIEW_OVERLAYS, true), e(def::VIEW_GIZMOS, true)];

static VIEW3D_CLUSTER: CtrlCluster = CtrlCluster {
    orientation: VIEW3D_ORIENTATION,
    snap: VIEW3D_SNAP,
    toggles: VIEW3D_TOGGLES,
    shading: VIEW3D_SHADING,
    view_toggles: VIEW3D_VIEW_TOGGLES,
};

/// Control cluster for a given host (empty for hosts without header controls).
pub fn with_idx(host: BHostType) -> &'static CtrlCluster {
    match host {
        BHostType::View3D => &VIEW3D_CLUSTER,
        _ => &EMPTY_CLUSTER,
    }
}
