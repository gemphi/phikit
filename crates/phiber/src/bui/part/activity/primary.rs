//! Primary vertical action sections (Top, Center, Bottom) for Activity Bar.

use super::super::super::bview::{BAction, BView};
use super::super::super::state::BKey;

pub struct ActivityPrimary;

impl ActivityPrimary {
    pub fn top_section(active_space: &str, active_tab: &str, drawer_open: bool) -> BView {
        BView::column_center(
            Some(4.0),
            vec![
                BView::button()
                    .with_icon("home")
                    .with_tooltip("Home (Layout Space)")
                    .with_active(active_space == "Layout" && active_tab == "home")
                    .with_action(BAction::Set(BKey::CtxSpace, "Layout".to_string())),
                BView::button()
                    .with_icon("message_square")
                    .with_tooltip("Social Chat & Direct Messages")
                    .with_active(active_tab == "chat" && drawer_open)
                    .with_action(BAction::Toggle(BKey::ActivityDrawer)),
                BView::button()
                    .with_icon("grid")
                    .with_tooltip("Workspaces & Spaces")
                    .with_active(active_tab == "spaces" && drawer_open)
                    .with_action(BAction::Toggle(BKey::ActivityDrawer)),
            ],
        )
    }

    pub fn center_section(active_tab: &str) -> BView {
        BView::column_center(
            Some(4.0),
            vec![
                BView::button()
                    .with_icon("sparkles")
                    .with_tooltip("AI Stream & Collaborative Viewport")
                    .with_active(active_tab == "stream")
                    .with_action(BAction::Set(BKey::ActivityTab, "stream".to_string())),
            ],
        )
    }

    pub fn bottom_section() -> BView {
        BView::column_center(
            Some(4.0),
            vec![
                BView::button()
                    .with_icon("settings")
                    .with_tooltip("Theme Settings & Social Marketplace")
                    .with_action(BAction::Set(BKey::Modal, "Settings".to_string())),
                BView::button()
                    .with_icon("user")
                    .with_tooltip("User Profile & Discord Presence")
                    .with_action(BAction::Set(BKey::Modal, "Profile".to_string())),
            ],
        )
    }
}
