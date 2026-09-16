//! Default host state - initial region visibility per host type.

use std::collections::HashMap;

use super::{BHostState, HostState};
use crate::store::hosts::BHostType;
use crate::store::regions::BRegionType;

impl HostState {
    pub fn new(host_type: BHostType) -> Self {
        let mut region_visible = HashMap::new();
        region_visible.insert(BRegionType::Header, true);
        region_visible.insert(BRegionType::Center, true);
        region_visible.insert(BRegionType::Footer, false);

        Self {
            host_type,
            region_visible,
            active_region: BRegionType::Center,
        }
    }
}

impl Default for HostState {
    fn default() -> Self {
        Self::new(BHostType::View3D)
    }
}

impl BHostState for HostState {
    fn host_type(&self) -> BHostType {
        self.host_type
    }
    fn active_region(&self) -> BRegionType {
        self.active_region
    }
    fn is_region_visible(&self, rtype: BRegionType) -> bool {
        *self.region_visible.get(&rtype).unwrap_or(&false)
    }
    fn toggle_region(&mut self, rtype: BRegionType) {
        let current = self.is_region_visible(rtype);
        self.region_visible.insert(rtype, !current);
    }
    fn set_region_visible(&mut self, rtype: BRegionType, visible: bool) {
        self.region_visible.insert(rtype, visible);
    }
    fn set_host(&mut self, host: BHostType) {
        self.host_type = host;
    }
    fn set_active_region(&mut self, rtype: BRegionType) {
        self.active_region = rtype;
    }
}

/// Create a host state with default region visibility for the given host type.
pub fn default_host_state(host: BHostType) -> HostState {
    let mut state = HostState::new(host);
    match host {
        BHostType::Time | BHostType::Seq | BHostType::Action | BHostType::Nla => {
            state.set_region_visible(BRegionType::Footer, true);
        }
        _ => {}
    }
    state
}
