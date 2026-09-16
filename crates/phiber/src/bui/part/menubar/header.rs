//! Editor area header menus builder (View, Select, Add, Object, etc.).

use super::super::super::bview::{BAction, BView};
use super::super::super::state::{BKey, BState};
use super::super::super::view::menu::MenuView;
use crate::store::hosts::BHostType;
use crate::store::menus::header;
use crate::store::parts::BPartType;

pub struct HeaderMenus;

impl HeaderMenus {
    pub fn build(state: &BState, host: BHostType, part: BPartType) -> Vec<BView> {
        let menus = header::items_for(host);
        let any_open = menus.iter().any(|m| {
            let label = m.label.unwrap_or("");
            let dk = BKey::Dropdown {
                host,
                part,
                key: format!("::{}", label),
            };
            state.get(&dk) == Some("true")
        });

        menus
            .iter()
            .map(|m| {
                let label = m.label.unwrap_or("");
                let dk = BKey::Dropdown {
                    host,
                    part,
                    key: format!("::{}", label),
                };
                let is_open = state.get(&dk) == Some("true");
                let ch: Vec<BView> = m
                    .items
                    .unwrap_or(&[])
                    .iter()
                    .map(|i| MenuView::item(i, state, host, part, label))
                    .collect();
                let mut dd = BView::dropdown_action(
                    label,
                    m.icon.map(|s| s.to_string()),
                    ch,
                    is_open,
                    BAction::Toggle(dk.clone()),
                );
                if any_open {
                    dd = dd.with_hover_action(BAction::Open(dk));
                }
                dd
            })
            .collect()
    }
}
