//! X-ray as Blender 5.0 has it (`view3d.toggle_xray`, `VIEW3D_HT_header`):
//! the button before the shading radio and Alt Z over the viewport flip the
//! switch of the shading in use — Solid's (off in a new file) or
//! Wireframe's (on) — and the button is pressed while it is on. In Material
//! Preview the switch still flips, the button is dimmed and the status bar
//! says X-ray is not available there. While on, the viewport draws through
//! at `1 − xray_alpha` (phiray `tests/xray.rs` holds the frames to Blender's).

use gpui::{Modifiers, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::part::filter::BFilter;
use phiber::v2::scene::{BShading, BXray, shading_key};
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState, CtxKey, ModalKey};
use phiber::v2::view::{BView, BViewItem};

const BUTTON: &str = "btn-icon-set-ctx.op.view3d-toggle_xray-xray";

/// The header's X-ray button: `(pressed, dimmed)`.
fn button(state: &BState) -> (bool, bool) {
    fn find(v: &BView) -> Option<(bool, bool)> {
        match v {
            BView::Button(c) if c.icon.as_deref() == Some("xray") => Some((c.active, c.muted)),
            BView::Row { list } | BView::Column { list } => list.iter().find_map(find),
            _ => None,
        }
    }
    find(&BFilter::new(BHostType::View3D).build(state)).expect("the X-ray button")
}

fn said() -> String {
    phiber::v2::state::log::latest().map(|e| e.text.clone()).unwrap_or_default()
}

#[gpui::test]
async fn alt_z_and_the_button_toggle_xray_for_the_shading_in_use(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state.set(ModalKey::Open, "");
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();
    let xray = |app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext| app.read_with(cx, |a, _| BXray::now(&a.state));
    let pressed = |app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext| app.read_with(cx, |a, _| button(&a.state));
    assert_eq!(xray(&app, cx), BXray::FACTORY, "a new file's");
    assert!(cx.debug_bounds(BUTTON).is_some(), "the button before the shading radio");
    assert_eq!(pressed(&app, cx), (false, false));

    // Alt Z over the viewport: Solid's X-ray on, drawn through at 0.5.
    let canvas = cx.debug_bounds("view3d-canvas").expect("the viewport").center();
    cx.simulate_mouse_move(canvas, None, Modifiers::none());
    cx.simulate_keystrokes("alt-z");
    cx.run_until_parked();
    assert!(xray(&app, cx).solid && said() == "X-Ray on");
    assert_eq!(pressed(&app, cx), (true, false));
    assert_eq!(xray(&app, cx).see_through(BShading::Solid), 0.5);

    // Wireframe keeps its own: on already; Alt Z there turns it off only.
    let shade = |app: &gpui::Entity<BPhiberApp>, cx: &mut gpui::VisualTestContext, s: BShading| {
        app.update(cx, |app, cx| {
            app.state.set(&shading_key(), s.slug());
            cx.notify();
        });
        cx.run_until_parked();
    };
    shade(&app, cx, BShading::Wireframe);
    assert_eq!(pressed(&app, cx), (true, false), "Wireframe's X-ray is on in a new file");
    cx.simulate_mouse_move(canvas, None, Modifiers::none());
    cx.simulate_keystrokes("alt-z");
    cx.run_until_parked();
    let now = xray(&app, cx);
    assert!(!now.wireframe && now.solid, "Wireframe's off, Solid's untouched: {now:?}");

    // Material Preview: the button is dimmed; the switch flips and says so.
    shade(&app, cx, BShading::Material);
    assert_eq!(pressed(&app, cx), (true, true), "Solid's switch shows, dimmed");
    cx.simulate_mouse_move(canvas, None, Modifiers::none());
    cx.simulate_keystrokes("alt-z");
    cx.run_until_parked();
    assert!(!xray(&app, cx).solid && said() == "X-Ray not available in current mode");
    assert_eq!(xray(&app, cx).see_through(BShading::Material), 0.0, "nothing draws through");

    // Back in Solid, the button turns it on again.
    shade(&app, cx, BShading::Solid);
    let at = cx.debug_bounds(BUTTON).expect("the button").center();
    cx.simulate_click(at, Modifiers::none());
    cx.run_until_parked();
    assert!(xray(&app, cx).solid);
}
