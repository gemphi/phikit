//! The profile: who the user is, as a modal shows it. Their mark, their
//! name and their address; then what is known of them, a name and what
//! it stands for; then how the app appears, a pill for each look there
//! is. It is the same view in every app: the user, what is true of them
//! now and the looks are the app's data; and under them what the person
//! prefers of how it appears (`profile/prefer.rs`).
//!
//! A press on a look turns the window to it, and tells the app the word
//! `look` and the look's name, so the app can keep it. A profile longer
//! than its panel is scrolled with the wheel.

mod axis;
mod prefer;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use super::{DIM, GAP, INK, ROW, person};
use crate::pui::core::light::label;
use crate::pui::core::{PDraw, PEvent, PLight, PPaint, PPoint, PRect, PView};
use crate::pui::layout::{PDry, PScroll, fold};
use crate::pui::window::{PDeed, PWish};
use crate::service::PTable;
use crate::theme::PShade;

/// The theme's tokens for the profile: how large the user's mark is, how
/// round a block of rows and a pill are, and the fill of each in a theme
/// with no light.
pub const FACE: &str = "profile.avatar";
pub const ROUND: &str = "profile.radius";
pub const FILL: &str = "profile.fill";
pub const LIT: &str = "profile.active";
/// How much of the width a row's name has.
pub const SPLIT: &str = "profile.split";

/// The rows of a user's table that are not shown as rows: `name` and
/// `mail` head the profile, `modal` says which modal it is, and `about`
/// and `appearance` are the headings over the rows and over the looks;
/// `prefer`, where it has one, over what the person prefers. A row whose name
/// starts with `look.` is what a look is called.
const HEAD: [&str; 7] = ["name", "mail", "modal", "about", "appearance", "prefer", axis::AXIS];
pub const LOOK: &str = "look.";
/// The word the app is told when a look is chosen, before its name.
pub const TOLD: &str = "look";

/// What is true of the user now: asked once a frame, so what it gives
/// may change while the window is open.
pub type PFact = Rc<dyn Fn() -> PTable>;

/// Where each press of the profile was drawn, and what it says.
type Spot = Vec<(PRect, String)>;
/// What a group holds, drawn from a height down; how far it ends.
type Fill<'a> = dyn FnMut(&mut dyn PPaint, f32, &mut Spot) -> f32 + 'a;

/// Who the user is.
pub struct PProfile {
    frame: PRect,
    user: PTable,
    fact: Option<PFact>,
    /// What `fact` gave last.
    live: PTable,
    /// The looks, and where the app is told that one was chosen.
    shade: Option<Rc<PShade>>,
    wish: Option<PWish>,
    /// Where each look's pill was drawn last, in the window.
    spot: RefCell<Vec<(PRect, String)>>,
    scroll: Cell<PScroll>,
    /// The groups folded shut: every other is open.
    shut: RefCell<Vec<String>>,
}

impl PProfile {
    pub fn new(user: PTable) -> PProfile {
        PProfile { frame: PRect::default(), user, fact: None, live: PTable::default(), shade: None, wish: None, spot: RefCell::default(), scroll: Cell::default(), shut: RefCell::default() }
    }

    /// The same profile with rows that are asked for once a frame.
    pub fn live(self, fact: Option<PFact>) -> PProfile {
        PProfile { fact, ..self }
    }

    /// The same profile with the looks to choose among.
    pub fn dressed(self, shade: Option<Rc<PShade>>, wish: Option<PWish>) -> PProfile {
        PProfile { shade, wish, ..self }
    }

    /// The rows: those of the user's table that are rows, then what is
    /// true now.
    fn row(&self) -> Vec<(&str, &str)> {
        let words = ["axis.", "value.", "place."];
        let shown = |name: &str| !HEAD.contains(&name) && !name.starts_with(LOOK) && !name.starts_with(prefer::WORD) && !words.iter().any(|word| name.starts_with(word));
        let own = self.user.list().iter().filter(|(name, _)| shown(name));
        own.chain(self.live.list()).map(|(name, value)| (name.as_str(), value.as_str())).collect()
    }

