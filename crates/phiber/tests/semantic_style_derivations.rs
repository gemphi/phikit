//! The semantic derivations stay on the ladder: every surface, host pane,
//! strip, tab and hover a theme hands out is one of its eighteen shades, a
//! gradient runs one step up to the slot's own shade, a lift moves whole
//! steps, and a colour off the ladder (the accent) keeps a lightness nudge.

use phiber::v2::{
    host::BHostType,
    theme::{BGradientAxis, BPaletteName, BResolvedUiStyle, BShadowStyle, BSurfaceTone, SWEEP_STOP, ui_style_defaults},
};

fn style(name: BPaletteName) -> BResolvedUiStyle {
    BResolvedUiStyle::from(ui_style_defaults(name))
}

fn on_ladder(style: &BResolvedUiStyle, color: (f32, f32, f32, f32)) -> u8 {
    style.colors.ladder.step_of(color).unwrap_or_else(|| panic!("{color:?} is not a shade of {:?}", style.colors.ladder))
}

#[test]
fn every_surface_tone_is_a_shade_and_a_gradient_is_one_step_up() {
    for name in BPaletteName::ALL.iter().copied() {
        let style = style(name);
        let panel = on_ladder(&style, style.colors.bg_panel);
        // A header, a footer and a card are one shade: an editor's header
        // and the accordions under it read as one surface, and so does a
        // modal's title bar against its cards.
        let header = style.surface(BSurfaceTone::Header);
        assert_eq!(on_ladder(&style, header.bottom), panel + 1, "a header sits on the card's step");
        assert_eq!(
            style.surface(BSurfaceTone::Footer).bottom,
            header.bottom,
            "footers share the header shade"
        );
        assert_eq!(style.surface(BSurfaceTone::Sidebar).bottom, style.colors.bg_panel);
        assert_eq!(style.surface(BSurfaceTone::Body).bottom, style.colors.bg_panel);
        assert_eq!(on_ladder(&style, style.surface(BSurfaceTone::Card).bottom), panel + 1, "a card is one step over the panel");
        assert_eq!(on_ladder(&style, style.surface(BSurfaceTone::Section).bottom), panel + 1, "a section shares it, as Blender's sub-panel shares the panel's grey");
        // Inward is lighter in a dark theme and darker in a light one; the
        // steps are the same either way.
        let inward = |from: f32, to: f32| if style.colors.ladder.light { to < from } else { to > from };
        assert!(inward(style.colors.bg_header.2, style.colors.bg_panel.2), "{name:?}: a body is a step inward from its header");
        assert!(inward(style.surface(BSurfaceTone::Card).bottom.2, style.colors.bg_control.2), "{name:?}: a control is a step inward from the card it sits on");
        for tone in [BSurfaceTone::Header, BSurfaceTone::Footer, BSurfaceTone::Sidebar, BSurfaceTone::Body, BSurfaceTone::Modal, BSurfaceTone::Card, BSurfaceTone::Section] {
            let gradient = style.surface(tone);
            let bottom = on_ladder(&style, gradient.bottom);
            if gradient.enabled {
                assert_eq!(on_ladder(&style, gradient.top), bottom + 1, "{name:?} {tone:?}: a sweep is one step up");
            } else {
                assert_eq!(gradient.top, gradient.bottom);
            }
        }
        let viewport = style.surface(BSurfaceTone::Viewport);
        assert!(!viewport.enabled && viewport.top == style.colors.bg_viewport, "the canvas tone is flat and the viewport token");
        // Every sweep runs left to right and is back on its base by half way,
        // so the lit edge reads against a border and the surface keeps its
        // colour; the canvas sweeps the same way when the theme sweeps at all.
        for tone in [BSurfaceTone::Header, BSurfaceTone::Body, BSurfaceTone::Card] {
            let sweep = style.surface(tone);
            if sweep.enabled {
                assert_eq!((sweep.axis, sweep.stop), (BGradientAxis::Trailing, SWEEP_STOP), "{name:?} {tone:?}");
            } else {
                assert_eq!(sweep.stop, 1.0);
            }
        }
        let canvas = style.viewport_gradient();
        assert_eq!(canvas.enabled, style.effects.gradients, "{name:?}: the canvas sweeps when the theme does");
        assert_eq!(canvas.bottom, style.viewport_surface());
        if canvas.enabled {
            assert_eq!(on_ladder(&style, canvas.top), on_ladder(&style, canvas.bottom) + 1, "{name:?}: a step lighter at the left edge");
            assert!(style.viewport_sweep_lift() > 0.0 && style.viewport_sweep_lift() < 0.1, "{name:?}: the tracer lifts its sky by that step: {}", style.viewport_sweep_lift());
        } else {
            assert_eq!(style.viewport_sweep_lift(), 0.0);
        }
        // A heading is a step over what it heads, in every theme — swept
        // like every surface where gradients are on, flat where not — so a
        // panel's heading lifts off its card and a sub-panel's off its well.
        let heading = style.heading_surface(BSurfaceTone::Card);
        assert_eq!(on_ladder(&style, heading.bottom), panel + 2, "{name:?}: a heading is a step over the card");
        assert_eq!(heading.enabled, style.effects.gradients, "{name:?}: swept when the theme sweeps");
        assert_eq!(heading, style.sheen(heading.bottom), "{name:?}: the heading is its shade's sweep");
        // A sub-panel is a branch of its panel's tree: recessed a step toward
        // the region per level, its heading a step over its own well.
        assert_eq!(on_ladder(&style, style.surface(BSurfaceTone::Nested(1)).bottom), panel + 1, "{name:?}: a sub-panel's well is its panel's shade — no darker strip");
        assert_eq!(on_ladder(&style, style.surface(BSurfaceTone::Nested(2)).bottom), panel + 1, "{name:?}: at every depth");
        assert_eq!(on_ladder(&style, style.heading_surface(BSurfaceTone::Nested(1)).bottom), panel + 2, "{name:?}: its heading a step over it, as a panel's");
    }
}

