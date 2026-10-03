//! One rule, one place — the invariants behind this round of layout fixes.
//!
//! Each test here pins a rule that used to be re-declared per site, which is
//! how the sites drifted apart in the first place:
//!   • a menu closes everything it opened,
//!   • the window controls are the same three buttons everywhere,
//!   • an expanded rail entry fills the rail,
//!   • a side area scrolls only when its host stacks content.

use phiber::v2::state::{BKey as _, CtxKey, ModalKey};
use phiber::v2::state::FileKey;
use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState, V_TRUE, MenuKey, menu};

// ── Menus ─────────────────────────────────────────────────────────────────

#[test]
fn opening_a_menu_closes_the_submenu_another_one_left_open() {
    let host = BHostType::Topbar.slug();
    let (file, sub, edit) = (
        MenuKey::Top(host, "File").path(),
        MenuKey::Nested(host, &["File", "Import"]).path(),
        MenuKey::Top(host, "Edit").path(),
    );
    let mut s = BState::new();
    menu::apply_toggle(&mut s, &file);
    menu::apply_open(&mut s, &sub);
    assert_eq!(s.get(&sub), Some(V_TRUE), "the submenu is open to begin with");

    menu::apply_toggle(&mut s, &edit);
    assert_ne!(s.get(&file), Some(V_TRUE), "the old menu closes");
    assert_ne!(
        s.get(&sub),
        Some(V_TRUE),
        "and takes its submenu with it — otherwise the submenu's popover \
         stays deployed, stacked over the menu that was just opened"
    );
}

#[test]
fn closing_a_menu_by_clicking_it_again_closes_its_submenu_too() {
    let host = BHostType::Topbar.slug();
    let (file, sub) = (MenuKey::Top(host, "File").path(), MenuKey::Nested(host, &["File", "Import"]).path());
    let mut s = BState::new();
    menu::apply_toggle(&mut s, &file);
    menu::apply_open(&mut s, &sub);
    menu::apply_toggle(&mut s, &file);
    assert_ne!(s.get(&sub), Some(V_TRUE));
}

// ── Window controls ───────────────────────────────────────────────────────

#[test]
fn the_window_controls_are_one_table() {
    use phiber::v2::view::draw::caption::WINDOW_CONTROLS;
    let slugs: Vec<&str> = WINDOW_CONTROLS.iter().map(|c| c.slug).collect();
    assert_eq!(slugs, vec!["min", "max", "close"]);
    // Only the maximise control changes icon with the surface's state, and
    // only the close control gets the red hover.
    for ctl in WINDOW_CONTROLS {
        let changes = ctl.icon(false) != ctl.icon(true);
        assert_eq!(
            changes,
            ctl.slug == "max",
            "{} must{} swap its icon when maximised",
            ctl.slug,
            if ctl.slug == "max" { "" } else { " not" }
        );
        assert_eq!(ctl.closes(), ctl.slug == "close");
    }
}

// ── Rails ─────────────────────────────────────────────────────────────────

fn bounds(cx: &mut gpui::VisualTestContext, id: &'static str) -> gpui::Bounds<gpui::Pixels> {
    cx.debug_bounds(id)
        .unwrap_or_else(|| panic!("{id} was not painted"))
}

async fn workspace(cx: &mut TestAppContext, space: BSpaceType) -> &mut gpui::VisualTestContext {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, space.slug());
        app.state.set(
            &phiber::v2::part::strip::expanded_key(BHostType::View3D, "tool"),
            V_TRUE,
        );
        cx.notify();
    });
    cx.run_until_parked();
    cx
}

#[gpui::test]
async fn an_expanded_rail_entry_fills_the_rail(cx: &mut TestAppContext) {
    let cx = workspace(cx, BSpaceType::Layout).await;
    let pane = bounds(cx, "pane-floating-Left-View3D");
    let entry = bounds(cx, "tab-cursor-Cursor");
    // Expanded, a toolbar button is the rail's width less its own gutter —
    // never a small centred pill in the middle of a wide strip.
    let slack = f32::from(pane.size.width) - f32::from(entry.size.width);
    assert!(
        slack < f32::from(pane.size.width) / 2.0,
        "an expanded rail entry must take the rail's width: rail={:?} entry={:?}",
        pane.size.width,
        entry.size.width
    );
}

