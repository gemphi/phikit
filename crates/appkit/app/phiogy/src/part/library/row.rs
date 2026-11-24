//! What each list of the library's forms holds now, as rows of words:
//! the tree's sections, where the library is, its items, what the place
//! is, and of the item opened what it is, its parts, its attributes and
//! the comments on it. `data/listing.tsv` says which column is what.

use std::path::Path;
use std::rc::Rc;

use appkit::pui::state::{PMemory, PState};
use appkit::service::PTable;
use appkit::theme::PShade;

use crate::data;
use crate::library::{Attribute, Item, Library, folders, item};

/// The lists, and what they are read from.
pub struct List {
    pub library: Rc<Library>,
    pub text: Rc<PTable>,
    pub shade: Rc<PShade>,
    pub attribute: Rc<Attribute>,
    pub ui: Rc<PMemory>,
}

/// A row of words.
type Row = Vec<String>;

/// Rows of a table of `data/filter.tsv`: those of that group, without it.
fn group(name: &str) -> Vec<Row> {
    data::row(data::FILTER).into_iter().filter(|cell| cell.first() == Some(&name)).map(|cell| cell[1..].iter().map(|c| c.to_string()).collect()).collect()
}

/// Every row of a table, as words.
fn every(table: &str) -> Vec<Row> {
    data::row(table).into_iter().map(|cell| cell.into_iter().map(str::to_string).collect()).collect()
}

/// A line of the tree: where it goes, its words, its icon, its depth.
fn line(at: String, words: &str, icon: &str, depth: usize) -> Row {
    vec![at, words.to_string(), icon.to_string(), depth.to_string()]
}

/// The folder at a path as the tree names it: its own name, or a drive's.
fn named(path: &Path) -> String {
    path.file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_else(|| path.to_string_lossy().trim_end_matches('\\').to_string())
}

impl List {
    /// Where the library is: what the window keeps, or Phiogy's shapes.
    pub fn at(&self) -> String {
        let kept = self.ui.get("ui/folder");
        let shape = || self.library.place.iter().find(|place| place.section == "own").and_then(|place| place.folder.clone());
        match kept.is_empty() {
            true => shape().map(|folder| format!("dir:{}", folder.to_string_lossy())).unwrap_or_default(),
            false => kept,
        }
    }

    /// The item opened.
    fn open(&self) -> Option<Item> {
        self.library.get(&self.ui.get("ui/item"))
    }

