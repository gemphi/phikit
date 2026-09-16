//! BSpace set - static spec arrays, labels, and icons.

use super::BSpaceSpec;
use super::def::*;

pub static ALL: &[BSpaceType] = &[
    BSpaceType::Layout,
    BSpaceType::Modeling,
    BSpaceType::Sculpting,
    BSpaceType::UvEditing,
    BSpaceType::TexturePaint,
    BSpaceType::Shading,
    BSpaceType::Animation,
    BSpaceType::Rendering,
    BSpaceType::Compositing,
    BSpaceType::GeometryNodes,
    BSpaceType::Scripting,
];

pub static ALL_SPECS: &[BSpaceSpec] = &[
    BSpaceSpec {
        idx: BSpaceType::Layout,
        items: super::layout::set::LAYOUT_AREAS,
    },
    BSpaceSpec {
        idx: BSpaceType::Modeling,
        items: super::modeling::set::MODELING_AREAS,
    },
    BSpaceSpec {
        idx: BSpaceType::Sculpting,
        items: super::sculpting::set::SCULPTING_AREAS,
    },
    BSpaceSpec {
        idx: BSpaceType::UvEditing,
        items: super::uv_editing::set::UV_EDITING_AREAS,
    },
    BSpaceSpec {
        idx: BSpaceType::TexturePaint,
        items: super::texture_paint::set::TEXTURE_PAINT_AREAS,
    },
    BSpaceSpec {
        idx: BSpaceType::Shading,
        items: super::shading::set::SHADING_AREAS,
    },
    BSpaceSpec {
        idx: BSpaceType::Animation,
        items: super::animation::set::ANIMATION_AREAS,
    },
    BSpaceSpec {
        idx: BSpaceType::Rendering,
        items: super::rendering::set::RENDERING_AREAS,
    },
    BSpaceSpec {
        idx: BSpaceType::Compositing,
        items: super::compositing::set::COMPOSITING_AREAS,
    },
    BSpaceSpec {
        idx: BSpaceType::GeometryNodes,
        items: super::geometry_nodes::set::GEOMETRY_NODES_AREAS,
    },
    BSpaceSpec {
        idx: BSpaceType::Scripting,
        items: super::scripting::set::SCRIPTING_AREAS,
    },
];

pub static LABELS: &[&str] = &[
    LAYOUT_LABEL,
    MODELING_LABEL,
    SCULPTING_LABEL,
    UV_EDITING_LABEL,
    TEXTURE_PAINT_LABEL,
    SHADING_LABEL,
    ANIMATION_LABEL,
    RENDERING_LABEL,
    COMPOSITING_LABEL,
    GEOMETRY_NODES_LABEL,
    SCRIPTING_LABEL,
];
pub static ICONS: &[&str] = &[
    LAYOUT_ICON,
    MODELING_ICON,
    SCULPTING_ICON,
    UV_EDITING_ICON,
    TEXTURE_PAINT_ICON,
    SHADING_ICON,
    ANIMATION_ICON,
    RENDERING_ICON,
    COMPOSITING_ICON,
    GEOMETRY_NODES_ICON,
    SCRIPTING_ICON,
];
