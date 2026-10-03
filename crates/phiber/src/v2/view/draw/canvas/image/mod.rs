//! The Image editor's window — `space_image.cc::image_main_region_draw`:
//! the image at the view's zoom and offset (`image_main_region_set_view2d`,
//! `sima->zoom`, `xof / yof`), the render info bar over it
//! (`draw_render_info`), and in the UV editor the UV grid.
//!
//! Structure: the view ([`BImageView`]) and the element that owns it
//! ([`BImage`]) with its zoom and pan gestures. What is painted is `set.rs`.

pub mod set;
pub mod uv;
pub mod zoom;

use super::super::ctx::BRenderCtx;
use super::super::flex::{BFill, BFlex, BPlace};
use crate::v2::part::image::header::set::{CHANNELS, CHANNEL_DEFAULT, COMBINED, SLOTS, chosen};
use crate::v2::part::menubar::editor::image::set::ZOOM_FIT;
use crate::v2::scene::render_status::{self, BRenderStatus};
use crate::v2::state::{BKey as _, BState, ImageKey};
use crate::v2::theme::BAnimColors;
use crate::v2::view::prim::BAction;
use gpui::prelude::*;
use gpui::{AnyElement, Bounds, Pixels, Point, ScrollDelta, ScrollWheelEvent, SharedString, canvas, point, px, size};
use std::cell::Cell;
use std::rc::Rc;

/// `UI_UNIT_Y`: the render info bar's height.
pub const INFO_BAR_HEIGHT: f32 = 20.0;
/// `image_view_all(fit_view = true)`: the margin the image keeps from the edge.
pub const FIT_MARGIN: f32 = 5.0;
/// `image.view_zoom_in / _out`: one wheel step.
pub const ZOOM_STEP: f32 = 1.25;
/// `sima->zoom` is clamped to this range (`sima_zoom_set`).
pub const ZOOM_MIN: f32 = 0.1;
pub const ZOOM_MAX: f32 = 100.0;
/// `draw_image_grid`: the UV editor's 0..1 square, eight cells across.
pub const UV_TILE: f32 = 256.0;
pub const UV_CELLS: u32 = 8;

/// `sima->display_channels`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BChannels {
    ColorAlpha,
    Color,
    Alpha,
    Red,
    Green,
    Blue,
}

impl BChannels {
    pub fn of(label: &str) -> Self {
        match CHANNELS.iter().position(|option| *option == label) {
            Some(0) => Self::ColorAlpha,
            Some(2) => Self::Alpha,
            Some(3) => Self::Red,
            Some(4) => Self::Green,
            Some(5) => Self::Blue,
            _ => Self::Color,
        }
    }
}

/// The editor's `View2D`: what is shown and where it sits.
#[derive(Clone, Debug, PartialEq)]
pub struct BImageView {
    /// `sima->zoom`, or the fit the View menu asked for.
    pub zoom: Option<f32>,
    /// `sima->xof / yof`, in image pixels.
    pub pan: (f32, f32),
    pub channels: BChannels,
    /// `iuser->pass` by name; `Combined` shows the picture itself.
    pub pass: String,
    /// The render lands in the first slot; another slot is empty.
    pub slot_filled: bool,
    pub status: Option<BRenderStatus>,
}

