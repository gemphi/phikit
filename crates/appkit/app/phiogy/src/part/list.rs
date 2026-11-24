//! A page that is a list: headings, a line of words, a sum on a ground
//! of its own, a field to type in, a row of tabs, and lines, each a face, a mark or an icon with
//! two lines of words beside it and what can be done at its end. The
//! hosts, friends, finding people, the shops and the bag are each such
//! a page, as on the web each is made of the kit's own rows
//! (`puikit/app/phi-kit/src/{hosts,friends,direct,shops,bag}.tsx`).
//!
//! A page says what it holds as a list of items and what a press on each
//! means; `Lister` lays them out, scrolls them, and says what was pressed.

use std::cell::{Cell, RefCell};

use appkit::pui::block::{PBand, PRow};
use appkit::pui::control::{PControl, PField, inside};
use appkit::pui::core::{PPaint, PPoint, PRect, PView, figure, show};
use appkit::pui::layout::{PDry, PScroll, ground, wrap};

mod item;

pub use super::tool::grouped;
pub use item::{Deed, Item, Seat};
use super::{DIM, Desk, GAP, INK, LINE, ROW, fit};

/// The theme's tokens: how high a line of a list is, and how far
/// down a page begins. How large what leads a line is, is the kit's
/// (`row.lead`).
pub const HIGH: &str = "list.row";
pub const TOP: &str = "list.top";

/// Where a field of a page stands in itself: as wide as the page's
/// column, as high as a row. The page puts it where its item is.
pub fn field(desk: &Desk, width: f32) -> PRect {
    PRect::new(0.0, 0.0, desk.column(width).1, desk.size(ROW))
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
    /// What can be done, the kit's group of pills, drawn so that it
    /// ends at `end`, its middle at that height; where it begins.
    fn deeds(&self, paint: &mut dyn PPaint, end: PPoint, deed: &[Deed<A>], spot: &mut Vec<(PRect, A)>) -> f32 {
        let pill: Vec<(&str, &str, bool)> = deed.iter().map(Deed::pill).collect();
        let high = pill.high(paint);
        let rect = pill.draw_end(paint, PPoint::new(end.x, end.y - high / 2.0));
        let begin = rect.first().map_or(end.x, |rect| rect.origin.x);
        spot.extend(rect.into_iter().zip(deed.iter().map(|one| one.act.clone())));
        begin - paint.size(GAP) / 4.0
    }

    /// Draw a page: its items down the column of `frame`, which is the
    /// part's own, at `origin` in the window. Every run of items
    /// between two headings is a section, and stands on a ground of
    /// its own: where each item stands is learned first with nothing
    /// drawn.
    pub fn draw(&self, paint: &mut dyn PPaint, origin: PPoint, frame: PRect, desk: &Desk, item: &[Item<A>], field: &[&PField]) {
        let stand = self.pass(&mut PDry(&*paint), origin, frame, desk, item, field);
        let ((start, wide), gap) = (desk.column(frame.size.width), paint.size(GAP));
        paint.clip(PRect { origin, size: frame.size });
        let mut from = None;
        for index in 0..=item.len() {
            let head = item.get(index).is_none_or(|one| matches!(one, Item::Head(_) | Item::Top(..)));
            match (head, from) {
                (true, Some(first)) => {
                    ground(paint, PPoint::new(origin.x + start, stand[first]), wide, stand[index] - gap / 2.0);
                    from = None;
                }
                (false, None) => from = Some(index),
                _ => {}
            }
        }
        paint.unclip();
        self.pass(paint, origin, frame, desk, item, field);
    }

    /// Draw the items; where each begins, and where the last ends.
    fn pass(&self, paint: &mut dyn PPaint, origin: PPoint, frame: PRect, desk: &Desk, item: &[Item<A>], field: &[&PField]) -> Vec<f32> {
        let (gap, line, row, high) = (paint.size(GAP), paint.size(LINE), paint.size(ROW), paint.size(HIGH));
        let (ink, dim) = (paint.color(INK), paint.color(DIM));
        let (start, wide) = desk.column(frame.size.width);
        let left = origin.x + start;
        let begin = origin.y + paint.size(TOP) - self.scroll.get().at();
        let (mut top, mut spot, mut stand) = (begin, Vec::new(), Vec::new());
        paint.clip(PRect { origin, size: frame.size });
        for one in item {
            stand.push(top);
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
                Item::Sum(name, sum) => {
                    // What it is a sum of at the left; the sum at the
                    // right, on its ground.
                    let size = figure::size(paint, sum);
                    let middle = top + size.height / 2.0;
                    paint.text(PPoint::new(left, middle - line / 2.0), &fit(name, wide - size.width - gap, paint), dim);
                    figure::end(paint, PPoint::new(left + wide, middle), sum, false);
                    top += size.height + gap / 4.0;
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
                    // As many on a line as the page is wide, the rest on
                    // the lines under it.
                    let pill: Vec<(&str, &str, bool)> = tab.iter().map(Deed::pill).collect();
                    let (rect, high) = pill.draw_lines(paint, PPoint::new(left, top), wide);
                    spot.extend(rect.into_iter().zip(tab.iter().map(|one| one.act.clone())));
                    top += high + gap / 2.0;
                }
                Item::Line(seat, title, sub, open, deed) => {
                    // The kit's row, with what can be done at its end.
                    let rect = PRect::new(left, top, wide, high);
                    open.iter().for_each(|act| spot.push((rect, act.clone())));
                    let end = self.deeds(paint, PPoint::new(left + wide, rect.center().y), deed, &mut spot);
                    (seat.lead(), title.as_str(), sub.as_str()).row(paint, rect, left + wide - end);
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
        stand.push(top);
        stand
    }

    /// What a press at a place of the part means, if anything: of the
    /// places it is in, the smallest, so what can be done at a line's
    /// end is not the line.
    pub fn press(&self, at: PPoint) -> Option<A> {
        let place = at.plus(self.origin.get());
        let (spot, area) = (self.spot.borrow(), |rect: &PRect| rect.size.width * rect.size.height);
        spot.iter().filter(|(rect, _)| rect.contains(place)).min_by(|one, other| area(&one.0).total_cmp(&area(&other.0))).map(|(_, act)| act.clone())
    }

    /// Where a press on the page does something, as it was drawn last.
    pub fn hands(&self) -> Vec<PRect> {
        self.spot.borrow().iter().map(|(rect, _)| *rect).collect()
    }

    /// Turn the wheel: `true` when the page moved.
    pub fn wheel(&self, by: f32) -> bool {
        let mut scroll = self.scroll.get();
        let any = scroll.turn(-by);
        self.scroll.set(scroll);
        any
    }
}
