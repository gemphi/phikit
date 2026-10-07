//! The UV editor's colours — `space_image`'s `wire_edit`, `uv_shadow`,
//! `face`, `vertex` in Blender's theme — from the palette in use, as every
//! canvas takes its colours: the edges in the text's light shade on a dark
//! outline (`edge_display_type` Outline), the Object Mode UVs in the dim
//! text shade, a face a wash of the text shade, a vertex the app's darkest.

use super::BResolvedUiStyle;
use crate::v2::theme::palette::BColor;

/// `space_image.face`: white at 10/255.
const FACE_ALPHA: f32 = 0.04;

/// What the UV layout is drawn in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BUvColors {
    /// `wire_edit`: an edge in Edit Mode.
    pub edge: BColor,
    /// The line under it (Outline).
    pub outline: BColor,
    /// `uv_shadow`: the UVs shown in Object Mode.
    pub shadow: BColor,
    /// `face`: a face in Edit Mode.
    pub face: BColor,
    /// `vertex`: a vertex in Edit Mode.
    pub vertex: BColor,
    /// `vertex_select`: a selected one. The accent, which in this theme is
    /// off the shade ladder because it means something
    /// (`ui-parity-status.md`'s ladder).
    pub selected: BColor,
    /// `TH_UV_OTHERS` is not it: Blender draws a **pinned** UV in its own
    /// colour (`TH_PIN`, red by default) because a pin is a different
    /// thing from a selection — it is what an unwrap solves around, and
    /// it holds whether or not the corner is selected.
    pub pinned: BColor,
}

impl BResolvedUiStyle {
    pub fn uv(&self) -> BUvColors {
        let colors = &self.colors;
        BUvColors {
            edge: colors.text,
            outline: colors.bg_app,
            shadow: colors.text_dim,
            face: (colors.text.0, colors.text.1, colors.text.2, FACE_ALPHA),
            vertex: colors.bg_app,
            selected: colors.accent,
            // `TH_PIN` is a flat red in every Blender theme, light and
            // dark alike, and is not derived from the UI palette — a pin
            // has to read as a pin whatever the editor is wearing. This
            // palette carries no red at all, so it is written out here
            // rather than bent out of the accent.
            pinned: (0.87, 0.21, 0.21, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::v2::state::BState;
    use crate::v2::theme::{BPaletteName, resolve_ui_style, style::ui_style_defaults};

    #[test]
    fn the_uv_layout_takes_the_palettes_shades() {
        let style = resolve_ui_style(ui_style_defaults(BPaletteName::Sphere), &BState::new());
        let uv = style.uv();
        assert_eq!(uv.edge, style.colors.text);
        assert!(uv.shadow.2 < uv.edge.2 && uv.outline.2 < uv.shadow.2, "Object Mode's UVs are dimmer than Edit Mode's, the outline darker still");
        assert!(uv.face.3 < 0.1, "a face is a wash");
        assert_eq!(uv.selected, style.colors.accent, "a selected corner is the accent");
        assert_ne!(uv.selected, uv.vertex, "and tells itself from an unselected one");
    }
}
