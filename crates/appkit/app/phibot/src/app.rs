//! The botanist as appkit sees it: its spaces, hosts, parts and modals
//! read from `data/`, and the view of each part.

use std::rc::Rc;

use appkit::pui::PApp;
use appkit::pui::core::PView;
use appkit::pui::form::PForm;
use appkit::pui::window::{PActivity, PHost, PModal, PPart, PSet, PSpace, PWish};
use appkit::service::PTable;
use appkit::theme::{PLook, PShade, PTheme};

use crate::data;
use crate::part::{Choose, Desk, Ground, Robot, Score};

pub struct Phibot {
    own: PTable,
    space: PSet<PSpace>,
    host: PSet<PHost>,
    part: PSet<PPart>,
    modal: PSet<PModal>,
    desk: Desk,
}

impl Phibot {
    /// The theme, with what the command line changes after it.
    pub fn look(change: &str) -> PLook {
        PLook::read(&format!("{}\n{change}", data::THEME))
    }

    /// The looks a person can turn to, each the theme with its rows over it.
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

    pub fn new(look: Rc<PLook>) -> Phibot {
        let (mut space, mut host) = (PSet::read(data::SPACE), PSet::read(data::HOST));
        PSpace::arrange(&mut space, data::LAYOUT);
        PHost::declare(&mut host, data::REGION);
        let desk = Desk::new(Rc::new(Phibot::shade(&look)), Rc::new(PTable::read(data::TEXT)));
        Phibot { own: PTable::read(data::APP), space, host, part: PSet::read(data::PART), modal: PSet::read(data::MODAL), desk }
    }

    pub fn desk(&self) -> &Desk {
        &self.desk
    }
}

impl PApp for Phibot {
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

    fn theme(&self) -> Rc<dyn PTheme> {
        self.desk.shade.clone()
    }

    fn activity(&self) -> PActivity {
        PActivity::read(data::ACTIVITY).hold(data::MENU)
    }

    fn user(&self) -> PTable {
        let look: String = self.desk.shade.list().iter().map(|name| format!("look.{name}\t{}\n", self.desk.text.get(&format!("look.{name}")))).collect();
        PTable::read(&format!("{}\n{look}", data::USER))
    }

    fn shade(&self) -> Option<Rc<PShade>> {
        Some(self.desk.shade.clone())
    }

    fn content(&self, part: &str) -> Option<Box<dyn PView>> {
        let desk = self.desk.clone();
        match part {
            // The customiser is the kit's form, as data/form.tsv lays it.
            "pick" => Some(Box::new(PForm::new("choose", data::FORM, Rc::new(Choose(desk.clone())), desk.text.clone(), desk.shade.clone()))),
            "robot" => Some(Box::new(Robot::new(desk))),
            "ground" => Some(Box::new(Ground::new(desk))),
            "score" => Some(Box::new(Score::new(desk))),
            _ => None,
        }
    }

    fn front(&self) -> &str {
        self.own.get("front")
    }

    fn side(&self) -> &str {
        self.own.get("side")
    }

    fn focus(&self) -> &str {
        self.own.get("focus")
    }

    fn tab(&self) -> bool {
        self.own.get("tab") != "no"
    }

    fn wish(&self) -> Option<PWish> {
        Some(self.desk.wish.clone())
    }
}