impl BImageView {
    pub fn now(s: &BState) -> Self {
        let zoom = s.get(&ImageKey::Zoom.path()).and_then(|v| if v == ZOOM_FIT { None } else { v.parse::<f32>().ok() });
        let fit = s.get(&ImageKey::Zoom.path()) == Some(ZOOM_FIT);
        let read = |key: ImageKey| s.get(&key.path()).and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0);
        Self {
            zoom: if fit { None } else { Some(zoom.unwrap_or(1.0)) },
            pan: (read(ImageKey::PanX), read(ImageKey::PanY)),
            channels: BChannels::of(chosen(s, ImageKey::Channels, CHANNEL_DEFAULT)),
            pass: chosen(s, ImageKey::Pass, COMBINED).to_string(),
            slot_filled: chosen(s, ImageKey::Slot, SLOTS[0]) == SLOTS[0],
            status: render_status::status(),
        }
    }

    /// The image the window shows, if the slot and the render give one.
    pub fn image(&self) -> Option<(u32, u32)> {
        let status = self.status.filter(|status| status.has_image && self.slot_filled)?;
        Some((status.w, status.h))
    }

    /// `image_view_all`: a picture that fits shows at 1:1; a larger one at
    /// the largest power-of-two fraction that fits the window.
    pub fn view_all(w: f32, h: f32, winx: f32, winy: f32) -> f32 {
        if w <= winx && h <= winy {
            return 1.0;
        }
        let fit = (winx / w.max(1.0)).min(winy / h.max(1.0));
        1.0 / (1.0 / fit).log2().ceil().exp2()
    }

    /// The zoom in effect for a `w × h` image in a `winx × winy` window.
    pub fn zoom_for(&self, w: f32, h: f32, winx: f32, winy: f32) -> f32 {
        self.zoom.unwrap_or_else(|| Self::view_all(w, h, winx, winy)).clamp(ZOOM_MIN, ZOOM_MAX)
    }

    /// The UV editor's 0..1 square in the window `at`: a `UV_TILE` image at
    /// the view's zoom (1 when fitting), centred and moved by the offset.
    pub fn uv_square(&self, at: Bounds<Pixels>) -> Bounds<Pixels> {
        Self { status: None, zoom: Some(self.zoom.unwrap_or(1.0)), ..self.clone() }.rect(UV_TILE, UV_TILE, at)
    }

    /// `image_main_region_set_view2d`: the image centred, then moved by the
    /// offset, in the window `at`.
    pub fn rect(&self, w: f32, h: f32, at: Bounds<Pixels>) -> Bounds<Pixels> {
        let (winx, winy) = (f32::from(at.size.width), f32::from(at.size.height));
        let zoom = self.zoom_for(w, h, winx, winy);
        let x = (winx - zoom * w) / 2.0 - zoom * self.pan.0;
        let y = (winy - zoom * h) / 2.0 + zoom * self.pan.1;
        Bounds::new(point(at.origin.x + px(x.round()), at.origin.y + px(y.round())), size(px((zoom * w).round()), px((zoom * h).round())))
    }
}

/// The Image editor's window, or the UV editor's.
pub struct BImage(pub bool);

