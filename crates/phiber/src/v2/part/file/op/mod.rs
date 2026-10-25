//! File-browser operators — the reducer half of the file browser.
//! Blender ref: `space_file/file_ops.cc`.
//!
//! The view writes intents (`file.nav = "file.parent"`, `file.path =
//! "<dir>"`, `file.bookmark_add`, …); the rules here turn them into state
//! transitions. Each one is a [`BRule`], run by `state::rule` after the
//! intent is stored, and each is pure over `BState`, so it is testable
//! without GPUI.

use std::path::Path;

use super::{history, listing};
use crate::v2::state::val::{V_EMPTY, V_FALSE, V_MODAL_FILE_BROWSER, V_MODAL_RENDER};
use crate::v2::state::{BKey, BPath, BState, FileKey, ModalKey, rule::BRule};

/// Operator names written into `file.nav` by the nav buttons.
pub const NAV_PREVIOUS: &str = "file.previous";
pub const NAV_NEXT: &str = "file.next";
pub const NAV_PARENT: &str = "file.parent";
pub const NAV_REFRESH: &str = "file.refresh";
pub const NAV_NEW_FOLDER: &str = "file.directory_new";

/// `file.sort` → the new column starts ascending (`FILE_OT_sort_column_ui_context`).
/// A repeated click uses the dedicated direction toggle instead.
pub struct BSortRule;

impl BRule for BSortRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == FileKey::Sort.path()
    }
    fn after(&self, _: &BPath, _: &str, s: &mut BState) {
        s.set(FileKey::SortInvert, V_FALSE);
    }
}

/// `file.nav` and `file.path` → the folder history (`FILE_OT_previous`,
/// `_next`, `_parent`, `_refresh`, and a typed or clicked path).
///
/// `file.path` is a navigation *request*: it records history and, when the
/// target is a file rather than a directory, promotes it to a selection
/// instead of a directory change (this is what makes clicking a `.blend`
/// row in the list open it).
pub struct BNavRule;

impl BRule for BNavRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == FileKey::Nav.path() || *path == FileKey::Path.path()
    }
    fn after(&self, path: &BPath, value: &str, s: &mut BState) {
        if *path == FileKey::Nav.path() {
            nav(s, value);
        } else {
            goto(s, value);
        }
    }
}

fn nav(s: &mut BState, op: &str) {
    match op {
        NAV_PREVIOUS => history::go_back(s),
        NAV_NEXT => history::go_forward(s),
        NAV_PARENT => {
            let current = listing::current_dir(s);
            if let Some(parent) = listing::parent_of(&current) {
                history::goto(s, &parent);
            }
        }
        // Listings are read fresh every frame, so a refresh only needs to
        // clear any stale error banner.
        NAV_REFRESH => s.set(FileKey::Error, V_EMPTY),
        // `file.directory_new`: "New Folder" in the folder shown, numbered
        // past any already there, and the browser goes into it.
        NAV_NEW_FOLDER => {
            let dir = listing::current_dir(s);
            let made = (0..100).map(|n| if n == 0 { "New Folder".to_string() } else { format!("New Folder {n}") }).map(|name| listing::join(&dir, &name)).find(|p| !Path::new(p).exists());
            match made.map(|p| std::fs::create_dir(&p).map(|_| p)) {
                Some(Ok(p)) => history::goto(s, &p),
                Some(Err(e)) => s.set(FileKey::Error, format!("Could not create a folder: {e}")),
                None => {}
            }
        }
        _ => {}
    }
    s.set(FileKey::Nav, V_EMPTY);
}

fn goto(s: &mut BState, target: &str) {
    if target.is_empty() {
        return;
    }
    if Path::new(target).is_file() {
        // A file, not a folder: go to its folder and pick it there — lit and
        // previewed, not opened (`BPickRule` names it).
        if let Some(dir) = listing::parent_of(target) {
            history::goto(s, &dir);
        }
        s.set(FileKey::Picked, target.to_string());
        s.set(FileKey::Filename, file_name_of(target));
        return;
    }
    s.remove(&FileKey::Picked.path());
    history::goto(s, target);
    // Entering a directory clears the pending filename, as Blender does.
    s.set(FileKey::Filename, V_EMPTY);
}

fn file_name_of(path: &str) -> String {
    path.rsplit(['/', '\\']).next().unwrap_or(path).to_string()
}

/// `file.purpose` → raise the browser when no modal is up. A job requested
/// from a menu (Open…, Save As…, Import ▸ …) invokes the browser window,
/// as Blender's `WM_OT_open_mainfile` does.
pub struct BPurposeRule;

