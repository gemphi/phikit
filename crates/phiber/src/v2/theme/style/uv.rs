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
    }
}
