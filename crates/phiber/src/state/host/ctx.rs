//! Host context sync - syncs host state with BContext.
//!
//! When the active host changes, the context's active_host and mode
//! are updated. When a region is toggled, the context's active_region
//! may change if the active region becomes hidden.

use super::{BHostState, HostState};
use crate::state::context::BContext;
use crate::store::regions::BRegionType;

/// Sync context when host type changes.
pub fn sync_host_change(ctx: &mut BContext, state: &HostState) {
    ctx.set_active_host(state.host_type(), ctx.active_zone);
}

/// Sync context when region visibility changes.
/// If the active region becomes hidden, fall back to Center.
pub fn sync_region_change(ctx: &mut BContext, state: &HostState) {
    if !state.is_region_visible(ctx.active_region) {
        ctx.active_region = state.active_region();
    }
}

/// Check if a region should be rendered for the current host.
pub fn should_render_region(state: &HostState, rtype: BRegionType) -> bool {
    state.is_region_visible(rtype)
}
