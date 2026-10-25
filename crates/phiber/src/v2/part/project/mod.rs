//! The Project Browser — Reality Composer Pro's browser over the open
//! scene's assets, its three columns as three regions of the host:
//!
//! - the navigator (`BProjectNav`, the Tools region): Collections ▸ All
//!   Files, then the Project — the scene, and a folder per kind with how
//!   many it holds;
//! - the toolbar over the cards (`BProjectBar`, the ToolHeader): import, a
//!   new scene, the card size, cards or a list, the order — the filter is
//!   the header's search field (`part::search`);
//! - the cards (`BProjectItem`, the window): every asset of the folder as
//!   a card — a material its swatch, an image its picture, the rest their
//!   kind's icon — or as a list;
//! - the inspector (`BProjectInfo`, the UI region): the card picked, its
//!   values, what it uses and what uses it; else "Select a file to view
//!   its properties."
//!
//! The assets are a phiecs world (`db`), built once per scene; the
//! Library's wardrobe is a world of its own (`library`).

pub mod check;
pub mod db;
pub mod library;
pub mod thumb;

use crate::v2::state::{BKey as _, BState, FileKey, OpKey, ProjectKey};
use crate::v2::state::val::V_TRUE;
use crate::v2::view::prim::{BAction, BCard, BColumn, BControl, BThumb};
use crate::v2::view::{BView, BViewItem};
use db::{BAssetKind, BProjectDb};
use phiecs::prelude::Word;

/// The card sizes the zoom steps through, px.
const SIZES: [f32; 5] = [64.0, 80.0, 96.0, 128.0, 160.0];
const DEFAULT_SIZE: f32 = 96.0;

fn folder(s: &BState) -> Option<BAssetKind> {
    s.get(ProjectKey::Folder).and_then(BAssetKind::by_slug)
}

fn size(s: &BState) -> f32 {
    s.get(ProjectKey::Size).and_then(|v| v.parse().ok()).unwrap_or(DEFAULT_SIZE)
}

/// The cards the browser shows: the folder's assets (every kind's under All
/// Files), filtered by name and ordered.
fn shown(db: &BProjectDb, s: &BState) -> Vec<Word> {
    let kinds: Vec<BAssetKind> = match folder(s) {
        Some(k) => vec![k],
        None => BAssetKind::ALL.to_vec(),
    };
    let filter = s.get(ProjectKey::Search).unwrap_or("").to_lowercase();
    let mut words: Vec<Word> = kinds.iter().flat_map(|&k| db.assets(k)).filter(|&w| filter.is_empty() || db.asset(w).is_some_and(|a| a.name.to_lowercase().contains(&filter))).collect();
    // Coined order until the sort is pressed; then by name, either way.
    match s.get(ProjectKey::Descending) {
        Some(V_TRUE) => words.sort_by(|a, b| db.asset(*b).map(|x| &x.name).cmp(&db.asset(*a).map(|x| &x.name))),
        Some(_) => words.sort_by(|a, b| db.asset(*a).map(|x| &x.name).cmp(&db.asset(*b).map(|x| &x.name))),
        None => {}
    }
    words
}

