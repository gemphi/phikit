//! Topbar menus builder (File, Edit, Render, Window, Help).

use super::super::super::bview::{BAction, BView};
use super::super::super::state::{BKey, BState};
use super::super::super::view::menu::MenuView;
use crate::store::hosts::BHostType;
use crate::store::menus::BMenu;
use crate::store::parts::BPartType;

pub struct TopbarMenus;

impl TopbarMenus {
    pub fn build(state: &BState, host: BHostType, part: BPartType) -> Vec<BView> {
        let menus: Vec<(BMenu, BKey)> = (0..5)
            .filter_map(BMenu::from_idx)
            .map(|m| {
                let l = m.label();
                let dk = BKey::Dropdown {
                    host,
                    part,
                    key: l.to_string(),
                };
                (m, dk)
            })
            .collect();

        let any_open = menus.iter().any(|(_, dk)| state.get(dk) == Some("true"));

        menus
            .into_iter()
            .map(|(m, dk)| {
                let l = m.label();
                let ch: Vec<BView> = m
                    .spec()
                    .items
                    .iter()
                    .map(|i| MenuView::item(i, state, host, part, &l))
                    .collect();
                let is_open = state.get(&dk) == Some("true");
                let mut dd =
                    BView::dropdown_action(l, None, ch, is_open, BAction::Toggle(dk.clone()));
                if any_open {
                    dd = dd.with_hover_action(BAction::Open(dk));
                }
                dd
            })
            .collect()
    }
}
