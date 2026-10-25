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
/// The rule a content header closes with, drawn below its band.
const RULE: f32 = phiber::v2::theme::style::metric::DEFAULT_STROKES.border;

/// A strip's one centre line: the centre of its cap band — the strip less
/// the rule under it and the menu gap above it. Every control sits on it.
fn line(strip: gpui::Bounds<px_t>) -> f32 {
    let band = f32::from(strip.size.height) - RULE;
    // The band's own centre: every control clears its top and bottom alike.
    f32::from(strip.origin.y) + band / 2.0
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
    // The header and the path strip are content headers: each is its band
    // plus the rule it closes with. The execute bar heads nothing — it is a
    // strip at the dialog's foot — so it is the band alone.
    for (name, strip) in [("header", header), ("path strip", path)] {
        assert_eq!(
            f32::from(strip.size.height),
            METRICS.h_header + RULE,
            "the {name} is a header's band plus its rule"
        );
    }
    assert_eq!(f32::from(execute.size.height), METRICS.h_header, "the execute bar is a header's height");
    assert_eq!(top(path), bottom(header), "the path strip sits directly under the header");

    let files = bounds(cx, "panel-body-surface");
    let sources = bounds(cx, "panel-sidebar-column");
    // A browser's columns stand a seam from everything round them, the gap
    // between the columns themselves (`region::pad`).
    let seam = phiber::v2::theme::ui_style_defaults(phiber::v2::theme::BPaletteName::DEFAULT).resize.gap;
    assert_eq!(top(files) - bottom(path), seam, "the file list sits a seam under the strips");
    assert_eq!(top(sources), top(files), "the source list and the file list start on one line");
    assert_eq!(bottom(sources), bottom(files), "and end on one line");
    assert_eq!(top(execute) - bottom(files), seam, "the execute bar sits a seam below them");
}

