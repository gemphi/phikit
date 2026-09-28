//! Typing in the file browser, driven through the real window.
//!
//! Blender's File View filters the list as you type in the search box and
//! navigates when you type a path and press Return. These drive the same
//! path the user does: click the field, press keys, read the rendered list.

use phiber::v2::state::{BKey as _, AppKey, CtxKey};
use phiber::v2::state::FileKey;
use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::part::file::listing;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState, edit};
use phiber::v2::view::prim::BAction;

/// The fixtures folder, which holds several `.blend` files with known names.
fn fixtures() -> String {
    concat!(env!("CARGO_MANIFEST_DIR"), "/../phiphx/tests/fixtures").to_string()
}

/// Names the file list would show for the current state.
fn listed(s: &BState) -> Vec<String> {
    listing::rows_for(s)
        .expect("the fixtures folder is readable")
        .into_iter()
        .map(|r| r.name)
        .collect()
}

#[gpui::test]
async fn typing_in_the_search_box_filters_the_list(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        app.state.set(&FileKey::Path.path(), fixtures());
        cx.notify();
    });
    cx.run_until_parked();

    app.update(cx, |app, cx| {
        let all = listed(&app.state);
        assert!(all.len() > 2, "the fixtures folder lists several files: {all:?}");

        // Clicking the search field puts the caret in it.
        app.handle_action(
            &BAction::Set(AppKey::Edit.path(), FileKey::Search.path().encode()),
            cx,
        );
        assert!(edit::is_editing(&app.state, &FileKey::Search.path()));

        // Typing "normal" leaves only NORMAL-MAP.blend.
        for ch in "normal".chars() {
            app.handle_key(&ch.to_string(), Some(&ch.to_string()), None, cx);
        }
        let filtered = listed(&app.state);
        assert!(
            filtered.iter().all(|n| n.to_lowercase().contains("normal")),
            "every listed file matches the query: {filtered:?}"
        );
        assert!(!filtered.is_empty(), "NORMAL-MAP.blend survives the filter");

        // Escape restores the list.
        app.handle_key("escape", None, None, cx);
        assert_eq!(listed(&app.state).len(), all.len(), "Escape clears the filter");
        assert!(!edit::is_editing(&app.state, &FileKey::Search.path()));
    });
}

#[gpui::test]
async fn typing_a_folder_path_navigates_there(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();

    app.update(cx, |app, cx| {
        // Caret in the directory field, then type the fixtures path.
        app.handle_action(&BAction::Set(AppKey::Edit.path(), FileKey::Path.path().encode()), cx);
        app.handle_key("delete", None, None, cx);
        for ch in fixtures().chars() {
            app.handle_key(&ch.to_string(), Some(&ch.to_string()), None, cx);
        }
        // Nothing moves until Return: this is a text field, not a live jump.
        app.handle_key("enter", None, None, cx);
        assert!(!edit::is_editing(&app.state, &FileKey::Path.path()), "Return drops the caret");
        assert_eq!(
            listing::current_dir(&app.state).replace('\\', "/"),
            fixtures().replace('\\', "/"),
            "the browser is in the typed folder"
        );
        assert!(
            listed(&app.state).iter().any(|n| n.ends_with(".blend")),
            "and it lists that folder's blend files"
        );
    });
}
