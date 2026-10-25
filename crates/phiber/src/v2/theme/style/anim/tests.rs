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
