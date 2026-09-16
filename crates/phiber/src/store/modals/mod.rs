//! Modal specs - typed definitions for help, profile, and settings modals.
//!
//! Modal state in `state/modal/` references these enums and structs.

pub mod def;
pub mod help;
pub mod profile;
pub mod set;
pub mod settings;
pub use def::BModalType;

use crate::store::spec::{BSpec, BSpecType};

pub use help::BHelpType;
pub use profile::BProfileType;
pub use settings::BSettingsType;

/// Identifies which modal is open.

impl BModalType {
    pub fn label(self) -> &'static str {
        set::LABELS[self as usize]
    }

    pub fn name(self) -> &'static str {
        set::LABELS[self as usize]
    }

    pub fn from_name(name: &str) -> Option<BModalType> {
        set::ALL.iter().find(|&&m| m.name() == name).copied()
    }

    pub fn icon(self) -> &'static str {
        set::ICONS[self as usize]
    }

    pub fn width(self) -> f32 {
        set::WIDTHS[self as usize]
    }

    pub fn all() -> &'static [BModalType] {
        set::ALL
    }

    /// Get the full spec for this modal.
    pub fn items(self) -> &'static BModalSpec {
        &set::ALL_SPECS[self as usize]
    }

    pub fn with_set(self) -> &'static BModalSpec {
        &set::ALL_SPECS[self as usize]
    }
}

/// Static spec for a modal.
///
/// Modals are leaves in the hierarchy: only `idx` and an empty `items` set.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BModalSpec {
    pub idx: BModalType,
    /// Child set — always empty for modals.
    pub items: &'static [BModalSpec],
}

impl BSpec for BModalSpec {
    fn idx(&self) -> BSpecType {
        BSpecType::BModal(self.idx)
    }
    fn items(&self) -> Vec<&dyn BSpec> {
        self.items.iter().map(|m| m as &dyn BSpec).collect()
    }
}

impl BModalSpec {
    /// Modals are leaves in the type hierarchy.
    pub fn items(&self) -> &'static [BModalSpec] {
        self.items
    }
}
