//! Area view — renders the active host within an area. Corresponds to store/areas/.
//!
//! The area-switch seam: resolves the active host from `BKey::AreaHost(default)`,
//! falling back to the area's `default_host`.

use super::super::bview::BView;
use super::super::state::{BKey, BState};
use super::BViewBuilder;
use super::host::HostView;
use crate::store::areas::BAreaSpec;
use crate::store::hosts::BHostType;

/// A single area holding its active host.
#[derive(Clone, Copy, Debug)]
pub struct AreaView {
    pub area: BAreaSpec,
    pub height: Option<f32>,
}

impl AreaView {
    pub fn new(area: BAreaSpec) -> Self {
        Self { area, height: None }
    }

    pub fn with_height(mut self, h: Option<f32>) -> Self {
        self.height = h;
        self
    }
}

impl BViewBuilder for AreaView {
    fn build(&self, state: &BState) -> BView {
        let host = state
            .get(&BKey::AreaHost(self.area.default_host))
            .and_then(BHostType::from_name)
            .unwrap_or(self.area.default_host);

        let host_view = HostView::new(host).build(state);
        BView::panel_sized(
            self.area.idx.panel_role(),
            "",
            vec![host_view],
            None,
            self.height,
        )
    }
}
