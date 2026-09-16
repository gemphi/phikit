//! Region view — renders the panes within a host region. Corresponds to store/regions/.

use super::super::bview::BView;
use super::super::state::BState;
use super::BViewBuilder;
use super::pane::{PaneView, PartContainer};
use crate::store::BPanelRole;
use crate::store::hosts::BHostType;
use crate::store::parts::{BPart, BPartType};
use crate::store::regions::BRegionType;

/// A single region (Header/Center/Footer/…) of a host.
#[derive(Clone, Copy, Debug)]
pub struct RegionView {
    pub host: BHostType,
    pub region: BRegionType,
}

impl RegionView {
    pub fn new(host: BHostType, region: BRegionType) -> Self {
        Self { host, region }
    }
}

impl BViewBuilder for RegionView {
    fn build(&self, state: &BState) -> BView {
        let host_spec = self.host.with_set();
        let Some(region_spec) = host_spec.with_set().iter().find(|r| r.idx == self.region) else {
            return BView::Empty;
        };
        let panes = region_spec.items;
        if panes.is_empty() {
            return BView::Empty;
        }

        match self.region {
            // Work region (Blender RGN_TYPE_WINDOW): the viewport fills 100% in
            // flow; Toolbar/Sidebar are absolute overlays painted LAST so they
            // sit on top of the viewport (HTML flex + absolute model).
            BRegionType::Center | BRegionType::Window => {
                let mut parts: Vec<&BPart> =
                    panes.iter().flat_map(|pane| pane.items.iter()).collect();
                parts.sort_by_key(|p| matches!(p.idx, BPartType::Toolbar | BPartType::Sidebar));
                let views: Vec<BView> = parts
                    .into_iter()
                    .map(|part| PartContainer::new(*part, self.host).build(state))
                    .collect();
                if views.is_empty() {
                    BView::Empty
                } else {
                    BView::panel(BPanelRole::Work, "", views)
                }
            }
            // Strip regions (Header/Footer/…): panes are left/center/right
            // segments inside the region's own styled strip.
            _ => {
                let children: Vec<BView> = panes
                    .iter()
                    .map(|p| PaneView::new(*p, self.host).build(state))
                    .collect();
                if children.is_empty() {
                    BView::Empty
                } else {
                    BView::panel(self.region.panel_role(), "", children)
                }
            }
        }
    }
}