#[test]
fn disabling_gradients_flattens_every_semantic_gradient_without_changing_base_color() {
    let mut style = style(BPaletteName::Sphere);
    style.effects.gradients = false;
    for tone in [BSurfaceTone::Header, BSurfaceTone::Footer, BSurfaceTone::Sidebar, BSurfaceTone::Body, BSurfaceTone::Viewport, BSurfaceTone::Modal, BSurfaceTone::Card, BSurfaceTone::Section] {
        let gradient = style.surface(tone);
        assert!(!gradient.enabled);
        assert_eq!(gradient.top, gradient.bottom);
    }
    let tab = style.tab_active_gradient(BHostType::Buts);
    assert!(!tab.enabled);
    assert_eq!(tab.top, tab.bottom);
}

#[test]
fn every_host_shares_the_panel_shade_and_the_chrome_its_own() {
    let style = style(BPaletteName::Tokyo);
    for host in [BHostType::View3D, BHostType::Buts, BHostType::Shader, BHostType::Outliner, BHostType::Time, BHostType::Image, BHostType::Text] {
        assert_eq!(style.host_background(host), style.colors.bg_panel, "{host:?} is the panel shade like every editor");
        assert_eq!(style.strip_gradient(host).bottom, style.colors.bg_header, "{host:?}'s strip is the header shade");
        assert_eq!(style.host_item_hover(host), style.colors.bg_hover);
        assert_eq!(style.host_item_active_gradient(host).bottom, style.colors.bg_active);
        // An editor's header is the card shade, not a shade of its own.
        assert_eq!(style.host_header_surface(host).bottom, style.surface(BSurfaceTone::Card).bottom);
    }
    assert_eq!(style.host_background(BHostType::Topbar), style.colors.bg_top);
    assert_eq!(style.host_background(BHostType::Statusbar), style.colors.bg_bottom);
    assert_eq!(style.host_background(BHostType::Activity), style.colors.bg_panel, "the activity bar is an area like any other: no darker shade");
    assert_eq!(style.host_body_surface(BHostType::Activity), style.host_body_surface(BHostType::View3D));
    assert_eq!(style.host_header_surface(BHostType::Topbar).bottom, style.colors.bg_top, "the top bar is its own chrome");
    // The active Properties tab is EXACTLY the surface of the region beside
    // it, flat. It used to sweep in from one step over the panel, which is a
    // gradient across the join between the tab and the panel it selects —
    // and a gradient on one side of that join makes the two read as two
    // things rather than as a hole cut in the strip
    // (`tests/the_properties_strip_is_a_strip.rs`).
    //
    // The panel it lands on is NOT `bg_panel` any more: the Properties body
    // takes the one step that an unselected tab used to draw as a chip, so the
    // strip is flat and the selected tab is the one carrying a shade. The claim
    // here is unchanged by that and is stated against the body rather than
    // against a token the body no longer equals.
    let tab = style.tab_active_gradient(BHostType::Buts);
    let panel = style.host_body_surface(BHostType::Buts);
    assert_eq!(tab.bottom, panel.bottom, "the active Properties tab lands on the panel");
    assert_eq!(tab.top, tab.bottom, "and is flat, so the join has no edge");
    assert_eq!(tab, panel, "it is the panel's own surface");
    assert_eq!(
        panel.bottom,
        style.step(style.colors.bg_panel, -1),
        "and the panel is the step under the strip, which is `bg_panel`",
    );
    let viewport = style.viewport_gradient();
    assert!(viewport.bottom == style.viewport_surface() && viewport.bottom.3 == 1.0);
    let node_grid = style.node_grid_stroke();
    assert_eq!(on_ladder(&style, (node_grid.color.0, node_grid.color.1, node_grid.color.2, 1.0)), on_ladder(&style, style.colors.bg_viewport) + 1, "the node grid is light: the step inward of the canvas");
    assert_eq!(node_grid.color.3, style.effects.node_grid_alpha);
}

