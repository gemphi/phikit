//! The asset library in the window, made only of the kit's blocks: the
//! form `library` of `data/form.tsv` (Import at its top right, the
//! places as tabs, the kinds and the order as filters, what the place
//! holds counted, its files as a grid of tiles), and the form `custom`
//! beside an opened model (its parts as tabs over the chosen part's
//! colour, its finish, what it is). This file only answers what each
//! list holds and what each path keeps; the kit lays them out.

use std::path::Path;
use std::rc::Rc;

use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};
use appkit::pui::form::{PForm, PListed};
use appkit::pui::state::{PMemory, PState};
use appkit::pui::window::{PDeed, PWish};

use super::Desk;
use crate::data;
use crate::library::{Dress, Library, item};

/// The word the window is told, with the chosen file's path, when the
/// person chose a file to import.
pub const IMPORT: &str = "import";

/// What only the window keeps of the library: the place, the kind and the
/// order shown, the file opened and its part chosen. A press that opens
/// something, or asks for a file, is a deed of the window's.
struct Ui {
    memory: Rc<PMemory>,
    wish: PWish,
    library: Rc<Library>,
}

impl PState for Ui {
    fn get(&self, path: &str) -> String {
        self.memory.get(path)
    }

    fn set(&self, path: &str, value: &str) {
        let wish = |deed: PDeed| self.wish.borrow_mut().push(deed);
        match (path, value) {
            ("ui/do", IMPORT) => wish(PDeed::Choose(IMPORT.to_string())),
            ("ui/do", _) => wish(PDeed::Show("library".to_string())),
            ("ui/item", key) => match self.library.get(key) {
                // A model opens beside its dress; any other file in the
                // system's own program for it.
                Some(item) if item.kind == "model" => {
                    self.memory.set(path, key);
                    self.memory.set("ui/part", "0");
                    wish(PDeed::Show("model".to_string()));
                }
                Some(item) => wish(PDeed::Launch(item.path.to_string_lossy().to_string())),
                None => {}
            },
            ("ui/place", _) => {
                self.library.again();
                self.memory.set(path, value);
            }
            _ => self.memory.set(path, value),
        }
    }
}

/// Rows of a table of `data/filter.tsv`: those of that group, without it.
fn group(name: &str) -> Vec<Vec<String>> {
    data::row(data::FILTER).into_iter().filter(|cell| cell.first() == Some(&name)).map(|cell| cell[1..].iter().map(|c| c.to_string()).collect()).collect()
}

/// Every row of a table, as text.
fn every(table: &str) -> Vec<Vec<String>> {
    data::row(table).into_iter().map(|cell| cell.into_iter().map(str::to_string).collect()).collect()
}

