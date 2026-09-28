//! The file browser dialog, measured on the painted UI.
//!
//! The dialog raises the File host, so what it shows is the host's regions:
//! a header, a path strip and an execute bar of one height, a source list
//! column beside the file list, and the content gap under the strips — the
//! same rules a workspace lays the editor out by, not a second set.

use phiber::v2::state::{BKey as _, CtxKey, ModalKey};
use phiber::v2::state::FileKey;
use gpui::{Pixels as px_t, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::state::{
    BState,
    val::{V_MODAL_FILE_BROWSER, V_TRUE},
};
use phiber::v2::theme::METRICS;

async fn dialog(cx: &mut TestAppContext, filter_open: bool) -> &mut gpui::VisualTestContext {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, "layout");
        app.state.set(ModalKey::Open, V_MODAL_FILE_BROWSER);
        if filter_open {
            app.state.set(&FileKey::FilterOpen.path(), V_TRUE);
        }
        cx.notify();
    });
    cx.run_until_parked();
    cx
}

fn bounds(cx: &mut gpui::VisualTestContext, id: &'static str) -> gpui::Bounds<px_t> {
    cx.debug_bounds(id).unwrap_or_else(|| panic!("{id} was not painted"))
}

fn top(b: gpui::Bounds<px_t>) -> f32 {
    f32::from(b.origin.y)
}
fn bottom(b: gpui::Bounds<px_t>) -> f32 {
    f32::from(b.origin.y) + f32::from(b.size.height)
}
/// A strip's one centre line: the centre of its cap band — the strip less
/// the menu gap above it. Every control in a strip sits on it.
fn line(strip: gpui::Bounds<px_t>) -> f32 {
    f32::from(strip.origin.y) + METRICS.menu_top_gap + (f32::from(strip.size.height) - METRICS.menu_top_gap) / 2.0
}

fn mid(b: gpui::Bounds<px_t>) -> f32 {
    f32::from(b.origin.y) + f32::from(b.size.height) / 2.0
}

#[gpui::test]
async fn the_strips_are_one_height_and_the_content_sits_a_gutter_below(cx: &mut TestAppContext) {
    let cx = dialog(cx, false).await;
    let header = bounds(cx, "region-Header-File");
    let path = bounds(cx, "region-ToolHeader-File");
    let execute = bounds(cx, "region-Execute-File");
    for (name, strip) in [("header", header), ("path strip", path), ("execute bar", execute)] {
        assert_eq!(f32::from(strip.size.height), METRICS.h_header, "the {name} is a header's height");
    }
    assert_eq!(top(path), bottom(header), "the path strip sits directly under the header");

    let files = bounds(cx, "panel-body-surface");
    let sources = bounds(cx, "panel-sidebar-column");
    assert_eq!(top(files) - bottom(path), METRICS.pad_md, "the file list sits a gutter under the strips");
    assert_eq!(top(sources), top(files), "the source list and the file list start on one line");
    assert_eq!(bottom(sources), bottom(files), "and end on one line");
    assert_eq!(top(execute) - bottom(files), METRICS.pad_md, "the execute bar sits a gutter below them");
}

#[gpui::test]
async fn the_fields_and_buttons_sit_on_the_strips_centre_line(cx: &mut TestAppContext) {
    let cx = dialog(cx, false).await;
    let path = bounds(cx, "region-ToolHeader-File");
    let directory = bounds(cx, "btn-label-set-ctx.edit-ctx.file.path-text");
    let search = bounds(cx, "btn-label-set-ctx.edit-ctx.file.search-text");
    let up = bounds(cx, "btn-icon-set-ctx.file.nav-file.parent-arrow_up");
    for (name, control) in [("directory", directory), ("search", search), ("up", up)] {
        assert!((mid(control) - line(path)).abs() < 0.51, "the {name} control is centred on the strip's line");
    }
    // The display-mode radio sits in the header's right pane; the
    // hidden-files eye, the filter toggle and its popover in the path
    // strip's right pane — each centred on its strip, right of the fields.
    let header = bounds(cx, "region-Header-File");
    let radio = bounds(cx, "btn-icon-set-ctx.file.display-Vertical List-list");
    assert!((mid(radio) - line(header)).abs() < 0.51, "the display-mode radio sits on the header's line");
    // A header toggle is a button like the switcher beside it: one height.
    let switcher = bounds(cx, "btn-label-toggle-ctx.switcher.file_browser.Center.file-text");
    assert_eq!(radio.size.height, switcher.size.height, "the display-mode toggles are as tall as the editor switcher");
    // The strip reads as the columns under it: the refresh ends where the
    // source list does, and the directory field starts where the file list does.
    let tools = bounds(cx, "region-Tools-File");
    let list = bounds(cx, "panel-body-surface");
    let refresh = bounds(cx, "btn-icon-set-ctx.file.nav-file.refresh-refresh");
    let pane = bounds(cx, "pane-Left-File");
    let source = bounds(cx, "panel-sidebar-column");
    assert_eq!(pane.origin.x, tools.origin.x, "the strip's left pane is the tools region");
    assert_eq!(pane.size.width, tools.size.width);
    assert!((f32::from(pane.origin.x) + METRICS.pad_md - f32::from(source.origin.x)).abs() < 0.51, "the folder moves start where the source list does");
    assert!((f32::from(refresh.origin.x) + f32::from(refresh.size.width) - (f32::from(source.origin.x) + f32::from(source.size.width))).abs() < 0.51, "the refresh ends where the source list does");
    assert!((f32::from(directory.origin.x) - f32::from(list.origin.x)).abs() < 0.51, "the directory field starts where the file list does");
    assert!(f32::from(tools.origin.x) + f32::from(tools.size.width) <= f32::from(list.origin.x) + 0.51);
    let hidden = bounds(cx, "btn-icon-set-ctx.file.show_hidden-true-eye_off");
    assert!((mid(hidden) - line(path)).abs() < 0.51, "the hidden-files eye is centred on the path strip's line");
    assert!(f32::from(hidden.origin.x) > f32::from(search.origin.x) + f32::from(search.size.width), "and right of the search");
    assert!(
        f32::from(directory.size.width) > 120.0 && f32::from(search.size.width) > 120.0,
        "the directory and the search take the strip's slack"
    );
    let execute = bounds(cx, "region-Execute-File");
    let filename = bounds(cx, "btn-label-set-ctx.edit-ctx.file.filename-text");
    // The execute bar carries no menu caps, so it has no cap band: its
    // controls centre on the bar itself.
    assert!((mid(filename) - mid(execute)).abs() < 0.51, "the file name field is centred on the bar");
}

