//! The asset library, as Reality Composer Pro's Project Browser and
//! Blender's file browser are: a tree of where things are (the drives,
//! the person's own folders, Phiogy's shapes and their Phiegg space, and
//! Phiegg's collections and clubs, and the sets they made), and what the
//! place chosen holds: its folders and files, a club's items, a set's
//! variants. A model is read from USD and drawn by the kit's raster into
//! pictures the tiles and the 3D view show. Everything it names is a
//! table of `data/`.

mod attribute;
pub mod item;
mod model;
mod place;
mod set;
mod shelf;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use appkit::pui::core::orbit::{ROOM, around, shot};
use appkit::pui::core::{PFilm, PPiece, PSun};
use appkit::service::PTable;
use appkit::theme::PTheme;

pub use attribute::{Attribute, paint};
pub use item::Item;
pub use model::{Model, Part};
pub use place::{Place, folders};
pub use set::Set;
pub use shelf::{Club, Collection, Shelf};

use crate::choice::Choice;
use crate::data;

/// What is listed: where, of which kind, in which order.
type Asked = (String, String, String);

/// The library.
pub struct Library {
    pub place: Vec<Place>,
    pub shelf: Shelf,
    kind: Vec<Vec<String>>,
    /// Its 3D pictures' settings (`data/model.tsv`), and its own words and
    /// numbers (`data/shelf.tsv`).
    pub set: PTable,
    pub own: PTable,
    /// Where its pictures are shown: the window's film.
    pub film: PFilm,
    /// What the person chose: their sets.
    choice: Rc<Choice>,
    model: RefCell<HashMap<PathBuf, Rc<Model>>>,
    listed: RefCell<HashMap<Asked, Rc<Vec<Item>>>>,
    shot: RefCell<HashSet<String>>,
}

impl Library {
    /// The drives and places of this system, Phiegg's shelf beside the
    /// site's file, and the person's sets.
    pub fn own(site: &Path, choice: Rc<Choice>) -> Library {
        let own = PTable::read(data::SHELF);
        let shelf = Shelf::read(&site.parent().map(|folder| folder.join(own.get("shelf"))).unwrap_or_default(), own.get("club"));
        Library { shelf, ..Library::at(Place::own(), choice) }
    }

    /// A library of those places and no shelf.
    pub fn at(place: Vec<Place>, choice: Rc<Choice>) -> Library {
        let kind = data::row(data::KIND).into_iter().map(|cell| cell.into_iter().map(str::to_string).collect()).collect();
        let (set, own) = (PTable::read(data::MODEL), PTable::read(data::SHELF));
        Library { place, shelf: Shelf::default(), kind, set, own, film: PFilm::default(), choice, model: RefCell::default(), listed: RefCell::default(), shot: RefCell::default() }
    }

    /// Where the library opens: Phiogy's own shapes.
    pub fn home(&self) -> String {
        let folder = self.place.iter().find(|place| place.section == "own").and_then(|place| place.folder.clone());
        folder.map(|folder| format!("dir:{}", folder.to_string_lossy())).unwrap_or_default()
    }

    /// The person's sets, by name.
    pub fn sets(&self) -> Vec<Set> {
        self.choice.starting("set/").into_iter().filter_map(|(name, value)| Set::kept(&name, &value)).collect()
    }

    /// Make a set of a model: its name the next free `Set n`.
    pub fn make(&self, base: &str, word: &str) -> String {
        let name = (1..).map(|n| format!("{word} {n}")).find(|name| self.sets().iter().all(|set| set.name != *name)).unwrap_or_default();
        let set = Set { name: name.clone(), count: self.own.get("count").parse().unwrap_or(1), base: base.to_string() };
        self.choice.put(&format!("set/{name}"), &set.value());
        name
    }

    /// What a place of the tree holds (`dir:`, `collection:`, `club:`,
    /// `set:` and its key), of a kind (`all`), in an order; read once,
    /// until the library is told to look again.
    pub fn item(&self, at: &str, kind: &str, order: &str) -> Rc<Vec<Item>> {
        let asked = (at.to_string(), kind.to_string(), order.to_string());
        if let Some(known) = self.listed.borrow().get(&asked) {
            return known.clone();
        }
        let most = self.set.get("most").parse().unwrap_or(usize::MAX);
        let (icon, wanted) = (self.own.get("folder"), |item: &Item| kind == "all" || item.kind == kind);
        let list: Vec<Item> = match at.split_once(':') {
            Some(("dir", path)) => item::list(Path::new(path), &self.kind, kind, order, most, icon),
            Some(("collection", id)) => self.shelf.collection(id).map(|one| one.club.iter().filter_map(|club| self.shelf.club(club)).map(|club| self.club_item(club)).collect()).unwrap_or_default(),
            Some(("club", id)) => self.shelf.club(id).map(|club| club.item.iter().filter(|item| wanted(item)).take(most).cloned().collect()).unwrap_or_default(),
            Some(("set", name)) => self.variants(name).into_iter().take(most).collect(),
            _ => Vec::new(),
        };
        let list = Rc::new(list);
        self.listed.borrow_mut().insert(asked, list.clone());
        list
    }

