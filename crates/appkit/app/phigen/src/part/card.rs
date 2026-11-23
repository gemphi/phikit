//! The cards: a few ways to begin, each a title and a line
//! (`data/card.tsv`). A press on one says what it stands for, or, when it
//! stands for nothing, only lets the person type. They are shown twice:
//! under the field of a chat nothing was said in, side by side; and in
//! the welcome modal, one under the other, under a heading and over a
//! button that closes it.

use appkit::pui::control::{PButton, PControl};
use appkit::pui::core::{PDraw, PEvent, PLight, PPaint, PPoint, PRect, PView, light, send, show};
use appkit::pui::layout::wrap;
use appkit::pui::window::PDeed;

use super::{DIM, Desk, GAP, INK, LINE, fit};

/// The theme's tokens for a card: how round and how high it is, how wide
/// the modal's button is, and how narrow the room may be before the cards
/// go one under the other. A card wears the surface's light.
pub const RADIUS: &str = "card.radius";
pub const HIGH: &str = "card.height";
pub const BUTTON: &str = "button.width";
pub const NARROW: &str = "card.narrow";

/// One way to begin.
struct Way {
    title: String,
    line: String,
    /// What a press says; nothing lets the person type.
    say: String,
}

/// A few ways to begin.
pub struct Card {
    frame: PRect,
    desk: Desk,
    way: Vec<Way>,
    /// In the welcome modal: under a heading, over a button that closes it.
    modal: bool,
    go: PButton,
}

impl Card {
    /// The cards a table gives: a card a line, as its slug, its title, its
    /// line and what it says, between tabs.
    pub fn new(desk: Desk, table: &str, modal: bool) -> Card {
        let line = table.lines().filter(|line| !line.trim().is_empty() && !line.starts_with('#'));
        let way = line.map(|line| {
            let cell: Vec<&str> = line.split('\t').map(str::trim).collect();
            let at = |index: usize| cell.get(index).copied().unwrap_or_default().to_string();
            Way { title: at(1), line: at(2), say: at(3) }
        });
        let go = PButton::new(desk.text.get("welcome.go"), PRect::default());
        Card { frame: PRect::default(), way: way.collect(), desk, modal, go }
    }

    /// How far down the first card is: under the modal's heading.
    fn top(&self) -> f32 {
        match self.modal {
            true => self.desk.size(GAP) * 2.0 + self.desk.size(LINE) * 2.0,
            false => self.desk.size(GAP),
        }
    }

    /// The rectangle of each card, in the part's own coordinates: across
    /// the body's column, side by side where there is room and it is not
    /// the modal, else stacked. Stacked, the cards share what the part has,
    /// less the modal's heading and button.
    fn rect(&self) -> Vec<PRect> {
        let (gap, count) = (self.desk.size(GAP), self.way.len().max(1) as f32);
        let (start, wide) = self.desk.column(self.frame.size.width);
        let beside = !self.modal && wide >= self.desk.size(NARROW);
        // What is kept under the cards: a gap, and in the modal its button.
        let foot = match self.modal {
            true => self.go.frame().size.height + gap * 2.0,
            false => gap,
        };
        // Stacked, the cards share what is left, so none is cut off.
        let left = self.frame.size.height - self.top() - foot;
        let high = match beside {
            true => self.desk.size(HIGH),
            false => self.desk.size(HIGH).min((left - gap * (count - 1.0)) / count),
        };
        let each = (wide - gap * (count - 1.0)) / count;
        let place = |index: usize| match beside {
            true => PRect::new(start + index as f32 * (each + gap), self.top(), each, high),
            false => PRect::new(start, self.top() + index as f32 * (high + gap), wide, high),
        };
        (0..self.way.len()).map(place).collect()
    }
}

/// A card's line set in the room it has: two lines at most, the second
/// cut short when there is more.
fn brief(text: &str, room: f32, paint: &dyn PPaint) -> Vec<String> {
    let mut row = wrap(text, room, &|text| paint.measure(text));
    let rest = row.split_off(row.len().min(1)).join(" ");
    row.extend((!rest.is_empty()).then(|| fit(&rest, room, paint)));
    row
}

impl PDraw for Card {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, line, radius) = (paint.size(GAP), paint.size(LINE), paint.size(RADIUS));
        let (ink, dim) = (paint.color(INK), paint.color(DIM));
        if self.modal {
            paint.text(origin.plus(PPoint::new(gap, gap)), self.desk.text.get("welcome.head"), ink);
            paint.text(origin.plus(PPoint::new(gap, gap + line)), self.desk.text.get("welcome.line"), dim);
            show(&self.go, paint, origin);
        }
        // Every card's words begin at the same height: the room is kept
        // for as many lines as the longest of them takes.
        let rect = self.rect();
        let room = rect.first().map_or(0.0, |rect| rect.size.width) - gap * 2.0;
        let set: Vec<Vec<String>> = self.way.iter().map(|way| brief(&way.line, room, paint)).collect();
        let most = set.iter().map(Vec::len).max().unwrap_or(1) as f32;
        for ((rect, way), row) in rect.into_iter().zip(&self.way).zip(&set) {
            let at = rect.moved(origin);
            light::surface(paint, at, radius, PLight::Rest, "");
            let top = at.origin.y + (at.size.height - line * (1.0 + most)) / 2.0;
            paint.text(PPoint::new(at.origin.x + gap, top), &fit(&way.title, room, paint), ink);
            for (index, text) in row.iter().enumerate() {
                paint.text(PPoint::new(at.origin.x + gap, top + line * (index + 1) as f32), text, dim);
            }
        }
    }
}

impl PView for Card {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        let (gap, wide, high) = (self.desk.size(GAP), self.desk.size(BUTTON), self.desk.size(LINE) * 2.0);
        self.frame = frame;
        self.go.set_frame(PRect::new(frame.size.width - gap - wide, frame.size.height - gap - high, wide, high));
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        let PEvent::PointerDown(at) = *event else { return false };
        let pressed = self.go.value();
        let closed = self.modal && send(&mut self.go, event) && self.go.value() != pressed;
        let way = self.rect().iter().position(|rect| rect.contains(at)).and_then(|index| self.way.get(index));
        let chosen = way.is_some();
        // A press on a card, or on the button, takes the modal away; what
        // the card stands for is said after.
        if self.modal && (closed || chosen) {
            self.desk.wish.borrow_mut().push(PDeed::Close);
        }
        way.filter(|way| !way.say.is_empty()).into_iter().for_each(|way| _ = self.desk.say(&way.say));
        closed || chosen
    }
}
