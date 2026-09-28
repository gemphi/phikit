//! How a `BSpaceType` finds its own spec.
//!
//! This file is the *index*, not the content. Each workspace owns a folder:
//! `space/<name>/mod.rs` says what the workspace is and
//! `space/<name>/set.rs` holds its areas and the hosts they offer.
//!
//! The lookup is an exhaustive `match`, not a table indexed by the enum's
//! discriminant — so adding a `BSpaceType` variant stops the build until its
//! workspace exists, and a workspace can be left out of the tab strip
//! (`BSpaceType::ALL`) without misindexing every later one.

use super::{BSpaceSpec, BSpaceType};
use super::{
    animation, compositing, geometry_nodes, layout, modeling, rendering, scripting, sculpting,
    shading, texture_paint, uv_editing,
};

impl BSpaceType {
    /// This workspace's spec — its label, icon and areas.
    pub fn spec(self) -> &'static BSpaceSpec {
        match self {
            BSpaceType::Layout => &layout::SPEC,
            BSpaceType::Modeling => &modeling::SPEC,
            BSpaceType::Sculpting => &sculpting::SPEC,
            BSpaceType::UvEditing => &uv_editing::SPEC,
            BSpaceType::TexturePaint => &texture_paint::SPEC,
            BSpaceType::Shading => &shading::SPEC,
            BSpaceType::Animation => &animation::SPEC,
            BSpaceType::Rendering => &rendering::SPEC,
            BSpaceType::Compositing => &compositing::SPEC,
            BSpaceType::GeometryNodes => &geometry_nodes::SPEC,
            BSpaceType::Scripting => &scripting::SPEC,
        }
    }
}
