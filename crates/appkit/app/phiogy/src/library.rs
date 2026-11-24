//! The asset library: the places a creator keeps things (Phiogy's own
//! shapes, their Phiegg space, and the folders the system keeps for
//! them, so it sees what the person sees), what each holds by kind, and
//! the models among them, read from USD and drawn by the kit's raster
//! into pictures that the grid's tiles and the 3D view show. Everything
//! it names is a table of `data/`.

mod dress;
pub mod item;
mod model;
mod place;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use appkit::pui::core::orbit::{around, shot};
use appkit::pui::core::{PFilm, PPiece, PSun};
use appkit::service::PTable;
use appkit::theme::PTheme;

pub use dress::Dress;
pub use item::Item;
pub use model::{Model, Part};
pub use place::Place;

use crate::data;

/// What is listed, by the place, the kind and the order.
type Asked = (String, String, String);

/// The library.
pub struct Library {
    pub place: Vec<Place>,
    kind: Vec<Vec<String>>,
    /// Its 3D pictures' settings (`data/model.tsv`).
    pub set: PTable,
    /// Where its pictures are shown: the window's film.
    pub film: PFilm,
    model: RefCell<HashMap<PathBuf, Rc<Model>>>,
    listed: RefCell<HashMap<Asked, Rc<Vec<Item>>>>,
    shot: RefCell<HashSet<String>>,
}

impl Library {
    /// The places of `data/place.tsv`, where this system keeps them.
    pub fn own() -> Library {
        Library::at(Place::own())
    }

    /// A library of those places.
    pub fn at(place: Vec<Place>) -> Library {
        let kind = data::row(data::KIND).into_iter().map(|cell| cell.into_iter().map(str::to_string).collect()).collect();
        Library { place, kind, set: PTable::read(data::MODEL), film: PFilm::default(), model: RefCell::default(), listed: RefCell::default(), shot: RefCell::default() }
    }

    /// The place of that key.
    pub fn find(&self, key: &str) -> Option<&Place> {
        self.place.iter().find(|place| place.key == key)
    }

    /// The files of a place of a kind (`all` for every kind), in an
    /// order; read once, until the library is told to look again.
    pub fn item(&self, place: &str, kind: &str, order: &str) -> Rc<Vec<Item>> {
        let asked = (place.to_string(), kind.to_string(), order.to_string());
        if let Some(known) = self.listed.borrow().get(&asked) {
            return known.clone();
        }
        let most = self.set.get("most").parse().unwrap_or(usize::MAX);
        let folder = self.find(place).and_then(|place| place.folder.clone());
        let list = Rc::new(folder.map(|folder| item::list(place, &folder, &self.kind, kind, order, most)).unwrap_or_default());
        self.listed.borrow_mut().insert(asked, list.clone());
        list
    }

    /// Look at the folders again: a file came, or one was turned.
    pub fn again(&self) {
        self.listed.borrow_mut().clear();
    }

    /// The file of a key (`place:name`), wherever it is listed.
    pub fn get(&self, key: &str) -> Option<Item> {
        let (place, _) = key.split_once(':')?;
        self.item(place, "all", "name").iter().find(|item| item.key == key).cloned()
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
        let colour = three(&self.set, "colour");
        let model = Rc::new(Model::read(path, colour));
        self.model.borrow_mut().insert(path.to_path_buf(), model.clone());
        model
    }

    /// A model's pieces, dressed as the person dressed it.
    pub fn dressed(&self, item: &Item, dress: &Dress) -> Vec<PPiece> {
        self.model(&item.path).dressed(|part| dress.colour(&item.key, part))
    }

    /// The light a model is shown in, as it is dressed.
    pub fn sun(&self, item: &Item, dress: &Dress) -> PSun {
        dress.sun(&item.key, &self.set)
    }

    /// The name of a model's tile picture in the film, drawn once for each
    /// way it is dressed: the tile shows it as it shows any picture.
    pub fn tile(&self, item: &Item, dress: &Dress, theme: &dyn PTheme) -> String {
        let model = self.model(&item.path);
        let name = format!("library/{}/{}", item.key, dress.mark(&item.key, model.part.len()));
        if self.shot.borrow().contains(&name) || model.part.is_empty() {
            return name;
        }
        let side = self.set.get("tile").parse().unwrap_or(192);
        let piece = self.dressed(item, dress);
        let camera = around(&piece, 0.6, self.set.get("camera.fov").parse().unwrap_or(30.0), theme.size(appkit::pui::core::orbit::ROOM).unwrap_or(1.0), 1.0);
        let (wide, high, pixel) = shot(&piece, &camera, &self.sun(item, dress), (side, side), theme);
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
