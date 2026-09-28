//! Folder back/forward stacks.
//! Blender ref: `folder_history.cc` (`folderlist_pushdir` / `popdir` /
//! `clear_next`).
//!
//! Both stacks are newline-separated path lists stored in one state value,
//! most recent last. Navigating to a new directory pushes the old one onto
//! Back and clears Forward, exactly like a browser.

use crate::v2::state::BKey as _;
use crate::v2::state::FileKey;
use crate::v2::state::{BPath, BState};

fn read(s: &BState, key: &BPath) -> Vec<String> {
    s.get(key)
        .unwrap_or("")
        .split('\n')
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect()
}

fn write(s: &mut BState, key: &BPath, stack: &[String]) {
    s.set(key, stack.join("\n"));
}

pub fn back_stack(s: &BState) -> Vec<String> {
    read(s, &FileKey::HistBack.path())
}
pub fn fwd_stack(s: &BState) -> Vec<String> {
    read(s, &FileKey::HistFwd.path())
}

pub fn can_back(s: &BState) -> bool {
    !back_stack(s).is_empty()
}
pub fn can_forward(s: &BState) -> bool {
    !fwd_stack(s).is_empty()
}

/// Move to `target`, remembering where we came from and dropping the forward
/// trail (`folderlist_clear_next`).
pub fn goto(s: &mut BState, target: &str) {
    let current = s.get(&FileKey::Path.path()).unwrap_or("").to_string();
    if current == target {
        return;
    }
    if !current.is_empty() {
        let mut back = back_stack(s);
        back.push(current);
        // Keep the stack bounded; nobody navigates back further than this.
        if back.len() > 64 {
            back.remove(0);
        }
        write(s, &FileKey::HistBack.path(), &back);
    }
    s.set(&FileKey::HistFwd.path(), String::new());
    s.set(&FileKey::Path.path(), target.to_string());
}

/// `FILE_OT_previous`.
pub fn go_back(s: &mut BState) {
    let mut back = back_stack(s);
    let Some(target) = back.pop() else {
        return;
    };
    let current = s.get(&FileKey::Path.path()).unwrap_or("").to_string();
    let mut fwd = fwd_stack(s);
    if !current.is_empty() {
        fwd.push(current);
    }
    write(s, &FileKey::HistBack.path(), &back);
    write(s, &FileKey::HistFwd.path(), &fwd);
    s.set(&FileKey::Path.path(), target);
}

/// `FILE_OT_next`.
pub fn go_forward(s: &mut BState) {
    let mut fwd = fwd_stack(s);
    let Some(target) = fwd.pop() else {
        return;
    };
    let current = s.get(&FileKey::Path.path()).unwrap_or("").to_string();
    let mut back = back_stack(s);
    if !current.is_empty() {
        back.push(current);
    }
    write(s, &FileKey::HistBack.path(), &back);
    write(s, &FileKey::HistFwd.path(), &fwd);
    s.set(&FileKey::Path.path(), target);
}
