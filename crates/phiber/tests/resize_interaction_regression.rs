//! Runtime regression for interactive area resizing (DEFECT 1 / Requirement 54).
//!
//! Feature: phiber-blender-space-parity
//! Validates: Requirements 54.1, 54.6, 54.7
//!
//! Drives a REAL drag over a rendered `BResizeHandle` in a `TestAppContext`
//! window: it resizes the window, selects a multi-area space, locates the
//! handle via `debug_bounds`, presses on it, drags the pointer across it,
//! releases, and asserts that (a) the two adjacent `ctx.area_size` fractions
//! changed in `BState` and (b) the rendered layout bounds of the two areas
//! shifted. This fails on the pre-fix code where the divider never tracked
//! the cursor (the `on_drag_move` lived on the handle child and computed a
//! bad delta), so `handle_resize` was never invoked with a nonzero delta.

use gpui::{Modifiers, MouseButton, Point, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::{BHostType, host_slug};
use phiber::v2::space::{BSpaceItem, BSpaceType};
use phiber::v2::state::{BState, area_size_key, key::K_SPACE};
use phiber::v2::view::BViewItem;
use phiber::v2::zone::BZoneType;

/// Point offset helper.
fn pt(x: f32, y: f32) -> Point<gpui::Pixels> {
    Point { x: px(x), y: px(y) }
}

/// Select a space and force every View3D-capable area to its View3D host so
/// the layout renders the same content the user sees.
fn select_space(state: &mut BState, space: BSpaceType) {
    state.set(&K_SPACE, space.slug());
    // Dismiss the seeded welcome window so the workspace is what is measured.
    state.set(&phiber::v2::state::key::K_MODAL, "");
    for area in space
        .spec()
        .areas
        .iter()
        .filter(|a| a.hosts.contains(&BHostType::View3D))
    {
        state.set(
            &phiber::v2::area::area_key(space, area),
            host_slug(BHostType::View3D),
        );
    }
}

/// Read a fraction from BState with a `None` sentinel so we can detect writes.
fn raw_fraction(
    state: &BState,
    space: BSpaceType,
    zone: BZoneType,
    index: usize,
) -> Option<String> {
    state
        .get_encoded(&area_size_key(space, zone, index))
        .map(|s| s.to_string())
}

/// End-to-end: dragging a horizontal divider in Shading's Right column
/// (Outliner over Properties) changes the two adjacent area fractions and
/// shifts the rendered layout.
#[gpui::test]
async fn dragging_a_shading_boundary_changes_adjacent_fractions_and_layout(
    cx: &mut TestAppContext,
) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));

    app.update(cx, |app, cx| {
        select_space(&mut app.state, BSpaceType::Shading);
        cx.notify();
    });
    cx.run_until_parked();

    // Shading Right stacks Outliner over Properties → one horizontal handle.
    let handle_id = "resize-right-0";
    let hb = cx
        .debug_bounds(handle_id)
        .unwrap_or_else(|| panic!("Shading right resize handle `{handle_id}` was not painted"));
    let center = hb.center();

    // Snapshot the adjacent fractions before the drag (Shading seeds these).
    let start = app.read_with(cx, |a, _| a.state.clone());
    let bi: f32 = raw_fraction(&start, BSpaceType::Shading, BZoneType::Right, 0)
        .map(|s| s.parse().unwrap())
        .unwrap_or(0.5);
    let bj: f32 = raw_fraction(&start, BSpaceType::Shading, BZoneType::Right, 1)
        .map(|s| s.parse().unwrap())
        .unwrap_or(0.5);

    // Drive a real vertical drag DOWN (horizontal divider ⇒ moves in Y).
    // Dragging the divider down grows the top area (index 0).
    drag_down(cx, center, 160.0);

    let state = app.read_with(cx, |a, _| a.state.clone());
    let fi: f32 = raw_fraction(&state, BSpaceType::Shading, BZoneType::Right, 0)
        .unwrap()
        .parse()
        .unwrap();
    let fj: f32 = raw_fraction(&state, BSpaceType::Shading, BZoneType::Right, 1)
        .unwrap()
        .parse()
        .unwrap();

    assert!(
        fi > bi + 0.02,
        "top area fraction should grow from {bi} after dragging down, got {fi}"
    );
    assert!(
        fj < bj - 0.02,
        "bottom area fraction should shrink from {bj}, got {fj}"
    );
    assert!(
        (fi + fj - (bi + bj)).abs() < 1e-2,
        "pair sum must be preserved, got {}",
        fi + fj
    );

    // The rebuilt tree must still contain the zone and handle (layout intact).
    let tree = BSpaceItem::new(BSpaceType::Shading).build(&state);
    assert!(
        matches!(tree, phiber::v2::view::BView::Space { .. }),
        "root must remain a Space"
    );

    // The handle bounds must have shifted DOWN with the boundary.
    cx.run_until_parked();
    let hb2 = cx
        .debug_bounds(handle_id)
        .expect("handle still painted after resize");
    assert!(
        f32::from(hb2.center().y) > f32::from(center.y) + 10.0,
        "resize handle should move down with the boundary: {} -> {}",
        f32::from(center.y),
        f32::from(hb2.center().y)
    );
}

