//! The account: the one thing the user's button leads to, as the web's
//! drawer is (`AccountDrawer` in
//! `puikit/app/phi-kit/src/AccountDrawer.tsx`). Who the person is;
//! their wallet, each coin and the whole in dollars; the look; and the
//! account itself: their profile, and, where Phiora answers and lets
//! anyone be one of its members to try things, being one of them and
//! being nobody again.
//!
//! It has one heading, its own, with its way out at the end of it:
//! its panel is bare (`data/modal.tsv`), as the web's drawer has one
//! title and no button to make it larger.
//!
//! Where Phiora answers a guest can sign in with a handle and a
//! password, which is drawn as marks and sent to Phiora alone.
//! Signed in, the account lists the sessions signed in as the person,
//! signs the others out, and saves a copy of everything Phiora keeps
//! about them where they say. Making an account, an email, changing a
//! password and deleting are the web's account dialog, which this
//! window does not have: see the crate's TODO.

mod own;

use std::cell::{Cell, RefCell};

use appkit::pui::control::{PControl, PField};
use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};
use appkit::pui::window::PDeed;
use appkit::service::PTable;
use serde_json::Value;

use super::list::{Deed, Item, Lister, Seat, field};
use super::{Desk, Tool};
use crate::data;

/// The modal the person's profile is, which is appkit's.
pub const PROFILE: &str = "profile";
/// The row of `data/act.tsv` for its way out.
pub const SHUT: &str = "shut";
/// How the rows of a coin's name and of its worth in dollars begin in
/// `data/wallet.tsv`.
pub const LABEL: &str = "label.";
pub const RATE: &str = "rate.";

/// What a press on the account means.
#[derive(Debug, Clone, PartialEq)]
enum Act {
    Look(String),
    Profile,
    /// Be the member of that id; or, with none, nobody again.
    Sign(Option<String>),
    /// Put the account away.
    Shut,
    /// Give the keys to the field of that number.
    Field(usize),
    /// Sign in with what the two fields hold.
    Enter,
    /// Sign the other sessions out; keep a copy of one's data.
    Alone,
    Export,
}

/// The account.
pub struct Account {
    frame: PRect,
    desk: Desk,
    list: Lister<Act>,
    /// A handle and a password, which is never drawn; which of the two
    /// has the keys; and what Phiora said when it refused.
    handle: PField,
    secret: PField,
    aim: Cell<usize>,
    fault: RefCell<String>,
}

impl Account {
    pub fn new(desk: Desk) -> Account {
        let mut secret = PField::new(PRect::default());
        secret.secret = true;
        Account { frame: PRect::default(), desk, list: Lister::default(), handle: PField::new(PRect::default()), secret, aim: Cell::default(), fault: RefCell::default() }
    }

    /// Sign in with what the fields hold. The password is emptied
    /// whatever Phiora says.
    fn enter(&mut self) {
        let said = self.desk.kept.enter(&self.handle.value(), &self.secret.value());
        self.secret.store(String::new());
        *self.fault.borrow_mut() = match said {
            Ok(()) => String::new(),
            Err(why) => Some(why).filter(|why| !why.is_empty()).unwrap_or_else(|| self.desk.text.get("account.refused").to_string()),
        };
    }

