//! Crumbs: where something is, said as the way to it. Each step of the
//! way stands before the thing itself, dim, with the mark of what comes
//! next after it in the ink; a press on a step goes there. The thing itself is
//! drawn by who draws the crumbs, after them.

use super::super::core::{PPaint, PPoint, PRect};
use super::{DIM, GAP, INK, LINE};

/// The icon between a step and what comes after it.
pub const NEXT: &str = "chevron-right";

/// The steps of a way. Any list of texts is one.
pub trait PCrumb {
    /// The steps, from the first.
    fn step(&self) -> Vec<&str>;

    /// Draw them from a place, the top of their line: where each step
    /// stands, and how far across the thing itself begins.
    fn lead(&self, paint: &mut dyn PPaint, at: PPoint) -> (Vec<PRect>, f32) {
        let (gap, line, dim, ink) = (paint.size(GAP), paint.size(LINE), paint.color(DIM), paint.color(INK));
        let mut across = at.x;
        let rect = self.step().into_iter().map(|step| {
            let wide = paint.measure(step);
            let rect = PRect::new(across, at.y, wide, line);
            paint.text(PPoint::new(across, at.y), step, dim);
            // The mark between, in the ink: it says the way goes on.
            paint.icon(NEXT, PRect::new(across + wide + gap / 4.0, at.y + (line - gap) / 2.0, gap, gap), ink);
            across += wide + gap * 1.5;
            rect
        });
        let rect: Vec<PRect> = rect.collect();
        (rect, across)
    }
}

impl PCrumb for [&str] {
    fn step(&self) -> Vec<&str> {
        self.to_vec()
    }
}

impl PCrumb for [String] {
    fn step(&self) -> Vec<&str> {
        self.iter().map(String::as_str).collect()
    }
}