/// Press on `start`, cross the drag threshold, then step the pointer down by
/// `dist` pixels in several moves so gpui's drag session dispatches
/// `on_drag_move` at the render root.
fn drag_down(cx: &mut gpui::VisualTestContext, start: Point<gpui::Pixels>, dist: f32) {
    let x = f32::from(start.x);
    let y0 = f32::from(start.y);
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
    // Threshold-crossing move starts the drag session.
    cx.simulate_mouse_move(pt(x, y0 + 4.0), Some(MouseButton::Left), Modifiers::none());
    cx.run_until_parked();
    let mut d = 40.0;
    while d <= dist {
        cx.simulate_mouse_move(pt(x, y0 + d), Some(MouseButton::Left), Modifiers::none());
        cx.run_until_parked();
        d += 40.0;
    }
    cx.simulate_mouse_move(pt(x, y0 + dist), Some(MouseButton::Left), Modifiers::none());
    cx.run_until_parked();
    cx.simulate_mouse_up(pt(x, y0 + dist), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
}

/// A second multi-area space (Layout Right: Outliner over Properties) also
/// resizes, proving the fix is generic across the Space→Zone→Area hierarchy.
#[gpui::test]
async fn dragging_a_layout_boundary_changes_adjacent_fractions(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));

    app.update(cx, |app, cx| {
        select_space(&mut app.state, BSpaceType::Layout);
        cx.notify();
    });
    cx.run_until_parked();

    let hb = cx
        .debug_bounds("resize-right-0")
        .unwrap_or_else(|| panic!("Layout right resize handle was not painted"));
    let center = hb.center();

    // Layout does not seed fractions → equal 0.5/0.5 split before the drag.
    drag_down(cx, center, 120.0);

    let state = app.read_with(cx, |a, _| a.state.clone());
    let fi = raw_fraction(&state, BSpaceType::Layout, BZoneType::Right, 0);
    let fj = raw_fraction(&state, BSpaceType::Layout, BZoneType::Right, 1);
    assert!(
        fi.is_some() && fj.is_some(),
        "Layout drag must write both adjacent fractions, got {fi:?}/{fj:?}"
    );
    let fi: f32 = fi.unwrap().parse().unwrap();
    assert!(
        fi > 0.5,
        "Layout top area fraction should grow from 0.5, got {fi}"
    );
}

