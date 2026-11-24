//! An item of the library: a file of a folder, a folder in it, an item
//! of one of Phiegg's clubs, or a variant of a set. A file is what it is
//! by the end of its name (`data/kind.tsv`), how large and how new; one
//! of an end the table does not have, and a hidden one, is not listed.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// An item: its key (`dir:` and a path, `club:` and the club and the
/// item, `set:` and the set and the variant), where its file is (nothing
/// for one with none), its name, its kind and icon, its size in bytes,
/// when it last changed, its picture (a key) or face (a cell of a
/// sheet), what it is (its attributes, in words), and for a set's
/// variant the colour each part of its model takes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub key: String,
    pub path: PathBuf,
    pub name: String,
    pub kind: String,
    pub icon: String,
    pub size: u64,
    pub when: SystemTime,
    pub picture: String,
    pub face: Option<(String, u32, u32)>,
    pub attribute: Vec<(String, String)>,
    pub variant: Vec<String>,
}

impl Item {
    /// An item with no file: its key, name, kind and icon.
    pub fn named(key: String, name: String, kind: &str, icon: &str) -> Item {
        Item { key, path: PathBuf::new(), name, kind: kind.to_string(), icon: icon.to_string(), size: 0, when: SystemTime::UNIX_EPOCH, picture: String::new(), face: None, attribute: Vec::new(), variant: Vec::new() }
    }

    /// The file or folder at a path, as the library lists it; nothing for
    /// a file of no kind the table has.
    pub fn file(path: &Path, table: &[Vec<String>], folder_icon: &str) -> Option<Item> {
        let meta = std::fs::metadata(path).ok()?;
        let name = path.file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_else(|| path.to_string_lossy().to_string());
        let (kind, icon) = match meta.is_dir() {
            true => ("folder", folder_icon),
            false => kind(table, path)?,
        };
        let picture = match kind {
            "picture" => format!("file:///{}", path.to_string_lossy().replace('\\', "/")),
            _ => String::new(),
        };
        let when = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        Some(Item { key: format!("dir:{}", path.to_string_lossy()), path: path.to_path_buf(), name, kind: kind.to_string(), icon: icon.to_string(), size: meta.len(), when, picture, ..Item::named(String::new(), String::new(), "", "") })
    }
}

/// What a file is by its end: its kind and icon, from rows of
/// `end kind icon`.
pub fn kind<'a>(table: &'a [Vec<String>], path: &Path) -> Option<(&'a str, &'a str)> {
    let end = path.extension()?.to_string_lossy().to_lowercase();
    table.iter().find(|row| row.first() == Some(&end)).map(|row| (row.get(1).map_or("", String::as_str), row.get(2).map_or("", String::as_str)))
}

/// The folders and then the files of a folder that are of a kind the
/// table has and, unless the kind is `all`, of that kind (folders are
/// listed with `all` only); in that order (`name`, `new`, `large`), no
/// more than `most`.
pub fn list(folder: &Path, table: &[Vec<String>], want: &str, order: &str, most: usize, folder_icon: &str) -> Vec<Item> {
    let Ok(read) = std::fs::read_dir(folder) else { return Vec::new() };
    let mut out: Vec<Item> = Vec::new();
    for entry in read.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        let Some(item) = Item::file(&entry.path(), table, folder_icon).filter(|_| !name.starts_with(['.', '$'])) else { continue };
        let wanted = match item.kind.as_str() {
            "folder" => want == "all",
            kind => want == "all" || want == kind,
        };
        if wanted {
            out.push(item);
        }
    }
    match order {
        "new" => out.sort_by_key(|item| std::cmp::Reverse(item.when)),
        "large" => out.sort_by_key(|item| std::cmp::Reverse(item.size)),
        _ => out.sort_by_key(|item| item.name.to_lowercase()),
    }
    // Folders first, as a browser lists them.
    out.sort_by_key(|item| item.kind != "folder");
    out.truncate(most);
    out
}

/// A size as a person reads it, in the units given, smallest first
/// (`size.unit` of the text: bytes, then each a thousand and more).
pub fn size(bytes: u64, unit: &str) -> String {
    let unit: Vec<&str> = unit.split_whitespace().collect();
    let unit = match unit.is_empty() {
        true => vec![""],
        false => unit,
    };
    let (mut value, mut at) = (bytes as f64, 0);
    while value >= 1024.0 && at < unit.len() - 1 {
        value /= 1024.0;
        at += 1;
    }
    match at {
        0 => format!("{bytes} {}", unit[0]),
        _ => format!("{value:.1} {}", unit[at]),
    }
}

/// A path in a folder for a file of that name that is not there yet:
/// the name itself, or with ` 2`, ` 3`... before its end. Nothing of the
/// person's is written over.
pub fn free(folder: &Path, name: &str) -> PathBuf {
    let path = Path::new(name);
    let (stem, end) = (path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(), path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default());
    (1..).map(|n| match n {
        1 => folder.join(name),
        n => folder.join(format!("{stem} {n}{end}")),
    }).find(|path| !path.exists()).unwrap_or_else(|| folder.join(name))
}

/// Begin turning a source into USD beside it, with the command
/// `data/convert.tsv` gives (`{in}` the file, `{out}` the USD), on a
/// thread of its own, away from the window. `true` where the command is
/// on the system's path to begin.
pub fn turn(source: &Path) -> bool {
    let row = appkit::service::PTable::read(crate::data::CONVERT);
    let out = source.with_extension(row.get("end"));
    let fill = |word: &str| word.replace("{in}", &source.to_string_lossy()).replace("{out}", &out.to_string_lossy());
    let argument: Vec<String> = row.get("argument").split_whitespace().map(fill).collect();
    let command = row.get("command").to_string();
    let found = std::env::var_os("PATH").into_iter().flat_map(|path| std::env::split_paths(&path).collect::<Vec<_>>()).any(|folder| [command.clone(), format!("{command}.exe")].iter().any(|name| folder.join(name).is_file()));
    // It runs on its own thread, so waiting for it blocks nothing.
    #[allow(clippy::disallowed_methods)]
    let run = move || _ = std::process::Command::new(&command).args(&argument).status();
    if found {
        std::thread::spawn(run);
    }
    found
}
