//! What an unconnected input socket shows for its default value.
//!
//! Ported from Blender's `std_node_socket_draw` (`drawnode.cc`): an input with
//! nothing plugged into it draws its `default_value` as a widget in the row — a
//! number field for a float or int, a checkbox for a bool, a colour swatch for
//! an RGBA, a stacked column of fields for a vector. An output, a hidden-value
//! socket, or an input that IS linked draws only the dot and the name
//! (`draw_node_socket_without_value`); the value is what the link would supply.
//!
//! This file is the pure decisions behind that: whether a row shows a value at
//! all, and — if it does — which widget, resolved to the form the painter
//! draws. Keeping the choice here, as one [`widget`] call returning a
//! [`BSocketWidget`], is what lets the painter be one `match` rather than a
//! ladder of `if let`s guessing at the default's shape.

use super::BSocketSide;
use crate::phiphx::blend::shade::socket::BSocketDefault;

/// Whether this socket draws a value widget: an INPUT, not linked, with a stored
/// default. Blender draws the default only on unconnected inputs.
pub fn shows_value(side: BSocketSide, linked: bool, default: Option<&BSocketDefault>) -> bool {
    side == BSocketSide::Input && !linked && default.is_some()
}

/// The widget an unconnected input draws for its default — the painter's
/// shapes, chosen here so the painter dispatches on the shape rather than on the
/// default's type.
#[derive(Clone, Debug, PartialEq)]
pub enum BSocketWidget {
    /// A right-aligned number, as Blender formats it: three decimals for a float
    /// (a simplification — Blender's precision is the property's own), the number
    /// for an int.
    Text(String),
    /// A checkbox, checked when the stored bool is true. `std_node_socket_draw`
    /// draws a `SOCK_BOOLEAN` through the same `layout->prop` as a number, and a
    /// boolean property is a checkbox — so a bool is a box, never the words
    /// "On"/"Off", which was phiber's and not Blender's.
    Check(bool),
    /// A colour swatch, filled with Blender's stored LINEAR RGBA. The display
    /// (sRGB/OCIO) transform a swatch gets in Blender is not applied yet, so a
    /// pale `0.8` grey reads a little darker here than in Blender; applying a
    /// transform phiber cannot check against Blender's colour management would be
    /// inventing one, so the honest swatch is the stored value, noted.
    Swatch([f32; 4]),
}

/// The widget `side`'s socket draws for `default`, or `None` when it draws
/// nothing: an output, a linked input, a socket with no default, or a vector.
///
/// A vector is `None` NOT because it has no value but because Blender draws an
/// unconnected non-compact vector (`SOCK_VECTOR`, `std_node_socket_draw`) as a
/// name row above a stacked column of one field per component — a socket row
/// several units tall, and only when the socket is unlinked. phiber lays every
/// socket out at one row today, and the card's height (`socket::body_height`)
/// is computed without per-socket link state, so a value widget that changes a
/// socket's height can't be placed faithfully yet. Drawing the vector's three
/// numbers on the single existing row would be inventing a one-line form Blender
/// does not use; the honest state is to draw nothing until link state is threaded
/// into the height calculation, which is the vector widget's real prerequisite.
pub fn widget(side: BSocketSide, linked: bool, default: Option<&BSocketDefault>) -> Option<BSocketWidget> {
    if !shows_value(side, linked, default) {
        return None;
    }
    match default? {
        BSocketDefault::Float(v) => Some(BSocketWidget::Text(format!("{v:.3}"))),
        BSocketDefault::Int(v) => Some(BSocketWidget::Text(v.to_string())),
        BSocketDefault::Bool(v) => Some(BSocketWidget::Check(*v)),
        BSocketDefault::Color(rgba) => Some(BSocketWidget::Swatch(*rgba)),
        BSocketDefault::Vector(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_an_unconnected_input_with_a_default_shows_a_value() {
        let d = BSocketDefault::Float(0.5);
        assert!(shows_value(BSocketSide::Input, false, Some(&d)), "unconnected input with a default");
        assert!(!shows_value(BSocketSide::Input, true, Some(&d)), "a linked input shows the link, not the value");
        assert!(!shows_value(BSocketSide::Output, false, Some(&d)), "an output never shows a value");
        assert!(!shows_value(BSocketSide::Input, false, None), "no default, nothing to show");
    }

    /// A scalar reads as Blender formats it; a bool is a checkbox; a colour is a
    /// swatch of its stored RGBA; a vector draws nothing until its taller row is
    /// threaded through the height calculation.
    #[test]
    fn each_default_resolves_to_the_widget_the_painter_draws() {
        let w = |d| widget(BSocketSide::Input, false, Some(&d));
        assert_eq!(w(BSocketDefault::Float(0.5)), Some(BSocketWidget::Text("0.500".into())));
        assert_eq!(w(BSocketDefault::Float(1.0)), Some(BSocketWidget::Text("1.000".into())));
        assert_eq!(w(BSocketDefault::Int(4)), Some(BSocketWidget::Text("4".into())));
        assert_eq!(w(BSocketDefault::Bool(true)), Some(BSocketWidget::Check(true)), "a bool is a checkbox, not On/Off text");
        assert_eq!(w(BSocketDefault::Bool(false)), Some(BSocketWidget::Check(false)));
        assert_eq!(w(BSocketDefault::Color([0.8, 0.1, 0.2, 1.0])), Some(BSocketWidget::Swatch([0.8, 0.1, 0.2, 1.0])));
        assert_eq!(w(BSocketDefault::Vector([0.0, 0.0, 1.0])), None, "a vector needs a taller row, not yet laid out");
    }

    #[test]
    fn a_linked_or_output_socket_draws_no_widget_whatever_its_default() {
        let d = BSocketDefault::Color([1.0, 0.0, 0.0, 1.0]);
        assert_eq!(widget(BSocketSide::Input, true, Some(&d)), None, "a linked input shows the link");
        assert_eq!(widget(BSocketSide::Output, false, Some(&d)), None, "an output never shows a value");
    }
}
