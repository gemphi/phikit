//! Panel part spec - collapsible property panels in the Center region.
//!
//! Ref: Blender's Properties editor panels (Render, Output, Scene, World, Object, etc.)
//! Each context tab has multiple sub-section accordion panels.

use crate::store::parts::{BPart, BPartSpec, BPartType};

pub mod def;
pub mod set;

/// Properties editor context tab enum.
/// Matches Blender's eSpaceButtons_Context (BCONTEXT) in DNA_space_types.h.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BButsContext {
    Render = 0,
    Scene = 1,
    World = 2,
    Object = 3,
    Data = 4,
    Material = 5,
    Texture = 6,
    Particle = 7,
    Physics = 8,
    Bone = 9,
    Modifier = 10,
    Constraint = 11,
    BoneConstraint = 12,
    RenderLayer = 13,
}

impl BButsContext {
    pub const ALL: &[BButsContext] = &[
        BButsContext::Render,
        BButsContext::Scene,
        BButsContext::World,
        BButsContext::Object,
        BButsContext::Data,
        BButsContext::Material,
        BButsContext::Texture,
        BButsContext::Particle,
        BButsContext::Physics,
        BButsContext::Bone,
        BButsContext::Modifier,
        BButsContext::Constraint,
        BButsContext::BoneConstraint,
        BButsContext::RenderLayer,
    ];

    pub fn label(self) -> &'static str {
        match self {
            BButsContext::Render => "Render",
            BButsContext::Scene => "Scene",
            BButsContext::World => "World",
            BButsContext::Object => "Object",
            BButsContext::Data => "Data",
            BButsContext::Material => "Material",
            BButsContext::Texture => "Texture",
            BButsContext::Particle => "Particle",
            BButsContext::Physics => "Physics",
            BButsContext::Bone => "Bone",
            BButsContext::Modifier => "Modifiers",
            BButsContext::Constraint => "Constraints",
            BButsContext::BoneConstraint => "Bone Constraints",
            BButsContext::RenderLayer => "Render Layers",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            BButsContext::Render => "render",
            BButsContext::Scene => "scene",
            BButsContext::World => "world",
            BButsContext::Object => "object",
            BButsContext::Data => "data",
            BButsContext::Material => "material",
            BButsContext::Texture => "texture",
            BButsContext::Particle => "particle",
            BButsContext::Physics => "physics",
            BButsContext::Bone => "bone",
            BButsContext::Modifier => "modifier",
            BButsContext::Constraint => "constraint",
            BButsContext::BoneConstraint => "bone_constraint",
            BButsContext::RenderLayer => "render_layer",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            BButsContext::Render => "render",
            BButsContext::Scene => "scene",
            BButsContext::World => "world",
            BButsContext::Object => "object",
            BButsContext::Data => "data",
            BButsContext::Material => "material",
            BButsContext::Texture => "texture",
            BButsContext::Particle => "particle",
            BButsContext::Physics => "physics",
            BButsContext::Bone => "bone",
            BButsContext::Modifier => "modifiers",
            BButsContext::Constraint => "constraint",
            BButsContext::BoneConstraint => "bone_constraint",
            BButsContext::RenderLayer => "render_layer",
        }
    }
}

/// A single context tab in the Properties editor (e.g. Render, Scene, Object).
/// Each tab maps to a BButsContext variant and contains multiple accordion sub-panels.
#[derive(Clone, Copy, Debug)]
pub struct PanelSection {
    pub idx: &'static str,
    pub ctx: BButsContext,
    pub label: &'static str,
    pub icon: &'static str,
    pub expanded: bool,
    /// Sub-panel accordions within this context tab.
    pub panels: &'static [PanelAccordion],
}

/// A single accordion panel within a context tab (e.g. "Dimensions", "Output" under Render).
#[derive(Clone, Copy, Debug)]
pub struct PanelAccordion {
    pub label: &'static str,
    pub icon: &'static str,
    pub expanded: bool,
    /// Static property rows displayed when this accordion is expanded.
    pub rows: &'static [PanelRow],
}

/// A single property row within an accordion panel.
#[derive(Clone, Copy, Debug)]
pub struct PanelRow {
    pub label: &'static str,
    pub value: &'static str,
}

/// Spec for a Panel part.
#[derive(Clone, Copy, Debug)]
pub struct PanelPartSpec {
    pub idx: BPartType,
    pub items: &'static [BPart],
}

impl PanelPartSpec {
    pub const fn new() -> Self {
        Self {
            idx: BPartType::Panel,
            items: &[],
        }
    }
}

impl BPartSpec for PanelPartSpec {
    fn idx(&self) -> BPartType {
        self.idx
    }
    fn items(&self) -> &'static [BPart] {
        self.items
    }
}
