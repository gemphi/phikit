//! A band: pills that belong together, on one ground. It lies as wide
//! as its pills, fits into a width, fills a width, or is a row of tabs
//! whose light moves from the tab that was chosen to the one that is.

use super::{GAP, HIGH, LINE, PPill, RADIUS};
use crate::pui::core::light::label;
use crate::pui::core::{PLight, PPaint, PPoint, PRect};
use crate::pui::layout::PDry;

/// Pills that belong together, on one ground.
pub trait PBand {
    /// Its pills, in order.
    fn pill(&self) -> Vec<&dyn PPill>;

    /// How high it is: a pill and half a gap.
    fn high(&self, paint: &dyn PPaint) -> f32 {
        (paint.size(LINE) + paint.size(GAP) / 2.0).max(paint.theme().size(HIGH).unwrap_or_default()) + paint.size(GAP) / 2.0
    }

    /// Its pills as lines no wider than that, by their place in it: as
    /// many on a line as fit at their own size.
    fn lined(&self, paint: &dyn PPaint, most: f32) -> Vec<Vec<usize>> {
        let gap = paint.size(GAP);
        let (mut lines, mut wide): (Vec<Vec<usize>>, f32) = (Vec::new(), gap / 2.0);
        for (index, pill) in self.pill().iter().enumerate() {
            let more = pill.size(paint).width + gap / 4.0;
            match lines.last_mut() {
                Some(line) if wide + more <= most => line.push(index),
                _ => {
                    lines.push(vec![index]);
                    wide = gap / 2.0;
                }
            }
            wide += more;
        }
        lines
    }

    /// How high it stands as tabs across a width: a line, or the lines
    /// its pills fold to (`draw_moving`).
    fn high_in(&self, paint: &dyn PPaint, wide: f32) -> f32 {
        let lines = self.lined(paint, wide).len().max(1) as f32;
        lines * self.high(paint) + (lines - 1.0) * paint.size(GAP) / 4.0
    }

    /// How wide it is, with the room between and round its pills.
    fn span(&self, paint: &dyn PPaint) -> f32 {
        let (gap, list) = (paint.size(GAP), self.pill());
        let all: f32 = list.iter().map(|pill| pill.size(paint).width).sum();
        all + list.len().saturating_sub(1) as f32 * gap / 4.0 + gap / 2.0
    }

    /// Draw it from a place in no more than that width: where its pills
    /// are too wide together, each gives up the same share of the room
    /// beside what it holds, down to a quarter of a gap. Where each
    /// stands. No pill, no ground.
    fn draw_in(&self, paint: &mut dyn PPaint, at: PPoint, most: f32) -> Vec<PRect> {
        let (gap, radius, list) = (paint.size(GAP), paint.size(RADIUS), self.pill());
        let count = list.len().max(1) as f32;
        let room = (gap - (self.span(paint) - most).max(0.0) / count).max(gap / 4.0);
        let wide = self.span(paint) - (gap - room) * count;
        let high = self.high(paint);
        match list.is_empty() {
            true => {}
            // Lit against the ground it stands on, a section's or a pane's,
            // so its edge shows and lines up with what is over it.
            false => label(paint, PRect::new(at.x, at.y, wide, high), radius, PLight::Rest, false, ""),
        }
        let mut left = at.x + gap / 4.0;
        let place = |pill: &&dyn PPill| {
            let rect = pill.lay(paint, PPoint::new(left, at.y + gap / 4.0), true, room);
            left += rect.size.width + gap / 4.0;
            rect
        };
        list.iter().map(place).collect()
    }