// ── Header cells ──────────────────────────────────────────────────────────

#[gpui::test]
async fn the_workspace_tabs_stay_put_when_the_space_changes(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    let mut centres = Vec::new();
    for space in [BSpaceType::Layout, BSpaceType::Shading, BSpaceType::Scripting] {
        app.update(cx, |app, cx| {
            app.state = BState::new();
            app.state.set(CtxKey::Space, space.slug());
            cx.notify();
        });
        cx.run_until_parked();
        let cell = bounds(cx, "pane-Center-Topbar");
        centres.push(f32::from(cell.origin.x) + f32::from(cell.size.width) / 2.0);
    }
    // The top bar divides its width three ways, so the centre cell sits on the
    // window's centre line whatever the menus to its left happen to be. With
    // hugging ends it slid sideways on every space change.
    for c in &centres {
        assert!(
            (c - centres[0]).abs() < 0.5,
            "the workspace tabs must not move between spaces: {centres:?}"
        );
    }
}

// ── Content rows ──────────────────────────────────────────────────────────

#[gpui::test]
async fn the_file_browsers_columns_run_the_full_height(cx: &mut TestAppContext) {
    
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        app.state.set(ModalKey::Open, "file_browser");
        cx.notify();
    });
    cx.run_until_parked();

    let body = bounds(cx, "panel-body-surface");
    let column = bounds(cx, "panel-sidebar-column");
    // A content row holds columns, not controls: the bookmarks column runs
    // the height of the row beside the file table, instead of floating at
    // whatever height its own panels happen to be.
    assert!(
        f32::from(column.size.height) >= f32::from(body.size.height) - 1.0,
        "the bookmarks column must fill the row: body={:?} column={:?}",
        body.size.height,
        column.size.height
    );
}

#[gpui::test]
async fn the_topbar_right_cell_survives_a_narrow_window(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        cx.notify();
    });
    for width in [1600.0f32, 900.0, 620.0, 480.0] {
        cx.simulate_resize(size(px(width), px(800.0)));
        cx.run_until_parked();
        let right = bounds(cx, "pane-Right-Topbar");
        let strip = bounds(cx, "region-Header-Topbar");
        let close = bounds(cx, "caption-close");
        let zones = bounds(cx, "topbar-zone-toggle-group");
        // Whatever the window width, the whole right-hand group is painted
        // inside it — the centre cell is what gives up room, not the ends.
        assert!(
            f32::from(close.origin.x) + f32::from(close.size.width) <= width + 0.5,
            "the window controls run off a {width}px window"
        );
        assert!(
            f32::from(zones.origin.x) >= f32::from(right.origin.x) - 0.5
                && f32::from(zones.size.width) > 0.0,
            "the zone toggles were squeezed out of the right cell at {width}px"
        );
        // Its icons ride the strip's line — the centre of the cap band, the
        // strip less the menu gap above it — with the menus beside them.
        // Below the wrap width the bar is two lines and they ride the
        // second (`filter::topbar_wraps`).
        let icon_mid = f32::from(close.origin.y) + f32::from(close.size.height) / 2.0;
        let metrics = phiber::v2::theme::METRICS;
        let (gap, line) = (metrics.menu_top_gap, metrics.h_header);
        let wrapped = f32::from(strip.size.height) > line + 0.5;
        let line_top = f32::from(strip.origin.y) + if wrapped { line + metrics.gap_xs } else { 0.0 };
        let strip_line = line_top + gap + (line - gap) / 2.0;
        assert!(
            (icon_mid - strip_line).abs() < 0.51,
            "the top bar's right-hand icons are off the strip's line at {width}px: {icon_mid} vs {strip_line}"
        );
    }
}


