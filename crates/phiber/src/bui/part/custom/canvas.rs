//! Custom viewport canvas mapper.

use super::super::super::bview::BView;
use crate::store::BPanelRole;
use crate::store::hosts::BHostType;

pub struct CustomCanvas;

impl CustomCanvas {
    pub fn build(host: BHostType) -> Vec<BView> {
        let host_spec = host.items();
        // Viewport center is empty — host name shows in header next to switcher.
        vec![BView::panel(
            BPanelRole::Viewport,
            host_spec.label().to_string(),
            vec![BView::empty()],
        )]
    }
}