#[gpui::test]
async fn the_fields_and_buttons_sit_on_the_strips_centre_line(cx: &mut TestAppContext) {
    let cx = dialog(cx, false).await;
    let header = bounds(cx, "region-Header-File");
    let path = bounds(cx, "region-ToolHeader-File");
    // The header is the columns under it read across, as Explorer's address
    // bar: the directory as breadcrumbs over the file list, the search over
    // the details column.
    let directory = bounds(cx, "crumbs");
    let search = bounds(cx, "btn-label-set-ctx.edit-ctx.file.search-text");
    let up = bounds(cx, "btn-icon-set-ctx.file.nav-file.parent-arrow_up");
    for (name, control) in [("directory", directory), ("search", search)] {
        assert!((mid(control) - line(header)).abs() < 0.51, "the {name} control is centred on the header's line");
    }
    assert!((mid(up) - line(path)).abs() < 0.51, "the folder moves are centred on the strip's line");
    // The strip's right pane is the details column: the display modes, the
    // filter toggle and its popover and the region gear spread across it,
    // from where the search over them starts.
    let radio = bounds(cx, "btn-icon-set-ctx.file.display-Vertical List-list");
    let details = bounds(cx, "panel-info-column");
    assert!((mid(radio) - line(path)).abs() < 0.51, "the display-mode radio sits on the path strip's line");
    assert!((f32::from(radio.origin.x) - f32::from(search.origin.x)).abs() < 0.51, "and starts where the search does");
    assert!((f32::from(search.origin.x) - f32::from(details.origin.x)).abs() < 0.51, "the search starts where the details column's cards do");
    // A header toggle is a tile, as every button on a strip is: the button
    // less its inset, so its fill clears the strip's edges. (The editor
    // switcher beside it is a menu cap, which meets the strip's bottom edge
    // for its dropdown to open out of, as a tab does.)
    let tile = phiber::v2::theme::style::metric::DEFAULT_DIMENSIONS.tile_size();
    assert_eq!(f32::from(radio.size.height), tile, "the display-mode toggles are tiles");
    // The strip reads as the columns under it: the folder tools (refresh,
    // then New Folder, as Blender's) end where the source list does, and the
    // directory field starts where the file list does.
    let tools = bounds(cx, "region-Tools-File");
    let list = bounds(cx, "panel-body-surface");
    let refresh = bounds(cx, "btn-icon-set-ctx.file.nav-file.refresh-refresh");
    let new_folder = bounds(cx, "btn-icon-set-ctx.file.nav-file.directory_new-folder_plus");
    assert!(new_folder.origin.x > refresh.origin.x, "New Folder follows the refresh");
    let pane = bounds(cx, "pane-Left-File");
    let source = bounds(cx, "panel-sidebar-column");
    assert_eq!(pane.origin.x, tools.origin.x, "the strip's left pane is the tools region");
    assert_eq!(pane.size.width, tools.size.width);
    let seam = phiber::v2::theme::ui_style_defaults(phiber::v2::theme::BPaletteName::DEFAULT).resize.gap;
    assert!((f32::from(pane.origin.x) + seam - f32::from(source.origin.x)).abs() < 0.51, "the folder moves start where the source list does");
    assert!((f32::from(new_folder.origin.x) + f32::from(new_folder.size.width) - (f32::from(source.origin.x) + f32::from(source.size.width))).abs() < 0.51, "the folder tools end where the source list does");
    assert!((f32::from(directory.origin.x) - f32::from(list.origin.x)).abs() < 0.51, "the directory field starts where the file list does");
    assert!(f32::from(tools.origin.x) + f32::from(tools.size.width) <= f32::from(list.origin.x) + 0.51);
    assert!(
        f32::from(directory.size.width) > 120.0 && f32::from(search.size.width) > 120.0,
        "the directory and the search take the strip's slack"
    );
    let execute = bounds(cx, "region-Execute-File");
    // Opening names no file, so the bar leads with what is picked, not a
    // name field (`file::execute`); Cancel stands in the right group.
    let cancel = bounds(cx, "btn-label-set-ctx.modal--text");
    // The execute bar carries no menu caps, so it has no cap band: its
    // controls centre on the bar itself.
    assert!((mid(cancel) - mid(execute)).abs() < 0.51, "Cancel is centred on the bar");
    let centre = f32::from(execute.origin.x) + f32::from(execute.size.width) / 2.0;
    assert!(f32::from(cancel.origin.x) > centre, "and stands right of its centre, in the right group");
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
    // Its top is the heading row's: the heading itself is a tile inset
    // within that row (`sort.rs`), so its own top sits below the line.
    let heading_row = bounds(cx, "table-header");
    let path = bounds(cx, "region-ToolHeader-File");
    [
        f32::from(source.origin.x) - f32::from(tools.origin.x),
        f32::from(list.origin.x) - (f32::from(source.origin.x) + f32::from(source.size.width)),
        top(heading_row) - bottom(path),
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
    let edge = phiber::v2::theme::ui_style_defaults(phiber::v2::theme::BPaletteName::DEFAULT).resize.gap;
    assert_eq!(in_dialog[0], edge, "the source cards sit a seam in from the host");
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

/// The dialog and the editor area are ONE browser, differing only in where
/// it is shown.
///
/// `BFileBrowserModal::build` raises `BHostItem::new(BHostType::File)` — the
/// same host the workspace lays out — so the regions, their order and their
/// rules come from one declaration. The only thing that knows the difference
/// is the execute bar, and only about Cancel: in the dialog it closes the
/// dialog, in an area it drops the purpose so the bar hides, which is
/// Blender's own split (`file.cancel` in an area only clears
/// `FileSelectParams`). This pins that, so a second browser cannot grow
/// quietly out of the first.
#[test]
fn the_dialog_and_the_area_are_the_same_browser() {
    use phiber::v2::host::{BHostItem, BHostType};
    use phiber::v2::modal::file_browser::BFileBrowserModal;
    use phiber::v2::view::layout::BRenderer;
    use phiber::v2::view::text::BAsciiRenderer;
    use phiber::v2::view::{BView, BViewItem};

    let mut s = BState::new();
    s.set(CtxKey::Space, "layout");
    s.set(ModalKey::Open, V_MODAL_FILE_BROWSER);

    // The view tree as text, which the ASCII renderer already writes for
    // every node — a shape the two can be compared by.
    let shape = |v: &BView| BAsciiRenderer.render(v, &s);
    let via_modal = BFileBrowserModal.build(&s);
    let via_host = BHostItem::new(BHostType::File).build(&s);
    assert_eq!(
        shape(&via_modal),
        shape(&via_host),
        "the dialog builds the File host, not a browser of its own",
    );
    assert!(!shape(&via_modal).is_empty(), "and it builds something");
}
