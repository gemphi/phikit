//! Actions — the store writes every primitive dispatches. A `BAction` is a
//! pure value: the path to write and the write semantics. No closures, no
//! callbacks — the view layer stays declarative and state-driven.

use crate::v2::state::BPath;
use crate::v2::view::BView;
use crate::v2::zone::BZoneType;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BAction {
    Set(BPath, String),
    Toggle(BPath),
    Open(BPath),
    /// Remove an override so subsequent reads use their declared fallback.
    Reset(BPath),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BZoneToggle {
    pub zone: BZoneType,
    pub visible: bool,
    pub action: BAction,
}

impl BView {
    pub fn zone_toggle_group(controls: Vec<BZoneToggle>) -> Self {
        Self::ZoneToggleGroup { controls }
    }
}
