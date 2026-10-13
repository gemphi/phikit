//! Opening a file no longer takes the screen. Its progress lives at the foot
//! of the window: while a file loads the status zone grows into a full-width
//! loading panel — the file being read, a running log of the stages with the
//! time each took, a Cancel, and a bar across the bottom edge — and collapses
//! back to the ordinary status bar when the scene settles. The viewport is
//! never covered, so the scene is watched as it appears.

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::modal::{BModalList, BModalType};
use phiber::v2::part::statusbar::BStatusbar;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState, CtxKey, ModalKey};
use phiber::v2::state::log;
use phiber::v2::scene::{load_progress_key, load_stage_key, selected_key};
use phiber::v2::view::{BView, BViewItem, BViewList};

/// The labels a built view carries, flattened — including a button's label and
/// a progress bar's stage.
fn labels(v: &BView, out: &mut Vec<String>) {
    match v {
        BView::Label { text } | BView::Field { text } => out.push(text.clone()),
        BView::ProgressBar { label, .. } => out.push(label.clone()),
        BView::Button(ctl) => {
            if let Some(label) = &ctl.label {
                out.push(label.clone());
            }
        }
        BView::Modal { body, .. } => labels(body, out),
        BView::Column { list } | BView::Row { list } | BView::ListRow { list, .. } => {
            list.iter().for_each(|c| labels(c, out))
        }
        BView::Card(card)
            if matches!(card.kind, phiber::v2::view::prim::BCardKind::Column(_)) =>
        {
            card.body.iter().for_each(|c| labels(c, out))
        }
        _ => {}
    }
}

/// Whether the view holds a wide (loading-panel) progress bar.
fn has_wide_bar(v: &BView) -> bool {
    match v {
        BView::ProgressBar { wide, .. } => *wide,
        BView::Column { list } | BView::Row { list } | BView::ListRow { list, .. } => {
            list.iter().any(has_wide_bar)
        }
        _ => false,
    }
}

fn panel(s: &BState) -> BView {
    BStatusbar::new(BHostType::Statusbar).build(s)
}

#[test]
fn the_status_bar_becomes_a_loading_panel_and_goes_back() {
    let mut s = BState::new();

    // Idle: the ordinary segment row — no Cancel, no loading bar.
    let idle = panel(&s);
    let mut idle_labels = Vec::new();
    labels(&idle, &mut idle_labels);
    assert!(!idle_labels.iter().any(|l| l == "Cancel"), "{idle_labels:?}");
    assert!(!has_wide_bar(&idle), "no loading bar when nothing is opening");

    // A file opens: the bar gives way to the panel.
    s.set(&selected_key(), "C:/files/THYMUS-BOLDROID-WAVE-003.blend");
    s.set(&load_progress_key(), "0.4");
    s.set(&load_stage_key(), "Reading objects 431/2104 Cube.003");
    let loading = panel(&s);
    let mut found = Vec::new();
    labels(&loading, &mut found);
    assert!(
        found.iter().any(|l| l.contains("THYMUS")),
        "it names the file being opened: {found:?}"
    );
    assert!(
        found.iter().any(|l| l == "Reading objects 431/2104 Cube.003"),
        "and the live stage it is on: {found:?}"
    );
    assert!(found.iter().any(|l| l == "Cancel"), "and a Cancel: {found:?}");
    assert!(has_wide_bar(&loading), "with a full-width bar across the foot");

    // Settled: back to the ordinary row.
    s.remove(&load_progress_key());
    let back = panel(&s);
    let mut back_labels = Vec::new();
    labels(&back, &mut back_labels);
    assert!(!back_labels.iter().any(|l| l == "Cancel"), "{back_labels:?}");
    assert!(!has_wide_bar(&back), "the panel collapses when the scene settles");
}

#[test]
fn the_panel_shows_the_stages_with_their_timings() {
    log::clear();
    log::push("Reading file");
    log::push("Parsing meshes"); // newest first

    let mut s = BState::new();
    s.set(&load_progress_key(), "0.6");
    s.set(&load_stage_key(), "Parsing meshes");

    let mut found = Vec::new();
    labels(&panel(&s), &mut found);
    assert!(found.iter().any(|l| l == "Reading file"), "{found:?}");
    assert!(found.iter().any(|l| l == "Parsing meshes"), "{found:?}");
    // The newer line carries the time since the one before it.
    assert!(
        found.iter().any(|l| l.starts_with('+') && (l.ends_with("ms") || l.ends_with('s'))),
        "a step shows its elapsed time: {found:?}"
    );
}

#[test]
fn opening_a_file_does_not_raise_a_modal() {
    let mut s = BState::new();
    // A load in flight: nothing is raised over the screen.
    s.set(&load_progress_key(), "0.4");
    s.set(&load_stage_key(), "Reading file");
    assert!(BModalList::new().items(&s).is_empty(), "the load is not a modal");

    // A dialog the user opened stays up and is not shoved aside by the load.
    s.set(ModalKey::Open, BModalType::FileBrowser.slug());
    let shown = BModalList::new().items(&s);
    assert!(matches!(&shown[0], BView::Modal { kind, .. } if *kind == BModalType::FileBrowser));
}

// ── Painted: the zone grows and collapses ──────────────────────────────────

async fn workspace(
    cx: &mut TestAppContext,
) -> (gpui::Entity<BPhiberApp>, &mut gpui::VisualTestContext) {
    log::clear();
    cx.update(|cx| cx.set_reduce_motion(true));
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();
    (app, cx)
}

fn height(cx: &mut gpui::VisualTestContext, id: &'static str) -> f32 {
    f32::from(cx.debug_bounds(id).unwrap_or_else(|| panic!("{id} was not painted")).size.height)
}

#[gpui::test]
async fn the_status_zone_expands_while_loading_and_collapses_after(cx: &mut TestAppContext) {
    let (app, cx) = workspace(cx).await;
    let folded = height(cx, "region-Header-Statusbar");

    app.update(cx, |app, cx| {
        app.state.set(&load_progress_key(), "0.4");
        app.state.set(&load_stage_key(), "Reading objects 431/2104 Cube.003");
        cx.notify();
    });
    cx.run_until_parked();
    let loading = height(cx, "region-Header-Statusbar");
    assert!(
        loading > folded + 1.0,
        "the status zone grows into the panel ({loading} vs {folded})"
    );
    // The bar runs the width of the window along the foot.
    let bar = cx.debug_bounds("load-progress-track").expect("the loading bar is painted");
    let region = cx.debug_bounds("region-Header-Statusbar").unwrap();
    assert!(
        f32::from(bar.size.width) > f32::from(region.size.width) * 0.8,
        "the bar runs across the foot ({} of {})",
        f32::from(bar.size.width),
        f32::from(region.size.width),
    );

    app.update(cx, |app, cx| {
        app.state.remove(&load_progress_key());
        app.state.remove(&load_stage_key());
        cx.notify();
    });
    cx.run_until_parked();
    let back = height(cx, "region-Header-Statusbar");
    assert!((back - folded).abs() < 1.0, "it collapses back ({back} vs {folded})");
    assert!(cx.debug_bounds("load-progress-track").is_none(), "the bar is gone");
}
