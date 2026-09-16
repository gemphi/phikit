//! Switcher part view — host editor & mode picker popup.

pub mod hosts_col;
pub mod modes_col;
pub mod popup;
pub mod trigger;

pub use hosts_col::SwitcherHostsCol;
pub use modes_col::SwitcherModesCol;
pub use popup::SwitcherPopup;
pub use trigger::SwitcherTrigger;

use super::super::bview::{ActionHandler, BAction, BView};
use super::super::theme::BPalette;
use super::super::{BKey, BState};
use crate::store::hosts::BHostType;
use crate::store::parts::BPartType;
use gpui::prelude::*;
use gpui::{AnyElement, IntoElement, SharedString, StatefulInteractiveElement, div};

pub struct SwitcherView<'a> {
    pub host: BHostType,
    pub state: &'a BState,
}

impl<'a> SwitcherView<'a> {
    pub fn new(host: BHostType, state: &'a BState) -> Self {
        Self { host, state }
    }

    pub fn build(self) -> BView {
        let host = self.host;
        let state = self.state;
        let is_open = state.get(&BKey::Dropdown {
            host,
            part: BPartType::Switcher,
            key: "switcher".to_string(),
        }) == Some("true");
        let action = BAction::Toggle(BKey::Dropdown {
            host,
            part: BPartType::Switcher,
            key: "switcher".to_string(),
        });
        let preview_host = state
            .get(&BKey::SwitcherPreviewHost)
            .and_then(BHostType::from_name);

        BView::switcher(host, is_open, preview_host, action)
    }
}

pub struct SwitcherElementView {
    pub host: BHostType,
    pub open: bool,
    pub preview_host: Option<BHostType>,
}

impl SwitcherElementView {
    pub fn new(host: BHostType, open: bool, preview_host: Option<BHostType>) -> Self {
        Self {
            host,
            open,
            preview_host,
        }
    }

    pub fn into_elm(
        self,
        p: &BPalette,
        action: Option<&BAction>,
        on_action: ActionHandler,
    ) -> AnyElement {
        let id = format!("switcher-{:?}", self.host);
        let trigger = SwitcherTrigger {
            host: self.host,
            open: self.open,
        }
        .build(p);

        let mut container = div().relative();

        if let Some(act) = action {
            let handler = on_action.clone();
            let act_clone = act.clone();
            let clickable_trigger = trigger
                .id(SharedString::from(format!("{}-btn", id)))
                .on_click(move |_, window, cx| {
                    handler(&act_clone, window, cx);
                });
            container = container.child(clickable_trigger);
        } else {
            container = container.child(trigger);
        }

        if self.open {
            let left_col = SwitcherHostsCol {
                host: self.host,
                id: id.clone(),
            }
            .build(p, on_action.clone());
            // Use preview_host if set, otherwise fall back to the current host.
            let display_host = self.preview_host.unwrap_or(self.host);
            let right_col = SwitcherModesCol {
                host: display_host,
                id: id.clone(),
            }
            .build(p, on_action.clone());
            let popup = SwitcherPopup { id }.build(p, left_col, right_col, action, on_action);
            container = container.child(popup);
        }

        container.into_any_element()
    }
}