    /// A group as an accordion: its heading, pressed to fold or open it,
    /// and while open what it holds on a ground of its own, measured
    /// first with nothing drawn. How far down it ends.
    #[allow(clippy::too_many_arguments)]
    fn group(&self, key: &str, said: &str, at: PPoint, wide: f32, paint: &mut dyn PPaint, spot: &mut Spot, fill: &mut Fill) -> f32 {
        let gap = paint.size(GAP);
        // A group the user's table gives no word has no heading: it is
        // its ground alone, and never folds.
        let open = said.is_empty() || !self.shut.borrow().iter().any(|one| one == key);
        let below = match said.is_empty() {
            true => at.y,
            false => {
                let (rect, below) = axis::heading(paint, at, wide, said, open);
                spot.push((rect, format!("{}{key}", axis::FOLD)));
                below
            }
        };
        if !open {
            return below + gap / 2.0;
        }
        let end = fill(&mut PDry(&*paint), below + gap / 2.0, &mut Vec::new());
        axis::body(paint, PRect::new(at.x, below, wide, end - below + gap / 2.0));
        fill(paint, below + gap / 2.0, spot) + gap
    }
}

impl PDraw for PProfile {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, line, face, round) = (paint.size(GAP), paint.size(ROW), paint.size(FACE), paint.size(ROUND));
        let (ink, dim, wide) = (paint.color(INK), paint.color(DIM), self.frame.size.width - gap * 2.0);
        // What is longer than the panel is scrolled, and drawn nowhere else.
        let (room, mut scroll) = (PRect { origin, size: self.frame.size }, self.scroll.get());
        let origin = origin.minus(PPoint::new(0.0, scroll.at()));
        paint.clip(room);
        let (left, name) = (origin.x + gap, self.user.get("name"));
        person(super::picture(&self.user).as_ref(), name, PRect::new(left, origin.y + gap, face, face), paint);
        let beside = PPoint::new(left + face + gap, origin.y + gap + face / 2.0 - line);
        paint.text(beside, name, ink);
        paint.text(beside.plus(PPoint::new(0.0, line)), self.user.get("mail"), dim);
        // What is known of them, how the app appears and what the person
        // prefers: each a group that folds, on a ground of its own.
        let split = wide * paint.theme().size(SPLIT).unwrap_or(0.4);
        let row = self.row();
        let set: Vec<Vec<String>> = row.iter().map(|(_, value)| fold(value, wide - split - gap * 2.0, &|text| paint.measure(text))).collect();
        let high = |set: &Vec<String>| (set.len().max(1) as f32 + 0.5) * line;
        let mut spot = Vec::new();
        let mut down = origin.y + gap * 2.0 + face;
        if !row.is_empty() {
            let said = self.user.get("about").to_string();
            down = self.group("about", &said, PPoint::new(left, down), wide, paint, &mut spot, &mut |paint, mut top, _| {
                for ((name, _), set) in row.iter().zip(&set) {
                    paint.text(PPoint::new(left + gap, top), name, dim);
                    for (index, text) in set.iter().enumerate() {
                        paint.text(PPoint::new(left + gap + split, top + index as f32 * line), text, ink);
                    }
                    top += high(set);
                }
                top
            });
        }
        if let Some(shade) = self.shade.clone() {
            let axes = axis::axes(&self.user);
            for (index, name) in axes.iter().enumerate() {
                // An axis of how it appears: mode, surface.
                let said = axis::said(&self.user, name).to_string();
                let user = &self.user;
                down = self.group(name, &said, PPoint::new(left, down), wide, paint, &mut spot, &mut |paint, top, spot| axis::values(user, &shade, index, paint, PPoint::new(left + gap, top), wide - gap * 2.0, spot));
            }
            if axes.is_empty() {
                // With no axes, a pill for each look, the one shown lit.
                let said = self.user.get("appearance").to_string();
                down = self.group("appearance", &said, PPoint::new(left, down), wide, paint, &mut spot, &mut |paint, top, spot| {
                    let mut at = PPoint::new(left + gap, top);
                    let mut end = top;
                    for look in shade.list() {
                        let word = Some(self.user.get(&format!("{LOOK}{look}"))).filter(|word| !word.is_empty()).unwrap_or(look);
                        let long = paint.measure(word) + gap * 2.0;
                        if at.x + long > left + wide - gap && at.x > left + gap {
                            at = PPoint::new(left + gap, at.y + line + gap * 1.5);
                        }
                        let rect = PRect::new(at.x, at.y, long, line + gap);
                        let (light, token) = match look == shade.name() {
                            true => (PLight::Strong, LIT),
                            false => (PLight::Soft, FILL),
                        };
                        label(paint, rect, round, light, true, token);
                        paint.text(PPoint::new(rect.origin.x + gap, rect.center().y - line / 2.0), word, ink);
                        spot.push((rect, look.to_string()));
                        at.x += long + gap / 2.0;
                        end = rect.origin.y + rect.size.height;
                    }
                    end
                });
            }
            // What the person prefers of how they appear, where the
            // user's table gives that a heading.
            if !self.user.get("prefer").is_empty() {
                let said = self.user.get("prefer").to_string();
                down = self.group("prefer", &said, PPoint::new(left, down), wide, paint, &mut spot, &mut |paint, top, spot| {
                    let (end, more) = self.prefer(&shade, paint, left + gap, top, wide - gap * 2.0);
                    spot.extend(more);
                    end
                });
            }
        }
        *self.spot.borrow_mut() = spot;
        paint.unclip();
        // How long all of it is, from its top to a gap under its foot.
        scroll.fit(down + gap - origin.y, room.size.height);
        self.scroll.set(scroll);
    }
}

