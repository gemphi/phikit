//! The settings: what a person can choose about Phigen. Its look, a pill
//! for each one `data/look.tsv` has; and under that the model that
//! answers, the roster. What is chosen is kept (`choice.rs`), so Phigen
//! opens the same way the next time.

use std::cell::RefCell;

use appkit::pui::core::{PDraw, PEvent, PLight, PPaint, PPoint, PRect, PView, light, send, show};

use super::{DIM, Desk, GAP, INK, LINE, Roster};

/// The theme's token for how round a pill is. A pill wears the label's
/// light: the one chosen its strongest, the others its faintest.
pub const ROUND: &str = "tab.radius";
/// The word the window's own profile tells when a look was chosen there,
/// before the look's name: the choice is kept as one made here is.
pub const TOLD: &str = "look ";

/// The settings window.
pub struct Setting {
    frame: PRect,
    desk: Desk,
    roster: Roster,
    /// Where each look's pill was drawn last, in the window.
    spot: RefCell<Vec<(PRect, String)>>,
}

impl Setting {
    pub fn new(desk: Desk) -> Setting {
        Setting { frame: PRect::default(), roster: Roster::new(desk.clone(), false), desk, spot: RefCell::default() }
    }

    /// How far down the pills are, and how high: under their heading.
    fn pill(&self) -> (f32, f32) {
        let (gap, line) = (self.desk.size(GAP), self.desk.size(LINE));
        (line + gap / 2.0, line + gap)
    }
}

impl PDraw for Setting {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, line, round) = (paint.size(GAP), paint.size(LINE), paint.size(ROUND));
        let (ink, dim) = (paint.color(INK), paint.color(DIM));
        let (top, high) = self.pill();
        // The look: a heading, and a pill for each look there is.
        paint.text(origin.plus(PPoint::new(gap, 0.0)), self.desk.text.get("setting.look"), dim);
        let (mut next, mut spot) = (origin.x + gap, Vec::new());
        for name in self.desk.shade.list() {
            let label = self.desk.text.get(&format!("look.{name}"));
            let rect = PRect::new(next, origin.y + top, paint.measure(label) + gap * 2.0, high);
            let strength = match name == self.desk.shade.name() {
                true => PLight::Strong,
                false => PLight::Soft,
            };
            light::label(paint, rect, round, strength, false, "");
            paint.text(PPoint::new(rect.origin.x + gap, rect.center().y - line / 2.0), label, ink);
            spot.push((rect, name.to_string()));
            next += rect.size.width + gap / 2.0;
        }
        *self.spot.borrow_mut() = spot;
        // The model: a heading, and under it the roster.
        paint.text(origin.plus(PPoint::new(gap, top + high + gap)), self.desk.text.get("model.label"), dim);
        show(&self.roster, paint, origin);
    }
}

impl PView for Setting {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        let (gap, line, (top, high)) = (self.desk.size(GAP), self.desk.size(LINE), self.pill());
        self.frame = frame;
        // The roster has what is left under the model's heading.
        let under = top + high + gap + line + gap / 2.0;
        self.roster.set_frame(PRect::new(0.0, under, frame.size.width, (frame.size.height - under).max(0.0)));
    }

    fn tick(&mut self) -> bool {
        self.roster.tick()
    }

    /// A look chosen in the profile is kept as one chosen here.
    fn tell(&mut self, word: &str) -> bool {
        word.strip_prefix(TOLD).is_some_and(|look| self.desk.dress(look))
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        // What was drawn is in the window's coordinates.
        let look = event.at().and_then(|at| {
            let place = at.plus(self.frame.origin);
            self.spot.borrow().iter().find(|(rect, _)| rect.contains(place)).map(|(_, name)| name.clone())
        });
        match (event, look) {
            (PEvent::PointerDown(_), Some(name)) => self.desk.dress(&name),
            _ => send(&mut self.roster, event),
        }
    }
}
