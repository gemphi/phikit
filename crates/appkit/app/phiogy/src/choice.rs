//! What a person chose: which look, which room. A choice a line, its
//! name and what was chosen between tabs, in a file of their own that is
//! read when Phiogy opens and written each time they choose. Where there
//! is no folder to keep it in, a choice lasts as long as the window.

use std::cell::RefCell;

use appkit::service::{PStore, PTable};

/// What a person chose.
#[derive(Debug, Default)]
pub struct Choice {
    row: RefCell<Vec<(String, String)>>,
    /// Where the choices are kept, and in which file, if they are.
    keep: Option<(PStore, String)>,
}

impl Choice {
    /// The choices kept in that file of a store, kept there from now on.
    pub fn kept(store: PStore, name: &str) -> Choice {
        let row = PTable::read(&store.read(name).unwrap_or_default()).list().to_vec();
        Choice { row: RefCell::new(row), keep: Some((store, name.to_string())) }
    }

    /// The folder the choices are kept in, if they are.
    pub fn place(&self) -> Option<String> {
        self.keep.as_ref().map(|(store, _)| store.path())
    }

    /// What was chosen under that name; nothing when nothing was.
    pub fn get(&self, name: &str) -> String {
        self.row.borrow().iter().rev().find(|(known, _)| known == name).map(|(_, value)| value.clone()).unwrap_or_default()
    }

    /// Every choice whose name begins so, with the rest of its name.
    pub fn starting(&self, prefix: &str) -> Vec<(String, String)> {
        self.row.borrow().iter().filter_map(|(name, value)| Some((name.strip_prefix(prefix)?.to_string(), value.clone()))).collect()
    }

    /// Choose something under that name, and keep it.
    pub fn put(&self, name: &str, value: &str) {
        let mut row = self.row.borrow_mut();
        row.retain(|(known, _)| known != name);
        row.push((name.to_string(), value.to_string()));
        let text: String = row.iter().map(|(name, value)| format!("{name}\t{value}\n")).collect();
        self.keep.iter().for_each(|(store, file)| _ = store.write(file, &text));
    }
}
