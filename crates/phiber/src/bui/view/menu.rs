//! MenuView — shared helper for converting MenuItem → BView.
//! Used by menubar, sidebar, switcher views for recursive menu item rendering.
//! Submenus, checkboxes, and radio items are handled here.

use super::super::bview::{BAction, BView};
use crate::bui::{BKey, BState};
use crate::store::hosts::BHostType;
use crate::store::menus::{MenuItem, MenuItemKind};
use crate::store::parts::BPartType;

pub struct MenuView;

impl MenuView {
    /// Recursively convert MenuItem → BView (submenus, checkboxes, radio).
    pub fn item(
        mi: &MenuItem,
        state: &BState,
        host: BHostType,
        part: BPartType,
        prefix: &str,
    ) -> BView {
        match mi.kind {
            MenuItemKind::Separator => BView::separator(),
            MenuItemKind::Check | MenuItemKind::Radio => {
                let label = mi.label.unwrap_or("");
                let mk = BKey::MenuItem {
                    host,
                    part,
                    label: label.to_string(),
                };
                let checked = state.get(&mk) == Some("true") || mi.checked;
                let icon = match (mi.kind, checked) {
                    (MenuItemKind::Check, true) => "check_square",
                    (MenuItemKind::Check, false) => "square",
                    (MenuItemKind::Radio, true) => "circle",
                    (MenuItemKind::Radio, false) => "circle_off",
                    _ => "square",
                };
                BView::button()
                    .with_icon(icon)
                    .with_label(label)
                    .with_active(checked)
                    .with_action(BAction::Toggle(mk))
            }
            MenuItemKind::Label => {
                let label = mi.label.unwrap_or("");
                let icon = mi.icon.map(|s| s.to_string());
                if let Some(sub) = mi.items {
                    let full_key = if prefix.is_empty() {
                        label.to_string()
                    } else {
                        format!("{}::{}", prefix, label)
                    };
                    let dk = BKey::Dropdown {
                        host,
                        part,
                        key: full_key.clone(),
                    };
                    let ch: Vec<BView> = sub
                        .iter()
                        .map(|i| Self::item(i, state, host, part, &full_key))
                        .collect();
                    BView::dropdown_action(
                        label,
                        icon,
                        ch,
                        state.get(&dk) == Some("true"),
                        BAction::Toggle(dk),
                    )
                } else {
                    // Leaf label — check for known action labels.
                    match label {
                        "Open..." => BView::button()
                            .with_icon(icon.as_deref().unwrap_or("folder-open"))
                            .with_label(label)
                            .with_action(BAction::Set(BKey::Modal, "File Browser".to_string())),
                        "Preferences..." => BView::button()
                            .with_icon(icon.as_deref().unwrap_or("settings"))
                            .with_label(label)
                            .with_action(BAction::Set(BKey::Modal, "Settings".to_string())),
                        _ => match icon {
                            Some(i) if !i.is_empty() => BView::label_icon(label, i),
                            _ => BView::label(label),
                        },
                    }
                }
            }
        }
    }
}
