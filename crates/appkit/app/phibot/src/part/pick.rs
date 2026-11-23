//! The customiser (RobotCustomizationView, SelectorViews): its title with
//! Reset and Randomize, the part chosen (Head, Body, Backpack), then for
//! that part its shape, its eyes (the head's), its material, the
//! material's colour, and the lights' colour. Every name is phiace's
//! tables; every control is the kit's pill, a colour a swatch.

use std::cell::RefCell;

use appkit::pui::block::PBand;
use appkit::pui::core::{PDraw, PEvent, PLight, PPaint, PPoint, PRect, PView, light};
use phiace::bot::botanist::Data;
use phiace::bot::botanist::look::Look;

use super::{DIM, Desk, GAP, INK, LINE, byte};

/// The theme's tokens: a swatch's side, the ring round the chosen one.
pub const SWATCH: &str = "bot.swatch";
pub const RING: &str = "bot.ring";

/// What a press on the customiser does.
#[derive(Debug, Clone, PartialEq)]
enum Act {
    Reset,
    Random,
    Part(usize),
    Shape(String),
    Face(usize),
    Material(String),
    Colour(usize),
    Light(usize),
}

pub struct Pick {
    frame: PRect,
    desk: Desk,
    spot: RefCell<Vec<(PRect, Act)>>,
}

impl Pick {
    pub fn new(desk: Desk) -> Pick {
        Pick { frame: PRect::default(), desk, spot: RefCell::default() }
    }

    /// A heading over a row of choices; where the row begins.
    fn heading(&self, paint: &mut dyn PPaint, at: PPoint, key: &str) -> f32 {
        paint.text(at, self.desk.text.get(key), paint.color(DIM));
        at.y + paint.size(LINE) + paint.size(GAP) / 4.0
    }

    /// A row of pills, each a choice; where the next row begins.
    fn pills(&self, paint: &mut dyn PPaint, at: PPoint, wide: f32, list: Vec<(&str, &str, bool, Act)>) -> f32 {
        let pill: Vec<(&str, &str, bool)> = list.iter().map(|(icon, word, on, _)| (*icon, *word, *on)).collect();
        let rect = pill.draw_in(paint, at, wide);
        let high = pill.high(paint);
        self.spot.borrow_mut().extend(rect.into_iter().zip(list.into_iter().map(|(.., act)| act)));
        at.y + high + paint.size(GAP) / 2.0
    }

    /// A row of round swatches, the chosen one ringed; where the next
    /// row begins.
    fn swatches(&self, paint: &mut dyn PPaint, at: PPoint, wide: f32, list: Vec<(appkit::pui::core::PColor, bool, Act)>) -> f32 {
        let (gap, side, ring) = (paint.size(GAP), paint.size(SWATCH), paint.size(RING));
        let across = ((wide + gap / 2.0) / (side + gap / 2.0)).floor().max(1.0) as usize;
        let rows = list.len().div_ceil(across);
        for (index, (colour, on, act)) in list.into_iter().enumerate() {
            let (x, y) = ((index % across) as f32, (index / across) as f32);
            let rect = PRect::new(at.x + ring + x * (side + gap / 2.0), at.y + ring + y * (side + gap / 2.0), side, side);
            on.then(|| light::label(paint, rect.inset(-ring), side / 2.0 + ring, PLight::Strong, false, ""));
            paint.round(rect, side / 2.0, colour);
            self.spot.borrow_mut().push((rect, act));
        }
        at.y + rows as f32 * (side + gap / 2.0) + ring * 2.0 + gap / 2.0
    }
}

