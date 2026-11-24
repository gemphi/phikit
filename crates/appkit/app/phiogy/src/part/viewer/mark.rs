//! Marks on a picture looked at, as Google Photos has them: lines
//! sketched and words set on it, each in one of the inks of
//! `data/mark.tsv`. Where a mark lies is a share of the picture's width
//! and height, so it lies on the same place however large the picture
//! is shown. The last mark can be taken back, and put back again, until
//! another is made. Kept, the marks are what the person chose for that
//! picture (`mark/<key>`): opened again, it has them.

use appkit::pui::core::{PColor, PPaint, PPoint, PRect};

use crate::data;

/// A mark: a line through its points, or words at a place, in an ink.
#[derive(Debug, Clone, PartialEq)]
pub enum Mark {
    Line { ink: usize, point: Vec<PPoint> },
    Word { ink: usize, at: PPoint, text: String },
}

/// The marks on a picture: those made, those taken back, and whether
/// what is made is what was kept.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Sheet {
    pub done: Vec<Mark>,
    undone: Vec<Mark>,
    pub kept: bool,
}

/// What divides marks as they are kept, and what divides a mark's parts.
const APART: char = '|';

/// A place as it is kept: its two shares.
fn place(point: PPoint) -> String {
    format!("{:.4},{:.4}", point.x, point.y)
}

fn read(word: &str) -> Option<PPoint> {
    let (x, y) = word.split_once(',')?;
    Some(PPoint::new(x.parse().ok()?, y.parse().ok()?))
}

/// The inks, as colours, in their order.
pub fn inks() -> Vec<PColor> {
    let share = |cell: Option<&&str>| cell.and_then(|n| n.parse::<f32>().ok()).unwrap_or_default() / 255.0;
    data::row(data::MARK).into_iter().map(|cell| PColor::new(share(cell.get(1)), share(cell.get(2)), share(cell.get(3)), 1.0)).collect()
}

impl Sheet {
    /// The marks as they were kept.
    pub fn kept(value: &str) -> Sheet {
        let mark = value.split(APART).filter_map(|one| {
            let mut part = one.splitn(3, ' ');
            let (kind, ink) = (part.next()?, part.next()?.parse().ok()?);
            let rest = part.next()?;
            match kind {
                "l" => Some(Mark::Line { ink, point: rest.split(';').filter_map(read).collect() }),
                "w" => {
                    let (at, text) = rest.split_once(' ')?;
                    Some(Mark::Word { ink, at: read(at)?, text: text.to_string() })
                }
                _ => None,
            }
        });
        Sheet { done: mark.collect(), undone: Vec::new(), kept: true }
    }

    /// What they are kept as.
    pub fn value(&self) -> String {
        let one = |mark: &Mark| match mark {
            Mark::Line { ink, point } => format!("l {ink} {}", point.iter().map(|p| place(*p)).collect::<Vec<_>>().join(";")),
            Mark::Word { ink, at, text } => format!("w {ink} {} {}", place(*at), text.replace(APART, "/")),
        };
        self.done.iter().map(one).collect::<Vec<_>>().join(&APART.to_string())
    }

    /// A new mark: nothing taken back can be put back after it.
    pub fn add(&mut self, mark: Mark) {
        self.done.push(mark);
        self.undone.clear();
        self.kept = false;
    }

    /// The mark being made now, the last: a point more of its line, a
    /// letter more of its words or one less.
    pub fn last(&mut self) -> Option<&mut Mark> {
        self.kept = false;
        self.done.last_mut()
    }

    /// Take the last mark back; put the last taken back again.
    pub fn undo(&mut self) -> bool {
        let one = self.done.pop();
        self.kept &= one.is_none();
        one.map(|one| self.undone.push(one)).is_some()
    }

    pub fn redo(&mut self) -> bool {
        let one = self.undone.pop();
        self.kept &= one.is_none();
        one.map(|one| self.done.push(one)).is_some()
    }

    /// Draw the marks over a picture shown there, a line as round dots
    /// along it as thick as `thick`, words in the ink at their place.
    pub fn draw(&self, paint: &mut dyn PPaint, shown: PRect, thick: f32) {
        let ink = inks();
        let colour = |index: &usize| ink.get(*index).copied().unwrap_or_default();
        let at = |p: PPoint| PPoint::new(shown.origin.x + p.x * shown.size.width, shown.origin.y + p.y * shown.size.height);
        for mark in &self.done {
            match mark {
                Mark::Line { ink, point } => {
                    let place: Vec<PPoint> = point.iter().map(|p| at(*p)).collect();
                    let dot = |paint: &mut dyn PPaint, p: PPoint| paint.round(PRect::new(p.x - thick / 2.0, p.y - thick / 2.0, thick, thick), thick / 2.0, colour(ink));
                    place.first().into_iter().for_each(|p| dot(paint, *p));
                    for pair in place.windows(2) {
                        let (a, b) = (pair[0], pair[1]);
                        let steps = ((b.x - a.x).hypot(b.y - a.y) / (thick / 3.0)).ceil().max(1.0) as usize;
                        (1..=steps).for_each(|step| {
                            let t = step as f32 / steps as f32;
                            dot(paint, PPoint::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t));
                        });
                    }
                }
                Mark::Word { ink, at: place, text } => paint.text(at(*place), text, colour(ink)),
            }
        }
    }
}
