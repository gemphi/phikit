//! The Project Browser's Library: the wardrobe (`phiplay::kit::footwear`)
//! as a phiecs world beside the scene's — each collection and each set a
//! word, each design a word `in` its set (and each set `in` its
//! collection) with its fields bound (`BDesign`), so the navigator counts a
//! folder by what is `in` it and the inspector reads a design's fields off
//! its word. It needs no open scene.
//!
//! Folders are spelt `wear` (every design), `wear:<collection>` and
//! `wear:<set>`; a design is picked as `wear:<design>`. Its pictures: the
//! card `wear:<design>`, its six views `wear-view:<design>:<n>`, its sketch
//! checks `wear-check:<design>:<n>` (`check`).

use super::check;
use crate::v2::state::{BKey as _, BState, OpKey, ProjectKey};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BCard, BColumn, BControl, BThumb};
use phiecs::prelude::*;
use phiplay::build::footwear::picture::BShot;
use phiplay::kit::footwear::{BCollection, BFootwearSet, BFootwearSpec, DESIGNS};
use std::cell::OnceCell;
use std::rc::Rc;

pub const PREFIX: &str = "wear";
const VIEW: &str = "wear-view:";
/// A card's picture edge, px: drawn once at the largest card size.
const EDGE: u32 = 160;

/// A design, as its word carries it.
#[derive(Clone, Debug)]
pub struct BDesign(pub &'static BFootwearSpec);
impl Aspect for BDesign {}

/// A folder of the Library.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BShelf {
    All,
    Collection(BCollection),
    Set(BFootwearSet),
}

pub struct BLibrary {
    pub world: World,
    pub within: Relation,
    shelves: Vec<(BShelf, Word)>,
}

impl BLibrary {
    fn build() -> BLibrary {
        let mut world = World::new();
        world.register::<BDesign>();
        let within = world.declare("in").expect("a relation");
        let mut shelves = vec![(BShelf::All, world.coin_bare(PREFIX))];
        for c in BCollection::ALL {
            let cw = world.coin_bare(&format!("{PREFIX}:{}", c.slug()));
            world.relate(cw, within, shelves[0].1);
            shelves.push((BShelf::Collection(c), cw));
            for s in c.sets() {
                let sw = world.coin_bare(&format!("{PREFIX}:{}", s.spec().slug));
                world.relate(sw, within, cw);
                shelves.push((BShelf::Set(s), sw));
            }
        }
        for d in DESIGNS.iter() {
            let word = world.coin_bare(&format!("{PREFIX}:{}", d.slug));
            world.bind(word, BDesign(d));
            if let Some(&(_, set)) = shelves.iter().find(|(s, _)| *s == BShelf::Set(d.set)) {
                world.relate(word, within, set);
            }
        }
        BLibrary { world, within, shelves }
    }

    fn word(&self, shelf: BShelf) -> Option<Word> {
        self.shelves.iter().find(|(s, _)| *s == shelf).map(|(_, w)| *w)
    }

    /// Whether `w` is in `shelf`, directly or through the shelves in it.
    fn holds(&self, w: Word, shelf: Word) -> bool {
        self.world.holds(w, self.within, shelf) || self.world.calls_for(w, self.within).into_iter().any(|(up, _)| self.holds(up, shelf))
    }

    /// The designs on `shelf`.
    pub fn designs(&self, shelf: BShelf) -> Vec<Word> {
        let Some(at) = self.word(shelf) else { return Vec::new() };
        self.world.words().iter().copied().filter(|&w| self.world.read::<BDesign>(w).is_some() && self.holds(w, at)).collect()
    }

    pub fn design(&self, word: Word) -> Option<&'static BFootwearSpec> {
        self.world.read::<BDesign>(word).map(|d| d.0)
    }

    pub fn find(&self, spelling: &str) -> Option<Word> {
        self.world.words().iter().copied().find(|&w| self.world.spelling(w) == spelling)
    }
}

thread_local! {
    static LIBRARY: OnceCell<Rc<BLibrary>> = const { OnceCell::new() };
}

pub fn library() -> Rc<BLibrary> {
    LIBRARY.with(|cell| cell.get_or_init(|| Rc::new(BLibrary::build())).clone())
}

/// The folder picked, when it is the Library's.
pub fn folder(s: &BState) -> Option<BShelf> {
    let at = s.get(ProjectKey::Folder)?;
    if at == PREFIX {
        return Some(BShelf::All);
    }
    let slug = at.strip_prefix(PREFIX)?.strip_prefix(':')?;
    BFootwearSet::by_slug(slug).map(BShelf::Set).or_else(|| BCollection::ALL.into_iter().find(|c| c.slug() == slug).map(BShelf::Collection))
}

fn slug(shelf: BShelf) -> String {
    match shelf {
        BShelf::All => PREFIX.into(),
        BShelf::Collection(c) => format!("{PREFIX}:{}", c.slug()),
        BShelf::Set(s) => format!("{PREFIX}:{}", s.spec().slug),
    }
}

/// The navigator's Library rows: Footwear, each collection, its sets.
pub fn nav(s: &BState) -> Vec<BView> {
    let lib = library();
    let at = folder(s);
    let row = |icon: &str, label: String, shelf: BShelf| {
        let n = lib.designs(shelf).len();
        BView::Button(BControl::new().with_icon(icon).with_label(format!("{label}  {n}")).with_active(at == Some(shelf)).with_action(BAction::Set(ProjectKey::Folder.path(), slug(shelf))))
    };
    let mut list = vec![row("shoe", "Footwear".into(), BShelf::All)];
    for c in BCollection::ALL {
        list.push(row("collection", format!("  {}", c.label()), BShelf::Collection(c)));
        for set in c.sets() {
            list.push(row("folder", format!("    {}", set.spec().label), BShelf::Set(set)));
        }
    }
    list
}