impl BRule for BPurposeRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == FileKey::Purpose.path()
    }
    fn after(&self, _: &BPath, value: &str, s: &mut BState) {
        // `export:<format>`: the format is kept for the commit
        // (`app/op.rs`), the purpose itself is plain.
        if let Some(format) = value.strip_prefix("export:") {
            s.set(crate::v2::state::OpKey::ExportFormat, format.to_string());
            s.set(FileKey::Purpose, super::purpose::BFilePurpose::Export.slug());
        }
        match super::purpose::BFilePurpose::from_slug(value) {
            Some(super::purpose::BFilePurpose::SaveImage) => seed_image_target(s),
            Some(super::purpose::BFilePurpose::Mixdown) => seed_mixdown_target(s),
            Some(super::purpose::BFilePurpose::SaveAs | super::purpose::BFilePurpose::SaveCopy) => seed_blend_target(s),
            _ => {}
        }
        if value.is_empty() {
            return;
        }
        match s.get(ModalKey::Open).filter(|m| !m.is_empty()) {
            None => s.set(ModalKey::Open, V_MODAL_FILE_BROWSER),
            // Blender's render window carries the Image editor's own menus,
            // and Image ▸ Save As there opens the file browser over it
            // (`IMAGE_OT_save_as` invokes a file select like any operator).
            // The render view is a modal here and modals are one at a time,
            // so the browser remembers it and `BFileReturnRule` puts it
            // back once the browser closes — committed or cancelled. This
            // was the whole of "Image ▸ Save As does nothing": the purpose
            // was set, the browser never raised, no PNG ever reached disk.
            Some(V_MODAL_RENDER) => {
                s.set(ModalKey::FileReturn, V_MODAL_RENDER);
                s.set(ModalKey::Open, V_MODAL_FILE_BROWSER);
            }
            Some(_) => {}
        }
    }
}

/// Close the browser: the modal it covered comes back, or none. A commit
/// calls this itself (a rule's own `set` runs no rule); a cancel is the
/// action `modal = ""`, which `BFileReturnRule` answers.
pub fn close_browser(s: &mut BState) {
    // A field of the browser left in edit mode would go on taking the keys
    // — Escape among them — after the browser is gone.
    crate::v2::state::edit::stop(s);
    let back = s.get(ModalKey::FileReturn).map(str::to_string);
    s.remove(ModalKey::FileReturn);
    s.set(ModalKey::Open, back.unwrap_or_else(|| V_EMPTY.to_string()));
}

/// `modal = ""` → put back the modal the file browser covered, if any.
pub struct BFileReturnRule;

impl BRule for BFileReturnRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == ModalKey::Open.path()
    }
    fn after(&self, _: &BPath, value: &str, s: &mut BState) {
        if value.is_empty() && s.get(ModalKey::FileReturn).is_some() {
            close_browser(s);
        }
    }
}

/// Open the browser where Properties → Output points, with the file name it
/// would use, the way `IMAGE_OT_save_as` seeds itself from the render output
/// settings rather than dropping the user in their home folder.
///
/// A `//` path resolves against the loaded `.blend`'s own folder, so this
/// needs the path the scene was opened from; without one it leaves the
/// browser wherever it already was.
fn seed_image_target(s: &mut BState) {
    let output = crate::v2::scene::output_set(s);
    let blend = s.get(&crate::v2::scene::loaded_path_key()).unwrap_or_default().to_string();
    let folder = Path::new(&blend).parent().map(Path::to_path_buf).unwrap_or_default();
    let file = output.file_of(&folder);
    let Some(name) = file.file_name().map(|n| n.to_string_lossy().to_string()) else { return };
    seed_target(s, &file, name);
}

/// `sound.mixdown` opens beside the `.blend`, on its name with Blender's
/// default container's extension (`sound_mixdown_check`: `.flac`).
fn seed_mixdown_target(s: &mut BState) {
    let blend = s.get(&crate::v2::scene::loaded_path_key()).unwrap_or_default().to_string();
    seed_target(s, Path::new(&blend), crate::v2::scene::mixdown::BMixdown::name_for(Path::new(&blend)));
}

/// `wm.save_as_mainfile` opens on the file that is open, named as it is,
/// so Return saves over it and a changed name saves beside it.
fn seed_blend_target(s: &mut BState) {
    let blend = s.get(&crate::v2::scene::loaded_path_key()).unwrap_or_default().to_string();
    let file = Path::new(&blend);
    let Some(name) = file.file_name().map(|n| n.to_string_lossy().to_string()).filter(|n| !n.is_empty()) else { return };
    seed_target(s, file, name);
}

/// The browser in `file`'s folder, on `name` — after `goto`, which clears
/// the pending name as entering a folder does.
fn seed_target(s: &mut BState, file: &Path, name: String) {
    if let Some(dir) = file.parent().filter(|d| !d.as_os_str().is_empty()) {
        history::goto(s, &dir.to_string_lossy());
    }
    s.set(FileKey::Filename, name);
}

mod select;
pub use select::{BBookmarkAddRule, BPickRule, BSelectRule, pick_or_commit, picked};

#[cfg(test)]
mod stacking_tests;
