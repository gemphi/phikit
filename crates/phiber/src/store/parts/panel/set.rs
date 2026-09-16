//! Panel set - host-to-panel association and section arrays.
//!
//! Buts sections match Blender's eSpaceButtons_Context (BCONTEXT) enum.
//! Each section has multiple accordion sub-panels with property rows.
//! Ref: DNA_space_types.h, buttons_context.c

use super::def;
use super::{BButsContext, PanelSection};
use crate::store::hosts::BHostType;

/// Blender Properties editor context tabs (BCONTEXT order).
pub static BUTS_SECTIONS: &[PanelSection] = &[
    PanelSection {
        idx: def::SECTION_RENDER_ID,
        ctx: BButsContext::Render,
        label: def::SECTION_RENDER_LABEL,
        icon: def::SECTION_RENDER_ICON,
        expanded: true,
        panels: def::RENDER_ACCORDIONS,
    },
    PanelSection {
        idx: def::SECTION_SCENE_ID,
        ctx: BButsContext::Scene,
        label: def::SECTION_SCENE_LABEL,
        icon: def::SECTION_SCENE_ICON,
        expanded: false,
        panels: def::SCENE_ACCORDIONS,
    },
    PanelSection {
        idx: def::SECTION_WORLD_ID,
        ctx: BButsContext::World,
        label: def::SECTION_WORLD_LABEL,
        icon: def::SECTION_WORLD_ICON,
        expanded: false,
        panels: def::WORLD_ACCORDIONS,
    },
    PanelSection {
        idx: def::SECTION_OBJECT_ID,
        ctx: BButsContext::Object,
        label: def::SECTION_OBJECT_LABEL,
        icon: def::SECTION_OBJECT_ICON,
        expanded: false,
        panels: def::OBJECT_ACCORDIONS,
    },
    PanelSection {
        idx: def::SECTION_DATA_ID,
        ctx: BButsContext::Data,
        label: def::SECTION_DATA_LABEL,
        icon: def::SECTION_DATA_ICON,
        expanded: false,
        panels: def::DATA_ACCORDIONS,
    },
    PanelSection {
        idx: def::SECTION_MATERIAL_ID,
        ctx: BButsContext::Material,
        label: def::SECTION_MATERIAL_LABEL,
        icon: def::SECTION_MATERIAL_ICON,
        expanded: false,
        panels: def::MATERIAL_ACCORDIONS,
    },
    PanelSection {
        idx: def::SECTION_TEXTURE_ID,
        ctx: BButsContext::Texture,
        label: def::SECTION_TEXTURE_LABEL,
        icon: def::SECTION_TEXTURE_ICON,
        expanded: false,
        panels: def::TEXTURE_ACCORDIONS,
    },
    PanelSection {
        idx: def::SECTION_PARTICLE_ID,
        ctx: BButsContext::Particle,
        label: def::SECTION_PARTICLE_LABEL,
        icon: def::SECTION_PARTICLE_ICON,
        expanded: false,
        panels: def::PARTICLE_ACCORDIONS,
    },
    PanelSection {
        idx: def::SECTION_PHYSICS_ID,
        ctx: BButsContext::Physics,
        label: def::SECTION_PHYSICS_LABEL,
        icon: def::SECTION_PHYSICS_ICON,
        expanded: false,
        panels: def::PHYSICS_ACCORDIONS,
    },
    PanelSection {
        idx: def::SECTION_BONE_ID,
        ctx: BButsContext::Bone,
        label: def::SECTION_BONE_LABEL,
        icon: def::SECTION_BONE_ICON,
        expanded: false,
        panels: def::BONE_ACCORDIONS,
    },
    PanelSection {
        idx: def::SECTION_MODIFIER_ID,
        ctx: BButsContext::Modifier,
        label: def::SECTION_MODIFIER_LABEL,
        icon: def::SECTION_MODIFIER_ICON,
        expanded: false,
        panels: def::MODIFIER_ACCORDIONS,
    },
    PanelSection {
        idx: def::SECTION_CONSTRAINT_ID,
        ctx: BButsContext::Constraint,
        label: def::SECTION_CONSTRAINT_LABEL,
        icon: def::SECTION_CONSTRAINT_ICON,
        expanded: false,
        panels: def::CONSTRAINT_ACCORDIONS,
    },
    PanelSection {
        idx: def::SECTION_BONE_CONSTRAINT_ID,
        ctx: BButsContext::BoneConstraint,
        label: def::SECTION_BONE_CONSTRAINT_LABEL,
        icon: def::SECTION_BONE_CONSTRAINT_ICON,
        expanded: false,
        panels: def::BONE_CONSTRAINT_ACCORDIONS,
    },
    PanelSection {
        idx: def::SECTION_RENDER_LAYER_ID,
        ctx: BButsContext::RenderLayer,
        label: def::SECTION_RENDER_LAYER_LABEL,
        icon: def::SECTION_RENDER_LAYER_ICON,
        expanded: false,
        panels: def::RENDER_LAYER_ACCORDIONS,
    },
];

pub static VIEW3D_SECTIONS: &[PanelSection] = &[
    PanelSection {
        idx: def::SECTION_ITEM_ID,
        ctx: BButsContext::Object,
        label: def::SECTION_ITEM_LABEL,
        icon: def::SECTION_ITEM_ICON,
        expanded: true,
        panels: def::ITEM_ACCORDIONS,
    },
    PanelSection {
        idx: def::SECTION_TOOL_ID,
        ctx: BButsContext::Object,
        label: def::SECTION_TOOL_LABEL,
        icon: def::SECTION_TOOL_ICON,
        expanded: false,
        panels: def::TOOL_ACCORDIONS,
    },
    PanelSection {
        idx: def::SECTION_VIEW_ID,
        ctx: BButsContext::Object,
        label: def::SECTION_VIEW_LABEL,
        icon: def::SECTION_VIEW_ICON,
        expanded: false,
        panels: def::VIEW_ACCORDIONS,
    },
];

pub static DEFAULT_SECTIONS: &[PanelSection] = &[];

/// Panel sections for a given host.
pub fn with_idx(host: BHostType) -> &'static [super::PanelSection] {
    match host {
        BHostType::Buts => BUTS_SECTIONS,
        BHostType::View3D => VIEW3D_SECTIONS,
        _ => DEFAULT_SECTIONS,
    }
}
