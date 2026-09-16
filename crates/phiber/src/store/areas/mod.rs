//! BAreaSpec specs - slots at zone positions that hold hosts.
//!
//! File layout:
//!   mod.rs - BAreaType, BAreaSpec, BSpec impl
//!   def.rs - static area assignments
//!   set.rs - static spec arrays and labels

pub mod def;
pub mod set;
pub use def::BAreaType;

use crate::store::hosts::{BHostType, host_spec};
use crate::store::spec::{BSpec, BSpecType};
use crate::store::zones::BZoneType;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BAreaSpec {
    pub idx: BAreaType,
    pub zone: BZoneType,
    pub default_host: BHostType,
    pub items: &'static [BHostType],
}

impl BSpec for BAreaSpec {
    fn idx(&self) -> BSpecType {
        BSpecType::BArea(self.idx)
    }

    fn items(&self) -> Vec<&dyn BSpec> {
        self.items
            .iter()
            .map(|&h| host_spec(h) as &dyn BSpec)
            .collect()
    }
}

impl BAreaSpec {
    /// Return the host set for this area.
    pub fn items(&self) -> &'static [BHostType] {
        self.items
    }
}

impl BAreaType {
    pub fn label(self) -> &'static str {
        set::AREA_LABELS[self as usize].0
    }

    pub fn all() -> &'static [BAreaType] {
        set::ALL_AREAS
    }

    /// Get the full spec for this area, including its host set.
    pub fn items(self) -> &'static BAreaSpec {
        &set::ALL_SPECS[self as usize]
    }

    pub fn with_set(self) -> &'static BAreaSpec {
        &set::ALL_SPECS[self as usize]
    }
}
