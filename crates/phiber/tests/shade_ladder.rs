//! The shade ladder is the palette: every surface, text and structural
//! token of every theme is one of its eighteen shades, the animation
//! editors' surfaces are too, the default ladder is the sphere's table to
//! the digit, and no renderer shades a colour by hand — the semantic layer
//! moves along the ladder for it.

use phiber::v2::theme::palette::shade::{SPHERE, set::SPHERE_TABLE};
use phiber::v2::theme::{BPaletteName, BResolvedUiStyle, BShadeLadder, palette_for, resolve_ui_style, ui_style_defaults};
use phiber::v2::state::BState;
use std::path::{Path, PathBuf};

fn expect_shade(ladder: &BShadeLadder, name: &str, color: (f32, f32, f32, f32)) -> u8 {
    ladder.step_of(color).unwrap_or_else(|| panic!("{name} = {color:?} is not a shade of the {ladder:?} ladder"))
}

#[test]
fn the_default_ladder_is_the_sphere_table_to_the_digit() {
    for (index, (hex, how)) in SPHERE_TABLE.iter().enumerate() {
        let ours = SPHERE.hex(index as u8 + 1);
        let channels = |text: &str| (0..3).map(|c| u8::from_str_radix(&text[1 + c * 2..3 + c * 2], 16).unwrap()).collect::<Vec<_>>();
        let gap: i16 = channels(&ours).iter().zip(channels(hex)).map(|(a, b)| (i16::from(*a) - i16::from(b)).abs()).max().unwrap();
        assert!(gap <= 1, "shade {} ({how}): {ours} against the table's {hex}", index + 1);
    }
}

#[test]
fn every_theme_token_is_a_shade_except_the_colours_that_mean_something() {
    for name in BPaletteName::ALL.iter().copied() {
        let palette = palette_for(name);
        let ladder = &palette.ladder;
        let surfaces = [
            ("bg_app", palette.bg_app, 2),
            ("bg_activity", palette.bg_activity, 3),
            ("bg_top", palette.bg_top, 3),
            ("bg_bottom", palette.bg_bottom, 3),
            ("bg_header", palette.bg_header, 4),
            ("bg_left", palette.bg_left, 5),
            ("bg_right", palette.bg_right, 5),
            ("bg_panel", palette.bg_panel, 5),
            ("bg_control", palette.bg_control, 7),
            ("bg_hover", palette.bg_hover, 8),
            ("bg_active", palette.bg_active, 9),
            ("bg_viewport", palette.bg_viewport, 10),
            ("text_dim", palette.text_dim, 12),
            ("text", palette.text, 16),
            ("border", palette.border, 9),
            ("sep", palette.sep, 2),
            ("row_alt", palette.row_alt, 6),
            ("viewport_grid", palette.viewport_grid, 11),
        ];
        for (token, color, step) in surfaces {
            assert_eq!(expect_shade(ladder, token, color), step, "{name:?}: {token} sits on the slot every theme gives it");
        }
        assert_eq!(ladder.step_of(palette.bg_modal_dim), Some(1), "{name:?}: the modal dim is shade 1 at an alpha");
        for (token, color) in [
            ("accent", palette.accent),
            ("axis_x", palette.axis_x),
            ("axis_y", palette.axis_y),
            ("wire_active", palette.wire_active),
            // Selection is the accent at an alpha, not a grey step. On the
            // ladder it had to share shade 8 with `bg_hover` and shade 9 with
            // `bg_active`, which made a hovered row look selected.
            ("row_selected", palette.row_selected),
            ("row_active", palette.row_active),
        ] {
            assert!(ladder.step_of(color).is_none(), "{name:?}: {token} carries meaning and stays off the ladder");
        }
        // The point of moving them off: a row's four states are four colours.
        let (accent_h, accent_s, accent_l, _) = palette.accent;
        for (token, color) in [("row_selected", palette.row_selected), ("row_active", palette.row_active)] {
            assert_eq!((color.0, color.1, color.2), (accent_h, accent_s, accent_l), "{name:?}: {token} is the accent's own hue");
        }
        assert!(palette.row_selected.3 < palette.row_active.3, "{name:?}: the active row is the stronger of the two");
        for (selection, control) in [
            ("row_selected vs bg_hover", palette.bg_hover),
            ("row_selected vs bg_active", palette.bg_active),
        ] {
            assert_ne!(palette.row_selected, control, "{name:?}: {selection} — selection must not be a control shade");
        }
        assert_ne!(palette.row_active, palette.bg_active, "{name:?}: the active row is not the active-control shade");
    }
}

