//! Modebar set — per-host mode tables.
//! Blender ref: `eContextObjectMode` in `DNA_object_types.h` (View3D),
//! `SI_MODE_*` (Image/UV), `SEQ_VIEW_*` (Sequencer), etc. Modes shown for a
//! Mesh object at the top of `space_view3d.py` — other object types use
//! subsets of these (Curve: Object/Edit; Empty/Camera: Object only).

use crate::v2::state::{BKey as _, CtxKey};
use crate::v2::host::BHostType;
use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BModeSpec {
    pub icon: &'static str,
    pub label: &'static str,
}

impl BSpec for BModeSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Modebar)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

const fn m(icon: &'static str, label: &'static str) -> BModeSpec {
    BModeSpec { icon, label }
}

// View3D — mirrors `eContextObjectMode` for a Mesh object.
static VIEW3D: &[BModeSpec] = &[
    m("object", "Object Mode"),
    m("edit", "Edit Mode"),
    m("brush", "Sculpt Mode"),
    m("brush", "Vertex Paint"),
    m("brush", "Weight Paint"),
    m("paintbrush", "Texture Paint"),
    m("particle", "Particle Edit"),
    m("pose", "Pose Mode"),
];
// Blender's seven outliner display modes (`eSpaceOutliner_Mode`), in the
// order `space_outliner.py` lists them.
static OUTLINER: &[BModeSpec] = &[
    m("scene", "View Layer"),
    m("scene", "Scenes"),
    m("blend", "Blender File"),
    m("layers", "Library Overrides"),
    m("sequencer", "Video Sequencer"),
    m("braces", "Data API"),
    m("trash", "Orphan Data"),
];
static IMAGE: &[BModeSpec] = &[m("image", "View"), m("brush", "Paint"), m("mask", "Mask")];
static SEQ: &[BModeSpec] = &[
    m("seq", "Sequencer"),
    m("image", "Preview"),
    m("layout", "Sequencer & Preview"),
];
static NODE: &[BModeSpec] = &[
    m("object", "Object"),
    m("world", "World"),
    m("brush", "Line Style"),
];
static ACTION: &[BModeSpec] = &[
    m("action", "Dope Sheet"),
    m("action", "Action Editor"),
    m("shapes", "Shape Key Editor"),
    m("pencil", "Grease Pencil"),
    m("mask", "Mask"),
    m("cache", "Cache File"),
];
static IPO: &[BModeSpec] = &[m("graph", "F-Curves"), m("drivers", "Drivers")];
static CLIP: &[BModeSpec] = &[m("clip", "Tracking"), m("mask", "Mask")];

pub fn modes_for(host: BHostType) -> &'static [BModeSpec] {
    match host {
        BHostType::View3D => VIEW3D,
        BHostType::Outliner => OUTLINER,
        BHostType::Image => IMAGE,
        BHostType::Seq => SEQ,
        BHostType::Node | BHostType::Shader | BHostType::Compositor | BHostType::GeoNode => NODE,
        BHostType::Action => ACTION,
        BHostType::Ipo => IPO,
        BHostType::Clip => CLIP,
        _ => &[],
    }
}

use crate::v2::state::{BPath, OutlinerKey, val::V_MODE_OBJECT};

/// Which state key a host's mode dropdown writes.
///
/// The Outliner's dropdown is a *display mode* selector, not an object-mode
/// selector. Pointing it at `ctx.mode` made picking "Scenes" in the Outliner
/// also drop the 3D viewport out of Object Mode, since both hosts read the
/// same key — and so did every other editor's: the Dope Sheet in the Action
/// Editor put the viewport back in Object Mode. The object's mode is the
/// 3D viewport's (`ctx.mode`); every other editor's mode is its own space's
/// (`<host>.mode`: `SpaceAction.mode`, `SpaceImage.mode`, …).
pub fn mode_key(host: BHostType) -> BPath {
    match host {
        BHostType::Outliner => OutlinerKey::DisplayMode.path(),
        BHostType::View3D => CtxKey::Mode.path(),
        _ => BPath::owned(vec![host.slug(), "mode"]),
    }
}

/// The mode a host shows before anything has been chosen.
pub fn default_mode(host: BHostType) -> &'static str {
    match host {
        // Blender opens the 3D viewport in Object Mode and the Outliner in
        // View Layer; every other host starts on its first declared mode.
        BHostType::View3D => V_MODE_OBJECT,
        BHostType::Outliner => "View Layer",
        _ => modes_for(host)
            .first()
            .map(|m| m.label)
            .unwrap_or(V_MODE_OBJECT),
    }
}
