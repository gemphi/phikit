//! End to end: switching the Layout's bottom editor to the Graph editor
//! paints the animation window with its channel region, and a press in the
//! window scrubs on the Graph editor's own view (`graph_create`: `0 .. efra`),
//! not the Timeline's. What the window paints for loaded curves — the
//! value axis, the colours, the vertices — is unit-tested in
//! `canvas/time/curve` and `canvas/time/grid`; this drives the built window.

use gpui::{Modifiers, MouseButton, Point, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BKey as _, CtxKey};

fn pt(x: f32, y: f32) -> Point<gpui::Pixels> {
    Point::new(px(x), px(y))
}

/// `ANIM_UI_get_channel_name_width`: the channel region on the window's left.
const CHANNEL_WIDTH: f32 = 200.0;

#[gpui::test]
async fn the_graph_editor_is_the_animation_window_on_its_own_view(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    let space = BSpaceType::Layout;
    let area = space.spec().areas.iter().find(|a| a.default_host == BHostType::Time).copied().expect("the Timeline area");
    assert!(area.hosts.contains(&BHostType::Ipo), "the Graph editor is offered where Blender offers it");
    app.update(cx, |app, cx| {
        app.state.remove(phiber::v2::state::ModalKey::Open);
        app.state.set(&phiber::v2::area::area_key(space, &area), BHostType::Ipo.slug());
        cx.notify();
    });
    cx.run_until_parked();

    let window = cx.debug_bounds("time-window").expect("the Graph editor window is painted");
    let header = cx.debug_bounds("region-Header-Ipo").expect("the Graph editor header is painted");
    assert!(window.origin.y > header.origin.y, "the window hangs under its header");
    assert!(f32::from(window.size.height) >= 40.0, "the window has room for the strip and a value line: {:?}", window.size);
    let (left, width) = (f32::from(window.origin.x), f32::from(window.size.width));
    let y = f32::from(window.origin.y) + f32::from(window.size.height) * 0.6;
    let frame = |app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext| -> i32 {
        app.read_with(cx, |a, _| a.state.get(CtxKey::Frame).map_or(1, |v| v.parse().unwrap()))
    };

    // The channel region takes the press without scrubbing.
    let before = frame(&app, cx);
    cx.simulate_mouse_down(pt(left + CHANNEL_WIDTH * 0.5, y), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(pt(left + CHANNEL_WIDTH * 0.5, y), MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(frame(&app, cx), before, "a press in the channel region is not a scrub");

    // The Graph editor opens on `0 .. efra`, so halfway across its window
    // is frame 125 — where the Timeline, on `−9 .. 260`, would give 126.
    let graph_width = width - CHANNEL_WIDTH;
    let press = pt(left + CHANNEL_WIDTH + graph_width * 0.5, y);
    cx.simulate_mouse_down(press, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(press, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let halfway = frame(&app, cx);
    assert!((halfway - 125).abs() <= 1, "a press halfway sets frame 125 on the Graph editor's view, got {halfway}");
    let press = pt(left + CHANNEL_WIDTH + graph_width * 0.1, y);
    cx.simulate_mouse_down(press, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(press, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    let tenth = frame(&app, cx);
    assert!((tenth - 25).abs() <= 1, "a tenth of the way is frame 25, got {tenth}");
    let _ = CtxKey::Frame.path();
}

/// A loaded action's keys are painted as vertices where the Graph editor's
/// view puts them: frame `f` of `0 .. 250` across the window, value `v` of
/// `−9 .. 10` up it — a selected key in the theme's `TH_VERTEX_SELECT`
/// colour, an unselected one in its `TH_VERTEX`, both `vertex_size` 6 and
/// round.
#[gpui::test]
async fn loaded_keys_are_painted_as_vertices_on_the_value_axis(cx: &mut TestAppContext) {
    use phiber::phiphx::blend::{BFCurve, BKey, FLAG_ACTIVE, FLAG_SELECTED};
    use phiber::phiphx::scene::{SceneAction, SceneModel};
    let key = |frame: f32, value: f32, selected: bool| BKey { selected, ..BKey::new([frame - 5.0, value], [frame, value], [frame + 5.0, value], 2) };
    let mut selected = BFCurve::new("location", 0, vec![key(50.0, 0.0, true), key(200.0, 5.0, true)]);
    selected.flag = FLAG_SELECTED | FLAG_ACTIVE;
    let unselected = BFCurve::new("location", 1, vec![key(125.0, -5.0, false)]);
    let action = SceneAction {
        name: "CubeAction".into(),
        frame_start: 50.0,
        frame_end: 200.0,
        fcurve_count: 2,
        keys: vec![50.0, 125.0, 200.0],
        users: vec!["Cube".into()],
        curves: vec![selected, unselected],
        ..Default::default()
    };
    let mut model = SceneModel { actions: vec![action], ..Default::default() };
    model.selection.selected = vec!["Cube".into()];

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    let space = BSpaceType::Layout;
    let area = space.spec().areas.iter().find(|a| a.default_host == BHostType::Time).copied().expect("the Timeline area");
    app.update(cx, |app, cx| {
        app.scene_cache = Some(("graph-test".into(), std::sync::Arc::new(model)));
        app.state.remove(phiber::v2::state::ModalKey::Open);
        app.state.set(&phiber::v2::area::area_key(space, &area), BHostType::Ipo.slug());
        cx.notify();
    });
    cx.run_until_parked();

    let window = cx.debug_bounds("time-window").expect("the Graph editor window is painted");
    let (left, top) = (f32::from(window.origin.x), f32::from(window.origin.y));
    let (width, height) = (f32::from(window.size.width) - CHANNEL_WIDTH, f32::from(window.size.height));
    let x_of = |frame: f32| left + CHANNEL_WIDTH + frame / 250.0 * width;
    let y_of = |value: f32| top + height - (value + 9.0) / 19.0 * height;
    // Every round 6 × 6 quad is a vertex: its origin and its fill. The
    // scene is recorded in device pixels, so sizes carry the scale factor.
    let scale = cx.update(|window, _| window.scale_factor());
    let vertex_size = format!(" {:.2}x{:.2} ", 6.0 * scale, 6.0 * scale);
    let vertex_corner = format!(" r{r:.2}/{r:.2}/{r:.2}/{r:.2}", r = 3.0 * scale);
    let vertices: Vec<(f32, f32, String)> = cx
        .scene_fingerprint()
        .into_iter()
        .filter(|line| line.starts_with("quad ") && line.contains(&vertex_size) && line.contains(&vertex_corner))
        .map(|line| {
            let origin = line.split(' ').nth(1).unwrap().to_string();
            let (x, y) = origin.split_once(',').unwrap();
            let fill = line.split(" bg ").nth(1).unwrap().split(" border ").next().unwrap().to_string();
            (x.parse::<f32>().unwrap() / scale, y.parse::<f32>().unwrap() / scale, fill)
        })
        .collect();
    assert_eq!(vertices.len(), 3, "three keys, three vertices: {vertices:?}");
    let near = |(x, y, _): &(f32, f32, String), frame: f32, value: f32| (x + 3.0 - x_of(frame)).abs() <= 1.5 && (y + 3.0 - y_of(value)).abs() <= 1.5;
    let at = |frame: f32, value: f32| vertices.iter().find(|v| near(v, frame, value)).unwrap_or_else(|| panic!("no vertex at frame {frame}, value {value}: {vertices:?}"));
    let orange = at(50.0, 0.0);
    let black = at(125.0, -5.0);
    at(200.0, 5.0);
    assert_ne!(orange.2, black.2, "a selected key and an unselected one differ");
    // The colours are the theme's `TH_VERTEX_SELECT` and `TH_VERTEX` stand-ins
    // (`theme/style/anim.rs`): phiber's warm accent, and the app background.
    let state = phiber::v2::state::BState::new();
    // The mode the window drew in, not a dark-pinned table: `ui_style_defaults`
    // is Dark, and `BThemeMode::DEFAULT` is Light, so pinning here compared the
    // painted keys against a different theme than the one that painted them.
    let (_, mode) = phiber::v2::theme::theme_in_force(&state);
    let colors = phiber::v2::theme::resolve_ui_style(phiber::v2::theme::style::ui_style_defaults_in(phiber::v2::theme::BPaletteName::Sphere, mode), &state).anim();
    let hue = |fill: &str| fill.split("h: ").nth(1).and_then(|rest| rest.split(',').next()).and_then(|h| h.trim().parse::<f32>().ok()).unwrap_or(-1.0);
    let lightness = |fill: &str| fill.split("l: ").nth(1).and_then(|rest| rest.split(',').next()).and_then(|l| l.trim().parse::<f32>().ok()).unwrap_or(-1.0);
    assert!((hue(&orange.2) - colors.vertex_select.0 / 360.0).abs() < 1e-3, "selected key in the theme's vertex_select: {}", orange.2);
    assert!((lightness(&black.2) - colors.vertex.2 / 100.0).abs() < 1e-3, "unselected key in the theme's vertex colour: {}", black.2);
    // `draw_fcurve_handle_vertices`: every Bézier key shows both handles as
    // 7 × 7 rings with a 1.5 px outline — six for these three keys — and
    // `draw_fcurve_handles` runs a line from each to its key.
    let ring_size = format!(" {:.2}x{:.2} ", 7.0 * scale, 7.0 * scale);
    let ring_border = format!(" w{b:.2}/{b:.2}/{b:.2}/{b:.2} ", b = 1.5 * scale);
    let rings: Vec<(f32, f32)> = painted_lines(cx)
        .into_iter()
        .filter(|line| line.starts_with("quad ") && line.contains(&ring_size) && line.contains(&ring_border))
        .map(|line| {
            let origin = line.split(' ').nth(1).unwrap().to_string();
            let (x, y) = origin.split_once(',').unwrap();
            (x.parse::<f32>().unwrap() / scale + 3.5, y.parse::<f32>().unwrap() / scale + 3.5)
        })
        .collect();
    assert_eq!(rings.len(), 6, "two handle rings per Bézier key: {rings:?}");
    assert!(rings.iter().any(|(x, y)| (x - x_of(45.0)).abs() <= 1.5 && (y - y_of(0.0)).abs() <= 1.5), "the first key's left handle sits at frame 45: {rings:?}");
    assert!(rings.iter().any(|(x, y)| (x - x_of(130.0)).abs() <= 1.5 && (y - y_of(-5.0)).abs() <= 1.5), "the unselected key's right handle at frame 130");
}

fn painted_lines(cx: &mut gpui::VisualTestContext) -> Vec<String> {
    cx.scene_fingerprint()
}
