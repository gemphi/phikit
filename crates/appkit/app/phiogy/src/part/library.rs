//! The asset library in the window, made only of the kit's blocks: the
//! form `tree` at the left (the drives, the person's folders, Phiogy's,
//! the folders down to where they are, Phiegg's collections and clubs,
//! the sets), `library` at the right (Up and Import, where it is, what to
//! show and in what order, the items as a grid or a list, what the place
//! is), and `item`, the inspector beside an item opened (what it is, a
//! colour for each part and a finish, its attributes, comments). Every
//! block is `data/form.tsv`'s; this file only says what a press does,
//! and `row.rs` what each list holds.

mod row;

use std::path::Path;
use std::rc::Rc;

use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};
use appkit::pui::form::{PForm, PListed};
use appkit::pui::state::{PMemory, PState};
use appkit::pui::window::{PDeed, PWish};

use super::Desk;
use crate::data;
use crate::library::{Attribute, Library, item};

/// The word the window is told, with the chosen file's path, when the
/// person chose a file to import.
pub const IMPORT: &str = "import";

/// What only the window keeps of the library: the place shown, the kind
/// and the order, the item opened and its part chosen, what is folded.
/// A press that opens something, or asks for a file, is a deed of the
/// window's.
struct Ui {
    memory: Rc<PMemory>,
    wish: PWish,
    library: Rc<Library>,
    /// The word a new set's name begins with.
    set: String,
}

impl Ui {
    /// The place over one: a folder's folder, a club's collection.
    fn up(&self, at: &str) -> String {
        match at.split_once(':') {
            Some(("dir", path)) => Path::new(path).parent().map(|up| format!("dir:{}", up.to_string_lossy())).unwrap_or_else(|| at.to_string()),
            Some(("club", id)) => self.library.shelf.club(id).map(|club| format!("collection:{}", club.collection)).unwrap_or_else(|| at.to_string()),
            _ => at.to_string(),
        }
    }

    /// Show another place.
    fn go(&self, at: &str) {
        self.library.again();
        self.memory.set("ui/folder", at);
    }
}

impl PState for Ui {
    fn get(&self, path: &str) -> String {
        self.memory.get(path)
    }

    fn set(&self, path: &str, value: &str) {
        let wish = |deed: PDeed| self.wish.borrow_mut().push(deed);
        let library = || PDeed::Show("library".to_string());
        match (path, value) {
            ("ui/do", IMPORT) => wish(PDeed::Choose(IMPORT.to_string())),
            ("ui/do", "up") => self.go(&self.up(&self.memory.get("ui/folder"))),
            ("ui/do", "set") => match self.library.get(&self.memory.get("ui/item")).filter(|item| item.kind == "model") {
                Some(item) => {
                    let name = self.library.make(&item.key, &self.set);
                    self.go(&format!("set:{name}"));
                    wish(library());
                }
                None => wish(library()),
            },
            ("ui/do", _) => wish(library()),
            ("ui/folder", at) => self.go(at),
            // A folder or a club opens in the library; any other item
            // beside the inspector.
            ("ui/item", key) => match self.library.get(key) {
                Some(item) if matches!(item.kind.as_str(), "folder" | "club") => self.go(&item.key),
                Some(_) => {
                    self.memory.set(path, key);
                    self.memory.set("ui/part", "0");
                    wish(PDeed::Show("model".to_string()));
                }
                None => {}
            },
            _ => self.memory.set(path, value),
        }
    }
}

/// The library's forms' source: its lists from its tables and from what
/// the places and the item opened hold now; its values in the window's
/// memory (`ui`) and with what the person chose (`look`, `note`).
pub fn source(desk: &Desk) -> Rc<PListed> {
    let attribute = Rc::new(Attribute::new(desk.choice.clone()));
    let ui = Rc::new(Ui { memory: desk.shelf.clone(), wish: desk.wish.clone(), library: desk.library.clone(), set: desk.text.get("set.word").to_string() });
    let list = row::List { library: desk.library.clone(), text: desk.text.clone(), shade: desk.shade.clone(), attribute: attribute.clone(), ui: desk.shelf.clone() };
    let rows = move |name: &str| list.rows(name);
    let state: Vec<(String, Rc<dyn PState>)> = vec![("ui".to_string(), ui), ("look".to_string(), attribute.clone()), ("note".to_string(), attribute)];
    Rc::new(PListed::new(data::LISTING, Rc::new(rows), desk.text.clone(), state))
}

/// A form of the library: the kit's, told when a file to import was
/// chosen.
pub struct Browse {
    form: PForm,
    desk: Desk,
}

impl Browse {
    /// The form of that name (`tree`, `library`, `item`) of `data/form.tsv`.
    pub fn new(name: &str, desk: Desk) -> Browse {
        let form = PForm::new(name, data::FORM, source(&desk), desk.text.clone(), desk.shade.clone());
        Browse { form, desk }
    }

    /// Bring a file into the person's own place (the first the table
    /// makes, their Phiegg space), and where it is no USD, turn it into
    /// USD beside it with the command `data/convert.tsv` gives. What
    /// happened is said under the place's counts.
    pub fn import(&self, from: &Path) {
        let (library, text) = (&self.desk.library, &self.desk.text);
        let Some(folder) = library.place.iter().find(|place| place.make).and_then(|place| place.folder.clone()) else { return };
        let name = from.file_name().map(|name| name.to_string_lossy().to_string()).unwrap_or_default();
        let to = item::free(&folder, &name);
        let copied = std::fs::create_dir_all(&folder).and_then(|_| std::fs::copy(from, &to));
        let said = match (copied, library.kind(&to).map(|(kind, _)| kind)) {
            (Err(error), _) => error.to_string(),
            (Ok(_), Some(kind)) if kind == "source" => match item::turn(&to) {
                true => text.fill("import.turned", "name", &name),
                false => text.fill("import.wait", "name", &name),
            },
            (Ok(_), _) => text.fill("import.done", "name", &name),
        };
        library.again();
        self.desk.shelf.set("ui/folder", &format!("dir:{}", folder.to_string_lossy()));
        self.desk.shelf.set("ui/said", &said);
    }
}

impl PDraw for Browse {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        self.form.draw(paint, origin);
    }
}

impl PView for Browse {
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
