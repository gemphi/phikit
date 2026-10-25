//! The file browser's directory is breadcrumbs, as Explorer's address bar
//! is: This PC › Local Disk (C:) › … on Windows, / › home › … elsewhere —
//! each crumb a folder's own path — and This PC is a folder too, listing
//! the drives, that a drive's root goes up to.

use phiber::v2::part::file::listing::{THIS_PC, parent_of, trail};

#[test]
fn a_windows_path_starts_at_this_pc_and_names_each_folder() {
    let crumbs = trail(r"C:\ProgramData\DockerDesktop");
    let names: Vec<&str> = crumbs.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names[0], THIS_PC);
    assert!(names[1].contains("C:"), "the drive, named as the source list names it: {names:?}");
    assert_eq!(&names[2..], ["ProgramData", "DockerDesktop"]);
    assert_eq!(crumbs[1].1, r"C:\");
    assert_eq!(crumbs[3].1, r"C:\ProgramData\DockerDesktop");
}

#[test]
fn a_posix_path_starts_at_the_root() {
    let crumbs = trail("/home/claude/ws");
    assert_eq!(crumbs, vec![("/".into(), "/".into()), ("home".into(), "/home".into()), ("claude".into(), "/home/claude".into()), ("ws".into(), "/home/claude/ws".into())]);
}

#[test]
fn a_drives_root_goes_up_to_this_pc_and_this_pc_is_the_top() {
    assert_eq!(parent_of(r"C:\").as_deref(), Some(THIS_PC));
    assert_eq!(parent_of(r"C:\Users").as_deref(), Some(r"C:\"));
    assert_eq!(parent_of(THIS_PC), None);
    assert_eq!(trail(THIS_PC), vec![(THIS_PC.to_string(), THIS_PC.to_string())]);
}

#[gpui::test]
async fn a_crumb_goes_to_its_folder_and_the_box_between_types_the_path(cx: &mut gpui::TestAppContext) {
    use gpui::{Modifiers, MouseButton, point, px, size};
    use phiber::v2::app::BPhiberApp;
    use phiber::v2::state::{AppKey, BKey as _, BState, CtxKey, FileKey, ModalKey};
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1600.0), px(900.0)));
    let dir = env!("CARGO_MANIFEST_DIR").to_string();
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(CtxKey::Space, "layout");
        app.state.set(ModalKey::Open, "file_browser");
        app.state.set(&FileKey::Path.path(), format!("{dir}/assets"));
        cx.notify();
    });
    cx.run_until_parked();
    let crumb = cx.debug_bounds("crumb-assets").expect("the folder's own crumb");
    let parent = cx.debug_bounds("crumb-phiber").expect("its parent's crumb");
    assert!(parent.origin.x < crumb.origin.x, "the crumbs read from the top down");
    cx.simulate_click(parent.center(), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(&FileKey::Path.path()).map(str::to_string)), Some(dir.clone()), "a crumb goes to its folder");
    // The box past the last crumb: a click there types the path.
    let field = cx.debug_bounds("crumbs").expect("the crumbs' box");
    let past = point(field.origin.x + field.size.width - px(8.0), field.center().y);
    cx.simulate_mouse_down(past, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(past, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(app.read_with(cx, |a, _| a.state.get(AppKey::Edit).map(str::to_string)), Some(FileKey::Path.path().encode()), "the caret is in the path");
}
