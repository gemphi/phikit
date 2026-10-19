//! Sub-panels are a tree, and shade is what says so. A nested accordion sits
//! on its parent's own row column — no extra indent, no guide down its left
//! edge — and is recessed a step toward the region, its heading a step over
//! its own well, so Properties' Sampling ▸ Render ▸ Denoise reads as three
//! levels in shade alone, in every theme.
//!
//! It used to step in by a `pad_sm` and carry a left guide. Both marked a
//! sub-panel out as a different kind of thing and left the space around it
//! uneven — wider on the left than the inset its parent's body gives every
//! other row. A nested accordion is an accordion: same card, one shade in.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState, ButsKey, CtxKey};
use phiber::v2::theme::{BPaletteName, BResolvedUiStyle, BSurfaceTone, ui_style_defaults};
use phiber::v2::view::prim::{BCard, BCardKind};

/// A card's selector: `card-<key>` for its frame, `card-<what>-<key>` for a
/// part of it.
fn id(what: &str, key: &str) -> &'static str {
    let part = if what.is_empty() { String::new() } else { format!("{what}-") };
    let key = if key.ends_with("-expanded-0") { key.to_string() } else { format!("{key}-expanded") };
    Box::leak(format!("card-{part}ctx-buts-render-{key}").into_boxed_str())
}

#[gpui::test]
async fn each_level_steps_in_by_one_indent_under_its_parents_rows(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(1000.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        app.state.set(ButsKey::NavbarTab, "Render");
        cx.notify();
    });
    cx.run_until_parked();

    let mut at = |what: &str, key: &str| {
        f32::from(cx.debug_bounds(id(what, key)).unwrap_or_else(|| panic!("{key} {what}")).origin.x)
    };
    // A card's first row, where its rows start.
    let sampling_rows = at("row", "sampling-expanded-0");
    let render = at("", "sampling_render");
    let render_rows = at("row", "sampling_render-expanded-0");
    let denoise = at("", "sampling_render_denoise");
    let viewport = at("", "sampling_viewport");
    // A sub-panel starts exactly where its parent's rows start, and its own
    // sub-panel exactly where *its* rows start: the body's inset is the only
    // space either side of it, so the gap around a nested accordion is the
    // same on all four sides.
    assert_eq!(render, sampling_rows, "Render starts on Sampling's row column");
    assert_eq!(denoise, render_rows, "Denoise starts on Render's row column");
    // Depth still reads left to right, because each level's rows are inset
    // inside the level above.
    assert!(denoise > render, "a grandchild sits right of its parent");
    // Siblings share a column.
    assert_eq!(viewport, render);
}

#[test]
fn a_card_pushed_into_a_sub_panel_moves_a_level_deeper_with_its_own() {
    let leaf = BCard::panel("Denoise").sub();
    let branch = BCard::panel("Render").body(vec![leaf.into()]).sub();
    assert_eq!(branch.kind, BCardKind::Sub { depth: 1 });
    let phiber::v2::view::BView::Card(leaf) = &branch.body[0] else { panic!("the leaf") };
    assert_eq!(leaf.kind, BCardKind::Sub { depth: 2 }, "the leaf went a level deeper with its branch");
    assert_eq!((branch.depth(), leaf.depth(), BCard::panel("Sampling").depth()), (1, 2, 0));
}

#[test]
fn each_level_is_recessed_a_step_and_its_heading_a_step_over_its_well() {
    for name in BPaletteName::ALL.iter().copied() {
        let style = BResolvedUiStyle::from(ui_style_defaults(name));
        let step = |color| style.colors.ladder.step_of(color).unwrap_or_else(|| panic!("{name:?}: {color:?} off the ladder"));
        let card = step(style.surface(BSurfaceTone::Card).bottom);
        let well = |depth| step(style.surface(BSurfaceTone::Nested(depth)).bottom);
        let heading = |tone| step(style.heading_surface(tone).bottom);
        assert_eq!(well(1), card, "{name:?}: a sub-panel shares its panel's shade — no darker strip");
        assert_eq!(well(2), card, "{name:?}: at every depth");
        // A heading's gradient STARTS on the card under it and sweeps a step
        // up from there: its bottom stop is its parent's own shade, so the
        // shading disappears into the parent and there is no seam across the
        // panel where the two meet. The step is the sweep's top, not a flat
        // band under it — taking the sheen over `step(surface, 1)` was saying
        // "heading" twice and paying for it with an edge.
        let top = |tone| step(style.heading_surface(tone).top);
        for (tone, well) in [(BSurfaceTone::Card, card), (BSurfaceTone::Nested(1), well(1)), (BSurfaceTone::Nested(2), well(2))] {
            assert_eq!(heading(tone), well, "{name:?}: a heading starts on the shade under it");
            let wanted = well + u8::from(style.effects.gradients);
            assert_eq!(top(tone), wanted, "{name:?}: and sweeps a step up when the theme sweeps");
        }
        let swept = style.heading_surface(BSurfaceTone::Nested(1));
        assert_eq!(swept.enabled, style.effects.gradients, "{name:?}: swept when the theme sweeps");
    }
}