#[test]
fn structural_strokes_sidebar_width_and_shadow_are_resolved_accessors() {
    let mut style = style(BPaletteName::Graphite);
    style.strokes.border = 2.0;
    style.strokes.hairline = 0.5;
    style.strokes.grid = 1.5;
    style.dimensions.sidebar_width = 180.0;
    style.dimensions.sidebar_min_width = 160.0;
    style.effects.surface_shadow = BShadowStyle::None;
    assert_eq!(style.border_stroke().width, 2.0);
    assert_eq!(style.border_stroke().color, style.colors.border);
    assert_eq!(style.separator_stroke().width, 0.5);
    assert_eq!(style.viewport_grid_stroke().width, 1.5);
    assert_eq!(style.node_grid_stroke().width, 1.5);
    assert_eq!(style.sidebar_min_width(), 180.0);
    assert_eq!(style.surface_shadow(), BShadowStyle::None);
}

#[test]
fn steps_stay_on_the_ladder_and_off_ladder_colours_nudge() {
    let style = style(BPaletteName::Sphere);
    let canvas = on_ladder(&style, style.colors.bg_viewport);
    let checker = style.image_checker_colors();
    assert_eq!(on_ladder(&style, checker.light), canvas);
    assert_eq!(on_ladder(&style, checker.dark), canvas - 1);
    // A selected row under the pointer is one step lighter than the selection
    // it already carries. `row_selected` is not always a ladder shade — in
    // several palettes it is an accent tint with its own alpha — so the rule
    // is stated the way the accent's is below: a step is a step whether the
    // colour it starts from is on the ladder or beside it.
    let selected = style.colors.row_selected;
    let lit = style.selected_hover(selected);
    match style.colors.ladder.step_of(selected) {
        Some(rung) => assert_eq!(on_ladder(&style, lit), rung + 1, "a ladder selection steps on the ladder"),
        None => {
            assert_eq!((lit.0, lit.1, lit.3), (selected.0, selected.1, selected.3), "an off-ladder selection keeps its hue, saturation and alpha");
            assert!(lit.2 > selected.2, "and lightens: {selected:?} -> {lit:?}");
        }
    }
    assert_eq!(on_ladder(&style, style.shade(style.colors.bg_panel, 0.06)), on_ladder(&style, style.colors.bg_panel) + 2, "a 0.06 lift is two steps");
    assert_eq!(style.shade(style.colors.bg_panel, 0.01), style.colors.bg_panel, "a lift under half a step is none");
    assert_eq!(on_ladder(&style, style.step(style.colors.text, 5)), 18, "steps clamp at the top");
    let accent = style.colors.accent;
    let nudged = style.step(accent, 1);
    assert_eq!((nudged.0, nudged.1), (accent.0, accent.1), "the accent keeps its hue and saturation");
    assert!((nudged.2 - accent.2 - 3.0).abs() < 1e-4, "and moves a step's worth of lightness");
}
