//! Host state set methods - pure state mutations.
//!
//! Convenience helpers that wrap the `BHostState` trait.

use super::{BHostState, HostState};
use crate::store::regions::BRegionType;

impl HostState {
    /// Show a region.
    pub fn show_region(&mut self, rtype: BRegionType) {
        self.set_region_visible(rtype, true);
    }

    /// Hide a region.
    pub fn hide_region(&mut self, rtype: BRegionType) {
        self.set_region_visible(rtype, false);
    }
}
