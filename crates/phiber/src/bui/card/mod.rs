//! Universal Card module — composable Card, Header, Body, Footer, List, and Item.

pub mod body;
pub mod footer;
pub mod header;
pub mod item;
pub mod list;

pub use body::CardBodyView;
pub use footer::CardFooterView;
pub use header::CardHeaderView;
pub use item::CardItemView;
pub use list::CardListView;

use super::bview::{ActionHandler, BAction, BView, CardShade};
use super::theme::BPalette;
use gpui::prelude::*;
use gpui::{AnyElement, IntoElement, div, px};

pub struct CardView {
    pub shade: CardShade,
    pub header: Option<String>,
    pub icon: Option<String>,
    pub badge: Option<String>,
    pub tool: Option<Box<BView>>,
    pub collapsible: bool,
    pub collapsed: bool,
    pub collapse_action: Option<BAction>,
    pub body: Option<Box<BView>>,
    pub footer: Option<String>,
    pub footer_action: Option<Box<BView>>,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl CardView {
    pub fn into_elm(self, p: &BPalette, on_action: ActionHandler) -> AnyElement {
        let bg_color = match self.shade {
            CardShade::App => p.bg_app,
            CardShade::Activity => p.bg_toolbar,
            CardShade::Panel => p.bg_panel,
            CardShade::Header => p.bg_header,
            CardShade::Viewport => p.bg_viewport,
            CardShade::Accent => p.accent,
        };

        let mut card = div()
            .flex()
            .flex_col()
            .bg(bg_color)
            .rounded(px(p.radius_sm))
            .border_1()
            .border_color(p.border);

        if let Some(w) = self.width {
            card = card.w(px(w));
        } else {
            card = card.w_full();
        }
        if let Some(h) = self.height {
            card = card.h(px(h));
        } else {
            card = card.h_full();
        }

        // 1. Header Slot
        if self.header.is_some() || self.icon.is_some() || self.collapsible {
            let header_view = CardHeaderView {
                title: self.header,
                icon: self.icon,
                badge: self.badge,
                tool: self.tool,
                collapsible: self.collapsible,
                collapsed: self.collapsed,
                collapse_action: self.collapse_action,
            };
            card = card.child(header_view.into_elm(p, on_action.clone()));
        }

        // 2. Body Slot
        if !self.collapsed {
            if let Some(body_view) = self.body {
                let body_elm = CardBodyView {
                    content: *body_view,
                };
                card = card.child(body_elm.into_elm(p, on_action.clone()));
            }
        }

        // 3. Footer Slot
        if !self.collapsed && (self.footer.is_some() || self.footer_action.is_some()) {
            let footer_elm = CardFooterView {
                text: self.footer,
                action: self.footer_action,
            };
            card = card.child(footer_elm.into_elm(p, on_action));
        }

        card.into_any_element()
    }
}