    /// What the account holds, from its top down.
    fn item(&self) -> Vec<Item<Act>> {
        let (text, kept) = (&self.desk.text, &self.desk.kept);
        let word = |name: &str| text.get(name).to_string();
        let me = kept.me();
        let said = |name: &str| me[kept.word(name)].as_str().unwrap_or_default().to_string();
        let guest = me[kept.word("me.guest")].as_bool().unwrap_or(true);
        let (name, under) = match guest {
            true => (word("account.guest"), word("account.nobody")),
            false => (said("me.name"), Some(format!("@{}", said("me.handle"))).filter(|handle| handle.len() > 1).unwrap_or_default()),
        };
        let shut = Tool::read(data::ACT).of(SHUT, false, text).map(|(icon, said)| Deed::icon(icon, said, Act::Shut));
        let mut item = vec![Item::Top(word("account.title"), shut.into_iter().collect()), Item::Line(Seat::Icon(self.desk.icon(self.desk.page.face()).to_string()), name, under, None, vec![Deed::word(text.get("account.profile"), false, Act::Profile)])];
        item.push(Item::Head(word("account.wallet")));
        item.extend(self.wallet());
        let look = self.desk.shade.list().into_iter().map(|look| Deed::word(text.get(&format!("look.{look}")), look == self.desk.shade.name(), Act::Look(look.to_string())));
        item.extend([Item::Head(word("account.look")), Item::Tabs(look.collect()), Item::Head(word("account.account"))]);
        // Being somebody: only where Phiora answers.
        match (kept.at.is_some(), guest) {
            // Phiora gave the site, and this window keeps nothing: it
            // was not asked who the person is. Else it did not answer.
            (false, _) => item.push(Item::Note(word(match self.desk.site.asked {
                Some((_, true)) => "account.unkept",
                _ => "account.offline",
            }))),
            (true, false) => {
                item.push(Item::Line(Seat::None, word("account.signed"), String::new(), None, vec![Deed::word(text.get("account.signOut"), false, Act::Sign(None))]));
                item.extend(self.signed());
            }
            (true, true) => {
                // A handle and a password, for an account made on Phiora.
                let enter = Deed::word(text.get("account.signIn"), false, Act::Enter);
                item.extend([Item::Field(0, word("account.handle"), Act::Field(0)), Item::Field(1, word("account.secret"), Act::Field(1)), Item::Line(Seat::None, String::new(), String::new(), None, vec![enter])]);
                item.extend(Some(Item::Note(self.fault.borrow().clone())).filter(|_| !self.fault.borrow().is_empty()));
                let demo = Some(kept.demo()).filter(|_| me[kept.word("me.demo")].as_bool().unwrap_or(false)).unwrap_or_default();
                item.push(Item::Note(word(match demo.is_empty() {
                    true => "account.nodemo",
                    false => "account.demo",
                })));
                item.extend(demo.into_iter().map(|(id, name, handle)| Item::Line(Seat::Icon(self.desk.icon(self.desk.page.face()).to_string()), name, format!("@{handle}"), None, vec![Deed::word(text.get("account.signIn"), false, Act::Sign(Some(id)))])));
            }
        }
        item.push(Item::Note(word("account.phione")));
        item
    }
}

impl PDraw for Account {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        self.list.draw(paint, origin, self.frame, &self.desk, &self.item(), &[&self.handle, &self.secret]);
    }
}

impl PView for Account {
    /// Where a press on it does something.
    fn hand(&self) -> Vec<PRect> {
        self.list.hands()
    }

    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
        self.handle.set_frame(field(&self.desk, frame.size.width));
        self.secret.set_frame(field(&self.desk, frame.size.width));
    }

    fn tick(&mut self) -> bool {
        (self.handle.focused, self.secret.focused) = (self.aim.get() == 0, self.aim.get() == 1);
        false
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        self.tick();
        match *event {
            // Enter signs in; any other key is the field's that has them.
            PEvent::KeyDown(PEvent::ENTER) => {
                self.enter();
                true
            }
            PEvent::KeyDown(_) => match self.aim.get() {
                0 => self.handle.respond(event),
                _ => self.secret.respond(event),
            },
            PEvent::Scroll(_, by) => self.list.wheel(by.y),
            PEvent::PointerDown(at) => {
                let Some(act) = self.list.press(at) else { return false };
                match act {
                    Act::Look(look) => _ = self.desk.dress(&look),
                    Act::Profile => self.desk.wish.borrow_mut().push(PDeed::Open(PROFILE.to_string())),
                    Act::Shut => self.desk.wish.borrow_mut().push(PDeed::Close),
                    Act::Field(index) => self.aim.set(index),
                    Act::Enter => self.enter(),
                    Act::Alone => _ = self.desk.kept.alone(),
                    Act::Export => self.export(),
                    Act::Sign(who) => {
                        // Another person has other documents, and begins
                        // where the site begins.
                        if self.desk.kept.sign(who.as_deref()) {
                            *self.desk.now.who.borrow_mut() = None;
                        }
                    }
                }
                true
            }
            _ => false,
        }
    }
}

/// Who the person is, as a line to print.
pub fn who(me: &Value, word: &dyn Fn(&str) -> String, text: &PTable) -> String {
    let said = |name: &str| me[word(name)].as_str().unwrap_or_default().to_string();
    match me[word("me.guest")].as_bool().unwrap_or(true) {
        true => text.get("account.guest").to_string(),
        false => format!("{} @{}", said("me.name"), said("me.handle")),
    }
}
