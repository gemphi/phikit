//! An editor's header too narrow for its controls clips them at its end, as
//! Blender's does; it never lays one control over another. The Timeline's
//! search was drawn across its jump arrows, so a click meant for an arrow
//! landed on the field.

use gpui::{Bounds, Pixels, TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{CtxKey, ModalKey};

fn overlap(a: &Bounds<Pixels>, b: &Bounds<Pixels>) -> f32 {
    let w = f32::from(a.right().min(b.right())) - f32::from(a.left().max(b.left()));
    let h = f32::from(a.bottom().min(b.bottom())) - f32::from(a.top().max(b.top()));
    if w > 0.5 && h > 0.5 { w * h } else { 0.0 }
}

/// The state key a control writes: `frame` for the frame field
/// (`…-edit-ctx.frame-number`) and for its arrows (`…-set-ctx.frame-5-…`).
fn key(selector: &str) -> Option<&str> {
    let at = selector.rfind("ctx.")? + 4;
    let rest = &selector[at..];
    Some(&rest[..rest.find('-').unwrap_or(rest.len())])
}

fn inside(outer: &Bounds<Pixels>, inner: &Bounds<Pixels>) -> bool {
    outer.left() <= inner.left() && outer.right() >= inner.right() && outer.top() <= inner.top() && outer.bottom() >= inner.bottom()
}

async fn crossings(cx: &mut TestAppContext, width: f32, space: BSpaceType) -> Vec<String> {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(width), px(800.0)));
    app.update(cx, |app, cx| {
        app.state.set(CtxKey::Space, space.slug());
        app.state.remove(ModalKey::Open);
        cx.notify();
    });
    cx.run_until_parked();
    let all = cx.all_debug_bounds();
    let headers: Vec<(String, Bounds<Pixels>)> = all.iter().filter(|(sel, _)| sel.starts_with("region-Header-")).cloned().collect();
    let controls: Vec<(String, Bounds<Pixels>)> = all.iter().filter(|(sel, _)| sel.starts_with("btn-") || sel.starts_with("search")).cloned().collect();
    let mut wrong = Vec::new();
    for (header, area) in &headers {
        // The controls whose middle is in this header — and only the part of
        // each the header shows: past its end a control is clipped away.
        let mine: Vec<&(String, Bounds<Pixels>)> = controls.iter().filter(|(_, b)| area.contains(&b.center())).collect();
        for (i, (a, ba)) in mine.iter().enumerate() {
            for (b, bb) in mine.iter().skip(i + 1) {
                // A number field's arrows are its own ends, over its box;
                // that is one control, not two crossing.
                if inside(ba, bb) || inside(bb, ba) || (a.contains("edit-ctx.") || b.contains("edit-ctx.")) && key(a) == key(b) {
                    continue;
                }
                if overlap(ba, bb) > 4.0 {
                    wrong.push(format!("{header} at {width}px: {a} {:?}+{:?} crosses {b} {:?}+{:?}", ba.origin.x, ba.size.width, bb.origin.x, bb.size.width));
                }
            }
        }
    }
    wrong
}

#[gpui::test]
async fn the_layout_space_headers_lay_no_control_over_another(cx: &mut TestAppContext) {
    let mut wrong = Vec::new();
    for width in [1400.0, 1000.0] {
        wrong.extend(crossings(cx, width, BSpaceType::Layout).await);
    }
    assert!(wrong.is_empty(), "{} crossings:\n  {}", wrong.len(), wrong.join("\n  "));
}

#[gpui::test]
async fn the_top_bar_wraps_before_its_search_reaches_its_menus(cx: &mut TestAppContext) {
    // At 13 px text the menus are wider than at 11, so the bar wraps sooner
    // (`filter::topbar_wraps`) rather than laying the search over Help.
    let wrong: Vec<String> = crossings(cx, 1100.0, BSpaceType::Layout).await.into_iter().filter(|line| line.contains("Topbar")).collect();
    assert!(wrong.is_empty(), "{} crossings:\n  {}", wrong.len(), wrong.join("\n  "));
}
