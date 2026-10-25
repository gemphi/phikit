//! End to end: in Preferences the category rail and the content beside it
//! share one grid. The rail's first cap sits level with the first card
//! heading — and, under Themes, with the first theme row, so *Themes* lines
//! up with *Graphite* — every cap is the button height, and the content
//! keeps the module's gap under the header.

use gpui::{Bounds, Pixels, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::state::{BPath, ModalKey, V_MODAL_SETTINGS};
use phiber::v2::theme::{BPaletteName, ui_style_defaults};

/// Show Preferences on `category`.
fn show(app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext, category: &str) {
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, V_MODAL_SETTINGS);
        app.state.set(&BPath::owned(vec![BHostType::Prefs.slug(), "navbar", "tab"]), category);
        cx.notify();
    });
    cx.run_until_parked();
}

fn top(bounds: &Bounds<Pixels>) -> f32 {
    f32::from(bounds.origin.y)
}

#[gpui::test]
async fn the_category_rail_lines_up_with_the_content_rows(cx: &mut TestAppContext) {
    let style = ui_style_defaults(BPaletteName::DEFAULT);
    // The card-head module: a heading, a row and a rail cap are all the
    // button plus its head room.
    let module = style.dimensions.card_head_height();

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));

    // Under Interface the first cap sits level with the first card heading.
    show(&app, cx, "Interface");
    let interface = cx.debug_bounds("tab-interface-Interface").expect("the Interface cap");
    let heading = cx.debug_bounds("card-head-ctx-prefs-interface-typography-expanded").expect("the first heading");
    assert_eq!(top(&interface), top(&heading), "the first cap sits level with the first heading");
    assert_eq!(f32::from(interface.size.height), module, "a cap is the card-head module");
    assert_eq!(f32::from(heading.size.height), module, "and so is a heading");

    // The content keeps the module's gap under the title bar — the region's
    // pad — while the rail runs flush, as Blender's navigation bar does.
    let region = cx.debug_bounds("region-Center-Prefs").expect("the content region");
    let rail = cx.debug_bounds("region-NavBar-Prefs").expect("the category rail");
    let body = cx.debug_bounds("panel-body-surface").expect("the content column");
    assert_eq!(top(&body) - top(&region), style.spacing.pad_md, "the content sits a content pad under the title bar");
    assert_eq!(top(&rail), top(&region), "the rail region starts where the content region does");
    assert_eq!(top(&interface) - top(&rail), style.spacing.pad_md + style.strokes.border + style.spacing.pad_sm, "the first cap sits the content's own inset down");

    // Under Themes the Theme section leads: its heading beside Interface,
    // its first theme row beside Themes — one row pitch down.
    show(&app, cx, "Appearance");
    let heading = cx.debug_bounds("card-head-ctx-prefs-appearance-theme-expanded").expect("the Theme section's heading");
    // The themes come in groups: the first group's heading is the first row.
    let first_row = cx.debug_bounds("card-head-ctx-prefs-appearance-theme_studio-expanded").expect("the first theme group");
    let interface = cx.debug_bounds("tab-interface-Interface").unwrap();
    let themes = cx.debug_bounds("tab-theme-Appearance").unwrap();
    assert_eq!(top(&interface), top(&heading), "Interface beside the Theme heading");
    assert_eq!(top(&themes), top(&first_row), "Appearance beside the first theme");
    assert_eq!(top(&themes) - top(&interface), top(&first_row) - top(&heading), "one pitch for caps and rows");
    assert_eq!(f32::from(first_row.size.height), module, "a theme row is the card-head module");
    // The open group lists its themes, at most eight.
    let listed = cx.all_debug_bounds().into_iter().filter(|(selector, _)| selector.starts_with("card-row-ctx-prefs-appearance-theme_studio-expanded-")).count();
    assert!(listed > 0 && listed <= 8, "the open group lists its themes: {listed}");
    let _ = BPaletteName::ALL;
}