    pub fn rows(&self, name: &str) -> Vec<Row> {
        let (at, text) = (self.at(), &self.text);
        let own = &self.library.own;
        match name {
            "volume" | "system" | "own" => {
                let place = self.library.place.iter().filter(|place| place.section == name);
                place.filter_map(|place| Some(line(format!("dir:{}", place.folder.as_ref()?.to_string_lossy()), match text.get(&place.word) {
                    "" => &place.word,
                    words => words,
                }, &place.icon, 0))).collect()
            }
            "folder" => self.tree(&at),
            "collection" => {
                let open = match at.split_once(':') {
                    Some(("collection", id)) => id.to_string(),
                    Some(("club", id)) => self.library.shelf.club(id).map(|club| club.collection.clone()).unwrap_or_default(),
                    _ => String::new(),
                };
                let mut out = Vec::new();
                for one in &self.library.shelf.collection {
                    out.push(line(format!("collection:{}", one.id), &one.name, own.get("collection"), 0));
                    let clubs = one.club.iter().filter(|_| one.id == open).filter_map(|id| self.library.shelf.club(id));
                    out.extend(clubs.map(|club| line(format!("club:{}", club.id), &club.name, own.get("club"), 1)));
                }
                out
            }
            "club" => self.library.shelf.club.iter().map(|club| line(format!("club:{}", club.id), &club.name, own.get("club"), 0)).collect(),
            "set" => self.library.sets().iter().map(|set| line(format!("set:{}", set.name), &set.name, own.get("set"), 0)).collect(),
            "path" => self.path(&at),
            "kind" | "sort" | "tool" | "back" => group(name),
            "paint" => every(data::PAINT),
            "finish" => every(data::FINISH),
            "item" => self.items(&at),
            "stat" => self.stat(&at),
            "about" => self.open().map(|one| self.about(&one)).unwrap_or_default(),
            "attribute" => self.open().map(|one| one.attribute.iter().map(|(name, value)| vec![name.clone(), name.clone(), value.clone()]).collect()).unwrap_or_default(),
            "part" => self.open().filter(|one| one.kind == "model").map(|one| self.library.model(&one.path).part.iter().enumerate().map(|(index, part)| vec![index.to_string(), part.name.clone()]).collect()).unwrap_or_default(),
            "comment" => self.open().map(|one| self.attribute.note(&one.key).into_iter().enumerate().map(|(index, said)| vec![index.to_string(), said]).collect()).unwrap_or_default(),
            "say" => self.open().map(|_| vec![vec!["say".to_string(), "item.say".to_string()]]).unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    /// The folders down to where the library is, and those in it, each
    /// set in by its depth.
    fn tree(&self, at: &str) -> Vec<Row> {
        let Some(("dir", path)) = at.split_once(':') else { return Vec::new() };
        let icon = self.library.own.get("folder");
        let path = Path::new(path);
        let over: Vec<&Path> = path.ancestors().collect::<Vec<_>>().into_iter().rev().collect();
        let mut out: Vec<Row> = over.iter().enumerate().map(|(depth, folder)| line(format!("dir:{}", folder.to_string_lossy()), &named(folder), icon, depth)).collect();
        out.extend(folders(path).iter().map(|folder| line(format!("dir:{}", folder.to_string_lossy()), &named(folder), icon, over.len())));
        out
    }

    /// Where the library is, as steps back up: each where it goes and
    /// its words.
    fn path(&self, at: &str) -> Vec<Row> {
        match at.split_once(':') {
            Some(("dir", path)) => {
                let over: Vec<&Path> = Path::new(path).ancestors().collect::<Vec<_>>().into_iter().rev().collect();
                over.iter().map(|folder| vec![format!("dir:{}", folder.to_string_lossy()), named(folder), String::new()]).collect()
            }
            Some(("club", id)) => {
                let club = self.library.shelf.club(id);
                let collection = club.and_then(|club| self.library.shelf.collection(&club.collection));
                let first = collection.map(|one| vec![format!("collection:{}", one.id), one.name.clone(), String::new()]);
                first.into_iter().chain(club.map(|club| vec![at.to_string(), club.name.clone(), String::new()])).collect()
            }
            Some(("collection", id)) => self.library.shelf.collection(id).map(|one| vec![vec![at.to_string(), one.name.clone(), String::new()]]).unwrap_or_default(),
            Some(("set", name)) => vec![vec![at.to_string(), name.to_string(), String::new()]],
            _ => Vec::new(),
        }
    }

    /// The items of where the library is, of the kind and in the order
    /// chosen: each its key, name, icon, picture, what it is, its size,
    /// and its face.
    fn items(&self, at: &str) -> Vec<Row> {
        let unit = self.text.get("size.unit");
        let list = self.library.item(at, &self.ui.get("ui/kind"), &self.ui.get("ui/sort"));
        list.iter().map(|one| {
            let face = one.face.as_ref().map(|(sheet, column, index)| format!("{sheet} {column} {index}")).unwrap_or_default();
            let size = match one.size {
                0 => String::new(),
                bytes => item::size(bytes, unit),
            };
            vec![one.key.clone(), one.name.clone(), one.icon.clone(), self.library.tile(one, &self.attribute, self.shade.as_ref()), self.text.get(&format!("kind.{}", one.kind)).to_string(), size, face]
        }).collect()
    }

    /// What the place is: a folder's counts and size, a club's attributes
    /// and how many it made, a collection's topic, a set's size; and what
    /// was last said.
    fn stat(&self, at: &str) -> Vec<Row> {
        let row = |key: &str, value: String| vec![key.to_string(), self.text.get(&format!("stat.{key}")).to_string(), value];
        let said = self.ui.get("ui/said");
        let first = (!said.is_empty()).then(|| vec!["said".to_string(), said, String::new()]);
        let rest: Vec<Row> = match at.split_once(':') {
            Some(("club", id)) => self.library.shelf.club(id).map(|club| {
                let own = club.attribute.iter().map(|(name, value)| vec![name.clone(), name.clone(), value.clone()]);
                own.chain([row("supply", club.supply.to_string()), row("file", club.item.len().to_string())]).collect()
            }).unwrap_or_default(),
            Some(("collection", id)) => self.library.shelf.collection(id).map(|one| vec![row("topic", one.topic.clone()), row("file", one.club.len().to_string())]).unwrap_or_default(),
            _ => {
                let all = self.library.item(at, "all", "name");
                let count = |kind: &str| all.iter().filter(|one| one.kind == kind).count().to_string();
                let bytes = all.iter().map(|one| one.size).sum();
                vec![row("file", all.len().to_string()), row("folder", count("folder")), row("model", count("model")), row("picture", count("picture")), row("video", count("video")), row("size", item::size(bytes, self.text.get("size.unit")))]
            }
        };
        first.into_iter().chain(rest).collect()
    }

    /// What an item is: its name, kind, where it is, its size, and of a
    /// model its parts and triangles.
    fn about(&self, one: &Item) -> Vec<Row> {
        let row = |key: &str, value: String| vec![key.to_string(), self.text.get(&format!("about.{key}")).to_string(), value];
        let mut out = vec![row("file", one.name.clone()), row("kind", self.text.get(&format!("kind.{}", one.kind)).to_string())];
        if !one.path.as_os_str().is_empty() {
            out.push(row("where", one.path.parent().map(|up| up.to_string_lossy().to_string()).unwrap_or_default()));
            out.push(row("size", item::size(one.size, self.text.get("size.unit"))));
        }
        if one.kind == "model" {
            let model = self.library.model(&one.path);
            out.push(row("part", model.part.len().to_string()));
            out.push(row("triangle", model.count.to_string()));
        }
        out
    }
}
