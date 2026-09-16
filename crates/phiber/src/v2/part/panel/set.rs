//! Panel set — dispatcher over per-context section+prop tables.
//! Each context file (`object.rs`, `render.rs`, …) exposes a `SECTIONS` static.
//! Ground truth: `bl_ui/properties_*.py` (registered panels) + RNA definitions
//! in `source/blender/makesrna/intern/rna_*.cc`.

use super::prop::BPropSpec;
use super::{
    collection, constraint, data, material, modifier, object, output, particles, physics, render,
    scene, texture, tool, view_layer, world,
};
use crate::v2::host::BHostType;
use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BSectionSpec {
    pub title: &'static str,
    pub props: &'static [BPropSpec],
}

impl BSpec for BSectionSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Panel)
    }
    fn label(&self) -> &'static str {
        self.title
    }
}

pub const fn sec(title: &'static str, props: &'static [BPropSpec]) -> BSectionSpec {
    BSectionSpec { title, props }
}

pub fn sections_for(host: BHostType, tab: &str) -> &'static [BSectionSpec] {
    if !matches!(host, BHostType::Buts) {
        return &[];
    }
    match tab {
        "Active Tool" => tool::SECTIONS,
        "Render" => render::SECTIONS,
        "Output" => output::SECTIONS,
        "View Layer" => view_layer::SECTIONS,
        "Scene" => scene::SECTIONS,
        "World" => world::SECTIONS,
        "Collection" => collection::SECTIONS,
        "Object" => object::SECTIONS,
        "Modifiers" => modifier::SECTIONS,
        "Particles" => particles::SECTIONS,
        "Physics" => physics::SECTIONS,
        "Constraints" => constraint::SECTIONS,
        "Data" => data::SECTIONS,
        "Material" => material::SECTIONS,
        "Texture" => texture::SECTIONS,
        _ => &[],
    }
}