#[gpui::test]
async fn resizing_and_switching_space_do_not_rebuild_the_viewport(cx: &mut TestAppContext) {
    use phiber::v2::view::draw::canvas::view3d::cache;

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();
    let settled = cache::builds();

    // Projecting the scene is the most expensive thing a frame does, and it
    // depends on the camera, the palette and the scene — so dragging the
    // window edge or switching space has no business paying for it. (When the
    // GPU draws, nothing is projected at all and this holds trivially; on a
    // machine without one it is the difference between a smooth drag and a
    // slideshow.)
    for width in [1200.0f32, 1100.0, 1000.0, 900.0, 820.0] {
        cx.simulate_resize(size(px(width), px(800.0)));
        cx.run_until_parked();
    }
    assert_eq!(cache::builds(), settled, "dragging the window edge re-projected the scene");

    for space in [BSpaceType::Shading, BSpaceType::Layout, BSpaceType::Modeling] {
        app.update(cx, |app, cx| {
            app.state.set(CtxKey::Space, space.slug());
            cx.notify();
        });
        cx.run_until_parked();
    }
    assert_eq!(cache::builds(), settled, "changing space re-projected the scene");
}

#[gpui::test]
async fn a_window_drag_restarts_the_trace_once_not_every_frame(cx: &mut TestAppContext) {
    use phiber::v2::view::draw::canvas::view3d::views_posted;

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();
    // Let any first-frame size settle before counting.
    std::thread::sleep(std::time::Duration::from_millis(200));
    app.update(cx, |_, cx| cx.notify());
    cx.run_until_parked();
    let before = views_posted();

    // A drag: a new width every frame, faster than the settle window.
    for width in [1270.0f32, 1255.0, 1240.0, 1220.0, 1200.0, 1180.0, 1160.0, 1140.0] {
        cx.simulate_resize(size(px(width), px(800.0)));
        cx.run_until_parked();
    }
    let during = views_posted() - before;

    // The pointer stops. While a size is waiting the viewport keeps asking
    // for frames (`request_animation_frame`); the test drives those frames
    // itself, since its executor does not tick a display.
    std::thread::sleep(std::time::Duration::from_millis(200));
    for _ in 0..3 {
        app.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
    }
    let after = views_posted() - before;

    if after == 0 {
        // No tracer in this environment (no adapter): nothing is posted at
        // all and there is nothing to measure.
        return;
    }
    // Every frame of the drag used to restart the trace at its own size.
    assert!(
        during <= 2,
        "an 8-frame drag restarted the tracer {during} times; it must hold one size"
    );
    assert_eq!(
        after - during,
        1,
        "the size the drag stopped at must be traced once when it settles"
    );
}

