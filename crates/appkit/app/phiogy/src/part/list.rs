//! A page that is a list: headings, a line of words, a field to type
//! in, a row of tabs, and lines, each a face, a mark or an icon with
//! two lines of words beside it and what can be done at its end. The
//! hosts, friends, finding people, the shops and the bag are each such
//! a page, as on the web each is made of the kit's own rows
//! (`puikit/app/phi-kit/src/{hosts,friends,direct,shops,bag}.tsx`).
//!
//! A page says what it holds as a list of items and what a press on each
//! means; `Lister` lays them out, scrolls them, and says what was pressed.

use std::cell::{Cell, RefCell};

use appkit::pui::control::{PControl, PField, inside};
use appkit::pui::core::{PFit, PLight, PPaint, PPoint, PRect, PView, light, show};
use appkit::pui::layout::{PScroll, wrap};

pub use super::tool::grouped;
use super::{DIM, Desk, GAP, ICON, INK, LINE, RADIUS, ROW, fit, tool};
use crate::site::Face;

/// The theme's tokens: how high a line of a list is, how large what
/// stands at its head, and how far down a page begins.
pub const HIGH: &str = "list.row";
pub const MARK: &str = "list.mark";
pub const TOP: &str = "list.top";

/// Where a field of a page stands in itself: as wide as the page's
/// column, as high as a row. The page puts it where its item is.
pub fn field(desk: &Desk, width: f32) -> PRect {
    PRect::new(0.0, 0.0, desk.column(width).1, desk.size(ROW))
}

/// What stands at the head of a line.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Seat {
    #[default]
    None,
    /// A cell of a sheet: a face, a place's mark.
    Face(Face),
    Icon(String),
    /// A picture by its key, whole.
    Picture(String),
}

/// What can be done at the end of a line, or in a row of tabs: its
/// words, or its icon where it has one; whether it is so; and what a
/// press means.
#[derive(Debug, Clone, PartialEq)]
pub struct Deed<A> {
    pub label: String,
    pub icon: String,
    pub on: bool,
    pub act: A,
}

impl<A> Deed<A> {
    pub fn word(label: &str, on: bool, act: A) -> Deed<A> {
        Deed { label: label.to_string(), icon: String::new(), on, act }
    }

    pub fn icon(icon: &str, label: &str, act: A) -> Deed<A> {
        Deed { label: label.to_string(), icon: icon.to_string(), on: false, act }
    }
}

/// One thing of a page, from its top down.
#[derive(Debug, Clone, PartialEq)]
pub enum Item<A> {
    Head(String),
    /// A heading with what can be done at its end: a page that has no
    /// head of a panel's over it, and its own way out.
    Top(String, Vec<Deed<A>>),
    /// Words that are not pressed, broken to fit.
    Note(String),
    /// The field of that number among the page's, what it says while
    /// empty, and what a press on it means.
    Field(usize, String, A),
    Tabs(Vec<Deed<A>>),
    /// A line: its head, two lines of words, what a press on it means
    /// if it means something, and what can be done at its end.
    Line(Seat, String, String, Option<A>, Vec<Deed<A>>),
}

/// What lays a page's items out and says what was pressed.
pub struct Lister<A> {
    scroll: Cell<PScroll>,
    origin: Cell<PPoint>,
    spot: RefCell<Vec<(PRect, A)>>,
}

impl<A> Default for Lister<A> {
    fn default() -> Lister<A> {
        Lister { scroll: Cell::default(), origin: Cell::default(), spot: RefCell::default() }
    }
}

impl<A: Clone> Lister<A> {
    /// A pill with words in it, lit as `light` says.
    fn pill(paint: &mut dyn PPaint, at: PPoint, said: &str, light: PLight) -> PRect {
        let (gap, line, ink, radius) = (paint.size(GAP), paint.size(LINE), paint.color(INK), paint.size(RADIUS));
        let rect = PRect::new(at.x, at.y, paint.measure(said) + gap, line + gap / 2.0);
        light::label(paint, rect, radius, light, false, "");
        paint.text(PPoint::new(at.x + gap / 2.0, at.y + gap / 4.0), said, ink);
        rect
    }

    /// What can be done, drawn so that the last ends at `end`, their
    /// middle at its height; where the first begins.
    fn deeds(&self, paint: &mut dyn PPaint, end: PPoint, deed: &[Deed<A>], spot: &mut Vec<(PRect, A)>) -> f32 {
        let (gap, line) = (paint.size(GAP), paint.size(LINE));
        let mut right = end.x;
        for one in deed.iter().rev() {
            let light = match one.on {
                true => PLight::Strong,
                false => PLight::Rest,
            };
            let rect = match one.icon.is_empty() {
                true => {
                    let wide = paint.measure(&one.label) + gap;
                    Self::pill(paint, PPoint::new(right - wide, end.y - (line + gap / 2.0) / 2.0), &one.label, light)
                }
                false => {
                    let rect = PRect::new(right - line - gap / 2.0, end.y - (line + gap / 2.0) / 2.0, line + gap / 2.0, line + gap / 2.0);
                    tool::button(paint, rect, &one.icon, Some(light));
                    rect
                }
            };
            spot.push((rect, one.act.clone()));
            right = rect.origin.x - gap / 4.0;
        }
        right
    }

