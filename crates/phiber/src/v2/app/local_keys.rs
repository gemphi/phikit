//! Which state keys never leave this session — the one place that question is
//! answered, for the two things that ask it.
//!
//! A collaborator must not receive my open menu, my caret or my camera
//! (`collab`), and a resumed launch must not come back mid-interaction
//! (`session`). The two lists are not the same — a workspace and the file
//! browser's folder are mine to keep but no one else's business — so they are
//! two lists, but the shape of the answer is one: a set of keys, a set of
//! families, and a key enum that owns each domain name.
//!
//! Nothing here spells a key prefix. `BKey::family` takes it from a key of
//! that domain, so a domain that moves takes its prefix with it
//! (`tests/key_budget.rs`); the lists used to be string literals, and a key
//! that moved would have gone on syncing, or stopped resuming, with nothing
//! in either file to say that it had.

use crate::v2::state::{AppKey, AreaKey, BKey as _, CtxKey, FileKey, ImageKey, MenuKey, ModalKey, SceneKey, View3dKey};

/// True when `encoded` is one of `keys`, or sits under one of `families`.
fn matches(encoded: &str, keys: &[String], families: &[String]) -> bool {
    keys.iter().any(|key| key == encoded) || families.iter().any(|family| encoded.starts_with(family.as_str()))
}

/// Keys that are each session's own and are never sent to a peer: presence
/// itself, open menus and dialogs, layout drags, the text caret, the viewport
/// camera, the import machinery, the file browser, the status log and the
/// chosen workspace.
pub fn never_sent(encoded: &str) -> bool {
    let keys = [
        ModalKey::Open.path().encode(),
        AppKey::Edit.path().encode(),
        AppKey::EditUndo.path().encode(),
        CtxKey::Space.path().encode(),
        // The scene itself syncs; only the loading machinery is local.
        crate::v2::scene::loaded_path_key().encode(),
        SceneKey::Loading.path().encode(),
    ];
    let families = [
        AppKey::CollabStatus.family(),
        MenuKey::Top("", "").family(),
        MenuKey::Panel("", "").family(),
        AreaKey::DragSpace.family(),
        View3dKey::Yaw.family(),
        AppKey::StatusLogOpen.family(),
        FileKey::Path.family(),
    ];
    matches(encoded, &keys, &families)
}

/// Keys a resumed session does not come back to: an open dialog, menu or
/// header popover, an area-resize drag, the import machinery or scene data,
/// and the file-browser keys that are about the current browse rather than a
/// preference — what is selected, an error, the nav intent and its history,
/// the add-bookmark intent, the open filter popover. A resumed browser sits
/// at its folder, not mid-interaction.
pub fn never_resumed(encoded: &str) -> bool {
    let keys = [
        ModalKey::Open.path().encode(),
        ModalKey::FileReturn.path().encode(),
        CtxKey::Selected.path().encode(),
        // The Image editor's shown name points into the session's image list,
        // which does not persist: resumed, it would dangle and paint nothing.
        ImageKey::Shown.path().encode(),
        AreaKey::Maximized.path().encode(),
        FileKey::Selected.path().encode(),
        FileKey::Error.path().encode(),
        FileKey::Nav.path().encode(),
        FileKey::HistBack.path().encode(),
        FileKey::HistFwd.path().encode(),
        FileKey::BookmarkAdd.path().encode(),
        FileKey::FilterOpen.path().encode(),
    ];
    let families = [
        MenuKey::Top("", "").family(),
        MenuKey::Panel("", "").family(),
        AreaKey::DragSpace.family(),
        // Every scene key but the last file is scene data or a transient flag
        // (loading, error, unsaved, the edit-delta markers, the 3D cursor, the
        // resume stash): reopening the file restores the scene, not these.
        SceneKey::Loading.family(),
    ];
    matches(encoded, &keys, &families)
}
