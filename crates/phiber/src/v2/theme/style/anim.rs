//! The animation editors' colours — Timeline, Dope Sheet, Graph, NLA — as
//! phiber's theme gives them: Blender's *geometry* with every colour from
//! the palette in use (Blender has a colour per slot: `space_action.back`,
//! `TH_CFRAME`…). Each field names the Blender slot it stands in for.

use super::BResolvedUiStyle;
use crate::v2::theme::palette::BColor;

/// `TH_GRID` is the step inward of `TH_BACK` (light on a dark canvas, as
/// Blender's `#545454` on `#3D3D3D`), the minor lines at half its alpha, and
/// the drawn line (`draw_image_grid`, the UV square's and the 3D floor's
/// alike) that shade lifted a tenth.
const GRID_OVER_BACK: i32 = 1;
pub const GRID_LINE_LIFT: f32 = 0.10;
const MINOR_ALPHA: f32 = 0.5;
/// `ANIM_draw_framerange`: the out-of-range curtain, translucent, three
/// steps under the canvas; its edge line a step further.
const CURTAIN_UNDER_BACK: i32 = -3;
const CURTAIN_ALPHA: f32 = 0.6;
const RANGE_LINE_UNDER_BACK: i32 = -4;
/// The preview-range curtain and the summary / object backdrops.
const PREVIEW_ALPHA: f32 = 0.28;
const BACKDROP_ALPHA: f32 = 0.5;
/// `anim_active` is the warm accent at 0.4, `anim_non_active` twenty points darker at 0.3, `strip` near-black at a half.
const ACTION_ALPHA: f32 = 0.4;
const NO_ACTION_ALPHA: f32 = 0.3;
const NO_ACTION_UNDER: f32 = -0.20;
const STRIP_ALPHA: f32 = 0.5;
/// `graph_main_region_draw`'s cursor: `TH_CFRAME` shaded −10 (the value line)
/// and −40 (the Drivers editor's input line), both at alpha −50.
const CURSOR_VALUE_SHADE: f32 = -0.04;
const CURSOR_TIME_SHADE: f32 = -0.16;
const CURSOR_ALPHA: f32 = 0.8;
/// An unselected handle: its selected colour darkened (`handle_*` under `handle_sel_*`).
const HANDLE_UNDER_SELECTED: f32 = -0.25;
/// `time_marker_line` is black at 96/255 (the dashed stalk), `_selected` white at 160/255.
const MARKER_LINE_ALPHA: f32 = 0.376;
const MARKER_LINE_SELECTED_ALPHA: f32 = 0.627;

