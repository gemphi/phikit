//! The picker's Hex row.
//!
//! Blender puts Hex last, after the sliders — `block_colorpicker` builds it with
//! the label `IFACE_("Hex")` (`interface_region_color_picker.cc:1008`) and
//! `colorpicker_hex_rna_cb` reads it back through `hex_to_rgba`.
//!
//! The spelling itself is `theme::{format_hex, parse_hex}`, ported there beside
//! `parse_hsla` because a colour's spellings belong with the descriptor that
//! validates them, not with one row that happens to show one.

use crate::v2::state::{BPath, BState, BValue};
use crate::v2::theme::BColor;
use crate::v2::view::BView;
use crate::v2::view::prim::{BCtrl, BCtrlKind};

/// The Hex field, last — where `block_colorpicker` puts it, after the sliders
/// (`interface_region_color_picker.cc:1008`, labelled `IFACE_("Hex")`).
///
/// It is a plain text control over the colour's OWN key, not a scratch buffer.
/// Blender needs one (`cpicker->hexcol`) because its button writes into a char
/// array and the colour lives in RNA; here the colour already IS text in the
/// store, and `parse_color` accepts the hex spelling, so typing a hex code into
/// this row writes the colour directly — the same key, the same `Set`, the same
/// resolver. A half-typed `#3D` is text the resolver cannot read, which is the
/// state the `"h, s, l, a"` field beside it is in for most of its own keystrokes:
/// the effective default shows until the spelling is whole again.
/// While this field has the caret it shows the state's own text, because that is
/// what the user is part-way through typing. Otherwise it shows the colour's hex —
/// not the state's text, which is the `"h, s, l, a"` spelling the other rows write.
/// Blender re-seeds `cpicker->hexcol` from the colour on every update for the same
/// reason: moving the square must change what the Hex field reads.
pub(super) fn row(key: &BPath, color: BColor, s: &BState) -> BView {
    let mut ctrl = BCtrl::read(BCtrlKind::Text, key.clone(), BValue::Text(String::new()), s).with_label("Hex");
    if !ctrl.editing {
        ctrl.value = BValue::Text(crate::v2::theme::format_hex(color));
    }
    BView::row(vec![BView::Label { text: "Hex".to_string() }, BView::Ctrl(Box::new(ctrl))])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hex_row_comes_last_and_reads_the_colour_not_the_stored_text() {
        // Blender puts Hex after the sliders (`block_colorpicker`, the
        // `IFACE_("Hex")` button). It is keyed to the colour itself, because
        // `parse_color` takes the hex spelling, so typing one writes the colour
        // with no scratch buffer in between.
        use crate::v2::state::BValue;
        let key = BPath::owned(vec!["ui_style", "colors", "bg_app"]);
        let views = super::super::rows(&key, (0.0, 0.0, 23.9, 1.0), &BState::new());
        let BView::Row { list } = views.last().expect("rows") else { panic!("{:?}", views.last()) };
        let [BView::Label { text }, BView::Ctrl(ctrl)] = &list[..] else { panic!("{list:?}") };
        assert_eq!(text, "Hex");
        assert_eq!(ctrl.key, key, "it writes the colour's own key");
        // The state holds nothing here, and it must still show the colour's hex
        // rather than an empty field or the `"h, s, l, a"` the other rows write.
        assert_eq!(ctrl.value, BValue::Text("#3D3D3D".to_string()), "the theme's panel grey");
        // And what it shows, the field would take back.
        let BValue::Text(shown) = &ctrl.value else { panic!() };
        assert!(crate::v2::theme::parse_color(shown).is_ok(), "{shown} is a colour the field accepts");
    }
}
