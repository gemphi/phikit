//! Host view — renders regions within a host. Corresponds to store/hosts/.
//!
//! The host is a dumb pass-through: it loops its region specs and builds each.
//! Per-host center content is static store data (`store/parts/custom`),
//! rendered by the `CustomView` leaf via `center_body`.

use super::super::bview::BView;
use super::super::state::BState;
use super::BViewBuilder;
use super::region::RegionView;
use crate::bui::part::CustomView;
use crate::store::BPanelRole;
use crate::store::hosts::BHostType;

/// A single editor host (View3D, Outliner, …).
#[derive(Clone, Copy, Debug)]
pub struct HostView {
    pub host: BHostType,
}

impl HostView {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewBuilder for HostView {
    fn build(&self, state: &BState) -> BView {
        // Dumb pass-through: loop the host spec's regions, build each.
        // Region-type behavior lives in RegionView; part rendering in PartView.
        let children: Vec<BView> = self
            .host
            .with_set()
            .with_set()
            .iter()
            .map(|r| RegionView::new(self.host, r.idx).build(state))
            .filter(|v| !matches!(v, BView::Empty))
            .collect();
        BView::panel(BPanelRole::Host, "", children)
    }
}

/// Host-specific center content — the body of the `Custom`/viewport part.
/// `CustomView` loops the store's static content (`store/parts/custom`).
pub fn center_body(host: BHostType, state: &BState) -> BView {
    CustomView::new(host, state).build()
}
