//! One socket, painted: its dot on the card's edge and its name inside it.
//!
//! Split from `mod.rs`, which owns the links. The two are drawn in one pass and
//! in that order — links first, then sockets over them — because a socket sits
//! ON a card's border and a link drawn afterwards would cross the dot it ends
//! at.

use super::{BNodeCard, BNodeInk};
use crate::v2::view::draw::canvas::node::socket::{BSocketRow, BSocketSide, DOT};
use gpui::{App, Bounds, Corners, Pixels, SharedString, TextRun, Window, point, px, quad};

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
}
