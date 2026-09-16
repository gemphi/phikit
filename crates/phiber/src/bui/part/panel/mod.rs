//! Panel view — vertical tab bar with Card accordion content.

pub mod accordion;
pub mod tabs;

pub use accordion::PanelAccordion;
pub use tabs::PanelTabs;

use super::super::bview::{BView, CardShade};
use super::super::theme::BPalette;
use super::super::{BKey, BState};
use crate::store::hosts::BHostType;
use crate::store::parts::BPartType;
use crate::store::parts::panel;

pub struct PanelView<'a> {
    pub host: BHostType,
    pub state: &'a BState,
}

impl<'a> PanelView<'a> {
    pub fn new(host: BHostType, state: &'a BState) -> Self {
        Self { host, state }
    }

    pub fn build(self) -> BView {
        let host = self.host;
        let state = self.state;
        let sections = panel::set::with_idx(host);
        if sections.is_empty() {
            return BView::label("No properties");
        }

        let expanded = state.get(&BKey::PanelExpanded) == Some("true");
        let p = BPalette::from_state(state);
        let active_tab = state
            .get(&BKey::Tab {
                host,
                part: BPartType::Panel,
            })
            .unwrap_or(sections[0].idx);

        let vtab_bar = PanelTabs::build(host, sections, active_tab, expanded, &p);
        let accordion_content = PanelAccordion::build(state, sections, active_tab);

        let right_card = BView::card()
            .with_shade(CardShade::Panel)
            .with_body(accordion_content);

        BView::row(None, vec![vtab_bar, right_card])
    }
}