fn pick(db: &BProjectDb, w: Word) -> BAction {
    BAction::Set(ProjectKey::Selected.path(), db.world.spelling(w).to_string())
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BProjectItem;

impl BViewItem for BProjectItem {
    fn build(&self, s: &BState) -> BView {
        if let Some(shelf) = library::folder(s) {
            return library::cards(s, shelf, size(s));
        }
        let Some(db) = db::current() else {
            return BView::from(BCard::column(BColumn::Body, vec![BView::label("Open a scene to browse its project.")]));
        };
        let selected = s.get(ProjectKey::Selected).unwrap_or("");
        let words = shown(&db, s);
        if words.is_empty() {
            return BView::from(BCard::column(BColumn::Body, vec![BView::label("Nothing here.")]));
        }
        let body = if s.get(ProjectKey::List) == Some(V_TRUE) {
            BView::column(
                words
                    .iter()
                    .filter_map(|&w| {
                        let a = db.asset(w)?;
                        Some(BView::Button(BControl::new().with_icon(a.kind.icon()).with_label(format!("{}   {}", a.name, a.detail)).with_active(db.world.spelling(w) == selected).with_action(pick(&db, w))))
                    })
                    .collect(),
            )
        } else {
            let edge = size(s);
            BView::grid(
                words
                    .iter()
                    .filter_map(|&w| {
                        let a = db.asset(w)?;
                        Some(BView::thumbnail(BThumb {
                            preview_path: thumb::key(&db, w),
                            icon: a.kind.icon().into(),
                            label: a.name.clone(),
                            subtitle: a.detail.clone(),
                            size: edge,
                            selected: db.world.spelling(w) == selected,
                            action: Some(pick(&db, w)),
                        }))
                    })
                    .collect(),
            )
        };
        BView::from(BCard::column(BColumn::Body, vec![body]))
    }
}

/// The navigator: Collections ▸ All Files; Project ▸ each kind the scene
/// has, with its count; Library ▸ Footwear and its sets.
#[derive(Clone, Copy, Debug)]
pub struct BProjectNav;

impl BViewItem for BProjectNav {
    fn build(&self, s: &BState) -> BView {
        let at = folder(s);
        let row = |icon: &str, label: String, here: bool, slug: &str| {
            BView::Button(BControl::new().with_icon(icon).with_label(label).with_active(here).with_action(BAction::Set(ProjectKey::Folder.path(), slug.to_string())))
        };
        // Each group a card on its own ground, as the file browser's source
        // list and the Inspector's panels are.
        let mut project = Vec::new();
        if let Some(db) = db::current() {
            for kind in BAssetKind::ALL {
                let n = db.assets(kind).len();
                if n > 0 {
                    project.push(row(kind.icon(), format!("{}  {n}", kind.label()), at == Some(kind), kind.slug()));
                }
            }
        }
        if project.is_empty() {
            project.push(BView::label("Open a scene to list its files."));
        }
        let cards = vec![
            BCard::panel("Collections").body(vec![row("grid", "All Files".into(), at.is_none() && library::folder(s).is_none(), "all")]).into(),
            BCard::panel("Project").body(project).into(),
            BCard::panel("Library").body(library::nav(s)).into(),
        ];
        BView::from(BCard::column(BColumn::Sidebar, cards))
    }
}

/// The toolbar over the cards: import and a new scene at the left; the
/// card size, cards or list, and the order at the right.
#[derive(Clone, Copy, Debug)]
pub struct BProjectBar;

impl BViewItem for BProjectBar {
    fn build(&self, s: &BState) -> BView {
        let icon = |name: &str, tip: &str, action: BAction| BView::Button(BControl::new().with_icon(name).with_tooltip(tip).with_action(action));
        let edge = size(s);
        let step = |up: bool| {
            let at = SIZES.iter().position(|&x| x >= edge).unwrap_or(2);
            let next = if up { (at + 1).min(SIZES.len() - 1) } else { at.saturating_sub(1) };
            BAction::Set(ProjectKey::Size.path(), SIZES[next].to_string())
        };
        let list = s.get(ProjectKey::List) == Some(V_TRUE);
        let descending = s.get(ProjectKey::Descending) == Some(V_TRUE);
        BView::row(vec![
            icon("import", "Import", BAction::Set(FileKey::Purpose.path(), "import".into())),
            icon("file_new", "New Scene", BAction::Set(OpKey::NewFile.path(), "general".into())),
            BView::spacer(),
            icon("zoom_out", "Smaller", step(false)),
            icon("zoom_in", "Larger", step(true)),
            BView::Button(BControl::new().with_icon(if list { "grid" } else { "list" }).with_tooltip(if list { "Show as Icons" } else { "Show as List" }).with_action(BAction::Set(ProjectKey::List.path(), (!list).to_string()))),
            BView::Button(BControl::new().with_icon("sort").with_tooltip(if descending { "Sort A to Z" } else { "Sort Z to A" }).with_active(s.get(ProjectKey::Descending).is_some()).with_action(BAction::Set(ProjectKey::Descending.path(), (!descending).to_string()))),
        ])
    }
}

/// The inspector: the picked asset's values and its links, or the prompt.
#[derive(Clone, Copy, Debug)]
pub struct BProjectInfo;

impl BViewItem for BProjectInfo {
    fn build(&self, s: &BState) -> BView {
        if let Some(view) = library::info(s) {
            return view;
        }
        let found = db::current().and_then(|db| {
            let spelling = s.get(ProjectKey::Selected)?;
            let w = db.world.words().iter().copied().find(|&w| db.world.spelling(w) == spelling)?;
            Some((db, w))
        });
        let Some((db, w)) = found else {
            return BView::from(BCard::column(BColumn::Sidebar, vec![BView::label("Select a file to view its properties.")]));
        };
        let Some(a) = db.asset(w) else { return BView::empty() };
        let mut rows = vec![BView::pair("Name", a.name.clone()), BView::pair("Kind", a.kind.label().trim_end_matches('s').to_string())];
        rows.extend(a.info.iter().map(|(k, v)| BView::pair(k.clone(), v.clone())));
        let names = |ws: Vec<Word>| ws.iter().filter_map(|&u| db.asset(u)).map(|u| u.name.clone()).collect::<Vec<_>>();
        let mut cards = vec![BCard::panel("Info").icon(a.kind.icon()).body(rows).into()];
        let uses = names(db.uses_of(w));
        if !uses.is_empty() {
            cards.push(BCard::panel(format!("Uses ({})", uses.len())).body(uses.into_iter().take(24).map(BView::label).collect()).into());
        }
        let users = names(db.used_by(w));
        if !users.is_empty() {
            cards.push(BCard::panel(format!("Used by ({})", users.len())).body(users.into_iter().take(24).map(BView::label).collect()).into());
        }
        BView::from(BCard::column(BColumn::Sidebar, cards))
    }
}
