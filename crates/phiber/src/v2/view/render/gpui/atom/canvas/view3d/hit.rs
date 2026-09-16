//! Selectable viewport object hit target.

use super::super::super::super::ctx::BRenderCtx;
use crate::v2::state::key::K_OBJECT;
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{ClickEvent, Div, SharedString, div, px};
use std::rc::Rc;

pub(super) struct BHit {
    pub x: f32,
    pub y: f32,
    pub name: String,
}

impl BHit {
    pub fn render(self, c: &BRenderCtx) -> gpui::Stateful<Div> {
        let size = c.style.dimensions.object_icon_size * 2.0;
        let handler = c.h.clone();
        let id = SharedString::from(format!("obj-{}", self.name));
        let action = BAction::Set(K_OBJECT, self.name);
        div()
            .id(id)
            .absolute()
            .left(gpui::relative(0.5 + self.x))
            .top(gpui::relative(0.5 + self.y))
            .ml(px(-size / 2.0))
            .mt(px(-size / 2.0))
            .w(px(size))
            .h(px(size))
            .cursor_pointer()
            .on_click(move |_event: &ClickEvent, window, app| {
                (Rc::clone(&handler))(&action, window, app);
            })
    }
}