    /// Draw it across the whole of a width: each pill is given the same
    /// share of what is left beside what it holds, so the band fills it.
    /// Too narrow for that, it is drawn as `draw_in` draws it.
    fn draw_full(&self, paint: &mut dyn PPaint, at: PPoint, wide: f32, right: bool) -> Vec<PRect> {
        let (gap, radius, list) = (paint.size(GAP), paint.size(RADIUS), self.pill());
        let count = list.len().max(1) as f32;
        let held: f32 = list.iter().map(|pill| pill.sized(paint, 0.0).width).sum();
        let room = (wide - gap / 2.0 - (count - 1.0) * gap / 4.0 - held) / count;
        match (room >= gap, list.is_empty()) {
            (_, true) => Vec::new(),
            (false, false) => self.draw_in(paint, at, wide),
            // Lit against the ground it stands on (`right`), so its edge
            // shows and lines up with the heading over it.
            (true, false) => {
                let high = self.high(paint);
                label(paint, PRect::new(at.x, at.y, wide, high), radius, PLight::Rest, right, "");
                let mut left = at.x + gap / 4.0;
                let place = |pill: &&dyn PPill| {
                    let rect = pill.lay(paint, PPoint::new(left, at.y + gap / 4.0), true, room);
                    left += rect.size.width + gap / 4.0;
                    rect
                };
                list.iter().map(place).collect()
            }
        }
    }

    /// Draw it across a width as tabs whose light moves: every pill at
    /// rest on the band's ground, and one strong light `share` of the way
    /// from where the pill `from` stands to where the pill `to` does, as
    /// the chosen tab changes. Too many for the width, they fold to
    /// lines, each in the middle of the width. Where each pill stands.
    fn draw_moving(&self, paint: &mut dyn PPaint, at: PPoint, wide: f32, from: usize, to: usize, share: f32) -> Vec<PRect> {
        let (gap, radius, high) = (paint.size(GAP), paint.size(RADIUS), self.high(paint));
        let off: Vec<Off> = self.pill().into_iter().map(Off).collect();
        let lines = self.lined(paint, wide);
        let rect = match lines.len() {
            0 | 1 => off.draw_full(&mut PDry(&*paint), at, wide, true),
            _ => {
                let mut rect = Vec::new();
                for (row, line) in lines.iter().enumerate() {
                    let size: Vec<_> = line.iter().map(|index| off[*index].size(paint)).collect();
                    let span = size.iter().map(|size| size.width).sum::<f32>() + (size.len() as f32 - 1.0) * gap / 4.0;
                    let (mut left, top) = (at.x + (wide - span) / 2.0, at.y + row as f32 * (high + gap / 4.0) + gap / 4.0);
                    for size in size {
                        rect.push(PRect::new(left, top, size.width, size.height));
                        left += size.width + gap / 4.0;
                    }
                }
                rect
            }
        };
        let (Some(first), Some(last)) = (rect.first(), rect.last()) else { return rect };
        // Tabs stand over the ground that holds what they choose, lit from
        // the left: theirs runs from the right.
        let ground = match lines.len() {
            0 | 1 => PRect::new(at.x, at.y, last.origin.x + last.size.width + gap / 4.0 - at.x, high),
            _ => PRect::new(at.x, at.y, wide, self.high_in(paint, wide)),
        };
        label(paint, ground, radius, PLight::Rest, true, "");
        let (a, b) = (rect.get(from).unwrap_or(first), rect.get(to).unwrap_or(first));
        let t = share.clamp(0.0, 1.0);
        let t = t * t * (3.0 - 2.0 * t);
        // Across, and down where the tabs fold to lines.
        let lit = PRect::new(a.origin.x + (b.origin.x - a.origin.x) * t, a.origin.y + (b.origin.y - a.origin.y) * t, a.size.width + (b.size.width - a.size.width) * t, a.size.height);
        label(paint, lit, radius, PLight::Strong, false, "");
        let held: Vec<f32> = off.iter().map(|pill| pill.sized(paint, 0.0).width).collect();
        off.iter().zip(&rect).zip(held).map(|((pill, r), held)| pill.lay(paint, r.origin, true, r.size.width - held)).collect()
    }

    /// Draw it from a place; where each pill stands.
    fn draw(&self, paint: &mut dyn PPaint, at: PPoint) -> Vec<PRect> {
        self.draw_in(paint, at, f32::MAX)
    }

