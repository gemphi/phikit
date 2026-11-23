//! Phiogy as appkit sees it: a name, four sets, a theme, the rows of the
//! activity bar with their menus and a user, all read from `data/`, and
//! the view of each part. The bar, its menu and the profile are appkit's
//! own.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use appkit::pui::PApp;
use appkit::pui::core::PView;
use appkit::pui::window::part::PICTURE;
use appkit::pui::window::{PActivity, PFact, PHost, PLevel, PModal, PPart, PRail, PSet, PSpace, PWish};
use appkit::service::{PStore, PTable};
use appkit::theme::{PLook, PShade, PTheme};

use crate::board::Board;
use crate::choice::Choice;
use crate::comp::Limit;
use crate::data;
use crate::fact;
use crate::kept::Kept;
use crate::part::{Account, Desk, Friend, Guide, Page, ROOM, Room, Shopping, Status, Title, Venue, Viewer};
use crate::peer::{Maker, Stand};
use crate::site::Site;

/// The app.
pub struct Phiogy {
    /// Its name, the host in front on a mobile window, the host in the
    /// bar's menu: `data/app.tsv`.
    own: PTable,
    space: PSet<PSpace>,
    host: PSet<PHost>,
    part: PSet<PPart>,
    modal: PSet<PModal>,
    desk: Desk,
}

impl Phiogy {
    /// Phiogy's look: `data/theme.tsv`, and after it any rows that change
    /// it, such as another size for the window.
    pub fn look(change: &str) -> PLook {
        PLook::read(&format!("{}\n{change}", data::THEME))
    }

    /// The looks Phiogy can be shown in, the first of them that look:
    /// `data/look.tsv` names each, and after its name a row that changes
    /// the theme for it, as `data/theme.tsv` writes one.
    pub fn shade(look: &PLook) -> PShade {
        let mut list: Vec<(String, String)> = Vec::new();
        for line in data::LOOK.lines().filter(|line| !line.trim().is_empty() && !line.starts_with('#')) {
            let (name, row) = line.split_once('\t').unwrap_or((line, ""));
            match list.iter_mut().find(|(known, _)| known == name.trim()) {
                Some((_, rows)) => *rows += &format!("{row}\n"),
                None => list.push((name.trim().to_string(), format!("{row}\n"))),
            }
        }
        PShade::new(list.into_iter().map(|(name, rows)| (name, look.with(&rows))).collect())
    }

    /// Where the web Phiogy's own file is: `data/app.tsv` says, from
    /// this crate's folder.
    pub fn source() -> String {
        format!("{}/{}", env!("CARGO_MANIFEST_DIR"), PTable::read(data::APP).get("site"))
    }

    /// A board with the limits of `data/limit.tsv` and nobody on it.
    pub fn board() -> Arc<Mutex<Board>> {
        Arc::new(Mutex::new(Board::new(Limit::own())))
    }

    /// Phiogy in that look, on the web Phiogy's own files, with no line
    /// and nothing kept.
    pub fn new(look: Rc<PLook>) -> Phiogy {
        let why = PTable::read(data::TEXT).get("status.off").to_string();
        Phiogy::holding(look, Site::open(&Phiogy::source()), Phiogy::board(), Err(why), None)
    }

    /// Phiogy in that look, on that site and that board, the line where
    /// `line` says or not there for the reason it gives. What the person
    /// chose before, the look and the room, is chosen again, and what
    /// they choose is kept in their own folder for Phiogy; with `keep`
    /// false nothing is.
    pub fn open(look: Rc<PLook>, site: Site, board: Arc<Mutex<Board>>, line: Result<String, String>, keep: bool) -> Phiogy {
        let store = PStore::user(PTable::read(data::APP).get("folder")).filter(|_| keep);
        Phiogy::holding(look, site, board, line, store)
    }

    /// Phiogy in that look, on that site and board, keeping what is
    /// chosen in that store, if there is one. It opens on the room the
    /// person left open, or the one the site begins in, or its first.
    pub fn holding(look: Rc<PLook>, site: Site, board: Arc<Mutex<Board>>, line: Result<String, String>, store: Option<PStore>) -> Phiogy {
        let own = PTable::read(data::APP);
        let (mut space, mut host) = (PSet::read(data::SPACE), PSet::read(data::HOST));
        PSpace::arrange(&mut space, data::LAYOUT);
        PHost::declare(&mut host, data::REGION);
        let shade = Rc::new(Phiogy::shade(&look));
        let choice = store.map(|store| Choice::kept(store, own.get("setting"))).unwrap_or_default();
        shade.turn(&choice.get("look"));
        let first = site.venue.iter().flat_map(|venue| &venue.plate).map(|plate| plate.id.clone()).next();
        let wanted = [choice.get(ROOM), site.start.clone()].into_iter().find(|id| site.plate(id).is_some() || id.starts_with(Page::own().id(crate::part::page::DIRECT)));
        let open = Rc::new(RefCell::new(wanted.or(first).unwrap_or_default()));
        let desk = Desk { site: Rc::new(site), board, text: Rc::new(PTable::read(data::TEXT)), icon: Rc::new(PTable::read(data::ICON)), look, shade, choice: Rc::new(choice), wish: PWish::default(), open, line: Rc::new(line), home: own.get("space").to_string(), view: Rc::default(), kept: Rc::new(Kept::none()), now: Rc::default(), page: Rc::new(Page::own()), stock: Rc::default(), call: Rc::default(), engine: Rc::new(|| -> Maker { Box::new(|| Box::new(Stand::default())) }), refused: Rc::default(), away: false, ring: Rc::default() };
        Phiogy { own, space, host, part: PSet::read(data::PART), modal: PSet::read(data::MODAL), desk }
    }