/// One editor's worth of colours, in the theme's `BColor` (h, s, l, a).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BAnimColors {
    /// `TH_BACK`: the window.
    pub back: BColor,
    /// `TH_GRID`, the line drawn over it (the UV square's and the 3D
    /// floor's), and the minor lines shaded up from it.
    pub grid: BColor,
    pub grid_line: BColor,
    pub grid_minor: BColor,
    /// `ANIM_draw_framerange`: outside the scene range, and its edge line.
    pub out_of_range: BColor,
    pub range_line: BColor,
    /// `TH_ANIM_PREVIEW_RANGE`.
    pub preview: BColor,
    /// `TH_TIME_SCRUB_BACKGROUND` / `TH_TIME_SCRUB_TEXT`.
    pub scrub_back: BColor,
    pub scrub_text: BColor,
    /// `TH_CFRAME`, its shadow, and the frame number in it.
    pub playhead: BColor,
    pub playhead_shadow: BColor,
    pub playhead_text: BColor,
    /// The Graph editor's cursor: the value line (`cursorVal`) and, in the
    /// Drivers editor, the input line (`cursorTime`), `TH_CFRAME` shaded.
    pub cursor_value: BColor,
    pub cursor_time: BColor,
    /// `regions.channels`: the channel region and its names.
    pub channel_back: BColor,
    pub channel_text: BColor,
    /// `TH_ANIM_ACTIVE` (the summary) and `TH_DOPESHEET_CHANNELOB` (an object).
    pub summary_back: BColor,
    pub object_back: BColor,
    /// `TH_KEYTYPE_KEYFRAME`, `_SELECT` (the warm accent) / `TH_KEYBORDER`.
    pub key: BColor,
    pub key_select: BColor,
    pub key_border: BColor,
    /// `TH_VERTEX` / `TH_VERTEX_SELECT`: the Graph editor's key dots.
    pub vertex: BColor,
    pub vertex_select: BColor,
    /// `TH_SCROLL_TEXT`: the value numbers.
    pub scale_text: BColor,
    /// `TH_HEADER` shaded +50: a muted curve.
    pub muted: BColor,
    /// `tui.xaxis / yaxis / zaxis / waxis`, and the colour past them.
    pub axis_x: BColor,
    pub axis_y: BColor,
    pub axis_z: BColor,
    pub axis_w: BColor,
    pub axis_other: BColor,
    /// The NLA editor: `TH_ANIM_ACTIVE` (an object's action line),
    /// `TH_ANIM_INACTIVE` (no action), `TH_NLA_TWEAK`; `TH_STRIP` /
    /// `TH_STRIP_SELECT`; a strip's outline, plain and active; the names on
    /// strips; the track backdrop in the channel region.
    pub action_line: BColor,
    pub action_none: BColor,
    pub tweak: BColor,
    pub strip: BColor,
    pub strip_select: BColor,
    pub strip_outline: BColor,
    pub strip_outline_active: BColor,
    pub strip_text: BColor,
    pub strip_text_active: BColor,
    pub track_back: BColor,
    /// `TH_SHADE2` (`acf_generic_channel_color`): an F-curve channel's backdrop.
    pub curve_back: BColor,
    /// `TH_HANDLE_FREE / _AUTO / _VECT / _ALIGN / _AUTOCLAMP`, indexed by
    /// `eBezTriple_Handle`, and their `_SEL_` counterparts.
    pub handle: [BColor; 5],
    pub handle_select: [BColor; 5],
    /// `TH_HANDLE_VERTEX` / `TH_HANDLE_VERTEX_SELECT`: the ring on a handle's end.
    pub handle_vertex: BColor,
    pub handle_vertex_select: BColor,
    /// `TH_TIME_MARKER_LINE`, `ICON_MARKER`'s shade and the name (`TH_TEXT`);
    /// selected: `_LINE_SELECTED`, `ICON_MARKER_HLT`, `TH_TEXT_HI`.
    pub marker_line: BColor,
    pub marker: BColor,
    pub marker_text: BColor,
    pub marker_line_selected: BColor,
    pub marker_selected: BColor,
    pub marker_text_selected: BColor,
}

impl BAnimColors {
    /// A colour as `[r, g, b, a]` in 0..1 — the form `BFill::Swatch` and
    /// the curve painter take.
    pub fn rgba(color: BColor) -> [f32; 4] {
        let rgba: gpui::Rgba = crate::v2::view::draw::color::to_hsla(color).into();
        [rgba.r, rgba.g, rgba.b, rgba.a]
    }

    pub fn rgb(color: BColor) -> [f32; 3] {
        let [r, g, b, _] = Self::rgba(color);
        [r, g, b]
    }
}

fn with_alpha(color: BColor, alpha: f32) -> BColor {
    (color.0, color.1, color.2, alpha)
}

impl BAnimColors {
    /// A colour `delta` lighter (or darker), for a line drawn on a surface.
    pub fn lift(color: BColor, delta: f32) -> BColor {
        (color.0, color.1, (color.2 + delta * 100.0).clamp(0.0, 100.0), color.3)
    }
}

