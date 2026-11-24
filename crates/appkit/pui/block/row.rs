//! Rows and tiles: one thing of a list, said two ways. A row lies: what
//! leads it (a face, an icon, a picture), what it is called, and a line
//! about it. A tile stands: the same thing with its picture large over
//! its words, on a ground of its own. Any `(lead, title, note)` is both.

use super::super::core::light::surface;
use super::super::core::{PFit, PLight, PPaint, PPoint, PRect};
use super::{DIM, GAP, ICON, INK, LINE, PText, RADIUS};

/// The theme's token for how large what leads a row is; a theme without
/// it has it as large as an icon.
pub const LEAD: &str = "row.lead";

/// What leads a row, or stands over a tile's words.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PLead<'a> {
    #[default]
    None,
    /// An icon, by its name.
    Icon(&'a str),
    /// A face: a cell of a sheet, drawn round.
    Face(&'a str, u32, u32),
    /// A cell of a sheet, with the theme's corners.
    Cell(&'a str, u32, u32),
    /// A picture by its key, whole, with the theme's corners.
    Picture(&'a str),
}

impl PLead<'_> {
    /// Draw it in a square: an icon at its own size in the middle, on a
    /// soft ground of its own (an icon alone looks as if it were
    /// missing), a face round, a cell or a picture filling it.
    pub fn draw(&self, paint: &mut dyn PPaint, rect: PRect) {
        let (icon, radius, ink) = (paint.size(ICON), paint.size(RADIUS), paint.color(INK));
        match *self {
            PLead::None => {}
            PLead::Icon(name) => {
                super::super::core::light::label(paint, rect, radius, super::super::core::PLight::Soft, true, "");
                paint.icon(name, PRect::new(rect.center().x - icon / 2.0, rect.center().y - icon / 2.0, icon, icon), ink)
            }
            PLead::Face(sheet, column, index) => paint.picture(sheet, rect, rect.size.width / 2.0, PFit::Cell { column, index }),
            PLead::Cell(sheet, column, index) => paint.picture(sheet, rect, radius, PFit::Cell { column, index }),
            PLead::Picture(key) => paint.picture(key, rect, radius, PFit::Whole),
        }
    }
}

/// One thing of a list.
pub trait PRow {
    /// What leads it.
    fn lead(&self) -> PLead<'_> {
        PLead::None
    }

    /// What it is called.
    fn title(&self) -> &str;

    /// A line about it, under its title.
    fn note(&self) -> &str {
        ""
    }

    /// Draw it lying in a place: what leads it at the left, in the
    /// middle of the place's height; its title, and its note under it,
    /// each cut to fit. `end` of the place's width is left free at its
    /// end, for what can be done with it there.
    fn row(&self, paint: &mut dyn PPaint, rect: PRect, end: f32) {
        let (gap, line, ink, dim) = (paint.size(GAP), paint.size(LINE), paint.color(INK), paint.color(DIM));
        let side = paint.theme().size(LEAD).unwrap_or(paint.size(ICON)).min(rect.size.height);
        let lead = self.lead();
        lead.draw(paint, PRect::new(rect.origin.x, rect.center().y - side / 2.0, side, side));
        let beside = match lead {
            PLead::None => rect.origin.x,
            _ => rect.origin.x + side + gap / 2.0,
        };
        let room = (rect.origin.x + rect.size.width - end - beside - gap / 2.0).max(0.0);
        let (title, note) = (self.title().fit(room, paint), self.note().fit(room, paint));
        let up = match note.is_empty() {
            true => line / 2.0,
            false => line,
        };
        paint.text(PPoint::new(beside, rect.center().y - up), &title, ink);
        match note.is_empty() {
            true => {}
            false => paint.text(PPoint::new(beside, rect.center().y), &note, dim),
        }
    }

    /// Draw it standing in a place, as a tile: a ground in that light,
    /// what leads it a square across the top, its title and its note
    /// under that. Where the square stands, and where what follows the
    /// words may begin.
    fn tile(&self, paint: &mut dyn PPaint, rect: PRect, light: PLight) -> (PRect, PPoint) {
        let (gap, line, radius, ink, dim) = (paint.size(GAP), paint.size(LINE), paint.size(RADIUS), paint.color(INK), paint.color(DIM));
        surface(paint, rect, radius, light, "");
        let side = rect.size.width - gap;
        let frame = PRect::new(rect.origin.x + gap / 2.0, rect.origin.y + gap / 2.0, side, side);
        self.lead().draw(paint, frame);
        let (left, top) = (frame.origin.x, rect.origin.y + side + gap * 0.75);
        let (title, note) = (self.title().fit(side, paint), self.note().fit(side, paint));
        paint.text(PPoint::new(left, top), &title, ink);
        paint.text(PPoint::new(left, top + line), &note, dim);
        (frame, PPoint::new(left, top + line * 2.0 + gap / 4.0))
    }
}

/// What leads it, what it is called, and a line about it.
impl PRow for (PLead<'_>, &str, &str) {
    fn lead(&self) -> PLead<'_> {
        self.0
    }

    fn title(&self) -> &str {
        self.1
    }

    fn note(&self) -> &str {
        self.2
    }
}
