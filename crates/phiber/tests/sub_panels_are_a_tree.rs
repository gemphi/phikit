//! Sub-panels are a tree, as the Outliner's rows are: each level steps in
//! by the same indent under its parent's rows, carries a guide down its
//! left edge, and is recessed a step toward the region — its heading a step
//! over its own well — so Properties' Sampling ▸ Render ▸ Denoise reads as
//! three levels in indent and in shade, in every theme.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState, ButsKey, CtxKey};
use phiber::v2::theme::{BPaletteName, BResolvedUiStyle, BSurfaceTone, METRICS, ui_style_defaults};
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
    let indent = METRICS.pad_sm;
    // A sub-panel sits one indent in from where its parent's rows start; its
    // own sub-panel one indent in from where *its* rows start.
    assert_eq!(render - sampling_rows, indent, "Render steps in under Sampling's rows");
    assert_eq!(denoise - render_rows, indent, "Denoise steps in under Render's rows");
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
        assert_eq!(heading(BSurfaceTone::Card), card + 1, "{name:?}: a panel heading is a step over its card");
        assert_eq!(heading(BSurfaceTone::Nested(1)), well(1) + 1, "{name:?}: a sub-panel heading a step over its well");
        assert_eq!(heading(BSurfaceTone::Nested(2)), well(2) + 1, "{name:?}: at every level");
        let swept = style.heading_surface(BSurfaceTone::Nested(1));
        assert_eq!(swept.enabled, style.effects.gradients, "{name:?}: swept when the theme sweeps");
    }
}