    /// The same app, its calls carried by what that makes: the real
    /// library where it is built in. As it is, a call this window joins
    /// is carried by a stand-in, which sends and hears nothing.
    pub fn carried(self, engine: Rc<dyn Fn() -> Maker>) -> Phiogy {
        Phiogy { desk: Desk { engine, ..self.desk }, ..self }
    }

    /// The same app on a line it does not serve: somebody else's, which
    /// it reaches as a page does.
    pub fn away(self) -> Phiogy {
        Phiogy { desk: Desk { away: true, ..self.desk }, ..self }
    }

    /// The same app, the person's own documents kept where those are:
    /// in their folder, and on Phiora when it answers. As it is, they
    /// last as long as the window.
    pub fn keeping(self, kept: Kept) -> Phiogy {
        let app = Phiogy { desk: Desk { kept: Rc::new(kept), ..self.desk }, ..self };
        // What the person wears is put on as the window opens.
        app.desk.dressed();
        app
    }

    /// What every part works on.
    pub fn desk(&self) -> &Desk {
        &self.desk
    }
}

impl PApp for Phiogy {
    fn name(&self) -> &str {
        self.own.get("name")
    }

    fn space(&self) -> &PSet<PSpace> {
        &self.space
    }

    fn host(&self) -> &PSet<PHost> {
        &self.host
    }

    fn part(&self) -> &PSet<PPart> {
        &self.part
    }

    fn modal(&self) -> &PSet<PModal> {
        &self.modal
    }

    /// The looks, of which one is shown: a person can turn to another.
    fn theme(&self) -> Rc<dyn PTheme> {
        self.desk.shade.clone()
    }

    fn activity(&self) -> PActivity {
        PActivity::read(data::ACTIVITY).hold(data::MENU)
    }

    /// The hosts the person is in, each a row of the bar with its mark.
    fn rail(&self) -> Option<PRail> {
        let desk = self.desk.clone();
        Some(Rc::new(move || desk.place()))
    }

    /// The rows of `data/user.tsv`, and what each look is called.
    fn user(&self) -> PTable {
        let look: String = self.desk.shade.list().iter().map(|name| format!("look.{name}\t{}\n", self.desk.text.get(&format!("look.{name}")))).collect();
        // Their picture, where the site has one of them.
        let face: String = self.desk.cell(&self.desk.site.me).map(|(sheet, column, index)| format!("{PICTURE}\t{sheet} {column} {index}\n")).unwrap_or_default();
        PTable::read(&format!("{}\n{look}{face}", data::USER))
    }

    fn fact(&self) -> Option<PFact> {
        let desk = self.desk.clone();
        Some(Rc::new(move || fact::fact(&desk)))
    }

    fn shade(&self) -> Option<Rc<PShade>> {
        Some(self.desk.shade.clone())
    }

    fn content(&self, part: &str) -> Option<Box<dyn PView>> {
        let desk = self.desk.clone();
        // A view is for a part or a modal the sets name.
        self.part.find(part).map(PPart::slug).or_else(|| self.modal.find(part).map(PModal::slug))?;
        match part {
            "title" => Some(Box::new(Title::new(desk))),
            "room" => Some(Box::new(Room::new(desk))),
            "status" => Some(Box::new(Status::new(desk))),
            "venue" => Some(Box::new(Venue::new(desk))),
            "guide" => Some(Box::new(Guide::new(desk))),
            "shop" => Some(Box::new(Shopping::new(desk))),
            "account" => Some(Box::new(Account::new(desk))),
            "friend" => Some(Box::new(Friend::new(desk, false))),
            "people" => Some(Box::new(Friend::new(desk, true))),
            "view" => Some(Box::new(Viewer::new(desk))),
            _ => None,
        }
    }

    fn front(&self) -> &str {
        self.own.get("front")
    }

    fn tab(&self) -> bool {
        self.own.get("tab") != "no"
    }

    fn side(&self) -> &str {
        self.own.get("side")
    }

    fn wish(&self) -> Option<PWish> {
        Some(self.desk.wish.clone())
    }
}
