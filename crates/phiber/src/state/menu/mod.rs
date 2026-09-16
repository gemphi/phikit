//! Menu state - tracks open menu paths and checked items.
#![allow(dead_code)]

pub mod ctx;
pub mod def;
pub mod r#use;

pub use def::MenuState;

use crate::store::menus::BMenu;

impl MenuState {
    /// Check whether a given menu key is currently open.
    /// A key is open if the open path exists and starts with the key's components.
    pub fn is_open(&self, key: &str) -> bool {
        let Some(path) = &self.open else {
            return false;
        };
        let parts: Vec<&str> = key.split("::").collect();
        path.len() >= parts.len() && path.iter().zip(parts.iter()).all(|(a, b)| a == b)
    }

    /// Open the menu at `key`, replacing an unrelated open path.
    /// If `key` is a descendant of the current path, it extends the path.
    pub fn open(&mut self, key: &str) {
        let target: Vec<String> = key.split("::").map(|s| s.to_string()).collect();
        if let Some(path) = &self.open {
            if path.len() < target.len() && target.starts_with(path) {
                // Opening a descendant of the current path - extend it.
                self.open = Some(target);
                return;
            }
        }
        self.open = Some(target);
    }

    /// Close the menu at `key`, truncating the open path if `key` is a prefix.
    /// If `key` matches the full path, the menu closes entirely.
    pub fn close(&mut self, key: &str) {
        let parts: Vec<String> = key.split("::").map(|s| s.to_string()).collect();
        if let Some(path) = &self.open {
            if path == &parts {
                self.open = None;
            } else if path.starts_with(&parts) && path.len() > parts.len() {
                self.open = Some(parts);
            }
        }
    }

    /// Toggle the menu at `key`.
    /// - If the path is exactly `key`, close it.
    /// - If the current path is a descendant of `key`, truncate to `key`.
    /// - If `key` is a descendant of the current path, extend to `key`.
    /// - Otherwise, close the current path and open `key`.
    pub fn toggle(&mut self, key: &str) {
        let target: Vec<String> = key.split("::").map(|s| s.to_string()).collect();
        if let Some(path) = &self.open {
            if path == &target {
                self.open = None;
                return;
            } else if target.starts_with(path) {
                self.open = Some(target);
                return;
            } else if path.starts_with(&target) && path.len() > target.len() {
                self.open = Some(target);
                return;
            }
        }
        self.open = Some(target);
    }

    /// Close every open menu.
    pub fn close_all(&mut self) {
        self.open = None;
    }

    /// Check whether a menu item label is currently checked.
    pub fn is_checked(&self, label: &str) -> bool {
        self.checked.contains(label)
    }

    /// Toggle a menu item's checked state by label.
    pub fn toggle_check(&mut self, label: &str) {
        if self.checked.contains(label) {
            self.checked.remove(label);
        } else {
            self.checked.insert(label.to_string());
        }
    }

    /// Build a stable dropdown key for a menu path.
    pub fn key_for(top: BMenu, path: &[usize]) -> String {
        let mut parts = vec![top.label().to_string()];
        for idx in path {
            parts.push(idx.to_string());
        }
        parts.join("::")
    }
}
