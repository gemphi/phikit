//! SeparatorView — horizontal or visual divider line, plus drag-to-resize.

use super::super::bview::{ActionHandler, BAction};
use super::super::state::BKey;
use super::super::theme::BPalette;
use gpui::prelude::*;
use gpui::{AnyElement, App, Context, Div, IntoElement, Pixels, Point, Render, Window, div, px};
use gpui::{ClickEvent, InteractiveElement, StatefulInteractiveElement};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug)]
enum ResizeKind {
    Left,
    Right,
    RightTop,
    Bottom,
}

impl ResizeKind {
    fn from_key(key: &BKey) -> Self {
        match key {
            BKey::LeftWidth => Self::Left,
            BKey::RightWidth => Self::Right,
            BKey::RightTopHeight => Self::RightTop,
            BKey::BottomHeight => Self::Bottom,
            _ => Self::Right,
        }
    }

    fn compute_new_size(self, initial: f32, pos: Point<Pixels>, start_x: f32, start_y: f32) -> i32 {
        match self {
            Self::Left => (initial + (pos.x.as_f32() - start_x)) as i32,
            Self::Right => (initial - (pos.x.as_f32() - start_x)) as i32,
            Self::RightTop => (initial + (pos.y.as_f32() - start_y)) as i32,
            Self::Bottom => (initial - (pos.y.as_f32() - start_y)) as i32,
        }
    }
}

#[derive(Debug)]
struct ResizeDrag {
    #[allow(dead_code)]
    kind: ResizeKind,
    initial: f32,
    start_x: f32,
    start_y: f32,
}

impl ResizeDrag {
    fn new(kind: ResizeKind, initial: f32) -> Self {
        Self {
            kind,
            initial,
            start_x: 0.0,
            start_y: 0.0,
        }
    }
}

struct ResizeGhost;
impl Render for ResizeGhost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

pub struct SeparatorView {
    pub vertical: bool,
}

impl SeparatorView {
    pub fn new(vertical: bool) -> Self {
        Self { vertical }
    }

    pub fn with_elm(&self, p: &BPalette) -> Div {
        if self.vertical {
            div()
                .w(px(p.w_separator))
                .h_full()
                .flex_shrink_0()
                .cursor_col_resize()
                .bg(p.bg_viewport)
                .flex()
                .items_center()
                .justify_center()
                .child(div().w(px(1.)).h_full().bg(p.bg_viewport))
        } else {
            div()
                .h(px(p.h_separator))
                .w_full()
                .flex_shrink_0()
                .cursor_row_resize()
                .bg(p.bg_viewport)
                .flex()
                .items_center()
                .justify_center()
                .child(div().h(px(1.)).w_full().bg(p.bg_viewport))
        }
    }

    pub fn into_elm(
        &self,
        p: &BPalette,
        action: Option<&BAction>,
        on_action: ActionHandler,
    ) -> AnyElement {
        let sep_el = self.with_elm(p);
        match action {
            Some(BAction::Resize(key, initial)) => {
                let handler = on_action;
                let drag: Arc<Mutex<ResizeDrag>> = Arc::new(Mutex::new(ResizeDrag::new(
                    ResizeKind::from_key(key),
                    *initial as f32,
                )));
                let id = format!("sep-resize-{}", key.key());
                let key_owned = key.clone();
                sep_el
                    .id(id)
                    .on_drag(drag.clone(), move |drag, _pos: Point<Pixels>, w, _cx| {
                        let drag = drag.clone();
                        let mp = w.mouse_position();
                        {
                            let mut d = drag.lock().unwrap();
                            d.start_x = mp.x.as_f32();
                            d.start_y = mp.y.as_f32();
                        }
                        _cx.new(|_| ResizeGhost)
                    })
                    .on_drag_move::<Arc<Mutex<ResizeDrag>>>(move |e, w, cx| {
                        let drag = e.drag(cx).clone();
                        let (initial, start_x, start_y) = {
                            let d = drag.lock().unwrap();
                            (d.initial, d.start_x, d.start_y)
                        };
                        let kind = ResizeKind::from_key(&key_owned);
                        let new_size =
                            kind.compute_new_size(initial, e.event.position, start_x, start_y);
                        handler(&BAction::Resize(key_owned.clone(), new_size), w, cx);
                    })
                    .into_any_element()
            }
            Some(act) => {
                let act = act.clone();
                let handler = on_action;
                sep_el
                    .id(format!("sep-{}", if self.vertical { "v" } else { "h" }))
                    .on_click(move |_e: &ClickEvent, w: &mut Window, cx: &mut App| {
                        handler(&act, w, cx);
                    })
                    .into_any_element()
            }
            None => sep_el.into_any_element(),
        }
    }

    /// Popup item — thin horizontal divider line.
    pub fn into_popup_item(&self, p: &BPalette) -> AnyElement {
        div()
            .h(px(1.))
            .w_full()
            .bg(p.border)
            .my(px(p.gap_xs))
            .into_any_element()
    }
}