/// The cards of the folder picked: each design its picture.
pub fn cards(s: &BState, shelf: BShelf, edge: f32) -> BView {
    let lib = library();
    let selected = s.get(ProjectKey::Selected).unwrap_or("");
    let filter = s.get(ProjectKey::Search).unwrap_or("").to_lowercase();
    let cards = lib
        .designs(shelf)
        .into_iter()
        .filter_map(|w| {
            let d = lib.design(w)?;
            let spelling = lib.world.spelling(w).to_string();
            (filter.is_empty() || d.label.to_lowercase().contains(&filter)).then(|| {
                BView::thumbnail(BThumb {
                    preview_path: spelling.clone(),
                    icon: "shoe".into(),
                    label: d.label.into(),
                    subtitle: d.set.spec().label.into(),
                    size: edge,
                    selected: spelling == selected,
                    action: Some(BAction::Set(ProjectKey::Selected.path(), spelling)),
                })
            })
        })
        .collect();
    BView::from(BCard::column(BColumn::Body, vec![BView::grid(cards)]))
}

fn tile(key: String, label: &str) -> BView {
    BView::thumbnail(BThumb { preview_path: key, icon: "shoe".into(), label: label.into(), subtitle: String::new(), size: 96.0, selected: false, action: None })
}

/// The inspector for a picked design: its fields, its six views, its
/// sketch check and the box that confirms it, and the button that opens
/// its set on the stage.
pub fn info(s: &BState) -> Option<BView> {
    let lib = library();
    let d = lib.design(lib.find(s.get(ProjectKey::Selected)?)?)?;
    let mut rows = vec![BView::pair("Name", d.label)];
    rows.extend(d.fields().into_iter().map(|(k, v)| BView::pair(k, v)));
    let mut cards: Vec<BView> = vec![BCard::panel("Design").icon("shoe").body(rows).into()];
    let views = BShot::SHEET.iter().enumerate().map(|(i, shot)| tile(format!("{VIEW}{}:{i}", d.slug), shot.label())).collect();
    cards.push(BCard::panel("Views").icon("view").body(vec![BView::grid(views)]).into());
    let checks = check::checks(d);
    if !checks.is_empty() {
        let mut body = vec![BView::grid(checks.iter().enumerate().map(|(i, c)| tile(check::key(d, i), c.label)).collect())];
        body.push(BView::check_row(s, "", ProjectKey::Confirmed(d.slug).path(), false, "Confirmed against the sketch"));
        cards.push(BCard::panel("Sketch Check").icon("image").body(body).into());
    }
    let open = BView::Button(BControl::new().with_icon("play").with_label(format!("Open {}", d.set.spec().label)).with_action(BAction::Set(OpKey::Wear.path(), d.set.spec().slug.into())));
    cards.push(BCard::panel(d.set.spec().label).icon("folder").body(vec![BView::label(d.set.spec().caption), open]).into());
    Some(BView::from(BCard::column(BColumn::Sidebar, cards)))
}

/// The picture a `wear:` or `wear-view:` key names, as (width, height,
/// sRGB RGBA).
pub fn image(key: &str) -> Option<(u32, u32, Vec<u8>)> {
    let (slug, shot) = match key.strip_prefix(VIEW) {
        Some(rest) => {
            let (slug, i) = rest.rsplit_once(':')?;
            (slug, *BShot::SHEET.get(i.parse::<usize>().ok()?)?)
        }
        None => (key.strip_prefix(PREFIX)?.strip_prefix(':')?, BShot::SideOuter),
    };
    let design = phiplay::kit::footwear::by_slug(slug)?;
    let card = crate::v2::scene::wear::shot(design, shot, EDGE);
    Some((card.width, card.height, card.rgba))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_library_counts_each_folder_by_what_is_in_it() {
        let lib = library();
        assert_eq!(lib.designs(BShelf::All).len(), DESIGNS.len());
        for c in BCollection::ALL {
            assert_eq!(lib.designs(BShelf::Collection(c)).len(), 24, "{}", c.label());
        }
        for set in BFootwearSet::ALL {
            let words = lib.designs(BShelf::Set(set));
            assert_eq!(words.len(), 4);
            assert!(words.iter().all(|&w| lib.design(w).is_some_and(|d| d.set == set)));
        }
        let mut s = BState::new();
        s.set(ProjectKey::Folder, "wear:ballet");
        assert_eq!(folder(&s), Some(BShelf::Set(BFootwearSet::Ballet)));
        s.set(ProjectKey::Folder, "wear:cut-off");
        assert_eq!(folder(&s), Some(BShelf::Collection(BCollection::CutOff)));
        s.set(ProjectKey::Folder, "wear");
        assert_eq!(folder(&s), Some(BShelf::All));
        s.set(ProjectKey::Folder, "mesh");
        assert_eq!(folder(&s), None);
        s.set(ProjectKey::Selected, "wear:tall-classique");
        assert!(info(&s).is_some(), "a picked design has an inspector");
        assert!(image("wear-view:tall-classique:5").is_some(), "its sole view");
    }
}