impl PDraw for Pick {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        self.spot.borrow_mut().clear();
        let (gap, ink) = (paint.size(GAP), paint.color(INK));
        let data = self.desk.data();
        let look = self.desk.bot.borrow().look();
        let (left, wide) = (origin.x + gap, self.frame.size.width - gap * 2.0);
        let text = &self.desk.text;
        // The title, then Reset and Randomize at its end.
        let mut top = origin.y + gap;
        paint.head(PPoint::new(left, top), text.get("title"), ink);
        let tool = [("undo", text.get("reset"), false), ("rotate", text.get("random"), false)];
        let icon: Vec<(&str, &str, bool)> = tool.iter().map(|(icon, _, on)| (*icon, "", *on)).collect();
        let rect = icon.draw_end(paint, PPoint::new(left + wide, top));
        self.spot.borrow_mut().extend(rect.into_iter().zip([Act::Reset, Act::Random]));
        top += icon.high(paint) + gap / 2.0;
        // Which part: the kit's band, one pill a part.
        let chosen = self.desk.chosen.get();
        let part: Vec<(&str, &str, bool, Act)> = data.part.row.iter().enumerate().map(|(i, row)| ("", row.get(3).map_or("", String::as_str), i == chosen, Act::Part(i))).collect();
        top = self.pills(paint, PPoint::new(left, top), wide, part);
        let name = data.part.row.get(chosen).and_then(|row| row.first()).cloned().unwrap_or_default();
        let count = data.part.rows(&name).next().and_then(|row| row.get(2)).and_then(|n| n.parse::<usize>().ok()).unwrap_or_default();
        let choice = look.part.iter().find(|(n, _)| *n == name).map(|(_, c)| c.clone()).unwrap_or_default();
        // Its shape.
        top = self.heading(paint, PPoint::new(left, top), "shape");
        let shape: Vec<String> = (1..=count).map(|n| format!("{name}{n}")).collect();
        let list = shape.iter().map(|s| ("", s.as_str(), *s == choice.shape, Act::Shape(s.clone()))).collect();
        top = self.pills(paint, PPoint::new(left, top), wide, list);
        // The head's eyes.
        if name == "head" {
            top = self.heading(paint, PPoint::new(left, top), "face");
            let list = data.face.row.iter().enumerate().map(|(i, row)| (row.get(1).map_or("", String::as_str), "", i == look.face, Act::Face(i))).collect();
            top = self.pills(paint, PPoint::new(left, top), wide, list);
        }
        // Its material, and the material's colours.
        top = self.heading(paint, PPoint::new(left, top), "material");
        let material = materials(&data);
        let list = material.iter().map(|m| ("", m.as_str(), *m == choice.material, Act::Material(m.clone()))).collect();
        top = self.pills(paint, PPoint::new(left, top), wide, list);
        top = self.heading(paint, PPoint::new(left, top), "colour");
        let colour = data.material.rows(&choice.material).enumerate().map(|(i, row)| (byte([3, 4, 5].map(|c| row.get(c).and_then(|v| v.parse().ok()).unwrap_or_default())), i == choice.colour, Act::Colour(i))).collect();
        top = self.swatches(paint, PPoint::new(left, top), wide, colour);
        // The lights' colour.
        top = self.heading(paint, PPoint::new(left, top), "light");
        let lights = (0..data.light.row.len()).map(|i| (super::light(&data, i), i == look.light, Act::Light(i))).collect();
        self.swatches(paint, PPoint::new(left, top), wide, lights);
    }
}

/// The materials, in the order the material table first names each.
fn materials(data: &Data) -> Vec<String> {
    let mut out: Vec<String> = data.material.keys().into_iter().map(str::to_string).collect();
    out.dedup();
    out
}

/// Change the chosen part's look, or the robot's.
fn apply(desk: &Desk, act: Act) {
    let part = desk.data().part.row.get(desk.chosen.get()).and_then(|row| row.first()).cloned().unwrap_or_default();
    match act {
        Act::Part(i) => desk.chosen.set(i),
        Act::Reset => desk.bot.borrow_mut().reset(),
        Act::Random => desk.bot.borrow_mut().change(random),
        other => desk.bot.borrow_mut().change(|look, _| {
            let choice = look.choice(&part);
            match (other, choice) {
                (Act::Shape(s), Some(c)) => c.shape = s,
                (Act::Material(m), Some(c)) => c.material = m,
                (Act::Colour(i), Some(c)) => c.colour = i,
                (Act::Face(i), _) => look.face = i,
                (Act::Light(i), _) => look.light = i,
                _ => {}
            }
        }),
    }
}

/// Every choice at random (the sample's Randomize), from the clock.
fn random(look: &mut Look, data: &Data) {
    let mut seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(7, |d| d.subsec_nanos() as u64 | 1);
    let mut roll = |n: usize| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % n.max(1) as u64) as usize
    };
    let material = materials(data);
    for (name, choice) in look.part.iter_mut() {
        let count = data.part.rows(name).next().and_then(|r| r.get(2)).and_then(|n| n.parse::<usize>().ok()).unwrap_or(1);
        choice.shape = format!("{name}{}", roll(count) + 1);
        choice.material = material.get(roll(material.len())).cloned().unwrap_or_default();
        choice.colour = roll(4);
    }
    (look.face, look.light) = (roll(data.face.row.len()), roll(data.light.row.len()));
}

impl PView for Pick {
    fn hand(&self) -> Vec<PRect> {
        self.spot.borrow().iter().map(|(rect, _)| *rect).collect()
    }

    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        let PEvent::PointerDown(at) = *event else { return false };
        let place = at.plus(self.frame.origin);
        let act = self.spot.borrow().iter().find(|(rect, _)| rect.contains(place)).map(|(_, act)| act.clone());
        act.is_some_and(|act| {
            apply(&self.desk, act);
            true
        })
    }
}
