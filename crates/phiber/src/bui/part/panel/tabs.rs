//! Vertical Tab Strip for N-Panel using fluent .with_* methods.

use super::super::super::bview::{BAction, BView};
use super::super::super::state::BKey;
use super::super::super::theme::BPalette;
use crate::store::hosts::BHostType;
use crate::store::parts::BPartType;
use crate::store::parts::panel::PanelSection;

pub struct PanelTabs;

impl PanelTabs {
    pub fn build(
        host: BHostType,
        sections: &[PanelSection],
        active_tab: &str,
        expanded: bool,
        p: &BPalette,
    ) -> BView {
        let tab_width: Option<f32> = if expanded {
            Some(p.w_channels)
        } else {
            Some(p.w_toolbar)
        };

        let mut tabs: Vec<BView> = sections
            .iter()
            .map(|s| {
                let is_active = s.idx == active_tab;
                let action = BAction::Set(
                    BKey::Tab {
                        host,
                        part: BPartType::Panel,
                    },
                    s.idx.to_string(),
                );
                BView::tab(s.label)
                    .with_icon(s.icon)
                    .with_icon_only(!expanded)
                    .with_active(is_active)
                    .with_action(action)
            })
            .collect();

        tabs.push(
            BView::button()
                .with_icon(if expanded {
                    "chevrons-left"
                } else {
                    "chevrons-right"
                })
                .with_action(BAction::Toggle(BKey::PanelExpanded)),
        );

        BView::vtab_bar_sized(tab_width, tabs)
    }
}
