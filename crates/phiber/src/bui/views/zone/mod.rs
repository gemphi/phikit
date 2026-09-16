//! Zone view — a grid slot. Corresponds to store/zones/.
//!
//! Zones render themselves: each zone owns its visibility rule, loops over its
//! area specs from the store, and matches on its own type to pick a container.
//! No sizing logic lives here — leaf panels own metrics (theme palette).
//! `SpaceView` only composes the built zone nodes.

use super::super::bview::BView;
use super::super::state::{BKey, BState};
use super::super::theme::BPalette;
use super::BViewBuilder;
use super::area::AreaView;
use crate::store::BPanelRole;
use crate::store::areas::BAreaSpec;
use crate::store::hosts::BHostType;
use crate::store::zones::BZoneType;

/// A single zone slot (Top/Left/Right/Center/Bottom/Full) with its areas.
#[derive(Clone, Debug)]
pub struct ZoneView {
    pub zone: BZoneType,
    pub areas: Vec<BAreaSpec>,
    /// Space-level trigger chrome (zone togglers) rendered inside the Top strip.
    pub chrome: Option<BView>,
}

impl ZoneView {
    pub fn new(zone: BZoneType) -> Self {
        Self {
            zone,
            areas: Vec::new(),
            chrome: None,
        }
    }

    pub fn with_areas(mut self, areas: Vec<BAreaSpec>) -> Self {
        self.areas = areas;
        self
    }

    pub fn with_chrome(mut self, chrome: BView) -> Self {
        self.chrome = Some(chrome);
        self
    }

    /// Whether this zone is currently visible (toggleable zones read `BKey::Zone`).
    pub fn is_visible(&self, state: &BState) -> bool {
        if !self.zone.is_toggleable() {
            return true;
        }
        match state.get(&BKey::Zone(self.zone)) {
            Some("true") => true,
            Some("false") => false,
            _ => true,
        }
    }

    /// Loop the zone's areas through `AreaView`, separated by plain separators.
    fn area_views(&self, state: &BState) -> Vec<BView> {
        let mut out: Vec<BView> = Vec::new();
        for (i, a) in self.areas.iter().enumerate() {
            if i > 0 {
                out.push(BView::separator());
            }
            out.push(AreaView::new(*a).build(state));
        }
        out
    }
}

impl BViewBuilder for ZoneView {
    fn build(&self, state: &BState) -> BView {
        if self.areas.is_empty() || !self.is_visible(state) {
            return BView::empty();
        }
        // No sizing logic here: loop the store's area specs and match the zone
        // type for its container. Leaf panels own metrics (theme palette).
        match self.zone {
            // Top strip: horizontal area row + space chrome; separator closes it.
            BZoneType::Top => {
                let mut inner = self.area_views(state);
                if let Some(chrome) = &self.chrome {
                    inner.push(BView::vseparator());
                    inner.push(chrome.clone());
                }
                BView::column(
                    None,
                    vec![
                        BView::panel(BPanelRole::TopArea, "", inner),
                        BView::separator(),
                    ],
                )
            }
            BZoneType::Left => BView::panel(BPanelRole::LeftArea, "", self.area_views(state)),
            BZoneType::Center | BZoneType::Full => {
                BView::panel(BPanelRole::CenterArea, "", self.area_views(state))
            }
            BZoneType::Right => BView::panel(BPanelRole::RightColumn, "", self.area_views(state)),
            // Bottom strips stack full-width; the global Statusbar is footer-height.
            BZoneType::Bottom => BView::column(
                None,
                self.areas
                    .iter()
                    .map(|a| {
                        let view = AreaView::new(*a);
                        if a.default_host == BHostType::Statusbar {
                            view.with_height(Some(BPalette::from_state(state).h_footer))
                                .build(state)
                        } else {
                            view.build(state)
                        }
                    })
                    .collect(),
            ),
        }
    }
}