    /// Draw a page: its items down the column of `frame`, which is the
    /// part's own, at `origin` in the window.
    pub fn draw(&self, paint: &mut dyn PPaint, origin: PPoint, frame: PRect, desk: &Desk, item: &[Item<A>], field: &[&PField]) {
        let (gap, line, row, high, mark) = (paint.size(GAP), paint.size(LINE), paint.size(ROW), paint.size(HIGH), paint.size(MARK));
        let (ink, dim, icon) = (paint.color(INK), paint.color(DIM), paint.size(ICON));
        let (start, wide) = desk.column(frame.size.width);
        let left = origin.x + start;
        let begin = origin.y + paint.size(TOP) - self.scroll.get().at();
        let (mut top, mut spot) = (begin, Vec::new());
        paint.clip(PRect { origin, size: frame.size });
        for one in item {
            match one {
                Item::Head(said) => {
                    paint.head(PPoint::new(left, top + gap / 2.0), said, ink);
                    top += paint.size("head.line") + gap;
                }
                Item::Top(said, deed) => {
                    let high = paint.size("head.line");
                    paint.head(PPoint::new(left, top + gap / 2.0), said, ink);
                    self.deeds(paint, PPoint::new(left + wide, top + gap / 2.0 + high / 2.0), deed, &mut spot);
                    top += high + gap;
                }
                Item::Note(said) => {
                    for words in wrap(said, wide, &|text| paint.measure(text)) {
                        paint.text(PPoint::new(left, top), &words, dim);
                        top += line;
                    }
                    top += gap / 2.0;
                }
                Item::Field(index, hint, act) => {
                    let rect = PRect::new(left, top, wide, row);
                    if let Some(typed) = field.get(*index) {
                        show(*typed, paint, rect.origin.minus(typed.frame().origin));
                        if typed.value().is_empty() {
                            paint.text(rect.origin.plus(inside(paint, row)), &fit(hint, wide - gap, paint), dim);
                        }
                    }
                    spot.push((rect, act.clone()));
                    top += row + gap / 2.0;
                }
                Item::Tabs(tab) => {
                    let mut across = left;
                    for one in tab {
                        let light = match one.on {
                            true => PLight::Strong,
                            false => PLight::Soft,
                        };
                        let rect = Self::pill(paint, PPoint::new(across, top), &one.label, light);
                        spot.push((rect, one.act.clone()));
                        across += rect.size.width + gap / 4.0;
                    }
                    top += line + gap;
                }
                Item::Line(seat, title, sub, open, deed) => {
                    let rect = PRect::new(left, top, wide, high);
                    open.iter().for_each(|act| spot.push((rect, act.clone())));
                    let place = PRect::new(left, rect.center().y - mark / 2.0, mark, mark);
                    let radius = paint.size(RADIUS);
                    match seat {
                        Seat::None => {}
                        Seat::Face(face) => paint.picture(&face.sheet, place, mark / 2.0, PFit::Cell { column: face.column, index: face.index }),
                        Seat::Icon(name) => paint.icon(name, PRect::new(place.center().x - icon / 2.0, place.center().y - icon / 2.0, icon, icon), ink),
                        Seat::Picture(key) => paint.picture(key, place, radius, PFit::Whole),
                    }
                    let beside = match seat {
                        Seat::None => left,
                        _ => left + mark + gap / 2.0,
                    };
                    let end = self.deeds(paint, PPoint::new(left + wide, rect.center().y), deed, &mut spot);
                    let room = (end - beside - gap / 2.0).max(0.0);
                    let both = !sub.is_empty();
                    let up = match both {
                        true => line,
                        false => line / 2.0,
                    };
                    paint.text(PPoint::new(beside, rect.center().y - up), &fit(title, room, paint), ink);
                    if both {
                        paint.text(PPoint::new(beside, rect.center().y), &fit(sub, room, paint), dim);
                    }
                    top += high;
                }
            }
        }
        paint.unclip();
        let mut scroll = self.scroll.get();
        scroll.fit(top - begin + paint.size(TOP) + gap, frame.size.height);
        self.scroll.set(scroll);
        self.origin.set(origin);
        // What is out of sight is no place to press.
        spot.retain(|(rect, _)| PRect { origin, size: frame.size }.contains(rect.center()));
        *self.spot.borrow_mut() = spot;
    }

    /// What a press at a place of the part means, if anything: of the
    /// places it is in, the smallest, so what can be done at a line's
    /// end is not the line.
    pub fn press(&self, at: PPoint) -> Option<A> {
        let place = at.plus(self.origin.get());
        let (spot, area) = (self.spot.borrow(), |rect: &PRect| rect.size.width * rect.size.height);
        spot.iter().filter(|(rect, _)| rect.contains(place)).min_by(|one, other| area(&one.0).total_cmp(&area(&other.0))).map(|(_, act)| act.clone())
    }

    /// Turn the wheel: `true` when the page moved.
    pub fn wheel(&self, by: f32) -> bool {
        let mut scroll = self.scroll.get();
        let any = scroll.turn(-by);
        self.scroll.set(scroll);
        any
    }
}
