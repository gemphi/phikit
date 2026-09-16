//! Host specs - editor host types and their specs.
//!
//! A host ONLY owns its regions. Regions own panes, panes own parts.
//!
//! File layout:
//!   mod.rs - BHostType enum, BHostSpec struct, BHostSet trait, BSpec impl
//!   set.rs - HOST_SET HashMap + host_spec() + all_host_specs()
//!   def.rs - static host data sets and shared region constants
//!   view3d/  - view3d host constants (REGIONS)
//!   uv_editor/ - uv_editor host constants

pub mod def;
pub mod set;

pub mod asset_browser;
pub mod clip_editor;
pub mod compositor;
pub mod console;
pub mod dope_sheet;
pub mod drivers;
pub mod empty;
pub mod file_browser;
pub mod geo_nodes;
pub mod graph_editor;
pub mod image_editor;
pub mod info;
pub mod logic_editor;
pub mod nla;
pub mod node_editor;
pub mod outliner;
pub mod prefs;
pub mod properties;
pub mod sequencer;
pub mod shader_editor;
pub mod spreadsheet;
pub mod statusbar;
pub mod text_editor;
pub mod timeline;
pub mod topbar;
pub mod uv_editor;
pub mod view3d;
pub use def::BHostType;

pub use set::SWITCHABLE_HOSTS;
pub use set::host_spec;

use crate::store::regions::BRegionSpec;
use crate::store::spec::{BSpec, BSpecType};

/// Editor host types - short Blender-like names.

impl BHostType {
    pub fn label(self) -> &'static str {
        set::host_label(self)
    }

    pub fn from_name(name: &str) -> Option<BHostType> {
        set::ALL_HOSTS
            .iter()
            .find(|&&h| format!("{:?}", h) == name)
            .copied()
    }

    pub fn items(self) -> &'static BHostSpec {
        set::host_spec(self)
    }

    pub fn with_set(self) -> &'static BHostSpec {
        set::host_spec(self)
    }
}

/// Implemented by every host spec to expose its type and child items.
pub trait BHostDef {
    fn idx(&self) -> BHostType;
    fn items(&self) -> &'static [BRegionSpec];
}

/// Static spec for a host.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BHostSpec {
    pub idx: BHostType,
    pub items: &'static [BRegionSpec],
}

impl BHostSpec {
    pub const fn new(idx: BHostType, items: &'static [BRegionSpec]) -> Self {
        Self { idx, items }
    }

    /// Label comes from the host set association, not from the spec itself.
    pub fn label(&self) -> &'static str {
        set::host_label(self.idx)
    }

    /// Icon comes from the host set association.
    pub fn icon(&self) -> &'static str {
        set::host_icon(self.idx)
    }

    /// Return the child region set for simple hierarchy loops.
    pub fn items(&self) -> &'static [BRegionSpec] {
        self.items
    }

    pub fn with_set(&self) -> &'static [BRegionSpec] {
        self.items
    }
}

impl BSpec for BHostSpec {
    fn idx(&self) -> BSpecType {
        BSpecType::BHost(self.idx)
    }
    fn items(&self) -> Vec<&dyn BSpec> {
        self.items.iter().map(|r| r as &dyn BSpec).collect()
    }
}

impl BHostDef for BHostSpec {
    fn idx(&self) -> BHostType {
        self.idx
    }
    fn items(&self) -> &'static [BRegionSpec] {
        self.items
    }
}
