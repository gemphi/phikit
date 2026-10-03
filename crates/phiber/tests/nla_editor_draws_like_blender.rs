//! End to end: the NLA editor is the animation window with Blender's rows
//! (`animfilter_nla`: the object, its action line, its tracks last first),
//! `NLATRACK_HEIGHT` tall, and paints what `draw_nla_main_data` paints —
//! the action's band across the view and each track's strips as outlined
//! boxes from `strip.start` to `strip.end`.

use gpui::{TestAppContext, px, size};
use phiber::phiphx::blend::{BNlaStrip, BNlaTrack, BlendNla, EXTEND_HOLD, STRIP_MUTED, STRIP_SELECT};
use phiber::phiphx::scene::{SceneAction, SceneModel};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::space::BSpaceType;

/// `NLATRACK_NAMEWIDTH`, `NLATRACK_HEIGHT`, `NLATRACK_SKIP`, `UI_TIME_SCRUB_MARGIN_Y`.
const CHANNEL_WIDTH: f32 = 200.0;
const ROW: f32 = 24.0;
const SKIP: f32 = 2.0;
const SCRUB: f32 = 23.0;

fn strip(name: &str, start: f32, end: f32, repeat: f32, flag: u32) -> BNlaStrip {
    BNlaStrip { name: name.into(), start, end, actstart: 1.0, actend: 49.0, repeat, scale: 1.0, flag, kind: 0, extend: EXTEND_HOLD, action: Some("Walk".into()), ..Default::default() }
}

/// The quads of the last frame as `(x, y, w, h, has_border)` in logical pixels.
fn quads(cx: &mut gpui::VisualTestContext) -> Vec<(f32, f32, f32, f32, bool)> {
    let scale = cx.update(|window, _| window.scale_factor());
    cx.scene_fingerprint()
        .into_iter()
        .filter(|line| line.starts_with("quad "))
        .map(|line| {
            let mut parts = line.split(' ');
            let origin = parts.nth(1).unwrap();
            let size = parts.next().unwrap();
            let (x, y) = origin.split_once(',').unwrap();
            let (w, h) = size.split_once('x').unwrap();
            let border = !line.contains(" w0.00/0.00/0.00/0.00 ");
            (x.parse::<f32>().unwrap() / scale, y.parse::<f32>().unwrap() / scale, w.parse::<f32>().unwrap() / scale, h.parse::<f32>().unwrap() / scale, border)
        })
        .collect()
}

#[gpui::test]
async fn the_nla_editor_stacks_the_action_line_and_the_tracks_with_their_strips(cx: &mut TestAppContext) {
    let nla = BlendNla {
        object: "Cube".into(),
        action: Some("Idle".into()),
        extend: EXTEND_HOLD,
        flag: 0,
        tracks: vec![
            BNlaTrack { name: "Base Layer".into(), flag: 0, strips: vec![strip("Walk", 1.0, 97.0, 2.0, 0)], address: 0 },
            BNlaTrack { name: "Overlay".into(), flag: 0, strips: vec![strip("Walk Again", 120.0, 168.0, 1.0, STRIP_SELECT | STRIP_MUTED)], address: 0 },
        ],
        ..Default::default()
    };
    let idle = SceneAction { name: "Idle".into(), frame_start: 10.0, frame_end: 30.0, fcurve_count: 1, keys: vec![10.0, 30.0], users: vec!["Cube".into()], curves: Vec::new(), ..Default::default() };
    let mut model = SceneModel { actions: vec![idle], nla: vec![nla], ..Default::default() };
    model.selection.selected = vec!["Cube".into()];

    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1400.0), px(800.0)));
    // The Animation workspace's Dope Sheet band is tall enough for four rows.
    let space = BSpaceType::Animation;
    let area = space.spec().areas.iter().find(|a| a.default_host == BHostType::Action).copied().expect("the Dope Sheet area");
    assert!(area.hosts.contains(&BHostType::Nla));
    app.update(cx, |app, cx| {
        app.scene_cache = Some(("nla-test".into(), std::sync::Arc::new(model)));
        app.state.remove(phiber::v2::state::ModalKey::Open);
        app.state.set(phiber::v2::state::CtxKey::Space, space.slug());
        app.state.set(&phiber::v2::area::area_key(space, &area), BHostType::Nla.slug());
        cx.notify();
    });
    cx.run_until_parked();

    let window = cx.debug_bounds("time-window").expect("the NLA window is painted");
    assert!(cx.debug_bounds("region-Header-Nla").is_some());
    let (left, top) = (f32::from(window.origin.x) + CHANNEL_WIDTH, f32::from(window.origin.y));
    let width = f32::from(window.size.width) - CHANNEL_WIDTH;
    // `nla_create`: the view is `sfra − 10 .. efra + 10`, so 269 frames wide.
    let x_of = |frame: f32| left + (frame + 9.0) / 269.0 * width;
    let row_top = |index: usize| top + SCRUB + SKIP + index as f32 * (ROW + SKIP);
    let painted = quads(cx);
    let near = |a: f32, b: f32| (a - b).abs() <= 1.5;
    // Row 1 is the action line: a band the width of the view, a skip down.
    assert!(
        painted.iter().any(|(x, y, w, _, _)| near(*x, left) && near(*w, width) && near(*y, row_top(1) + SKIP)),
        "the action's band runs across the view on the second row: {painted:?}"
    );
    // Rows 2 and 3 are the tracks, last first: Overlay's strip at 120 .. 168,
    // then Base Layer's at 1 .. 97, each an outlined box.
    let strip_box = |start: f32, end: f32, row: usize| {
        painted.iter().any(|(x, y, w, h, border)| *border && near(*x, x_of(start)) && near(*w, x_of(end) - x_of(start)) && near(*y, row_top(row)) && near(*h, ROW))
    };
    assert!(strip_box(120.0, 168.0, 2), "the upper track's strip is the box on row 2: {painted:?}");
    assert!(strip_box(1.0, 97.0, 3), "the lower track's strip is the box on row 3");
    // The repeated strip carries a line at its one whole repeat, frame 49.
    assert!(
        painted.iter().any(|(x, y, w, _, _)| near(*x, x_of(49.0).floor()) && near(*w, 1.0) && *y > row_top(3) && *y < row_top(3) + ROW),
        "a repeat line at frame 49"
    );
}
