//! A file of a place, as the library lists it: what it is by the end of
//! its name (`data/kind.tsv`), how large and how new. A file of an end
//! the table does not have, a folder and a hidden file are not listed.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// A file of a place: its key (`place:name`), where it is, its name, its
/// kind and icon, its size in bytes and when it last changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub key: String,
    pub path: PathBuf,
    pub name: String,
    pub kind: String,
    pub icon: String,
    pub size: u64,
    pub when: SystemTime,
}

/// What a file is by its end: its kind and icon, from rows of
/// `end kind icon`.
pub fn kind<'a>(table: &'a [Vec<String>], path: &Path) -> Option<(&'a str, &'a str)> {
    let end = path.extension()?.to_string_lossy().to_lowercase();
    table.iter().find(|row| row.first() == Some(&end)).map(|row| (row.get(1).map_or("", String::as_str), row.get(2).map_or("", String::as_str)))
}

/// The files of a folder that are of a kind the table has and, unless
/// the kind is `all`, of that kind; in that order (`name`, `new`,
/// `large`), no more than `most`.
pub fn list(place: &str, folder: &Path, table: &[Vec<String>], want: &str, order: &str, most: usize) -> Vec<Item> {
    let Ok(read) = std::fs::read_dir(folder) else { return Vec::new() };
    let mut out: Vec<Item> = Vec::new();
    for entry in read.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let Ok(meta) = entry.metadata() else { continue };
        let Some((kind, icon)) = kind(table, &path) else { continue };
        let wanted = want == "all" || want == kind;
        if !meta.is_file() || name.starts_with('.') || !wanted {
            continue;
        }
        let when = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        out.push(Item { key: format!("{place}:{name}"), path, name, kind: kind.to_string(), icon: icon.to_string(), size: meta.len(), when });
    }
    match order {
        "new" => out.sort_by_key(|item| std::cmp::Reverse(item.when)),
        "large" => out.sort_by_key(|item| std::cmp::Reverse(item.size)),
        _ => out.sort_by_key(|item| item.name.to_lowercase()),
    }
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
