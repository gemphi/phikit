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
    /// A column of number fields, one per component — Blender's non-compact
    /// vector (`std_node_socket_draw` puts a name row above `column->prop`, which
    /// draws a field per component). Unlike the others this is TALLER than one
    /// row, so it changes the socket's height, which is why the layout asks
    /// [`rows_tall`] for the count and only an unlinked input ever gets it.
    Fields(Vec<String>),
}

impl BSocketWidget {
    /// How many rows this widget's socket occupies. Every widget but [`Fields`]
    /// sits inside the socket's own name row, so the socket is one row tall; a
    /// `Fields` column adds one row per component below the name
    /// (`BSocketWidget::Fields`).
    pub fn rows_tall(&self) -> f32 {
        match self {
            Self::Fields(f) => 1.0 + f.len() as f32,
            Self::Text(_) | Self::Check(_) | Self::Swatch(_) => 1.0,
        }
    }
}

/// How many rows `side`'s socket occupies, given its link state — one, unless it
/// is an unlinked input drawing a vector's field column (`BSocketWidget::Fields`).
/// Kept apart from [`widget`] so the layout can ask the height without formatting
/// the numbers, which only the painter needs.
pub fn rows_tall(side: BSocketSide, linked: bool, default: Option<&BSocketDefault>) -> f32 {
    if !shows_value(side, linked, default) {
        return 1.0;
    }
    match default {
        // One name row plus a field per component.
        Some(BSocketDefault::Vector(v)) => 1.0 + v.len() as f32,
        _ => 1.0,
    }
}

/// The widget `side`'s socket draws for `default`, or `None` when it draws
/// nothing: an output, a linked input, a socket with no default, or a vector.
///
/// A vector becomes a `Fields` column of one number per component — Blender's
/// non-compact vector. Each component reads to three decimals, the same
/// simplification as a float; Blender labels the fields X/Y/Z, which phiber does
/// not draw yet (the numbers alone, right-aligned like a scalar).
pub fn widget(side: BSocketSide, linked: bool, default: Option<&BSocketDefault>) -> Option<BSocketWidget> {
    if !shows_value(side, linked, default) {
        return None;
    }
    match default? {
        BSocketDefault::Float(v) => Some(BSocketWidget::Text(format!("{v:.3}"))),
        BSocketDefault::Int(v) => Some(BSocketWidget::Text(v.to_string())),
        BSocketDefault::Bool(v) => Some(BSocketWidget::Check(*v)),
        BSocketDefault::Color(rgba) => Some(BSocketWidget::Swatch(*rgba)),
        BSocketDefault::Vector(v) => Some(BSocketWidget::Fields(v.iter().map(|c| format!("{c:.3}")).collect())),
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
    /// swatch of its stored RGBA; a vector is a field per component.
    #[test]
    fn each_default_resolves_to_the_widget_the_painter_draws() {
        let w = |d| widget(BSocketSide::Input, false, Some(&d));
        assert_eq!(w(BSocketDefault::Float(0.5)), Some(BSocketWidget::Text("0.500".into())));
        assert_eq!(w(BSocketDefault::Float(1.0)), Some(BSocketWidget::Text("1.000".into())));
        assert_eq!(w(BSocketDefault::Int(4)), Some(BSocketWidget::Text("4".into())));
        assert_eq!(w(BSocketDefault::Bool(true)), Some(BSocketWidget::Check(true)), "a bool is a checkbox, not On/Off text");
        assert_eq!(w(BSocketDefault::Bool(false)), Some(BSocketWidget::Check(false)));
        assert_eq!(w(BSocketDefault::Color([0.8, 0.1, 0.2, 1.0])), Some(BSocketWidget::Swatch([0.8, 0.1, 0.2, 1.0])));
        assert_eq!(
            w(BSocketDefault::Vector([0.0, 0.5, 1.0])),
            Some(BSocketWidget::Fields(vec!["0.000".into(), "0.500".into(), "1.000".into()])),
            "a vector is a field per component",
        );
    }

    /// Only an unlinked vector input is taller than one row; everything else —
    /// a scalar, a swatch, a linked or output socket — is a single row.
    #[test]
    fn a_vector_input_is_the_only_socket_taller_than_one_row() {
        let vec = BSocketDefault::Vector([0.0, 0.0, 1.0]);
        assert_eq!(rows_tall(BSocketSide::Input, false, Some(&vec)), 4.0, "name row plus three fields");
        assert_eq!(rows_tall(BSocketSide::Input, true, Some(&vec)), 1.0, "a linked vector collapses to one row");
        assert_eq!(rows_tall(BSocketSide::Output, false, Some(&vec)), 1.0, "an output is one row");
        assert_eq!(rows_tall(BSocketSide::Input, false, Some(&BSocketDefault::Float(0.5))), 1.0, "a scalar fits its row");
        assert_eq!(rows_tall(BSocketSide::Input, false, None), 1.0, "no default, one row");
    }

    #[test]
    fn a_linked_or_output_socket_draws_no_widget_whatever_its_default() {
        let d = BSocketDefault::Color([1.0, 0.0, 0.0, 1.0]);
        assert_eq!(widget(BSocketSide::Input, true, Some(&d)), None, "a linked input shows the link");
        assert_eq!(widget(BSocketSide::Output, false, Some(&d)), None, "an output never shows a value");
    }
}