    /// Draw it from a place in lines no wider than that: as many pills
    /// on a line as fit, each line a band of its own under the last.
    /// Where each pill stands, and how high the lines are together.
    fn draw_lines(&self, paint: &mut dyn PPaint, at: PPoint, most: f32) -> (Vec<PRect>, f32) {
        let (gap, high) = (paint.size(GAP), self.high(paint));
        let mut lines: Vec<Lined> = Vec::new();
        let mut wide = gap / 2.0;
        for pill in self.pill() {
            let more = pill.size(paint).width + gap / 4.0;
            match lines.last_mut() {
                Some(line) if wide + more <= most || line.0.is_empty() => line.0.push(pill),
                _ => {
                    lines.push(Lined(vec![pill]));
                    wide = gap / 2.0;
                }
            }
            wide += more;
        }
        let mut rect = Vec::new();
        for (index, line) in lines.iter().enumerate() {
            rect.extend(line.draw_in(paint, PPoint::new(at.x, at.y + index as f32 * (high + gap / 4.0)), most));
        }
        let lines = lines.len() as f32;
        (rect, lines * high + (lines - 1.0).max(0.0) * gap / 4.0)
    }

    /// Draw it so that it ends at a place.
    fn draw_end(&self, paint: &mut dyn PPaint, end: PPoint) -> Vec<PRect> {
        let wide = self.span(paint);
        self.draw(paint, PPoint::new(end.x - wide, end.y))
    }
}

impl<T: PPill> PBand for [T] {
    fn pill(&self) -> Vec<&dyn PPill> {
        self.iter().map(|pill| pill as &dyn PPill).collect()
    }
}

/// A line of a band's pills, a band of its own (`draw_lines`).
struct Lined<'a>(Vec<&'a dyn PPill>);

impl PBand for Lined<'_> {
    fn pill(&self) -> Vec<&dyn PPill> {
        self.0.clone()
    }
}

/// A pill drawn as at rest, whether it is on or not: a tab whose light
/// is laid apart from it (`draw_moving`).
struct Off<'a>(&'a dyn PPill);

impl PPill for Off<'_> {
    fn icon(&self) -> &str {
        self.0.icon()
    }

    fn label(&self) -> &str {
        self.0.label()
    }
}

/// The theme's token for how much of its way a tab's light moves in a
/// frame. A theme without it has the light stand where it is at once.
pub const STEP: &str = "tab.step";

/// Where a row of tabs' light is: on its way from the tab that was
/// chosen to the one that is, and how far.
#[derive(Debug)]
pub struct PTabMove {
    from: std::cell::Cell<usize>,
    to: std::cell::Cell<usize>,
    share: std::cell::Cell<f32>,
}

/// A light at rest on the first tab: it has arrived, so a form's tabs
/// ask for no frame until another is chosen.
impl Default for PTabMove {
    fn default() -> PTabMove {
        PTabMove { from: std::cell::Cell::new(0), to: std::cell::Cell::new(0), share: std::cell::Cell::new(1.0) }
    }
}

impl PTabMove {
    /// Another tab is chosen: the light sets out for it from where it is.
    pub fn to(&self, tab: usize) {
        match tab == self.to.get() {
            true => {}
            false => {
                self.from.set(self.to.replace(tab));
                self.share.set(0.0);
            }
        }
    }

    /// A frame passes: `true` while the light is still on its way.
    pub fn tick(&self, theme: &dyn crate::theme::PTheme) -> bool {
        let step = theme.size(STEP).unwrap_or(1.0).max(f32::EPSILON);
        let was = self.share.get();
        self.share.set((was + step).min(1.0));
        was < 1.0
    }

    /// From which tab, to which, and how far.
    pub fn now(&self) -> (usize, usize, f32) {
        let share = match self.from.get() == self.to.get() {
            true => 1.0,
            false => self.share.get(),
        };
        (self.from.get(), self.to.get(), share)
    }
}