/// The seam must track the cursor 1:1 in pixels. This is the regression that
/// specifically guards DEFECT 1's root cause: the drag delta was divided by
/// the whole *window* dimension instead of the zone's true laid-out extent,
/// so the boundary crept only a fraction of the cursor travel. With the zone
/// extent measured during prepaint, dragging the divider by `D` pixels moves
/// the rendered handle by ~`D` pixels.
///
/// Validates: Requirements 54.1, 54.6, 54.7
#[gpui::test]
async fn seam_tracks_cursor_one_to_one_in_pixels(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    // A tall window where the resizable Right zone occupies only the middle
    // band — so window-height and zone-height differ a lot. On the buggy code
    // the handle would move far less than the cursor.
    cx.simulate_resize(size(px(1280.0), px(1000.0)));

    app.update(cx, |app, cx| {
        select_space(&mut app.state, BSpaceType::Shading);
        cx.notify();
    });
    cx.run_until_parked();

    let hb = cx
        .debug_bounds("resize-right-0")
        .unwrap_or_else(|| panic!("Shading right resize handle was not painted"));
    let center = hb.center();
    let y0 = f32::from(center.y);

    // Drag the seam down by a modest, sub-zone amount, in small continuous
    // steps (mirroring a real pointer drag) so the origin latches ~2px from
    // the press point rather than after a large jump.
    let dist = 90.0;
    drag_down_smooth(cx, center, dist);
    cx.run_until_parked();

    let hb2 = cx
        .debug_bounds("resize-right-0")
        .expect("handle still painted after resize");
    let moved = f32::from(hb2.center().y) - y0;

    // The seam should follow the cursor closely. Allow generous slack for the
    // gap/handle thickness and min-clamp, but the old whole-window divisor
    // produced roughly `dist * zone/window` (well under half of `dist` in this
    // tall window), which this lower bound rejects.
    assert!(
        moved > dist * 0.75,
        "seam should track the cursor ~1:1: dragged {dist}px, handle moved only {moved}px"
    );
    assert!(
        moved < dist * 1.25,
        "seam should not overshoot the cursor: dragged {dist}px, handle moved {moved}px"
    );

    // Fractions must still sum to ~1.0 across the zone (normalize invariant).
    let state = app.read_with(cx, |a, _| a.state.clone());
    let fi: f32 = raw_fraction(&state, BSpaceType::Shading, BZoneType::Right, 0)
        .unwrap()
        .parse()
        .unwrap();
    let fj: f32 = raw_fraction(&state, BSpaceType::Shading, BZoneType::Right, 1)
        .unwrap()
        .parse()
        .unwrap();
    assert!(
        (fi + fj - 1.0).abs() < 1e-2,
        "adjacent fractions must sum to ~1.0, got {}",
        fi + fj
    );
}

/// Zone seam: dragging the Center|Right boundary LEFT grows the Right zone's
/// pixel width (`ctx.zone_size.<space>.right`). Before this feature, zone
/// boundaries had no grab handle at all ("resizing missing on the zones").
///
/// Validates: state::zone_size + gpui::space seam handles + root ZoneDrag dispatch.
#[gpui::test]
async fn zone_seam_drag_resizes_right_column(cx: &mut TestAppContext) {
    use phiber::v2::state::zone_size_key;

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));

    app.update(cx, |app, cx| {
        select_space(&mut app.state, BSpaceType::Layout);
        cx.notify();
    });
    cx.run_until_parked();

    let hb = cx
        .debug_bounds("zone-seam-right")
        .expect("Center|Right zone seam handle was not painted");
    let x0 = f32::from(hb.center().x);

    // Drag the seam LEFT by 100px — the right column should grow ~100px.
    let center = hb.center();
    let y = f32::from(center.y);
    cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::none());
    let mut d = 3.0_f32;
    while d < 100.0 {
        cx.simulate_mouse_move(pt(x0 - d, y), Some(MouseButton::Left), Modifiers::none());
        cx.run_until_parked();
        d += 3.0;
    }
    cx.simulate_mouse_move(
        pt(x0 - 100.0, y),
        Some(MouseButton::Left),
        Modifiers::none(),
    );
    cx.run_until_parked();
    cx.simulate_mouse_up(pt(x0 - 100.0, y), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();

    let state = app.read_with(cx, |a, _| a.state.clone());
    let w: f32 = state
        .get_encoded(&zone_size_key(BSpaceType::Layout, BZoneType::Right))
        .expect("drag must write ctx.zone_size.Layout.right")
        .parse()
        .unwrap();

    cx.run_until_parked();
    let hb2 = cx
        .debug_bounds("zone-seam-right")
        .expect("seam still painted");
    let shifted = x0 - f32::from(hb2.center().x);

    assert!(
        shifted > 60.0,
        "seam should track the cursor ~1:1 leftward: moved only {shifted}px"
    );
    assert!(w > 100.0, "right column width should be recorded, got {w}");
}

