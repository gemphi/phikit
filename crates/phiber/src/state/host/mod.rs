//! Host state - per-host runtime state.
//!
//! Tracks region visibility and active region per host instance.
//! Uses the new 3-region model: Header, Center, Footer.
//!
//! Per-host-type state (View3DState, OutlinerState, etc.) and
//! per-host-type effects (view functions) will live in sub-modules.

pub mod ctx;
pub mod def;
pub mod r#use;

use std::collections::HashMap;

use crate::store::hosts::BHostType;
use crate::store::regions::BRegionType;

/// Trait every host state must implement.
pub trait BHostState {
    fn host_type(&self) -> BHostType;
    fn active_region(&self) -> BRegionType;
    fn is_region_visible(&self, rtype: BRegionType) -> bool;
    fn toggle_region(&mut self, rtype: BRegionType);
    fn set_region_visible(&mut self, rtype: BRegionType, visible: bool);
    fn set_host(&mut self, host: BHostType);
    fn set_active_region(&mut self, rtype: BRegionType);
}

/// Runtime state for a single host instance.
#[derive(Clone, Debug)]
pub struct HostState {
    pub host_type: BHostType,
    /// Which regions are visible, keyed by `BRegionType` (Header, Center, Footer).
    pub region_visible: HashMap<BRegionType, bool>,
    /// Active region (which region has focus).
    pub active_region: BRegionType,
}
