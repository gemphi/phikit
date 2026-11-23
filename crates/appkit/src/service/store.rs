//! A store: the folder an app keeps a person's own files in, so that what
//! they did is there the next time: their chats, their settings. Where no
//! such folder can be had, in a browser, nothing is kept yet.

use std::path::PathBuf;

/// A folder an app reads and writes its own files in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PStore {
    root: PathBuf,
}

impl PStore {
    /// A store in that folder.
    pub fn at(root: &str) -> PStore {
        PStore { root: PathBuf::from(root) }
    }

    /// The store of the app of that name for the person using it: a
    /// folder of the app's name where the system keeps such things. On
    /// Windows that is `%APPDATA%`; elsewhere `$XDG_DATA_HOME`, or
    /// `~/.local/share`. `None` when the system names no such place.
    pub fn user(app: &str) -> Option<PStore> {
        let named = |name: &str| std::env::var_os(name).map(PathBuf::from).filter(|path| !path.as_os_str().is_empty());
        let home = || named("HOME").map(|home| home.join(".local").join("share"));
        let base = named("APPDATA").or_else(|| named("XDG_DATA_HOME")).or_else(home)?;
        Some(PStore { root: base.join(app) })
    }

    /// Where the folder is, as a person would read it.
    pub fn path(&self) -> String {
        self.root.to_string_lossy().to_string()
    }

    /// The text of the file of that name, or `None` when there is none.
    pub fn read(&self, name: &str) -> Option<String> {
        std::fs::read_to_string(self.root.join(name)).ok()
    }

    /// Write the file of that name, making the folder if it is not there.
    /// `false` when it could not be written.
    pub fn write(&self, name: &str, text: &str) -> bool {
        std::fs::create_dir_all(&self.root).and_then(|_| std::fs::write(self.root.join(name), text)).is_ok()
    }
}
