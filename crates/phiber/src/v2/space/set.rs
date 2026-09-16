//! Space set — one `BSpaceSpec` per `BSpaceType`. Content lives in
//! `space::<name>` sibling modules.

use super::{BSpaceSpec, BSpaceType};
use super::{
    animation, compositing, geometry_nodes, layout, modeling, rendering, scripting, sculpting,
    shading, texture_paint, uv_editing,
};

pub static SET: &[BSpaceSpec] = &[
    BSpaceSpec {
        kind: BSpaceType::Layout,
        label: layout::LABEL,
        icon: layout::ICON,
        areas: layout::AREAS,
    },
    BSpaceSpec {
        kind: BSpaceType::Modeling,
        label: modeling::LABEL,
        icon: modeling::ICON,
        areas: modeling::AREAS,
    },
    BSpaceSpec {
        kind: BSpaceType::Sculpting,
        label: sculpting::LABEL,
        icon: sculpting::ICON,
        areas: sculpting::AREAS,
    },
    BSpaceSpec {
        kind: BSpaceType::UvEditing,
        label: uv_editing::LABEL,
        icon: uv_editing::ICON,
        areas: uv_editing::AREAS,
    },
    BSpaceSpec {
        kind: BSpaceType::TexturePaint,
        label: texture_paint::LABEL,
        icon: texture_paint::ICON,
        areas: texture_paint::AREAS,
    },
    BSpaceSpec {
        kind: BSpaceType::Shading,
        label: shading::LABEL,
        icon: shading::ICON,
        areas: shading::AREAS,
    },
    BSpaceSpec {
        kind: BSpaceType::Animation,
        label: animation::LABEL,
        icon: animation::ICON,
        areas: animation::AREAS,
    },
    BSpaceSpec {
        kind: BSpaceType::Rendering,
        label: rendering::LABEL,
        icon: rendering::ICON,
        areas: rendering::AREAS,
    },
    BSpaceSpec {
        kind: BSpaceType::Compositing,
        label: compositing::LABEL,
        icon: compositing::ICON,
        areas: compositing::AREAS,
    },
    BSpaceSpec {
        kind: BSpaceType::GeometryNodes,
        label: geometry_nodes::LABEL,
        icon: geometry_nodes::ICON,
        areas: geometry_nodes::AREAS,
    },
    BSpaceSpec {
        kind: BSpaceType::Scripting,
        label: scripting::LABEL,
        icon: scripting::ICON,
        areas: scripting::AREAS,
    },
];