impl BResolvedUiStyle {
    /// The animation editors' colours in this theme.
    pub fn anim(&self) -> BAnimColors {
        // The animation window is the canvas shade every other canvas is.
        let back = self.viewport_surface();
        let colors = &self.colors;
        // Free: warm; auto: the x axis's red; vector: the z axis's blue;
        // aligned: the y axis's green; auto-clamped: the accent.
        let handle_select = [colors.brand_warm, colors.axis_x, colors.axis_z, colors.axis_y, colors.accent];
        BAnimColors {
            back,
            grid: self.step(back, GRID_OVER_BACK),
            grid_line: BAnimColors::lift(self.step(back, GRID_OVER_BACK), GRID_LINE_LIFT),
            grid_minor: with_alpha(self.step(back, GRID_OVER_BACK), MINOR_ALPHA),
            out_of_range: with_alpha(self.step(back, CURTAIN_UNDER_BACK), CURTAIN_ALPHA),
            range_line: self.step(back, RANGE_LINE_UNDER_BACK),
            preview: with_alpha(colors.brand_warm, PREVIEW_ALPHA),
            scrub_back: colors.bg_header,
            scrub_text: colors.text_dim,
            playhead: colors.accent,
            playhead_shadow: colors.bg_app,
            // `TH_HEADER_TEXT_HI`, white in Blender's theme: the number sits
            // on the accent, and the palette's text — dark in a light mode —
            // vanished into it.
            playhead_text: (0.0, 0.0, 100.0, 1.0),
            cursor_value: with_alpha(self.shade(colors.accent, CURSOR_VALUE_SHADE), CURSOR_ALPHA),
            cursor_time: with_alpha(self.shade(colors.accent, CURSOR_TIME_SHADE), CURSOR_ALPHA),
            channel_back: colors.bg_left,
            channel_text: colors.text,
            summary_back: with_alpha(colors.bg_active, BACKDROP_ALPHA), // a surface, not the selection accent
            object_back: with_alpha(colors.row_alt, BACKDROP_ALPHA),
            key: colors.text,
            key_select: colors.brand_warm,
            key_border: colors.bg_app,
            vertex: colors.bg_app,
            vertex_select: colors.brand_warm,
            scale_text: colors.text,
            muted: colors.text_dim,
            axis_x: colors.axis_x,
            axis_y: colors.axis_y,
            axis_z: colors.axis_z,
            axis_w: colors.brand_warm,
            axis_other: colors.accent,
            action_line: with_alpha(colors.brand_warm, ACTION_ALPHA),
            action_none: with_alpha(self.shade(colors.brand_warm, NO_ACTION_UNDER), NO_ACTION_ALPHA),
            tweak: colors.online,
            strip: with_alpha(colors.bg_app, STRIP_ALPHA),
            strip_select: colors.brand_warm,
            strip_outline: colors.bg_app,
            strip_outline_active: colors.text,
            strip_text: colors.text,
            strip_text_active: colors.bg_app,
            track_back: with_alpha(colors.row_alt, BACKDROP_ALPHA),
            curve_back: with_alpha(colors.row_alt, BACKDROP_ALPHA),
            handle: handle_select.map(|color| self.shade(color, HANDLE_UNDER_SELECTED)),
            handle_select,
            handle_vertex: colors.bg_app,
            handle_vertex_select: colors.brand_warm,
            marker_line: with_alpha(colors.bg_app, MARKER_LINE_ALPHA),
            marker: colors.text_dim,
            marker_text: colors.text,
            marker_line_selected: with_alpha(colors.text, MARKER_LINE_SELECTED_ALPHA),
            marker_selected: colors.brand_warm,
            marker_text_selected: colors.text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::BState;
    use crate::v2::theme::{BPaletteName, resolve_ui_style, style::ui_style_defaults};

    #[test]
    fn the_editors_take_their_colours_from_the_palette_in_use() {
        let state = BState::new();
        let style = resolve_ui_style(ui_style_defaults(BPaletteName::Sphere), &state);
        let anim = style.anim();
        let viewport = style.viewport_surface();
        assert_eq!(anim.back, viewport, "the window is the canvas shade every canvas is");
        assert_eq!(style.colors.ladder.step_of(anim.grid), Some(11), "the grid is the step inward of the canvas");
        assert_eq!((anim.grid_minor.0, anim.grid_minor.1, anim.grid_minor.2), (anim.grid.0, anim.grid.1, anim.grid.2), "the minor lines are the grid's shade");
        assert!(anim.grid_minor.3 < anim.grid.3, "at half its alpha");
        assert_eq!(anim.playhead, style.colors.accent, "the playhead is the theme's accent");
        assert!(anim.cursor_time.2 < anim.cursor_value.2 && anim.cursor_value.2 < anim.playhead.2, "the cursor lines are the playhead darkened, the input line more");
        assert_eq!((anim.cursor_value.3, anim.cursor_time.3), (CURSOR_ALPHA, CURSOR_ALPHA));
        assert!(anim.handle.iter().zip(&anim.handle_select).all(|(plain, selected)| plain.0 == selected.0 && plain.2 < selected.2), "each handle type keeps its hue and darkens when unselected");
        assert_eq!(anim.handle_select[1], style.colors.axis_x, "auto handles take the x axis's red");
        assert_eq!(anim.vertex_select, style.colors.brand_warm);
        assert_eq!(anim.out_of_range.3, CURTAIN_ALPHA);
        let [r, g, b, a] = BAnimColors::rgba((0.0, 0.0, 100.0, 1.0));
        assert!((r - 1.0).abs() < 1e-6 && (g - 1.0).abs() < 1e-6 && (b - 1.0).abs() < 1e-6 && a == 1.0);
        let other = resolve_ui_style(ui_style_defaults(BPaletteName::Tokyo), &state).anim();
        assert_ne!(other.back, anim.back, "another palette gives the editors other colours");
    }
}