/// Zone seam: dragging the mid|Bottom boundary UP grows the Bottom strip's
/// pixel height (`ctx.zone_size.<space>.bottom`).
#[gpui::test]
async fn zone_seam_drag_resizes_bottom_strip(cx: &mut TestAppContext) {
    use phiber::v2::state::zone_size_key;

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));

    app.update(cx, |app, cx| {
        select_space(&mut app.state, BSpaceType::Layout);
        cx.notify();
    });
    cx.run_until_parked();

    let hb = cx
        .debug_bounds("zone-seam-bottom")
        .expect("mid|Bottom zone seam handle was not painted");
    let center = hb.center();
    let y0 = f32::from(center.y);
    let x = f32::from(center.x);

    cx.simulate_mouse_down(center, MouseButton::Left, Modifiers::none());
    let mut d = 3.0_f32;
    while d < 80.0 {
        cx.simulate_mouse_move(pt(x, y0 - d), Some(MouseButton::Left), Modifiers::none());
        cx.run_until_parked();
        d += 3.0;
    }
    cx.simulate_mouse_move(pt(x, y0 - 80.0), Some(MouseButton::Left), Modifiers::none());
    cx.run_until_parked();
    cx.simulate_mouse_up(pt(x, y0 - 80.0), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();

    let state = app.read_with(cx, |a, _| a.state.clone());
    let h: f32 = state
        .get_encoded(&zone_size_key(BSpaceType::Layout, BZoneType::Bottom))
        .expect("drag must write ctx.zone_size.Layout.bottom")
        .parse()
        .unwrap();
    assert!(h > 80.0, "bottom strip height should be recorded, got {h}");

    cx.run_until_parked();
    let hb2 = cx
        .debug_bounds("zone-seam-bottom")
        .expect("seam still painted");
    let shifted = y0 - f32::from(hb2.center().y);
    assert!(
        shifted > 50.0,
        "seam should track the cursor ~1:1 upward: moved only {shifted}px"
    );
}

/// Smooth continuous downward drag in small steps from the press point, so the
/// running-delta origin latches within the drag threshold of the press.
fn drag_down_smooth(cx: &mut gpui::VisualTestContext, start: Point<gpui::Pixels>, dist: f32) {
    let x = f32::from(start.x);
    let y0 = f32::from(start.y);
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
    let mut d = 3.0_f32;
    while d < dist {
        cx.simulate_mouse_move(pt(x, y0 + d), Some(MouseButton::Left), Modifiers::none());
        cx.run_until_parked();
        d += 3.0;
    }
    cx.simulate_mouse_move(pt(x, y0 + dist), Some(MouseButton::Left), Modifiers::none());
    cx.run_until_parked();
    cx.simulate_mouse_up(pt(x, y0 + dist), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
}

/// DEFECT 1 root-cause guard: the resize handle must present a real,
/// grabbable hit band along the boundary — not a hairline stranded between
/// two dead gaps. A thin/occluded seam is why the user reported "resizing
/// does nothing": there was effectively nothing to grab. This asserts the
/// rendered handle spans at least the resolved hit width along its
/// fixed (grab) axis in a real window, and that pressing exactly on the seam
/// center starts a resize (fractions change), i.e. the seam is not swallowed
/// by the adjacent View3D orbit surface.
///
/// Validates: Requirements 54.1, 54.3
#[gpui::test]
async fn resize_handle_presents_a_grabbable_band_and_wins_the_press(cx: &mut TestAppContext) {
    use phiber::v2::theme::{BPaletteName, ui_style_defaults};

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));

    app.update(cx, |app, cx| {
        select_space(&mut app.state, BSpaceType::Shading);
        cx.notify();
    });
    cx.run_until_parked();

    let hb = cx
        .debug_bounds("resize-right-0")
        .unwrap_or_else(|| panic!("Shading right resize handle was not painted"));

    // Right zone stacks vertically → the handle is a horizontal bar; its
    // grabbable axis is the height. It must be at least the metric hit band.
    let band = f32::from(hb.size.height);
    let hit_width = ui_style_defaults(BPaletteName::DEFAULT).resize.hit_width;
    assert!(
        band >= hit_width - 0.5,
        "resize seam is not grabbable: band height {band}px < hit width {hit_width}px"
    );

    // Pressing exactly on the seam center and dragging must resize (the press
    // is not stolen by adjacent area content).
    let center = hb.center();
    let before = app.read_with(cx, |a, _| a.state.clone());
    let bi: f32 = raw_fraction(&before, BSpaceType::Shading, BZoneType::Right, 0)
        .map(|s| s.parse().unwrap())
        .unwrap_or(0.5);
    drag_down(cx, center, 80.0);
    let after = app.read_with(cx, |a, _| a.state.clone());
    let fi: f32 = raw_fraction(&after, BSpaceType::Shading, BZoneType::Right, 0)
        .unwrap()
        .parse()
        .unwrap();
    assert!(
        (fi - bi).abs() > 0.02,
        "pressing the seam center did not resize (fraction {bi} -> {fi}); the seam was swallowed"
    );
}

