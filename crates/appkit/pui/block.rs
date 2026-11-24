//! Blocks: the shared parts a page is made of. Each is a trait, and all
//! of how it looks is the trait's own default: an app has data, says
//! which block the data is, and draws nothing itself. A rule about
//! looks is one change, here and in the theme.
//!
//! * `PPill`: something to press: an icon, words, or both, and whether
//!   it is on. `(icon, words, on)` is one.
//! * `PBand`: pills that belong together, on one ground: a title's
//!   tools, tabs, what can be done with a person. Any list of pills is
//!   one.
//! * `PFacts`: a table: a row each, what it is at the left, its value
//!   ending at the right. Any list of two texts is one.
//! * `PFace`: somebody's face on its ground: `(cell of a sheet, name)`.
//! * `PCard`: a face large on a ground with a name and a line about
//!   them: `(cell of a sheet, name, line)`. `PDeck`: cards that belong
//!   together, in rows that fold; any list of cards is one.
//! * `PRow`: one thing of a list: what leads it, its title, a line
//!   about it; lying as a row or standing as a tile with its picture
//!   large. `(lead, title, note)` is one.
//! * `PSay`: what somebody said: their face, who and when, the words,
//!   the pictures posted with it, what it was marked with.
//! * `PText`: a text cut to fit a width.
//! * `PCrumb`: the way to something, each step before it with the mark
//!   of what comes next. Any list of texts is one.
//! * `PTip`: what something is called, said by it under the pointer,
//!   on a ground that floats. Any text is one.
//!
//! A block is drawn in place and says where it stands, so the page that
//! holds it knows where a press does what. A page puts whole sections
//! on grounds with `layout::section`.

mod band;
mod card;
mod swatch;
mod crumb;
mod face;
mod row;
mod say;
mod tip;

pub use band::{PBand, PTabMove};
pub use card::{CARD, FACE, PCard, PDeck};
pub use crumb::PCrumb;
pub use face::PFace;
pub use row::{LEAD, PLead, PRow};
pub use say::{PSaid, PSay};
pub use swatch::PSwatch;
pub use tip::PTip;

use super::core::light::label;
use super::core::{PLight, PPaint, PPoint, PRect, PSize};

/// The theme's tokens every block is drawn with.
pub const GAP: &str = "gap";
pub const LINE: &str = "text.line";
pub const INK: &str = "text.ink";
pub const DIM: &str = "text.dim";
pub const ICON: &str = "icon.size";
pub const RADIUS: &str = "row.radius";
/// How high a pill is at least: a theme without it has pills a line and
/// half a gap high.
pub const HIGH: &str = "pill.high";
/// The line under a table's row, and how thick it is.
pub const RULE: &str = "line.ink";
pub const WIDTH: &str = "line.width";
/// The word that ends a text cut short.
pub const CUT: &str = "text.cut";

/// A text that can be cut to fit.
pub trait PText {
    fn text(&self) -> &str;

    /// The text when it is no wider than `width`; else as much of its
    /// start as fits with the theme's mark for a text cut short.
    fn fit(&self, width: f32, paint: &dyn PPaint) -> String {
        let mark = paint.theme().word(CUT).unwrap_or_default().to_string();
        let whole = self.text();
        let fits = |end: usize| paint.measure(&format!("{}{mark}", &whole[..end])) <= width;
        // Where a letter ends, in order; the longest start that fits is
        // found by halving, so a long text is measured a few times, not
        // once a letter.
        let end: Vec<usize> = whole.char_indices().map(|(at, letter)| at + letter.len_utf8()).collect();
        let cut = || end[..end.partition_point(|end| fits(*end))].last().copied().unwrap_or(0);
        match paint.measure(whole) <= width {
            true => whole.to_string(),
            false => format!("{}{mark}", whole[..cut()].trim_end()),
        }
    }
}

impl PText for str {
    fn text(&self) -> &str {
        self
    }
}

/// Something to press.
pub trait PPill {
    /// Its icon; nothing for one with words alone.
    fn icon(&self) -> &str {
        ""
    }

    /// Its words; nothing for one with an icon alone.
    fn label(&self) -> &str {
        ""
    }