#[gpui::test]
async fn the_cpu_painter_stays_idle_while_the_gpu_draws(cx: &mut TestAppContext) {
    use phiber::v2::view::draw::canvas::view3d::{on_gpu, rasterised};

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();
    if !on_gpu() {
        // No adapter here: the CPU painter is the renderer and must run.
        return;
    }
    // The painter stands in until the tracer's first image lands
    // (`the_viewport_always_draws_something`), which on a busy machine takes
    // a while: wait for the frame it stops at, then measure.
    for _ in 0..300 {
        let painted = rasterised();
        app.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
        if rasterised() == painted {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    // With the GPU drawing there is nothing for the CPU painter to fill. It
    // used to rasterise an empty, supersampled, viewport-sized frame anyway —
    // and upload it as a texture — on every frame, which was three quarters
    // of a debug-build frame and the reason zone drags crawled.
    let before = rasterised();
    for width in [1240.0f32, 1200.0, 1160.0] {
        cx.simulate_resize(size(px(width), px(800.0)));
        cx.run_until_parked();
    }
    for _ in 0..3 {
        app.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
    }
    assert_eq!(rasterised(), before, "the CPU painter rasterised a frame the GPU was drawing");
}

#[gpui::test]
async fn one_frame_reads_the_scene_once(cx: &mut TestAppContext) {
    use phiber::v2::scene::object_builds;

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();

    // The Viewport, the overlay, the Outliner and the frame trace each want
    // the scene. Each ask used to rebuild the whole object list — a String and
    // a materials Vec per object, several times a frame, for a list that
    // cannot change within one pass.
    let before = object_builds();
    for _ in 0..5 {
        app.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
    }
    let per_frame = (object_builds() - before) as f32 / 5.0;
    assert!(
        per_frame <= 1.0,
        "a frame read the scene {per_frame} times; the readers must share one build"
    );
}

// ── Progress ──────────────────────────────────────────────────────────────

#[gpui::test]
async fn the_progress_bar_holds_its_place_as_the_stage_changes(cx: &mut TestAppContext) {
    use phiber::v2::scene::{load_progress_key, load_stage_key};

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    let mut seen = Vec::new();
    // Stage labels change on every tick while a file is read; a bar that sat
    // after its label slid sideways with every object.
    for (fraction, stage) in [
        ("0.1", "Reading file"),
        ("0.4", "Reading objects 431/2104 Cube.003"),
        ("0.9", "Evaluating modifiers"),
    ] {
        app.update(cx, |app, cx| {
            app.state = BState::new();
            app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
            app.state.set(&load_progress_key(), fraction);
            app.state.set(&load_stage_key(), stage);
            cx.notify();
        });
        cx.run_until_parked();
        seen.push(bounds(cx, "statusbar-progress-track"));
    }
    for b in &seen {
        assert_eq!(
            (f32::from(b.origin.x), f32::from(b.size.width)),
            (f32::from(seen[0].origin.x), f32::from(seen[0].size.width)),
            "the bar must not move or resize as the stage label changes"
        );
    }
}

// ── Gradients ─────────────────────────────────────────────────────────────

#[test]
fn the_selected_properties_tab_is_the_panel_beside_it() {
    use phiber::v2::theme::{
        BGradientAxis, BPaletteName, resolve_ui_style, ui_style_defaults,
    };
    use phiber::v2::view::draw::tab::cap::BCapSide;

    let style = resolve_ui_style(ui_style_defaults(BPaletteName::DEFAULT), &BState::new());
    let fill = style.tab_active_gradient(BHostType::Buts);
    // The tab is exactly the panel's own surface, flat. It used to sweep in
    // from one step over it; a gradient on one side of the join between the
    // tab and the panel it selects is what made the two read as two matching
    // boxes rather than as one continuous area — a hole cut in the strip
    // onto what it selects (`tests/the_properties_strip_is_a_strip.rs`).
    assert_eq!(fill.bottom, style.colors.bg_panel);
    assert_eq!(fill.top, fill.bottom, "flat, so the join has no edge");
    assert_eq!(fill, style.host_body_surface(BHostType::Buts));

    // And it runs along the rail-to-panel axis, whichever side the rail is on.
    assert_eq!(
        BCapSide::Right.gradient_axis(),
        BGradientAxis::Trailing,
        "a left-hand rail hands off to the right"
    );
    assert_eq!(
        BCapSide::Left.gradient_axis(),
        BGradientAxis::Leading,
        "a right-hand rail hands off to the left"
    );
    assert_eq!(BCapSide::Below.gradient_axis(), BGradientAxis::Down);
}

// ── Asset shelf ───────────────────────────────────────────────────────────

#[test]
fn a_shelf_exists_only_in_a_mode_that_paints() {
    use phiber::v2::part::assetshelf::{BAssetShelf, BAssetShelfHeader, has_shelf};
    use phiber::v2::part::modebar::set::mode_key;
    use phiber::v2::view::BViewItem;

    // Blender polls the shelf per mode; a space that is not painting has no
    // shelf at all. Declaring one unconditionally is what left the Image
    // Editor with a row of brushes and an empty band under it.
    assert!(has_shelf(BHostType::Image, "Paint"));
    assert!(!has_shelf(BHostType::Image, "View"));
    assert!(has_shelf(BHostType::View3D, "Sculpt Mode"));
    assert!(!has_shelf(BHostType::View3D, "Object Mode"));

    // And with no shelf, both regions render nothing, so the host drops them
    // instead of reserving space.
    let mut s = BState::new();
    s.set(&mode_key(BHostType::Image), "View");
    assert!(BAssetShelf::new(BHostType::Image).build(&s).is_empty());
    assert!(BAssetShelfHeader::new(BHostType::Image).build(&s).is_empty());
}

#[test]
fn the_shelf_is_folded_until_asked_for_and_its_tiles_are_operations() {
    use phiber::v2::part::assetshelf::{BAssetShelf, k_brush, k_expanded};
    use phiber::v2::part::modebar::set::mode_key;
    use phiber::v2::view::{BView, BViewItem};

    let mut s = BState::new();
    s.set(&mode_key(BHostType::Image), "Paint");
    // In a painting mode but folded: the header offers it, the shelf is away.
    assert!(BAssetShelf::new(BHostType::Image).build(&s).is_empty());

    s.set(&k_expanded(BHostType::Image), "true");
    let view = BAssetShelf::new(BHostType::Image).build(&s);
    let BView::Row { list } = &view else { panic!("the shelf is a row of tiles") };
    assert_eq!(list.len(), 6);
    // Every tile selects its brush, and the first reads as active until one
    // is chosen. They used to be bare icons that did nothing at all.
    for child in list {
        let BView::Button(ctl) = child else { panic!("a tile is a button") };
        assert!(ctl.action.is_some(), "{:?} does nothing", ctl.tooltip);
    }
    let BView::Button(first) = &list[0] else { panic!() };
    assert!(first.active, "the mode's first brush is active by default");

    s.set(&k_brush(BHostType::Image), "Smear");
    let view = BAssetShelf::new(BHostType::Image).build(&s);
    let BView::Row { list } = &view else { panic!() };
    let active: Vec<_> = list
        .iter()
        .filter_map(|c| match c {
            BView::Button(ctl) if ctl.active => ctl.tooltip.clone(),
            _ => None,
        })
        .collect();
    assert_eq!(active, vec!["Smear".to_string()], "exactly one brush is active");
}

// ── State must not move things ────────────────────────────────────────────

#[gpui::test]
async fn selecting_a_file_row_does_not_move_it(cx: &mut TestAppContext) {
    
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    let mut seen = Vec::new();
    for selected in ["", "Downloads"] {
        app.update(cx, |app, cx| {
            app.state = BState::new();
            app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
            app.state.set(ModalKey::Open, "file_browser");
            if !selected.is_empty() {
                app.state.set(&FileKey::Selected.path(), selected);
            }
            cx.notify();
        });
        cx.run_until_parked();
        seen.push(bounds(cx, "panel-body-surface"));
    }
    // Selection is paint. A row used to shrink by a border and gain a margin
    // when selected, so picking one nudged it and everything below it.
    assert_eq!(
        (f32::from(seen[0].size.height), f32::from(seen[0].origin.y)),
        (f32::from(seen[1].size.height), f32::from(seen[1].origin.y)),
        "selecting a row must not move the list"
    );
}

#[test]
fn no_row_changes_its_measurements_with_its_state() {
    // The two row primitives both used to be shorter when selected. A state
    // change may alter what a row paints; it may never alter what it
    // measures, or hovering and selecting shift the layout around them.
    for file in ["list.rs", "table.rs"] {
        let src = std::fs::read_to_string(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("src/v2/view/draw")
                .join(file),
        )
        .unwrap_or_else(|e| panic!("{file}: {e}"));
        assert!(
            !src.contains("let height = if selected"),
            "{file} still sizes a row by its selection"
        );
    }
}

// ── Areas ─────────────────────────────────────────────────────────────────

#[test]
fn only_stacking_hosts_scroll_their_side_area() {
    // A canvas host fills its area; scrolling one left a dead band under the
    // canvas where the scroll view stopped short of the area's bottom.
    for host in [
        BHostType::View3D,
        BHostType::Image,
        BHostType::Uv,
        BHostType::Shader,
    ] {
        assert!(host.fills_content(), "{host:?} is a canvas host");
    }
    for host in [BHostType::Outliner, BHostType::Buts, BHostType::Prefs] {
        assert!(!host.fills_content(), "{host:?} stacks content and scrolls");
    }
}

#[test]
fn preferences_declares_its_categories_as_a_navbar_region() {
    use phiber::v2::region::BRegionType;
    let kinds: Vec<BRegionType> = phiber::v2::host::pref::REGIONS
        .iter()
        .map(|r| r.kind)
        .collect();
    assert!(
        kinds.contains(&BRegionType::NavBar),
        "Preferences must use the same NavBar region Properties does, not a \
         Left pane of its Center region: {kinds:?}"
    );
}