impl PView for PProfile {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    /// Once a frame it asks what is true now.
    fn tick(&mut self) -> bool {
        self.live = self.fact.as_ref().map(|fact| fact()).unwrap_or_default();
        false
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        let at = match *event {
            PEvent::PointerDown(at) => at,
            // The wheel turned away from the hand goes up the profile.
            PEvent::Scroll(_, by) => {
                let mut scroll = self.scroll.get();
                let any = scroll.turn(-by.y);
                self.scroll.set(scroll);
                return any;
            }
            _ => return false,
        };
        // What was drawn is in the window's coordinates.
        let place = at.plus(self.frame.origin);
        let look = self.spot.borrow().iter().find(|(rect, _)| rect.contains(place)).map(|(_, look)| look.clone());
        // A heading folds its group or opens it.
        if let Some(key) = look.as_deref().and_then(|look| look.strip_prefix(axis::FOLD)) {
            let mut shut = self.shut.borrow_mut();
            match shut.iter().position(|one| one == key) {
                Some(at) => _ = shut.remove(at),
                None => shut.push(key.to_string()),
            }
            return true;
        }
        // A preference is put over the look, and the app told all that is
        // preferred now.
        let prefer = look.as_ref().zip(self.shade.as_ref()).and_then(|(look, shade)| PProfile::chose(shade, look));
        if let Some(kept) = prefer {
            self.wish.iter().for_each(|wish| wish.borrow_mut().push(PDeed::Tell(format!("{} {kept}", prefer::TOLD))));
            return true;
        }
        let turned = look.as_ref().is_some_and(|look| self.shade.as_ref().is_some_and(|shade| shade.turn(look)));
        if let (true, Some(look), Some(wish)) = (turned, look, &self.wish) {
            wish.borrow_mut().push(PDeed::Tell(format!("{TOLD} {look}")));
        }
        turned
    }
}
