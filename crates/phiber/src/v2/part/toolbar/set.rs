//! Toolbar set — per-host tool tables.
//! Blender ref: `bl_ui/space_toolsystem_toolbar.py`; per-mode tool registration
//! via `bpy.types.WorkSpaceTool` subclasses.

use crate::v2::host::BHostType;
use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BToolSpec {
    pub icon: &'static str,
    pub label: &'static str,
}

impl BSpec for BToolSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Toolbar)
    }
    fn label(&self) -> &'static str {
        self.label
    }
    fn icon(&self) -> &'static str {
        self.icon
    }
}

const fn t(icon: &'static str, label: &'static str) -> BToolSpec {
    BToolSpec { icon, label }
}

static VIEW3D: &[BToolSpec] = &[
    t("cursor", "Cursor"),
    t("select", "Select Box"),
    t("move", "Move"),
    t("rotate", "Rotate"),
    t("scale", "Scale"),
    t("transform", "Transform"),
    t("annotate", "Annotate"),
    t("measure", "Measure"),
    t("add-cube", "Add Cube"),
];
/// The UV editor's toolset: it edits a mesh's UVs, so it transforms.
static UV: &[BToolSpec] = &[
    t("cursor", "Cursor"),
    t("select", "Select"),
    t("move", "Move"),
    t("rotate", "Rotate"),
    t("scale", "Scale"),
];
/// The Image editor's, which is a different editor and has a different
/// toolbar: the manual lists exactly two tools for it, Sample and Annotate
/// (`editors/image/introduction`, "Toolbar"). It shares no tool with the UV
/// editor — there is nothing in a picture to move, rotate or scale — and
/// the render window wore Cursor / Select / Move / Rotate / Scale down its
/// left edge because the two hosts were reading one list.
static IMAGE: &[BToolSpec] = &[
    t("pipette", "Sample"),
    t("annotate", "Annotate"),
];
static NODE: &[BToolSpec] = &[
    t("select", "Select Box"),
    t("move", "Move"),
    t("rotate", "Rotate"),
    t("scale", "Scale"),
];
static CLIP: &[BToolSpec] = &[
    t("select", "Select Box"),
    t("move", "Move"),
    t("rotate", "Rotate"),
    t("scale", "Scale"),
    t("track", "Track"),
    t("grab", "Grab"),
];
static FILE: &[BToolSpec] = &[
    t("list", "List View"),
    t("grid", "Grid View"),
    t("thumb", "Thumbnails"),
];

pub fn tools_for(host: BHostType) -> &'static [BToolSpec] {
    match host {
        BHostType::View3D => VIEW3D,
        BHostType::Uv => UV,
        BHostType::Image => IMAGE,
        BHostType::Node | BHostType::Shader | BHostType::Compositor | BHostType::GeoNode => NODE,
        BHostType::Clip => CLIP,
        BHostType::File | BHostType::Asset => FILE,
        _ => &[],
    }
}
