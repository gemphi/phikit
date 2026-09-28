//! File-browser operators — the reducer half of the file browser.
//! Blender ref: `space_file/file_ops.cc`.
//!
//! The view writes intents (`file.nav = "file.parent"`, `file.path =
//! "<dir>"`, `file.bookmark_add`, …); the rules here turn them into state
//! transitions. Each one is a [`BRule`], run by `state::rule` after the
//! intent is stored, and each is pure over `BState`, so it is testable
//! without GPUI.

use std::path::Path;

use super::history;
use super::listing;
use super::sidebar;
use crate::v2::modal::splash::recent;
use crate::v2::state::val::{V_EMPTY, V_FALSE, V_MODAL_FILE_BROWSER};
use crate::v2::state::{BKey, BPath, BState, FileKey, ModalKey, rule::BRule};

/// Operator names written into `file.nav` by the nav buttons.
pub const NAV_PREVIOUS: &str = "file.previous";
pub const NAV_NEXT: &str = "file.next";
pub const NAV_PARENT: &str = "file.parent";
pub const NAV_REFRESH: &str = "file.refresh";

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
        _ => {}
    }
    s.set(FileKey::Nav, V_EMPTY);
}

fn goto(s: &mut BState, target: &str) {
    if target.is_empty() {
        return;
    }
    if Path::new(target).is_file() {
        // A file, not a folder: select it rather than trying to enter it.
        s.set(FileKey::Selected, target.to_string());
        s.set(FileKey::Filename, file_name_of(target));
        return;
    }
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
        if value == super::purpose::BFilePurpose::SaveImage.slug() {
            seed_image_target(s);
        }
        if value == super::purpose::BFilePurpose::SaveAs.slug() || value == super::purpose::BFilePurpose::SaveCopy.slug() {
            seed_blend_target(s);
        }
        if !value.is_empty() && s.get(ModalKey::Open).filter(|m| !m.is_empty()).is_none() {
            s.set(ModalKey::Open, V_MODAL_FILE_BROWSER);
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
    if let Some(dir) = file.parent().filter(|d| !d.as_os_str().is_empty()) {
        history::goto(s, &dir.to_string_lossy());
    }
    // After `goto`, which clears the pending name as entering a folder does.
    s.set(FileKey::Filename, name);
}

/// `wm.save_as_mainfile` opens on the file that is open, named as it is,
/// so Return saves over it and a changed name saves beside it.
fn seed_blend_target(s: &mut BState) {
    let blend = s.get(&crate::v2::scene::loaded_path_key()).unwrap_or_default().to_string();
    let file = Path::new(&blend);
    let Some(name) = file.file_name().map(|n| n.to_string_lossy().to_string()).filter(|n| !n.is_empty()) else { return };
    if let Some(dir) = file.parent().filter(|d| !d.as_os_str().is_empty()) {
        history::goto(s, &dir.to_string_lossy());
    }
    s.set(FileKey::Filename, name);
}

/// `file.bookmark_add` → store the current directory in the next free slot
/// (`FILE_OT_bookmark_add`), then clear the intent.
pub struct BBookmarkAddRule;

impl BRule for BBookmarkAddRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == FileKey::BookmarkAdd.path()
    }
    fn after(&self, path: &BPath, _: &str, s: &mut BState) {
        let dir = listing::current_dir(s);
        sidebar::add_bookmark(s, &dir);
        s.remove(path);
    }
}

/// `file.selected` → a commit: record it in the welcome window's recent
/// list and dismiss whatever picker was on screen (the file browser modal,
/// or the splash itself).
pub struct BSelectRule;

impl BRule for BSelectRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == FileKey::Selected.path()
    }
    fn after(&self, _: &BPath, value: &str, s: &mut BState) {
        if value.is_empty() {
            return;
        }
        if super::purpose::purpose(s) == super::purpose::BFilePurpose::SaveImage {
            save_render_to(s, value);
            s.set(ModalKey::Open, V_EMPTY);
            s.remove(&FileKey::Purpose.path());
            return;
        }
        recent::push_recent(s, value);
        s.set(ModalKey::Open, V_EMPTY);
    }
}

/// Write the Render Result where the browser was pointed, and report it the
/// way Blender reports a save — in the status bar and the Info log, since a
/// silent write leaves no way to tell a finished save from a refused one.
///
/// The format comes from the Output panel rather than from the typed
/// extension, which is what Blender does: the extension follows the format,
/// not the other way round.
fn save_render_to(s: &mut BState, target: &str) {
    use crate::v2::part::text::info::{BReportLevel, push};
    let (shot, _) = crate::v2::scene::render_job::shot();
    let Some(shot) = shot else {
        push(s, BReportLevel::Error, "No render to save. Render an image first (F12).");
        return;
    };
    // A render still in flight is refused by the writer itself, with its own
    // message — the guard is not repeated here, so there is one place that
    // decides whether a shot can be written.
    let output = crate::v2::scene::output_set(s);
    let pass = crate::v2::scene::pass_set(s, Default::default());
    let file = output.file_of(Path::new(target).parent().unwrap_or(Path::new("")));
    // `file_of` is given the typed path's folder and the panel's own name,
    // so a name typed in the browser wins over the panel's while the format
    // still decides the extension.
    let file = match Path::new(target).file_name() {
        Some(name) => file.with_file_name(name).with_extension(output.format.extension().trim_start_matches('.')),
        None => file,
    };
    let written = crate::v2::scene::written_pass(&shot, pass);
    match crate::v2::scene::save_render_to(&shot, &output, pass, &file) {
        Ok(()) => {
            let note = if written.len() > 1 { format!(" ({} passes)", written.len()) } else { String::new() };
            push(s, BReportLevel::Info, &format!("Saved: '{}'{note}", file.display()));
        }
        Err(why) => push(s, BReportLevel::Error, &format!("Cannot save: {why}")),
    }
}
