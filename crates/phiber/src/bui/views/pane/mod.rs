//! Pane view — maps parts to part views. Corresponds to store/panes/.
//! `PartContainer` sizes Toolbar/Sidebar; `PaneView` wraps a pane's parts.

use super::super::bview::BView;
use super::super::state::{BKey, BState};
use super::super::theme::BPalette;
use super::BViewBuilder;
use super::part::PartView;
use crate::store::hosts::BHostType;
use crate::store::panes::BPaneSpec;
use crate::store::parts::{BPart, BPartType};

/// A single part container with optional width constraints.
#[derive(Clone, Copy, Debug)]
pub struct PartContainer {
    pub part: BPart,
    pub host: BHostType,
}

impl PartContainer {
    pub fn new(part: BPart, host: BHostType) -> Self {
        Self { part, host }
    }
}

impl BViewBuilder for PartContainer {
    fn build(&self, state: &BState) -> BView {
        let item = PartView::new(self.part.idx, self.host).build(state);
        let p = BPalette::from_state(state);
        let (w, h) = match self.part.idx {
            BPartType::Toolbar => {
                let width = if state.get(&BKey::Expanded {
                    host: self.host,
                    part: BPartType::Toolbar,
                }) == Some("true")
                {
                    p.w_panel
                } else {
                    p.w_toolbar
                };
                (Some(width), None)
            }
            BPartType::Sidebar => {
                let width = if state.get(&BKey::Expanded {
                    host: self.host,
                    part: BPartType::Sidebar,
                }) == Some("true")
                {
                    p.w_panel
                } else {
                    p.w_sidebar
                };
                (Some(width), None)
            }
            _ => (None, None),
        };

        if w.is_some() || h.is_some() {
            BView::panel_sized(self.part.idx.panel_role(), "", vec![item], w, h)
        } else {
            BView::panel(self.part.idx.panel_role(), "", vec![item])
        }
    }
}

/// A pane wrapping its parts.
#[derive(Clone, Copy, Debug)]
pub struct PaneView {
    pub pane: BPaneSpec,
    pub host: BHostType,
}

impl PaneView {
    pub fn new(pane: BPaneSpec, host: BHostType) -> Self {
        Self { pane, host }
    }
}

impl BViewBuilder for PaneView {
    fn build(&self, state: &BState) -> BView {
        if self.pane.items.is_empty() {
            return BView::Empty;
        }

        let part_views: Vec<BView> = self
            .pane
            .items
            .iter()
            .map(|p| PartContainer::new(*p, self.host).build(state))
            .collect();

        let children = if part_views.len() == 1 {
            part_views
        } else {
            let p = BPalette::from_state(state);
            vec![BView::row(Some(p.gap_xs), part_views)]
        };

        BView::panel(self.pane.idx.panel_role(), "", children)
    }
}