/// Post-decomposition geometry regression for both focused seam renderers.
/// The transparent hit targets are wider than the visible layout gap, but
/// remain absolute children: sibling allocation stays at the resolved gap and
/// each hit band is centered over that boundary. The same rendered Layout
/// also keeps its View3D viewport inside the bounded center slot.
///
/// Validates: Requirements 7.6, 7.7, 11.3, 13.8
#[gpui::test]
async fn focused_seam_overlays_are_grabbable_zero_footprint_and_bounded(cx: &mut TestAppContext) {
    use phiber::v2::theme::{BPaletteName, ui_style_defaults};

    fn close(actual: f32, expected: f32, message: &str) {
        assert!(
            (actual - expected).abs() <= 0.75,
            "{message}: expected {expected}px, got {actual}px"
        );
    }

    let resize = ui_style_defaults(BPaletteName::DEFAULT).resize;
    assert!(resize.hit_width > resize.gap);

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));

    // Area boundary: Shading's Right zone stacks two area slots vertically.
    app.update(cx, |app, cx| {
        select_space(&mut app.state, BSpaceType::Shading);
        cx.notify();
    });
    cx.run_until_parked();

    let upper = cx.debug_bounds("slot-right-0").expect("upper area slot");
    let lower = cx.debug_bounds("slot-right-1").expect("lower area slot");
    let area_hit = cx.debug_bounds("resize-right-0").expect("area hit band");
    let area_gap_start = f32::from(upper.origin.y + upper.size.height);
    let area_gap_end = f32::from(lower.origin.y);
    close(
        area_gap_end - area_gap_start,
        resize.gap,
        "area allocation gap",
    );
    close(
        f32::from(area_hit.size.height),
        resize.hit_width,
        "area hit-band height",
    );
    close(
        f32::from(area_hit.center().y),
        (area_gap_start + area_gap_end) / 2.0,
        "area hit-band center",
    );

    // Zone boundary: Layout's Center and Right slots remain separated only by
    // the resolved gap even though the absolute hit target overlaps both.
    app.update(cx, |app, cx| {
        select_space(&mut app.state, BSpaceType::Layout);
        cx.notify();
    });
    cx.run_until_parked();

    let center = cx.debug_bounds("slot-center-0").expect("center zone slot");
    let right = cx.debug_bounds("slot-right-0").expect("right zone slot");
    let zone_hit = cx
        .debug_bounds("zone-seam-right")
        .expect("center/right zone hit band");
    let zone_gap_start = f32::from(center.origin.x + center.size.width);
    let zone_gap_end = f32::from(right.origin.x);
    close(
        zone_gap_end - zone_gap_start,
        resize.gap,
        "zone allocation gap",
    );
    close(
        f32::from(zone_hit.size.width),
        resize.hit_width,
        "zone hit-band width",
    );
    close(
        f32::from(zone_hit.center().x),
        (zone_gap_start + zone_gap_end) / 2.0,
        "zone hit-band center",
    );

    let viewport = cx
        .debug_bounds("view3d-canvas")
        .expect("bounded View3D viewport");
    assert!(viewport.size.width > px(1.0) && viewport.size.height > px(1.0));
    assert!(viewport.origin.x >= center.origin.x && viewport.origin.y >= center.origin.y);
    assert!(
        viewport.origin.x + viewport.size.width <= center.origin.x + center.size.width
            && viewport.origin.y + viewport.size.height <= center.origin.y + center.size.height,
        "View3D viewport escaped its bounded center slot: viewport={viewport:?}, slot={center:?}"
    );
}
