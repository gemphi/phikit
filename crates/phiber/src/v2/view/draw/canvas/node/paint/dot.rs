//! One socket, painted: its dot on the card's edge and its name inside it.
//!
//! Split from `mod.rs`, which owns the links. The two are drawn in one pass and
//! in that order — links first, then sockets over them — because a socket sits
//! ON a card's border and a link drawn afterwards would cross the dot it ends
//! at.

use super::{BNodeCard, BNodeInk};
use crate::v2::view::draw::canvas::node::socket::{BSocketRow, BSocketSide, DOT, value};
use gpui::{App, Bounds, Corners, Pixels, Rgba, SharedString, TextRun, Window, point, px, quad, size};

/// One socket: its dot on the card's edge, and its name inside the card.
pub(super) fn paint_socket(
    card: &BNodeCard,
    row: &BSocketRow,
    ink: &BNodeInk,
    at: Bounds<Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    let (x, y) = card.socket_at(row);
    // `NODE_SOCKSIZE` is the radius, in widget units.
    let r = DOT * card.unit;
    let dot = Bounds::new(
        point(at.origin.x + px(x - r), at.origin.y + px(y - r)),
        gpui::size(px(r * 2.0), px(r * 2.0)),
    );
    let to_hsla = crate::v2::view::draw::color::to_hsla;
    window.paint_quad(quad(
        dot,
        Corners::all(px(r)),
        to_hsla(crate::v2::theme::socket_color(row.kind)),
        gpui::Edges::all(px(ink.ring)),
        to_hsla(ink.outline),
        gpui::BorderStyle::Solid,
    ));
    // The name, inside the card: an input reads from the left edge, an output
    // is pushed against the right one, which is Blender's
    // `LayoutAlign::Expand` for inputs against `LayoutAlign::Right` for
    // outputs (`node_update_basis_socket`).
    let run = TextRun {
        len: row.name.len(),
        font: window.text_style().font(),
        color: to_hsla(ink.label),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let size = px(card.unit * 0.55);
    let line = window.text_system().shape_line(
        SharedString::from(row.name.clone()),
        size,
        &[run],
        Some(px(card.width - r * 4.0)),
    );
    let text_x = match row.side {
        BSocketSide::Input => x + r * 2.0,
        BSocketSide::Output => x - r * 2.0 - f32::from(line.width()),
    };
    let line_height = size * 1.3;
    let _ = line.paint(
        point(at.origin.x + px(text_x), at.origin.y + px(y) - line_height / 2.0),
        line_height,
        gpui::TextAlign::Left,
        None,
        window,
        cx,
    );
    paint_value(card, row, ink, at, window, cx);
}

/// The default value an unconnected input shows: a right-aligned number for a
/// scalar, a colour swatch for an RGBA. A linked input, an output, a socket with
/// no default, or a vector draws nothing — `value::widget` resolves all of that
/// to one of two shapes, so this is a single `match`, not a ladder of guesses.
///
/// Right-aligned against the card's inner edge so a row reads "Roughness 0.500"
/// — Blender draws the value in a field that fills the row with the label on
/// the left; phiber keeps the painted label it already draws and sets the value
/// opposite it, which is the same reading without a field box to draw.
fn paint_value(
    card: &BNodeCard,
    row: &BSocketRow,
    ink: &BNodeInk,
    at: Bounds<Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    let (x, y) = card.socket_at(row);
    let r = DOT * card.unit;
    // The inner-right edge of the row, a dot's width in from the card's border.
    let inner_right = x + card.width - r * 2.0;
    let to_hsla = crate::v2::view::draw::color::to_hsla;
    match value::widget(row.side, row.linked, row.default.as_ref()) {
        // A scalar: one right-aligned number on the socket's own row.
        Some(value::BSocketWidget::Text(text)) => {
            paint_number(&text, card.unit, inner_right, y, ink, at, window, cx);
        }
        // A vector: a number per component, stacked one row apart below the
        // name — the field column `std_node_socket_draw` draws. The socket's
        // row was laid out this many units taller (`value::rows_tall`), so the
        // fields land inside the card, not over the socket below.
        Some(value::BSocketWidget::Fields(parts)) => {
            for (i, part) in parts.iter().enumerate() {
                let row_y = y + (i as f32 + 1.0) * card.unit;
                paint_number(part, card.unit, inner_right, row_y, ink, at, window, cx);
            }
        }
        Some(value::BSocketWidget::Check(on)) => {
            // A square box at the row's inner edge, filled when checked — the
            // boolean property's checkbox. Blender draws the check inline with
            // the label; phiber keeps its own painted label and sets the box at
            // the value position, where the other widgets sit.
            let s = card.unit * 0.5;
            let box_ = Bounds::new(
                point(at.origin.x + px(inner_right - s), at.origin.y + px(y - s / 2.0)),
                size(px(s), px(s)),
            );
            // Filled with the label ink when checked, transparent (just its
            // border) when not — a box you can see either way.
            let fill = if on { to_hsla(ink.label) } else { gpui::Hsla::transparent_black() };
            window.paint_quad(quad(
                box_,
                Corners::all(px(card.unit * 0.1)),
                fill,
                gpui::Edges::all(px(ink.ring)),
                to_hsla(ink.outline),
                gpui::BorderStyle::Solid,
            ));
        }
        Some(value::BSocketWidget::Swatch([rr, gg, bb, aa])) => {
            // A swatch the height of a short field, its right edge at the row's
            // inner edge, centred on the socket's own line.
            let (sw, sh) = (card.unit * 0.9, card.unit * 0.42);
            let swatch = Bounds::new(
                point(at.origin.x + px(inner_right - sw), at.origin.y + px(y - sh / 2.0)),
                size(px(sw), px(sh)),
            );
            let fill: gpui::Hsla = Rgba { r: rr, g: gg, b: bb, a: aa }.into();
            window.paint_quad(quad(
                swatch,
                Corners::all(px(card.unit * 0.1)),
                fill,
                gpui::Edges::all(px(ink.ring)),
                to_hsla(ink.outline),
                gpui::BorderStyle::Solid,
            ));
        }
        None => {}
    }
}

/// One right-aligned number, its right edge at `inner_right` and its middle at
/// `y` — the shared body of a scalar value and of each component of a vector's
/// field column.
#[allow(clippy::too_many_arguments)]
fn paint_number(
    text: &str,
    unit: f32,
    inner_right: f32,
    y: f32,
    ink: &BNodeInk,
    at: Bounds<Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    let run = TextRun {
        len: text.len(),
        font: window.text_style().font(),
        color: crate::v2::view::draw::color::to_hsla(ink.label),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let size_px = px(unit * 0.55);
    let line = window.text_system().shape_line(SharedString::from(text.to_string()), size_px, &[run], None);
    let line_height = size_px * 1.3;
    let _ = line.paint(
        point(at.origin.x + px(inner_right) - line.width(), at.origin.y + px(y) - line_height / 2.0),
        line_height,
        gpui::TextAlign::Left,
        None,
        window,
        cx,
    );
}
