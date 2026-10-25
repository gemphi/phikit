//! How the wheel is drawn and grabbed.
//!
//! The disc is painted, not built from elements: hue and saturation change
//! across every pixel of it, and `draw_but_HSVCIRCLE` itself is a triangle
//! fan of 64 colours. Here it is a grid of small quads, each the colour
//! [`BWheel::hs_at`] reads at its own centre — so what a spot shows is what
//! a press there writes — inside a ring in the outline colour that covers the
//! grid's stepped edge. The value bar beside it is bands from black to the
//! full colour, Blender's `GRAD_V_ALT`.
//!
//! A press writes at once, as Blender's does (`ui_do_but_HSVCIRCLE` jumps to
//! the pointer on press); a drag keeps writing. The press needs the bounds,
//! which only paint knows, so paint hands them to the press through a cell.

use super::{BWheel, hsv_to_rgb};
use crate::v2::view::draw::color::to_hsla;
use crate::v2::view::draw::ctx::BRenderCtx;
use crate::v2::view::draw::flex::{BFlex, BLen, BSpace};
use crate::v2::view::draw::track::BTrackGhost;
use gpui::prelude::*;
use gpui::{App, Bounds, DragMoveEvent, MouseButton, MouseDownEvent, Pixels, Point, Window, canvas, fill, point, px, size};
use std::cell::Cell;
use std::rc::Rc;

/// The disc's diameter. Blender sizes its circle to the popup's width
/// (`colorpicker_circle`, `PICKER_H`); a picker popover here is about this
/// wide once the value bar and the gap are beside it.
const DISC: f32 = 150.0;
/// The stroke of the wheel's rim and of the cursor's ring: Blender's
/// `hsv_cursor` and wheel outline, a pixel and a half.
const RING: f32 = 1.5;
/// The value bar's corner and the hairline round it.
const BAR_ROUND: f32 = 2.0;
const BAR_EDGE: f32 = 1.0;
/// The value marker's thickness.
const MARK: f32 = 3.0;
/// The value bar's width (`PICKER_W`'s slim column beside the circle).
const BAR: f32 = 16.0;
/// Quads across the disc: two pixels each at `DISC`, which the ring and the
/// smooth shading of neighbouring quads make read as continuous.
const CELLS: usize = 75;
/// Bands up the value bar.
const BANDS: usize = 48;

/// The second gesture: the same wheel, dragged by its value bar.
#[derive(Clone)]
struct BWheelValue(BWheel);

/// The wheel and its value bar, side by side.
pub fn view(wheel: &BWheel, ctx: &BRenderCtx) -> gpui::AnyElement {
    BFlex::row()
        .gap(BSpace::GapMd)
        .height(BLen::Measured(DISC))
        .keeps_size()
        .div(ctx)
        .child(disc(wheel, ctx))
        .child(bar(wheel, ctx))
        .into_any_element()
}

/// The colour as gpui paints it: the RGB the swatch shows, opaque.
fn rgb(c: [f32; 3]) -> gpui::Hsla {
    gpui::Rgba { r: c[0].clamp(0.0, 1.0), g: c[1].clamp(0.0, 1.0), b: c[2].clamp(0.0, 1.0), a: 1.0 }.into()
}