    /// A club as an item of its collection: its world's picture.
    fn club_item(&self, club: &Club) -> Item {
        Item { picture: club.world.clone(), attribute: club.attribute.clone(), ..Item::named(format!("club:{}", club.id), club.name.clone(), "club", self.own.get("club")) }
    }

    /// A set's variants: its model's file in the colours drawn for each.
    fn variants(&self, name: &str) -> Vec<Item> {
        let Some(set) = self.sets().into_iter().find(|set| set.name == name) else { return Vec::new() };
        let Some(base) = self.get(&set.base) else { return Vec::new() };
        let parts = self.model(&base.path).part.len();
        (0..set.count).map(|index| {
            let variant = set.variant(index, parts);
            let attribute = self.model(&base.path).part.iter().zip(&variant).map(|(part, colour)| (part.name.clone(), colour.clone())).collect();
            Item { key: format!("set:{name}/{index}"), name: format!("{name} #{}", index + 1), variant, attribute, ..base.clone() }
        }).collect()
    }

    /// Look at the folders again: a file came, or one was turned.
    pub fn again(&self) {
        self.listed.borrow_mut().clear();
    }

    /// The item of a key, wherever it is.
    pub fn get(&self, key: &str) -> Option<Item> {
        match key.split_once(':')? {
            ("dir", path) => Item::file(Path::new(path), &self.kind, self.own.get("folder")),
            ("club", rest) => match rest.split_once('/') {
                Some((club, _)) => self.shelf.club(club)?.item.iter().find(|item| item.key == key).cloned(),
                None => self.shelf.club(rest).map(|club| self.club_item(club)),
            },
            ("set", rest) => {
                let (name, _) = rest.split_once('/')?;
                self.variants(name).into_iter().find(|item| item.key == key)
            }
            _ => None,
        }
    }

    /// What a file is by the end of its name: its kind and icon.
    pub fn kind(&self, path: &Path) -> Option<(String, String)> {
        item::kind(&self.kind, path).map(|(kind, icon)| (kind.to_string(), icon.to_string()))
    }

    /// The model of a USD file, read once.
    pub fn model(&self, path: &Path) -> Rc<Model> {
        if let Some(known) = self.model.borrow().get(path) {
            return known.clone();
        }
        let model = Rc::new(Model::read(path, three(&self.set, "colour")));
        self.model.borrow_mut().insert(path.to_path_buf(), model.clone());
        model
    }

    /// A model's pieces, each part in the colour the person chose for it,
    /// or the one its variant drew, or its own.
    pub fn dressed(&self, item: &Item, attribute: &Attribute) -> Vec<PPiece> {
        self.model(&item.path).dressed(|part| attribute.colour(&item.key, part).or_else(|| item.variant.get(part).and_then(|key| paint(key))))
    }

    /// The light a model is shown in, as its finish has it.
    pub fn sun(&self, item: &Item, attribute: &Attribute) -> PSun {
        attribute.sun(&item.key, &self.set)
    }

    /// The picture a tile shows: a model drawn once for each way it is
    /// set, in the film; any other item's own picture.
    pub fn tile(&self, item: &Item, attribute: &Attribute, theme: &dyn PTheme) -> String {
        if item.kind != "model" {
            return item.picture.clone();
        }
        let model = self.model(&item.path);
        let name = format!("library/{}/{}", item.key, attribute.mark(&item.key, model.part.len()));
        if self.shot.borrow().contains(&name) || model.part.is_empty() {
            return name;
        }
        let side = self.set.get("tile").parse().unwrap_or(192);
        let piece = self.dressed(item, attribute);
        let camera = around(&piece, 0.6, self.set.get("camera.fov").parse().unwrap_or(30.0), theme.size(ROOM).unwrap_or(1.0), 1.0);
        let (wide, high, pixel) = shot(&piece, &camera, &self.sun(item, attribute), (side, side), theme);
        self.film.show(&name, wide, high, pixel);
        self.shot.borrow_mut().insert(name.clone());
        name
    }
}

/// Three numbers of a row of a table.
pub fn three(set: &PTable, key: &str) -> [f32; 3] {
    let v: Vec<f32> = set.get(key).split_whitespace().filter_map(|n| n.parse().ok()).collect();
    [0, 1, 2].map(|i| v.get(i).copied().unwrap_or_default())
}