#[test]
fn the_animation_editors_surfaces_are_shades_of_the_canvas() {
    for name in BPaletteName::ALL.iter().copied() {
        let style = resolve_ui_style(ui_style_defaults(name), &BState::new());
        let ladder = &style.colors.ladder;
        let anim = style.anim();
        assert_eq!(anim.back, style.viewport_surface(), "{name:?}: the window is the canvas");
        assert_eq!(expect_shade(ladder, "back", anim.back), 10);
        assert_eq!(expect_shade(ladder, "grid", anim.grid), 11, "the grid is light: the step inward of the canvas");
        assert_eq!(expect_shade(ladder, "grid_minor", anim.grid_minor), 11);
        assert_eq!(expect_shade(ladder, "scrub_back", anim.scrub_back), 4, "the scrub strip is a header");
        assert_eq!(expect_shade(ladder, "channel_back", anim.channel_back), 5, "the channel region is a body");
        // A body is one step inward from its header: lighter in a dark
        // theme, darker in a light one.
        if ladder.light {
            assert!(anim.channel_back.2 < anim.scrub_back.2, "{name:?}: a light theme's body is darker than its header");
        } else {
            assert!(anim.channel_back.2 > anim.scrub_back.2, "{name:?}: a body is lighter than its header");
        }
        assert_eq!(expect_shade(ladder, "out_of_range", anim.out_of_range), 7);
        assert_eq!(expect_shade(ladder, "range_line", anim.range_line), 6);
        for (token, color) in [("scale_text", anim.scale_text), ("key", anim.key), ("channel_text", anim.channel_text), ("muted", anim.muted), ("vertex", anim.vertex), ("key_border", anim.key_border), ("track_back", anim.track_back), ("curve_back", anim.curve_back), ("summary_back", anim.summary_back), ("object_back", anim.object_back), ("strip", anim.strip)] {
            expect_shade(ladder, token, color);
        }
        assert!(ladder.step_of(anim.playhead).is_none(), "the playhead is the accent");
    }
}

/// Every `.rs` under `dir`, recursively.
fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_renderer_shades_a_colour_by_hand() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2");
    let mut files = Vec::new();
    for dir in ["view", "part", "host", "app"] {
        sources(&root.join(dir), &mut files);
    }
    let offenders: Vec<String> = files
        .iter()
        .filter(|path| !path.components().any(|c| c.as_os_str() == "tests" || c.as_os_str() == "tests.rs"))
        .filter_map(|path| {
            let text = std::fs::read_to_string(path).ok()?;
            let hit = text.lines().enumerate().find(|(_, line)| {
                let code = line.split("//").next().unwrap_or("");
                code.contains(".shade(") || code.contains("host_lightness(")
            })?;
            Some(format!("{}:{}: {}", path.strip_prefix(&root).unwrap().display(), hit.0 + 1, hit.1.trim()))
        })
        .collect();
    assert!(offenders.is_empty(), "renderers take shades from the semantic layer, they do not shade colours themselves:\n  {}", offenders.join("\n  "));
    // The layer itself moves in whole steps.
    let style: BResolvedUiStyle = resolve_ui_style(ui_style_defaults(BPaletteName::DEFAULT), &BState::new());
    let stepped = style.step(style.colors.bg_panel, 2);
    assert_eq!(style.colors.ladder.step_of(stepped), Some(7), "two steps over the body is the control shade");
}