/// The library's forms' source: its lists from its tables and from what
/// the places and the opened model hold now; its values in the window's
/// memory (`ui`) and with what the person chose (`look`).
pub fn source(desk: &Desk) -> Rc<PListed> {
    let (library, text, shade) = (desk.library.clone(), desk.text.clone(), desk.shade.clone());
    let dress = Rc::new(Dress::new(desk.choice.clone()));
    let ui = Rc::new(Ui { memory: desk.shelf.clone(), wish: desk.wish.clone(), library: library.clone() });
    let (now, worn) = (ui.clone(), dress.clone());
    let unit = text.get("size.unit").to_string();
    let rows = move |name: &str| -> Vec<Vec<String>> {
        let list = || library.item(&now.get("ui/place"), &now.get("ui/kind"), &now.get("ui/sort"));
        let open = || library.get(&now.get("ui/item"));
        match name {
            "place" => library.place.iter().map(|place| vec![place.key.clone(), place.word.clone(), place.icon.clone()]).collect(),
            "kind" | "sort" | "tool" | "back" => group(name),
            "paint" => every(data::PAINT),
            "finish" => every(data::FINISH),
            "item" => list().iter().map(|one| {
                let picture = match one.kind.as_str() {
                    "model" => library.tile(one, &worn, shade.as_ref()),
                    "picture" => format!("file:///{}", one.path.to_string_lossy().replace('\\', "/")),
                    _ => String::new(),
                };
                vec![one.key.clone(), one.name.clone(), one.icon.clone(), picture, text.get(&format!("kind.{}", one.kind)).to_string(), item::size(one.size, &unit)]
            }).collect(),
            "stat" => {
                let all = library.item(&now.get("ui/place"), "all", "name");
                let count = |kind: &str| all.iter().filter(|one| one.kind == kind).count().to_string();
                let bytes = all.iter().map(|one| one.size).sum();
                let said = now.get("ui/said");
                let first = (!said.is_empty()).then(|| vec!["said".to_string(), said, String::new()]);
                let row = |key: &str, value: String| vec![key.to_string(), text.get(&format!("stat.{key}")).to_string(), value];
                first.into_iter().chain([row("file", all.len().to_string()), row("model", count("model")), row("source", count("source")), row("picture", count("picture")), row("video", count("video")), row("size", item::size(bytes, &unit))]).collect()
            }
            "part" => open().map(|one| library.model(&one.path).part.iter().enumerate().map(|(index, part)| vec![index.to_string(), part.name.clone()]).collect()).unwrap_or_default(),
            "about" => open().map(|one| {
                let model = library.model(&one.path);
                let row = |key: &str, value: String| vec![key.to_string(), text.get(&format!("about.{key}")).to_string(), value];
                vec![row("file", one.name.clone()), row("part", model.part.len().to_string()), row("triangle", model.count.to_string()), row("size", item::size(one.size, &unit))]
            }).unwrap_or_default(),
            _ => Vec::new(),
        }
    };
    let state: Vec<(String, Rc<dyn PState>)> = vec![("ui".to_string(), ui), ("look".to_string(), dress)];
    Rc::new(PListed::new(data::LISTING, Rc::new(rows), desk.text.clone(), state))
}

/// A form of the library: the kit's, told when a file to import was
/// chosen.
pub struct Shelf {
    form: PForm,
    desk: Desk,
}

impl Shelf {
    /// The form of that name (`library`, `custom`) of `data/form.tsv`.
    pub fn new(name: &str, desk: Desk) -> Shelf {
        let form = PForm::new(name, data::FORM, source(&desk), desk.text.clone(), desk.shade.clone());
        Shelf { form, desk }
    }

    /// Bring a file into the person's own place (the first the table
    /// makes, their Phiegg space), and where it is no USD, turn it into
    /// USD beside it with the command `data/convert.tsv` gives. What
    /// happened is said under the place's counts.
    pub fn import(&self, from: &Path) {
        let library = &self.desk.library;
        let text = &self.desk.text;
        let Some(place) = library.place.iter().find(|place| place.make) else { return };
        let Some(folder) = place.folder.clone() else { return };
        let name = from.file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_default();
        let to = crate::library::item::free(&folder, &name);
        let copied = std::fs::create_dir_all(&folder).and_then(|_| std::fs::copy(from, &to));
        let said = match (copied, library.kind(&to).map(|(kind, _)| kind)) {
            (Err(error), _) => error.to_string(),
            (Ok(_), Some(kind)) if kind == "source" => match crate::library::item::turn(&to) {
                true => text.fill("import.turned", "name", &name),
                false => text.fill("import.wait", "name", &name),
            },
            (Ok(_), _) => text.fill("import.done", "name", &name),
        };
        library.again();
        self.desk.shelf.set("ui/place", &place.key);
        self.desk.shelf.set("ui/said", &said);
    }
}

impl PDraw for Shelf {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        self.form.draw(paint, origin);
    }
}

impl PView for Shelf {
    fn frame(&self) -> PRect {
        self.form.frame()
    }

    fn set_frame(&mut self, frame: PRect) {
        self.form.set_frame(frame);
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        self.form.respond(event)
    }

    fn tick(&mut self) -> bool {
        self.form.tick()
    }

    fn beat(&self) -> Option<u64> {
        self.form.beat()
    }

    fn hand(&self) -> Vec<PRect> {
        self.form.hand()
    }

    /// `import` and a path: the file the person chose to import.
    fn tell(&mut self, word: &str) -> bool {
        match word.split_once(' ') {
            Some((IMPORT, path)) => {
                self.import(Path::new(path));
                true
            }
            _ => false,
        }
    }
}
