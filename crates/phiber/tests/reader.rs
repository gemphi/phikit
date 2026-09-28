//! The reader: Blender's manual and release notes read inside the window.
//!
//! Every Welcome and Help link but the Phiber website opens a document in
//! the reader — a sheet half the window wide over the workspace —
//! and closing the reader puts back the window it covered.

use phiber::v2::state::{BKey as _, CtxKey, ModalKey};
use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::modal::reader::{DOCS, doc_target, find};
use phiber::v2::modal::{about::ABOUT_LINKS, help::WEB_LINKS, splash};
use phiber::v2::state::{
    BState,
    val::{V_MODAL_HELP, V_MODAL_READER, V_MODAL_SPLASH},
};
use phiber::v2::view::prim::BAction;

#[test]
fn every_document_a_link_names_is_in_the_store() {
    let targets = splash::GETTING_STARTED
        .iter()
        .chain(splash::FOOT_LINKS)
        .map(|e| e.target)
        .chain(WEB_LINKS.iter().map(|e| e.target))
        .chain(ABOUT_LINKS.iter().map(|l| l.url));
    for target in targets {
        match doc_target(target) {
            Some(slug) => assert!(find(slug).is_some(), "no document for {target}"),
            None => assert!(
                target.starts_with("https://phiace.dev"),
                "{target} leaves the app; only the Phiber website may"
            ),
        }
    }
    assert!(DOCS.len() >= 10);
    for doc in DOCS {
        assert!(!doc.blocks.is_empty(), "{} is empty", doc.slug);
        assert_eq!(DOCS.iter().filter(|d| d.slug == doc.slug).count(), 1, "{} declared twice", doc.slug);
    }
}

#[test]
fn opening_a_document_raises_the_reader_and_closing_it_returns() {
    let mut s = BState::new();
    s.set(ModalKey::Open, V_MODAL_SPLASH);
    let open = splash::set::link_action("doc:whats_new");
    assert!(matches!(&open, BAction::Set(p, v) if p == &ModalKey::ReaderDoc.path() && v == "whats_new"));
    phiber::v2::state::action::apply_action(&mut s, &open);
    assert_eq!(s.get(ModalKey::Open), Some(V_MODAL_READER));
    assert_eq!(s.get(ModalKey::ReaderReturn), Some(V_MODAL_SPLASH));
    assert_eq!(phiber::v2::modal::reader::current(&s).slug, "whats_new");

    // Another document while reading swaps the page, not the return.
    phiber::v2::state::action::apply_action(&mut s, &splash::set::link_action("doc:manual"));
    assert_eq!(s.get(ModalKey::ReaderReturn), Some(V_MODAL_SPLASH));

    phiber::v2::state::action::apply_action(&mut s, &BAction::Set(ModalKey::Open.path(), String::new()));
    assert_eq!(s.get(ModalKey::Open), Some(V_MODAL_SPLASH), "closing the reader restores the welcome window");
    assert_eq!(s.get(ModalKey::ReaderReturn), None);
}

#[test]
fn the_phiber_website_still_leaves_the_app() {
    let action = splash::set::link_action("https://phiace.dev");
    assert!(matches!(&action, BAction::Set(p, v) if p.encode() == "ctx.link" && v == "https://phiace.dev"));
}

#[gpui::test]
async fn the_reader_is_a_sheet_half_the_window_wide(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1500.0), px(900.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, "layout");
        app.state.set(ModalKey::Open, V_MODAL_HELP);
        app.handle_action(&splash::set::link_action("doc:help"), cx);
        cx.notify();
    });
    cx.run_until_parked();
    let card = cx.debug_bounds("modal-card-Reader").expect("the reader is up");
    let width = f32::from(card.size.width);
    // A modal keeps one seam from the window's edge, as an area keeps from
    // the next; every gap in the system is that seam.
    let pad = phiber::v2::theme::ui_style_defaults(phiber::v2::theme::BPaletteName::DEFAULT).resize.gap;
    let sheet = (1500.0 - 2.0 * pad) / 2.0;
    assert!((width - sheet).abs() < 1.0, "sheet width {width}, expected {sheet}");
    assert!(f32::from(card.size.height) >= 900.0 - 2.0 * pad - 1.0, "the sheet takes the window's height");
    assert!(cx.debug_bounds("panel-body-surface").is_some(), "the page scrolls in a body panel");
}
