//! The browser's commits: adding a bookmark, and picking a file — which
//! records it in the recent list and dismisses the picker.

use super::super::{listing, sidebar};
use super::close_browser;
use crate::v2::modal::splash::recent;
use crate::v2::state::{BKey, BPath, BState, FileKey, rule::BRule};

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
        let purpose = super::super::purpose::purpose(s);
        if purpose == super::super::purpose::BFilePurpose::SaveImage {
            super::super::save_image::BImageSave::to(s, value);
            s.remove(&FileKey::Purpose.path());
        } else if !matches!(purpose, super::super::purpose::BFilePurpose::Mixdown | super::super::purpose::BFilePurpose::OpenImage) {
            // A mixdown or an image names no `.blend` for the recent list.
            recent::push_recent(s, value);
        }
        close_browser(s);
    }
}

/// `file.picked` → the name field takes the picked file's name, so Save As
/// writes over it only when the button says so.
pub struct BPickRule;

impl BRule for BPickRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == FileKey::Picked.path()
    }
    fn after(&self, _: &BPath, value: &str, s: &mut BState) {
        if let Some(name) = value.rsplit(['/', '\\']).next().filter(|name| !name.is_empty()) {
            s.set(FileKey::Filename, name.to_string());
        }
    }
}

/// What a click on a file does: the first picks it (lit, previewed, named);
/// a click on the file already picked commits it — a double click opens.
pub fn pick_or_commit(s: &BState, path: &str) -> crate::v2::view::prim::BAction {
    let key = if s.get(&FileKey::Picked.path()) == Some(path) { FileKey::Selected } else { FileKey::Picked };
    crate::v2::view::prim::BAction::Set(key.path(), path.to_string())
}

/// The file the browser has picked, else the one last committed.
pub fn picked(s: &BState) -> &str {
    s.get(&FileKey::Picked.path()).filter(|p| !p.is_empty()).or_else(|| s.get(&FileKey::Selected.path())).unwrap_or("")
}