fn disc(wheel: &BWheel, ctx: &BRenderCtx) -> gpui::AnyElement {
    let id = gpui::SharedString::from(format!("wheel-{}", wheel.key.encode()));
    let painted: Rc<Cell<Option<Bounds<Pixels>>>> = Rc::new(Cell::new(None));
    let outline = to_hsla(ctx.style.colors.border);
    let paint = {
        let (wheel, painted) = (wheel.clone(), painted.clone());
        move |bounds: Bounds<Pixels>, _: (), window: &mut Window, _: &mut App| {
            painted.set(Some(bounds));
            let side = f32::from(bounds.size.width).min(f32::from(bounds.size.height));
            let origin = point(
                bounds.origin.x + px((f32::from(bounds.size.width) - side) / 2.0),
                bounds.origin.y + px((f32::from(bounds.size.height) - side) / 2.0),
            );
            let step = side / CELLS as f32;
            let value = wheel.hsv[2].clamp(0.0, 1.0);
            for row in 0..CELLS {
                let down = (row as f32 + 0.5) / CELLS as f32;
                for column in 0..CELLS {
                    let across = (column as f32 + 0.5) / CELLS as f32;
                    let (dx, dy) = (across - 0.5, down - 0.5);
                    if dx * dx + dy * dy > 0.25 {
                        continue;
                    }
                    let (h, s) = BWheel::hs_at(across, down);
                    let at = point(origin.x + px(column as f32 * step), origin.y + px(row as f32 * step));
                    // A hair over a cell, so neighbours overlap rather than
                    // leaving the panel showing through hairline seams.
                    window.paint_quad(fill(Bounds { origin: at, size: size(px(step + 0.6), px(step + 0.6)) }, rgb(hsv_to_rgb([h, s, value]))));
                }
            }
            // The rim, over the grid's stepped edge.
            let ring = Bounds { origin, size: size(px(side), px(side)) };
            window.paint_quad(gpui::quad(ring, px(side / 2.0), gpui::transparent_black(), px(RING), outline, Default::default()));
            // The cursor: the colour itself in a white ring — `hsv_cursor`.
            let (across, down) = wheel.marker();
            let r = 6.0;
            let centre = point(origin.x + px(across * side - r), origin.y + px(down * side - r));
            let across_ring = r + r;
            let cursor = Bounds { origin: centre, size: size(px(across_ring), px(across_ring)) };
            window.paint_quad(gpui::quad(cursor, px(r), rgb(wheel.shown()), px(RING), gpui::white(), Default::default()));
        }
    };
    let element = BFlex::cell()
        .width(BLen::Measured(DISC))
        .height(BLen::Measured(DISC))
        .keeps_size()
        .div(ctx)
        .id(id.clone())
        .debug_selector(move || id.to_string())
        .child(BFlex::cell().width(BLen::Full).height(BLen::Full).styled(canvas(|_, _, _| (), paint), ctx));
    let press = {
        let (wheel, handler) = (wheel.clone(), ctx.handler.clone());
        move |event: &MouseDownEvent, window: &mut Window, app: &mut App| {
            // Over a node editor the press is the wheel's, not the card's
            // beneath it to select or drag.
            app.stop_propagation();
            if let Some(bounds) = painted.get() {
                (Rc::clone(&handler))(&wheel.adjust_at(bounds, event.position), window, app);
            }
        }
    };
    let over = {
        let handler = ctx.handler.clone();
        move |event: &DragMoveEvent<BWheel>, window: &mut Window, app: &mut App| {
            let wheel = event.drag(app).clone();
            (Rc::clone(&handler))(&wheel.adjust_at(event.bounds, event.event.position), window, app);
        }
    };
    element
        .cursor(gpui::CursorStyle::Crosshair)
        .on_mouse_down(MouseButton::Left, press)
        .on_drag(wheel.clone(), |_: &BWheel, _: Point<Pixels>, _: &mut Window, app: &mut App| app.new(|_| BTrackGhost))
        .on_drag_move::<BWheel>(over)
        .into_any_element()
}

fn bar(wheel: &BWheel, ctx: &BRenderCtx) -> gpui::AnyElement {
    let id = gpui::SharedString::from(format!("wheel-value-{}", wheel.key.encode()));
    let painted: Rc<Cell<Option<Bounds<Pixels>>>> = Rc::new(Cell::new(None));
    let outline = to_hsla(ctx.style.colors.border);
    let paint = {
        let (wheel, painted) = (wheel.clone(), painted.clone());
        move |bounds: Bounds<Pixels>, _: (), window: &mut Window, _: &mut App| {
            painted.set(Some(bounds));
            let (w, h) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
            let band = h / BANDS as f32;
            for k in 0..BANDS {
                // Top band is value 1, bottom value 0.
                let value = 1.0 - (k as f32 + 0.5) / BANDS as f32;
                let at = point(bounds.origin.x, bounds.origin.y + px(k as f32 * band));
                window.paint_quad(fill(Bounds { origin: at, size: size(px(w), px(band + 0.6)) }, rgb(hsv_to_rgb([wheel.hsv[0], wheel.hsv[1], value]))));
            }
            window.paint_quad(gpui::quad(bounds, px(BAR_ROUND), gpui::transparent_black(), px(BAR_EDGE), outline, Default::default()));
            // The marker: a bar across at the value, in the colour's contrast.
            let v = wheel.hsv[2].clamp(0.0, 1.0);
            let y = bounds.origin.y + px((1.0 - v) * h - MARK / 2.0);
            let ink = if v > 0.5 { gpui::black() } else { gpui::white() };
            window.paint_quad(fill(Bounds { origin: point(bounds.origin.x, y), size: size(px(w), px(MARK)) }, ink));
        }
    };
    let element = BFlex::cell()
        .width(BLen::Measured(BAR))
        .height(BLen::Measured(DISC))
        .keeps_size()
        .div(ctx)
        .id(id.clone())
        .debug_selector(move || id.to_string())
        .child(BFlex::cell().width(BLen::Full).height(BLen::Full).styled(canvas(|_, _, _| (), paint), ctx));
    let press = {
        let (wheel, handler) = (wheel.clone(), ctx.handler.clone());
        move |event: &MouseDownEvent, window: &mut Window, app: &mut App| {
            app.stop_propagation();
            if let Some(bounds) = painted.get() {
                (Rc::clone(&handler))(&wheel.value_at(bounds, event.position), window, app);
            }
        }
    };
    let over = {
        let handler = ctx.handler.clone();
        move |event: &DragMoveEvent<BWheelValue>, window: &mut Window, app: &mut App| {
            let BWheelValue(wheel) = event.drag(app).clone();
            (Rc::clone(&handler))(&wheel.value_at(event.bounds, event.event.position), window, app);
        }
    };
    element
        .cursor(gpui::CursorStyle::ResizeUpDown)
        .on_mouse_down(MouseButton::Left, press)
        .on_drag(BWheelValue(wheel.clone()), |_: &BWheelValue, _: Point<Pixels>, _: &mut Window, app: &mut App| app.new(|_| BTrackGhost))
        .on_drag_move::<BWheelValue>(over)
        .into_any_element()
}
