//! Toolbar item button builder.

use super::super::super::bview::{BAction, BView};
use super::super::super::state::BKey;
use crate::store::hosts::BHostType;
use crate::store::parts::BPartType;
use crate::store::parts::toolbar::ToolEntry;

pub struct ToolbarItem;

impl ToolbarItem {
    pub fn build(
        host: BHostType,
        tools: &[ToolEntry],
        current_tool: Option<&str>,
        exp: bool,
    ) -> Vec<BView> {
        let mut btns: Vec<BView> = tools
            .iter()
            .map(|t| {
                let active = current_tool == Some(t.idx) || (current_tool.is_none() && t.active);
                let btn = if exp {
                    BView::button().with_icon(t.icon).with_label(t.label)
                } else {
                    BView::button().with_icon(t.icon).with_tooltip(t.label)
                };
                btn.with_action(BAction::Set(BKey::CtxTool, t.idx.to_string()))
                    .with_active(active)
            })
            .collect();

        btns.push(
            BView::button()
                .with_icon(if exp { "chevron_left" } else { "chevron_right" })
                .with_tooltip(if exp { "Collapse" } else { "Expand" })
                .with_action(BAction::Toggle(BKey::Expanded {
                    host,
                    part: BPartType::Toolbar,
                })),
        );

        btns
    }
}
