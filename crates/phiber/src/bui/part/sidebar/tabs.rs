//! Sidebar tab strip builder using fluent .with_* methods.

use super::super::super::bview::{BAction, BView};
use super::super::super::state::BKey;
use crate::store::hosts::BHostType;
use crate::store::parts::{BPartType, sidebar};

pub struct SidebarTabs;

impl SidebarTabs {
    pub fn build(host: BHostType, active: Option<&str>, exp: bool) -> Vec<BView> {
        let mut tabs: Vec<BView> = sidebar::set::with_idx(host)
            .iter()
            .map(|t| {
                let is_active = active == Some(t.idx);
                let action = BAction::Set(
                    BKey::Tab {
                        host,
                        part: BPartType::Sidebar,
                    },
                    t.idx.to_string(),
                );
                BView::tab(t.label)
                    .with_icon(t.icon)
                    .with_icon_only(!exp)
                    .with_active(is_active)
                    .with_action(action)
            })
            .collect();

        tabs.push(
            BView::button()
                .with_icon(if exp { "chevron_left" } else { "chevron_right" })
                .with_tooltip(if exp { "Collapse" } else { "Expand" })
                .with_action(BAction::Toggle(BKey::Expanded {
                    host,
                    part: BPartType::Sidebar,
                })),
        );

        tabs
    }
}
