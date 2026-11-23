//! Phigen as appkit sees it: a name, four sets, a theme, the rows of the
//! activity bar with their menus and a user, all read from `data/`, and the view of each
//! part and of each modal that is its own. The bar, the tabs and the
//! profile are appkit's own.

use std::cell::RefCell;
use std::rc::Rc;

use appkit::pui::PApp;
use appkit::pui::core::{PEvent, PView};
use appkit::pui::window::{PActivity, PFact, PHost, PLevel, PModal, PPart, PSet, PSpace, PStage, PWish};
use appkit::service::{PStore, PTable};
use appkit::theme::{PLook, PShade, PTheme};

use crate::choice::Choice;
use crate::data;
use crate::fact;
use crate::history::History;
use crate::part::{Ask, Card, Desk, Find, Hello, Roster, Setting, Talk, Thread, Title};
use crate::model::Model;
use crate::reply::{Reply, Stand};

/// The app.
pub struct Phigen {
    /// Its name, the host in front on a mobile window, the part that takes
    /// the keys: `data/app.tsv`.
    own: PTable,
    space: PSet<PSpace>,
    host: PSet<PHost>,
    part: PSet<PPart>,
    modal: PSet<PModal>,
    desk: Desk,
}

impl Phigen {
    /// Phigen's look: `data/theme.tsv`, and after it any rows that change
    /// it, such as another size for the window.
    pub fn look(change: &str) -> PLook {
        PLook::read(&format!("{}\n{change}", data::THEME))
    }

    /// The looks Phigen can be shown in, the first of them that look:
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

    /// Phigen in that look, answered by the stand-in.
    pub fn new(look: Rc<PLook>) -> Phigen {
        let form = PTable::read(data::TEXT).get("reply").to_string();
        Phigen::answered(look, Box::new(Stand::new(&form)))
    }

    /// Phigen in that look, answered by the model `data/model.tsv` names,
    /// with the chats kept from before and kept from now on: in the file
    /// `data/app.tsv` names, in the person's own folder for Phigen. Where
    /// there is no such folder nothing is kept, and where no model can be
    /// called, in a browser, the stand-in answers. With `keep` false the
    /// chats and the choices from before are left out and none is kept.
    pub fn open(look: Rc<PLook>, keep: bool) -> Phigen {
        let (own, text) = (PTable::read(data::APP), PTable::read(data::TEXT));
        let model = Model::read(&PTable::read(data::MODEL), &text).filter(|_| !cfg!(target_family = "wasm"));
        let (reply, greeting): (Box<dyn Reply>, &str) = match model {
            Some(model) => (Box::new(model), text.get("greeting.model")),
            None => (Box::new(Stand::new(text.get("reply"))), text.get("greeting")),
        };
        Phigen::kept(look, reply, greeting, PStore::user(own.get("folder")).filter(|_| keep))
    }

    /// Phigen in that look, answered by whoever `reply` is, with the chats
    /// kept in that store and kept there from now on. What the person
    /// chose before, the look and the model, is chosen again, and what
    /// they choose is kept there too. With no store nothing is.
    pub fn kept(look: Rc<PLook>, reply: Box<dyn Reply>, greeting: &str, store: Option<PStore>) -> Phigen {
        let own = PTable::read(data::APP);
        let history = match &store {
            Some(store) => History::read(&store.read(own.get("keep")).unwrap_or_default(), reply, greeting).kept(store.clone(), own.get("keep")),
            None => History::new(reply, greeting),
        };
        let app = Phigen::holding(look, history);
        let choice = store.map(|store| Choice::kept(store, own.get("setting"))).unwrap_or_default();
        app.desk.shade.turn(&choice.get("look"));
        app.desk.history.borrow_mut().choose(&choice.get("model"));
        Phigen { desk: Desk { choice: Rc::new(choice), ..app.desk.clone() }, ..app }
    }

    /// Phigen in that look, answered by whoever `reply` is.
    pub fn answered(look: Rc<PLook>, reply: Box<dyn Reply>) -> Phigen {
        let greeting = PTable::read(data::TEXT).get("greeting").to_string();
        Phigen::holding(look, History::new(reply, &greeting))
    }

    /// Phigen in that look, with that history.
    pub fn holding(look: Rc<PLook>, history: History) -> Phigen {
        let text = Rc::new(PTable::read(data::TEXT));
        let history = Rc::new(RefCell::new(history));
        let (mut space, mut host) = (PSet::read(data::SPACE), PSet::read(data::HOST));
        PSpace::arrange(&mut space, data::LAYOUT);
        PHost::declare(&mut host, data::REGION);
        let own = PTable::read(data::APP);
        let shade = Rc::new(Phigen::shade(&look));
        let desk = Desk { history, text, look, shade, choice: Rc::default(), wish: PWish::default(), home: own.get("space").to_string(), empty: own.get("empty").to_string(), full: own.get("front").to_string() };
        Phigen { own, space, host, part: PSet::read(data::PART), modal: PSet::read(data::MODAL), desk }
    }

    /// What every part works on.
    pub fn desk(&self) -> &Desk {
        &self.desk
    }

    /// Whether nothing was said yet, in any chat: a person's first time.
    pub fn fresh(&self) -> bool {
        self.desk.history.borrow().list().iter().all(|chat| chat.title().is_none())
    }

    /// Type a line into a stage and press Enter, as a hand would.
    pub fn key(stage: &mut PStage, text: &str) {
        text.chars().chain(['\n']).for_each(|key| _ = stage.send(&PEvent::KeyDown(key)));
    }
}

impl PApp for Phigen {
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

    /// The rows of `data/user.tsv`, and what each look is called.
    fn user(&self) -> PTable {
        let look: String = self.desk.shade.list().iter().map(|name| format!("look.{name}\t{}\n", self.desk.text.get(&format!("look.{name}")))).collect();
        PTable::read(&format!("{}\n{look}", data::USER))
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
        let icon = self.part.find(part).map_or("", PPart::icon);
        match part {
            "thread" => Some(Box::new(Thread::new(desk))),
            "title" => Some(Box::new(Title::new(desk, self.own.get("pick")))),
            "talk" => Some(Box::new(Talk::new(desk))),
            "ask" => Some(Box::new(Ask::new(desk))),
            "find" => Some(Box::new(Find::new(desk, icon, data::FIND))),
            "hello" => Some(Box::new(Hello::new(desk))),
            "card" => Some(Box::new(Card::new(desk, data::CARD, false))),
            "welcome" => Some(Box::new(Card::new(desk, data::CARD, true))),
            // The modal the pick opens, where choosing closes it; and the
            // settings, which hold the same roster.
            "model" => Some(Box::new(Roster::new(desk, true))),
            "setting" => Some(Box::new(Setting::new(desk))),
            _ => None,
        }
    }

    fn front(&self) -> &str {
        self.own.get("front")
    }

    fn focus(&self) -> &str {
        self.own.get("focus")
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
