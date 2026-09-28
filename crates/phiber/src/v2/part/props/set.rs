//! Panel set — dispatcher over per-context section+prop tables.
//! Each context file (`object.rs`, `render.rs`, …) exposes a `SECTIONS` static.
//! Ground truth: `bl_ui/properties_*.py` (registered panels) + RNA definitions
//! in `source/blender/makesrna/intern/rna_*.cc`.

use super::prop::BPropSpec;
use super::collection;
use super::constraint;
use super::data;
use super::material;
use super::modifier;
use super::object;
use super::output;
use super::particles;
use super::physics;
use super::render;
use super::scene;
use super::texture;
use super::tool;
use super::view_layer;
use super::world;
use crate::v2::host::BHostType;
use crate::v2::part::BPartType;
use crate::v2::spec::{BSpec, BSpecKind};

/// One panel of a Properties tab — the parts of Blender's `PanelType` that a
/// declared panel needs (`bl_ui/properties_*.py`, Cycles' `addon/ui.py`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BSectionSpec {
    /// `bl_label`.
    pub title: &'static str,
    /// What `draw()` lays out, one row per property.
    pub props: &'static [BPropSpec],
    /// What `draw_header()` lays out: controls in the header, before the
    /// title, with no label (`layout.prop(rd, "use_motion_blur", text="")`).
    pub header: &'static [BPropSpec],
    /// `bl_options = {'DEFAULT_CLOSED'}`: closed until the user opens it.
    pub closed: bool,
    /// `bl_options = {'HIDE_HEADER'}`: no header strip, never folds.
    pub headless: bool,
    /// Sub-panels — the panels whose `bl_parent_id` is this one.
    pub list: &'static [BSectionSpec],
}

impl BSpec for BSectionSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Part(BPartType::Panel)
    }
    fn label(&self) -> &'static str {
        self.title
    }
}

/// A panel that is open by default, with a header and no sub-panels.
pub const fn sec(title: &'static str, props: &'static [BPropSpec]) -> BSectionSpec {
    BSectionSpec { title, props, header: &[], closed: false, headless: false, list: &[] }
}

impl BSectionSpec {
    /// `DEFAULT_CLOSED`.
    pub const fn closed(mut self) -> Self {
        self.closed = true;
        self
    }

    /// `HIDE_HEADER`.
    pub const fn headless(mut self) -> Self {
        self.headless = true;
        self
    }

    /// `draw_header`: the properties drawn in the header.
    pub const fn header(mut self, props: &'static [BPropSpec]) -> Self {
        self.header = props;
        self
    }

    /// The sub-panels, in Blender's order.
    pub const fn subs(mut self, list: &'static [BSectionSpec]) -> Self {
        self.list = list;
        self
    }

    /// Every property this panel and its sub-panels declare, header ones
    /// included.
    pub fn all_props(&self) -> Vec<&'static BPropSpec> {
        let mut out: Vec<&'static BPropSpec> = self.header.iter().chain(self.props.iter()).collect();
        for child in self.list {
            out.extend(child.all_props());
        }
        out
    }
}

/// Every Properties tab, in Blender's order.
pub const TABS: &[&str] = &[
    "Active Tool", "Render", "Output", "View Layer", "Scene", "World", "Collection", "Object",
    "Modifiers", "Particles", "Physics", "Constraints", "Data", "Material", "Texture",
];

/// The property declared under `key`, whichever tab holds it — so a write to
/// `ctx.buts.<key>` can be described by its label and, for an enum, the
/// chosen option's label ("Render Engine: Phiray").
pub fn prop_named(key: &str) -> Option<&'static BPropSpec> {
    TABS.iter()
        .flat_map(|tab| sections_for(BHostType::Buts, tab))
        .flat_map(|s| s.all_props())
        .find(|p| p.key == key)
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
