//! Zone specs - grid positions for areas in the workspace layout.
//!
//! Zones are named grid slots: Top, Left, Right, Center, Bottom, Full.
//! Each area in a workspace is assigned to a zone. The grid system
//! uses fractions to subdivide within a zone.
//!
//! Toggleable zones (Left, Right, Bottom) have hardcoded icon + tooltip
//! data in BZoneSpec. The space view shows togglers dynamically based
//! on which zones the active space contains.

pub mod def;
pub mod set;
pub use def::BZoneType;

use crate::store::spec::{BSpec, BSpecType};

impl BZoneType {
    pub fn label(self) -> &'static str {
        set::LABELS[self as usize]
    }

    pub fn from_name(name: &str) -> Option<BZoneType> {
        set::ALL_ZONES
            .iter()
            .find(|&&z| format!("{:?}", z) == name)
            .copied()
    }

    pub fn icon(self) -> &'static str {
        self.with_set().icon
    }

    pub fn tooltip(self) -> &'static str {
        self.with_set().tooltip
    }

    pub fn is_toggleable(self) -> bool {
        self.with_set().toggleable
    }

    pub fn toggleable_zones() -> &'static [BZoneType] {
        set::TOGGLEABLE
    }

    pub fn all() -> &'static [BZoneType] {
        set::ALL_ZONES
    }

    /// Get the full spec for this zone.
    pub fn items(self) -> &'static BZoneSpec {
        &set::ALL_SPECS[self as usize]
    }

    pub fn with_set(self) -> &'static BZoneSpec {
        &set::ALL_SPECS[self as usize]
    }
}

/// Static spec for a zone — hardcoded toggle data drives the UI.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BZoneSpec {
    pub idx: BZoneType,
    /// Icon name for the zone toggler button (empty if not toggleable).
    pub icon: &'static str,
    /// Tooltip text for the zone toggler button (empty if not toggleable).
    pub tooltip: &'static str,
    /// Whether this zone can be toggled on/off in the space view.
    pub toggleable: bool,
    /// Default width/height fraction for this zone.
    pub fraction: f32,
}

impl BSpec for BZoneSpec {
    fn idx(&self) -> BSpecType {
        BSpecType::BZone(self.idx)
    }
    fn items(&self) -> Vec<&dyn BSpec> {
        vec![]
    }
}