#[gpui::test]
async fn the_filter_popover_lists_menu_rows_from_its_left_edge(cx: &mut TestAppContext) {
    let cx = dialog(cx, true).await;
    let panel = bounds(cx, "popover-panel");
    let hidden = bounds(cx, "menu-Show Hidden Files");
    let folders = bounds(cx, "menu-Folders");
    assert_eq!(f32::from(hidden.origin.x), f32::from(folders.origin.x), "every row starts on one line");
    let inset = f32::from(hidden.origin.x) - f32::from(panel.origin.x);
    assert!(inset > 0.0 && inset < METRICS.pad_md * 2.0, "rows run from the panel's left edge, inset {inset}");
    assert_eq!(
        f32::from(hidden.size.width),
        f32::from(folders.size.width),
        "rows are the panel's width, not each its own"
    );
}

/// Where the file browser's parts sit relative to its host: the source
/// column's inset from the host's left edge, the one gutter between the
/// source cards and the file list, the list's inset from the host's right,
/// and the tops of both columns.
fn geometry(cx: &mut gpui::VisualTestContext) -> [f32; 5] {
    let tools = bounds(cx, "region-Tools-File");
    let source = bounds(cx, "panel-sidebar-column");
    // The file list's first column heading: `panel-body-surface` is not
    // unique once other editors share the window.
    let list = bounds(cx, "file-sort-name");
    let path = bounds(cx, "region-ToolHeader-File");
    [
        f32::from(source.origin.x) - f32::from(tools.origin.x),
        f32::from(list.origin.x) - (f32::from(source.origin.x) + f32::from(source.size.width)),
        top(list) - bottom(path),
        top(source) - bottom(path),
        f32::from(source.size.width) + (f32::from(source.origin.x) - f32::from(tools.origin.x)),
    ]
}

/// The source list's cards are the surfaces: its column draws no frame
/// round them, sits one gutter in from the host, and one gutter — not two —
/// from the file list. And a workspace area showing the File Browser lays
/// it out exactly as the dialog does: one layout, whatever holds the host.
#[gpui::test]
async fn the_columns_are_one_gutter_apart_and_a_workspace_area_matches_the_dialog(cx: &mut TestAppContext) {
    let dialog_cx = dialog(cx, false).await;
    let in_dialog = geometry(dialog_cx);
    assert_eq!(in_dialog[0], METRICS.pad_md, "the source cards sit one gutter in from the host");
    // The two columns are a seam apart — the gap between two areas of a
    // workspace — not a content pad, which spaced them twice as far.
    let seam = phiber::v2::theme::ui_style_defaults(phiber::v2::theme::BPaletteName::DEFAULT).resize.gap;
    let source = bounds(dialog_cx, "panel-sidebar-column");
    let files = bounds(dialog_cx, "panel-body-surface");
    let apart = f32::from(files.origin.x) - (f32::from(source.origin.x) + f32::from(source.size.width));
    assert!((apart - seam).abs() < 0.51, "the source cards and the file list are a zone seam apart: {apart}");
    // And the list ends as far in from the host's right edge as the source
    // cards start from its left: nothing stands beside the list.
    let host = bounds(dialog_cx, "region-Center-File");
    let right = f32::from(host.origin.x) + f32::from(host.size.width) - (f32::from(files.origin.x) + f32::from(files.size.width));
    assert!((right - in_dialog[0]).abs() < 0.51, "the list's right inset {right} is the source cards' left one {}", in_dialog[0]);
    assert!((in_dialog[2] - in_dialog[3]).abs() <= 1.0, "both columns start on one line: {in_dialog:?}");

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, "layout");
        app.state.set(&phiber::v2::state::AreaKey::Host("Layout", "Center", "view3d").path(), "file");
        cx.notify();
    });
    cx.run_until_parked();
    let in_area = geometry(cx);
    for (n, (a, d)) in in_area.iter().zip(in_dialog.iter()).enumerate() {
        assert!((a - d).abs() < 0.51, "measure {n}: the area {in_area:?} and the dialog {in_dialog:?} lay the host out alike");
    }
}
