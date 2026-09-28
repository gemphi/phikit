//! The layout fingerprint: where every selector-tagged element lands, and
//! every quad, shadow and sprite the frame paints — in every space, at two
//! window sizes, and with each main modal open.
//!
//! This is the oracle for refactors of the render layer. Moving layout
//! decisions from one place to another must not move a single element, and
//! this is what proves it didn't: it compares the whole window against the
//! recorded golden, not a handful of hand-picked bounds.
//!
//! Record a new golden (only when a layout change is intended):
//!     UPDATE_LAYOUT_GOLDEN=1 cargo test -p phiber --test layout_fingerprint

use phiber::v2::state::{CtxKey, ModalKey};
use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState};

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/layout_fingerprint.txt");
const SIZES: &[(f32, f32)] = &[(1600.0, 900.0), (1100.0, 700.0)];
const MODALS: &[&str] = &["settings", "file_browser", "splash", "about", "render", "spaces", "collab", "profile", "command", "help", "progress", "reader"];

fn frame(cx: &mut gpui::VisualTestContext, label: &str, out: &mut String) {
    // Sorted, and without `image` lines: the viewport's traced frame arrives
    // from the render thread whenever it is ready, so whether it is in this
    // capture — and where the quads fall around it in paint order — varies
    // run to run. It is scene content, not chrome; everything else is exact.
    let mut painted: Vec<String> = cx
        .scene_fingerprint()
        .into_iter()
        .filter(|line| !line.starts_with("image "))
        .collect();
    painted.sort();
    for line in painted {
        out.push_str(&format!("{label} | {line}\n"));
    }
    for (selector, b) in cx.all_debug_bounds() {
        out.push_str(&format!(
            "{label} | {selector} | {:.2} {:.2} {:.2} {:.2}\n",
            f32::from(b.origin.x),
            f32::from(b.origin.y),
            f32::from(b.size.width),
            f32::from(b.size.height)
        ));
    }
}

#[gpui::test]
async fn the_whole_window_lands_where_the_golden_says(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    let mut out = String::new();
    for &(w, h) in SIZES {
        cx.simulate_resize(size(px(w), px(h)));
        for space in BSpaceType::ALL {
            app.update(cx, |app, cx| {
                app.state = BState::new();
                app.state.set(CtxKey::Space, space.slug());
                cx.notify();
            });
            cx.run_until_parked();
            frame(cx, &format!("{w}x{h} {}", space.slug()), &mut out);
        }
        for modal in MODALS {
            app.update(cx, |app, cx| {
                app.state = BState::new();
                app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
                app.state.set(ModalKey::Open, *modal);
                if *modal == "progress" {
                    // The progress screen is only up while a file is loading;
                    // without a load in flight it dismisses itself mid-capture.
                    app.state.set(&phiber::v2::scene::load_progress_key(), "0.4");
                    app.state.set(&phiber::v2::scene::load_stage_key(), "Reading objects 431/2104");
                }
                cx.notify();
            });
            cx.run_until_parked();
            frame(cx, &format!("{w}x{h} modal:{modal}"), &mut out);
        }
    }
    if std::env::var("UPDATE_LAYOUT_GOLDEN").is_ok() {
        std::fs::write(GOLDEN, &out).expect("write golden");
        eprintln!("recorded {} bounds to {GOLDEN}", out.lines().count());
        return;
    }
    let golden = std::fs::read_to_string(GOLDEN).expect("no golden — record one with UPDATE_LAYOUT_GOLDEN=1");
    if golden == out {
        return;
    }
    let (g, o): (Vec<_>, Vec<_>) = (golden.lines().collect(), out.lines().collect());
    let gs: std::collections::BTreeSet<_> = g.iter().copied().collect();
    let os: std::collections::BTreeSet<_> = o.iter().copied().collect();
    let gone: Vec<_> = gs.difference(&os).take(15).collect();
    let new: Vec<_> = os.difference(&gs).take(15).collect();
    panic!(
        "layout moved: {} golden line(s) no longer hold, {} new\n--- expected (first 15)\n{}\n--- got (first 15)\n{}",
        gs.difference(&os).count(),
        os.difference(&gs).count(),
        gone.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n"),
        new.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n")
    );
}