    /// Whether it is on.
    fn on(&self) -> bool {
        false
    }

    /// How large it is with that much room beside what it holds: as
    /// high as a line and half a gap, or the theme's pill, if higher.
    fn sized(&self, paint: &dyn PPaint, room: f32) -> PSize {
        let (gap, line, icon) = (paint.size(GAP), paint.size(LINE), paint.size(ICON));
        let (mark, between) = match (self.icon().is_empty(), self.label().is_empty()) {
            (true, _) => (0.0, 0.0),
            (false, true) => (icon, 0.0),
            (false, false) => (icon, gap / 4.0),
        };
        PSize { width: mark + between + paint.measure(self.label()) + room, height: (line + gap / 2.0).max(paint.theme().size(HIGH).unwrap_or_default()) }
    }

    /// How large it is: what it holds, and a gap.
    fn size(&self, paint: &dyn PPaint) -> PSize {
        self.sized(paint, paint.size(GAP))
    }

    /// Draw it at a place with that much room beside what it holds;
    /// where it stands. On, it is lit at full strength. At rest it has
    /// a soft ground of its own, unless it is `held`: then it stands on
    /// the ground of the band that holds it.
    fn lay(&self, paint: &mut dyn PPaint, at: PPoint, held: bool, room: f32) -> PRect {
        let (gap, line, icon, radius, ink) = (paint.size(GAP), paint.size(LINE), paint.size(ICON), paint.size(RADIUS), paint.color(INK));
        let rect = PRect { origin: at, size: self.sized(paint, room) };
        match (self.on(), held) {
            (true, _) => label(paint, rect, radius, PLight::Strong, false, ""),
            (false, false) => label(paint, rect, radius, PLight::Soft, false, ""),
            (false, true) => {}
        }
        let left = at.x + room / 2.0;
        let words = match self.icon().is_empty() {
            true => left,
            false => {
                paint.icon(self.icon(), PRect::new(left, rect.center().y - icon / 2.0, icon, icon), ink);
                left + icon + gap / 4.0
            }
        };
        match self.label().is_empty() {
            true => {}
            false => paint.text(PPoint::new(words, rect.center().y - line / 2.0), self.label(), ink),
        }
        // A pill is pressed for something: the pointer is a hand over it.
        paint.press(rect);
        rect
    }

    /// Draw it at a place; where it stands.
    fn draw(&self, paint: &mut dyn PPaint, at: PPoint, held: bool) -> PRect {
        let room = paint.size(GAP);
        self.lay(paint, at, held, room)
    }
}

/// An icon, words, and whether it is on.
impl PPill for (&str, &str, bool) {
    fn icon(&self) -> &str {
        self.0
    }

    fn label(&self) -> &str {
        self.1
    }

    fn on(&self) -> bool {
        self.2
    }
}

/// A table: rows that name a thing and give its value.
pub trait PFacts {
    /// Its rows: what each is, and its value.
    fn row(&self) -> Vec<(&str, &str)>;

    /// Draw it from a place: a row each, what it is at the left, its
    /// value ending at the right, a line under it. How far down it
    /// ends.
    fn table(&self, paint: &mut dyn PPaint, at: PPoint, wide: f32) -> f32 {
        let (gap, line, rule) = (paint.size(GAP), paint.size(LINE), paint.size(WIDTH));
        let (ink, dim, ruled) = (paint.color(INK), paint.color(DIM), paint.color(RULE));
        let mut top = at.y;
        for (name, value) in self.row() {
            paint.text(PPoint::new(at.x, top + gap / 4.0), name, dim);
            let value = value.fit((wide - paint.measure(name) - gap).max(0.0), paint);
            paint.text(PPoint::new(at.x + wide - paint.measure(&value), top + gap / 4.0), &value, ink);
            top += line + gap / 2.0;
            paint.fill(PRect::new(at.x, top - rule, wide, rule), ruled);
        }
        top
    }
}

impl PFacts for [(String, String)] {
    fn row(&self) -> Vec<(&str, &str)> {
        self.iter().map(|(name, value)| (name.as_str(), value.as_str())).collect()
    }
}
