//! Context sync - methods that affect other state through context.
//!
//! These methods bridge between user intent (switch host, switch space,
//! change mode) and the resulting context updates.

use super::{BContext, BMode};
use crate::store::hosts::BHostType;
use crate::store::spaces::BSpaceType;
use crate::store::zones::BZoneType;

impl BContext {
    pub fn set_active_host(&mut self, host: BHostType, zone: BZoneType) {
        self.active_host = host;
        self.active_zone = zone;
        self.mode = BMode::for_host(host);
    }

    pub fn set_space(&mut self, space: BSpaceType) {
        self.active_space = space;
    }

    pub fn set_mode(&mut self, mode: BMode) {
        self.mode = mode;
    }
}
