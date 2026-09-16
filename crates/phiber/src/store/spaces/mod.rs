//! BSpace level - workspace tabs and their zone/area grids.
//!
//! A BSpace is a named workspace tab. It defines a grid of areas.
//! Each space IS a grid. The space is the glue of zones.
//!
//! Hierarchy: BSpace -> Zone -> BAreaSpec -> Host -> BRegion -> BPane -> Part
//!
//! Store modules are plural. State modules are singular.

pub mod animation;
pub mod compositing;
pub mod def;
pub mod geometry_nodes;
pub mod layout;
pub mod modeling;
pub mod rendering;
pub mod scripting;
pub mod sculpting;
pub mod set;
pub mod shading;
pub mod texture_paint;
pub mod uv_editing;
pub use def::{BObjectMode, BSpaceType};

use crate::store::areas::BAreaSpec;
use crate::store::spec::{BSpec, BSpecType};
use crate::store::zones::BZoneType;

/// Workspace tab types - mirrors Blender's workspaces.
/// Ref: Blender Manual - workspaces.rst lines 70-97

impl BSpaceType {
    pub fn label(self) -> &'static str {
        set::LABELS[self as usize]
    }

    pub fn from_name(name: &str) -> Option<BSpaceType> {
        set::ALL
            .iter()
            .find(|&&s| format!("{:?}", s) == name)
            .copied()
    }

    pub fn icon(self) -> &'static str {
        set::ICONS[self as usize]
    }

    pub fn all() -> &'static [BSpaceType] {
        set::ALL
    }

    /// Get the full spec for this space, including its area set.
    pub fn items(self) -> &'static BSpaceSpec {
        &set::ALL_SPECS[self as usize]
    }

    pub fn with_set(self) -> &'static BSpaceSpec {
        &set::ALL_SPECS[self as usize]
    }

    /// Zones in this space, each holding areas.
    pub fn zones(self) -> &'static [BZoneType] {
        match self {
            BSpaceType::Layout => self::layout::set::ZONES,
            BSpaceType::Modeling => self::modeling::set::ZONES,
            BSpaceType::Sculpting => self::sculpting::set::ZONES,
            BSpaceType::UvEditing => self::uv_editing::set::ZONES,
            BSpaceType::TexturePaint => self::texture_paint::set::ZONES,
            BSpaceType::Shading => self::shading::set::ZONES,
            BSpaceType::Animation => self::animation::set::ZONES,
            BSpaceType::Rendering => self::rendering::set::ZONES,
            BSpaceType::Compositing => self::compositing::set::ZONES,
            BSpaceType::GeometryNodes => self::geometry_nodes::set::ZONES,
            BSpaceType::Scripting => self::scripting::set::ZONES,
        }
    }

    /// Grid column fractions (left-to-right).
    pub fn columns(self) -> &'static [f32] {
        match self {
            BSpaceType::Layout => self::layout::set::COLUMNS,
            BSpaceType::Modeling => self::modeling::set::COLUMNS,
            BSpaceType::Sculpting => self::sculpting::set::COLUMNS,
            BSpaceType::UvEditing => self::uv_editing::set::COLUMNS,
            BSpaceType::TexturePaint => self::texture_paint::set::COLUMNS,
            BSpaceType::Shading => self::shading::set::COLUMNS,
            BSpaceType::Animation => self::animation::set::COLUMNS,
            BSpaceType::Rendering => self::rendering::set::COLUMNS,
            BSpaceType::Compositing => self::compositing::set::COLUMNS,
            BSpaceType::GeometryNodes => self::geometry_nodes::set::COLUMNS,
            BSpaceType::Scripting => self::scripting::set::COLUMNS,
        }
    }
}

/// Static spec for a space: idx + child area specs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BSpaceSpec {
    pub idx: BSpaceType,
    /// Child area specs in this space, each positioned at a zone.
    pub items: &'static [BAreaSpec],
}

impl BSpec for BSpaceSpec {
    fn idx(&self) -> BSpecType {
        BSpecType::BSpace(self.idx)
    }
    fn items(&self) -> Vec<&dyn BSpec> {
        self.items.iter().map(|a| a as &dyn BSpec).collect()
    }
}

impl BSpaceSpec {
    /// Return the child area set for simple hierarchy loops.
    pub fn items(&self) -> &'static [BAreaSpec] {
        self.items
    }
}
