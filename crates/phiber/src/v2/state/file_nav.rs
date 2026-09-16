//! File-browser operators — the reducer half of the file browser.
//!
//! The view writes intents (`ctx.file.nav = "file.parent"`, `ctx.file.path =
//! "<dir>"`); this module turns them into state transitions. Previously
//! nothing consumed those keys at all: the Back / Forward / Up / Refresh
//! buttons wrote a value that no code ever read, so they did nothing.
//!
//! Pure `fn(&mut BState)` like every other operator here (`menu`,
//! `space_switch`, `area_resize`), so it is testable without GPUI.

use std::path::Path;

use super::{BPath, BState, val::V_EMPTY};
use crate::v2::part::custom::file::{history, keys, listing, purpose::purpose};

/// Operator names written into `ctx.file.nav` by the nav buttons.
pub const NAV_PREVIOUS: &str = "file.previous";
pub const NAV_NEXT: &str = "file.next";
pub const NAV_PARENT: &str = "file.parent";
pub const NAV_REFRESH: &str = "file.refresh";

/// True when `path` is one of the keys this module reduces.
pub fn handles(path: &BPath) -> bool {
    let encoded = path.encode();
    encoded == "ctx.file.nav" || encoded == "ctx.file.path"
}

/// Apply a committed write to `ctx.file.nav` or `ctx.file.path`.
///
/// `ctx.file.path` is treated as a navigation *request*: it records history
/// and, when the target is a file rather than a directory, promotes it to a
/// selection instead of a directory change (this is what makes clicking a
/// `.blend` row in the list open it).
pub fn apply(state: &mut BState, path: &BPath, value: &str) {
    match path.encode().as_str() {
        "ctx.file.nav" => apply_nav(state, value),
        "ctx.file.path" => apply_goto(state, value),
        _ => {}
    }
}

fn apply_nav(state: &mut BState, op: &str) {
    match op {
        NAV_PREVIOUS => history::go_back(state),
        NAV_NEXT => history::go_forward(state),
        NAV_PARENT => {
            let current = listing::current_dir(state);
            if let Some(parent) = listing::parent_of(&current) {
                history::goto(state, &parent);
            }
        }
        // Listings are read fresh every frame, so a refresh only needs to
        // clear any stale error banner.
        NAV_REFRESH => {
            state.set(&keys::k_error(), V_EMPTY);
        }
        _ => {}
    }
    state.set(&keys::k_nav(), V_EMPTY);
}

fn apply_goto(state: &mut BState, target: &str) {
    if target.is_empty() {
        return;
    }
    if Path::new(target).is_file() {
        // A file, not a folder: select it rather than trying to enter it.
        state.set(&keys::k_selected(), target.to_string());
        state.set(&keys::k_filename(), file_name_of(target));
        return;
    }
    history::goto(state, target);
    // Entering a directory clears the pending filename, as Blender does.
    state.set(&keys::k_filename(), V_EMPTY);
}

fn file_name_of(path: &str) -> String {
    path.rsplit(['/', '\\']).next().unwrap_or(path).to_string()
}

/// The execute button's label for the current purpose.
pub fn exec_label(state: &BState) -> &'static str {
    purpose(state).exec_label()
}