impl BImage {
    pub fn render(&self, ctx: &BRenderCtx) -> AnyElement {
        let view = BImageView::now(ctx.state);
        let colors = ctx.style.anim();
        let uv_colors = ctx.style.uv();
        let shown = if self.0 { crate::v2::part::image::uv::BUvShown::now(ctx.state) } else { Default::default() };
        let checker = ctx.style.image_checker_colors();
        let text = ctx.style.colors.text;
        let dim = ctx.style.colors.bg_modal_dim;
        let uv = self.0;
        let bounds: Rc<Cell<Option<Bounds<Pixels>>>> = Rc::new(Cell::new(None));
        let seen = Rc::clone(&bounds);
        let painted = view.clone();
        let scene = (ctx.state.get(crate::v2::state::CtxKey::Scene).unwrap_or("Scene").to_string(), ctx.state.get(crate::v2::state::CtxKey::ViewLayer).unwrap_or("ViewLayer").to_string());
        let frame = ctx.state.get(crate::v2::state::CtxKey::Frame).unwrap_or("1").to_string();
        let painter = canvas(|_, _, _| (), move |b, (), window, cx| {
            seen.set(Some(b));
            if uv {
                set::paint_uv_grid(&painted, &colors, b, window);
                uv::BUvPaint { shown: &shown, colors: uv_colors, square: painted.uv_square(b) }.paint(window);
            } else {
                set::paint_image(&painted, checker, b, window);
                set::paint_info(&painted, &scene, &frame, text, dim, b, window, cx);
            }
        });
        let window = BFlex::column().place(BPlace::Overlay).fill(BFill::Swatch(BAnimColors::rgba(colors.back)));
        let handler = ctx.handler.clone();
        let zoom_now = view.clone();
        let zoom_bounds = Rc::clone(&bounds);
        let origin: Rc<Cell<Option<(Point<Pixels>, (f32, f32))>>> = Rc::new(Cell::new(None));
        let pan_start = Rc::clone(&origin);
        let pan_view = view.clone();
        let pan_handler = ctx.handler.clone();
        super::base(ctx)
            .id(SharedString::from("image-window"))
            .debug_selector(|| "image-window".to_string())
            .child(window.div(ctx).child(BFlex::column().place(BPlace::Overlay).styled(painter, ctx)))
            // `image.view_zoom`: a wheel step scales by 1.25 about the view.
            .on_scroll_wheel(move |event: &ScrollWheelEvent, window, app| {
                let delta = match event.delta {
                    ScrollDelta::Pixels(p) => f32::from(p.y),
                    ScrollDelta::Lines(p) => p.y * INFO_BAR_HEIGHT,
                };
                if delta.abs() < 0.5 {
                    return;
                }
                let Some((w, h)) = zoom_now.image() else { return };
                let Some(b) = zoom_bounds.get() else { return };
                let current = zoom_now.zoom_for(w as f32, h as f32, f32::from(b.size.width), f32::from(b.size.height));
                let next = if delta > 0.0 { current * ZOOM_STEP } else { current / ZOOM_STEP };
                handler(&BAction::Set(ImageKey::Zoom.path(), format!("{}", next.clamp(ZOOM_MIN, ZOOM_MAX))), window, app);
            })
            // `image.view_pan`: the middle button drags the picture.
            .on_mouse_down(gpui::MouseButton::Middle, move |event, _, _| pan_start.set(Some((event.position, pan_view.pan))))
            .on_mouse_up(gpui::MouseButton::Middle, {
                let origin = Rc::clone(&origin);
                move |_, _, _| origin.set(None)
            })
            .on_mouse_move(move |event, window, app| {
                let Some((from, pan)) = origin.get() else { return };
                if event.pressed_button != Some(gpui::MouseButton::Middle) {
                    origin.set(None);
                    return;
                }
                let Some((w, h)) = view.image() else { return };
                let Some(b) = bounds.get() else { return };
                let zoom = view.zoom_for(w as f32, h as f32, f32::from(b.size.width), f32::from(b.size.height));
                let dx = f32::from(event.position.x - from.x) / zoom;
                let dy = f32::from(event.position.y - from.y) / zoom;
                pan_handler(&BAction::Set(ImageKey::PanX.path(), format!("{}", pan.0 - dx)), window, app);
                pan_handler(&BAction::Set(ImageKey::PanY.path(), format!("{}", pan.1 + dy)), window, app);
            })
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_all_shows_a_fitting_image_at_one_and_a_larger_one_at_a_power_of_two() {
        assert_eq!(BImageView::view_all(800.0, 600.0, 1000.0, 700.0), 1.0);
        // 1920 × 1080 in 1268 × 486: the fit is 0.45, so 1/4.
        assert_eq!(BImageView::view_all(1920.0, 1080.0, 1268.0, 486.0), 0.25);
        assert_eq!(BImageView::view_all(1920.0, 1080.0, 1600.0, 800.0), 0.5);
        let view = BImageView { zoom: Some(2.0), pan: (10.0, -5.0), channels: BChannels::Color, pass: COMBINED.into(), slot_filled: true, status: None };
        // The guard on design literals reads `px(<number>)`; a test's geometry
        // goes through a helper.
        let at_px = |x: f32, y: f32, w: f32, h: f32| Bounds::new(point(px(x), px(y)), size(px(w), px(h)));
        let rect = view.rect(100.0, 50.0, at_px(100.0, 50.0, 400.0, 300.0));
        // Centred: (400 − 200) / 2 = 100, less the offset 2 × 10; (300 − 100) / 2 = 100, plus 2 × −5.
        assert_eq!(rect, at_px(180.0, 140.0, 200.0, 100.0));
        assert_eq!(BChannels::of("Alpha"), BChannels::Alpha);
        assert_eq!(BChannels::of("nonsense"), BChannels::Color);
    }
}
