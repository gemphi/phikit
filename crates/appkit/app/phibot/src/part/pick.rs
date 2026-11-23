//! The customiser (RobotCustomizationView, SelectorViews): its title with
//! Reset and Randomize, the part chosen (Head, Body, Backpack), then for
//! that part its shape, its eyes (the head's), its material, the
//! material's colour, and the lights' colour. Every name is phiace's
//! tables; every control is the kit's pill, a colour a swatch. Each group
//! stands on a ground of its own (the kit's section, its room the
//! theme's), across the whole width, and the room left in the pane is
//! shared out between the groups so they fill it.

use std::cell::RefCell;

use appkit::pui::block::PBand;
use appkit::pui::core::{PColor, PDraw, PEvent, PLight, PPaint, PPoint, PRect, PView, light};
use appkit::pui::layout::{PDry, section};
use phiace::bot::botanist::Data;
use phiace::bot::botanist::look::Look;

use super::{Desk, GAP, INK, LINE, byte};

/// The theme's tokens: a swatch's side, the ring round the chosen one,
/// the room a section has round what stands on it, and the least room
/// between two sections.
pub const SWATCH: &str = "bot.swatch";
pub const RING: &str = "bot.ring";
pub const PAD: &str = "section.side";
pub const SPACE: &str = "section.space";

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

/// What a group holds: pills (an icon, words, whether on, what a press
/// does), or swatches.
enum Hold {
    Pill(Vec<(String, String, bool, Act)>),
    Swatch(Vec<(PColor, bool, Act)>),
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

    /// The groups the chosen part has: a heading's key and what it holds.
    fn groups(&self, data: &Data, look: &Look) -> Vec<(&'static str, Hold)> {
        let chosen = self.desk.chosen.get();
        let part = data.part.row.iter().enumerate().map(|(i, row)| (String::new(), row.get(3).cloned().unwrap_or_default(), i == chosen, Act::Part(i))).collect();
        let name = data.part.row.get(chosen).and_then(|row| row.first()).cloned().unwrap_or_default();
        let count = data.part.rows(&name).next().and_then(|row| row.get(2)).and_then(|n| n.parse::<usize>().ok()).unwrap_or_default();
        let choice = look.part.iter().find(|(n, _)| *n == name).map(|(_, c)| c.clone()).unwrap_or_default();
        let shape = (1..=count).map(|n| format!("{name}{n}")).map(|s| (String::new(), s.clone(), s == choice.shape, Act::Shape(s))).collect();
        let face = data.face.row.iter().enumerate().map(|(i, row)| (row.get(1).cloned().unwrap_or_default(), String::new(), i == look.face, Act::Face(i))).collect();
        let material = materials(data).into_iter().map(|m| (String::new(), m.clone(), m == choice.material, Act::Material(m))).collect();
        let colour = data.material.rows(&choice.material).enumerate().map(|(i, row)| (byte([3, 4, 5].map(|c| row.get(c).and_then(|v| v.parse().ok()).unwrap_or_default())), i == choice.colour, Act::Colour(i))).collect();
        let lights = (0..data.light.row.len()).map(|i| (super::light(data, i), i == look.light, Act::Light(i))).collect();
        let eyes = match name.as_str() {
            "head" => vec![("face", Hold::Pill(face))],
            _ => Vec::new(),
        };
        let rest = vec![("material", Hold::Pill(material)), ("colour", Hold::Swatch(colour)), ("light", Hold::Swatch(lights))];
        [vec![("", Hold::Pill(part)), ("shape", Hold::Pill(shape))], eyes, rest].into_iter().flatten().collect()
    }

    /// One group on its ground, its heading over what it holds, from a
    /// place across a width; where it ends.
    fn group(&self, paint: &mut dyn PPaint, at: PPoint, wide: f32, key: &str, hold: &Hold) -> f32 {
        let (gap, line, ink) = (paint.size(GAP), paint.size(LINE), paint.color(INK));
        let mut top = at.y;
        if !key.is_empty() {
            paint.text(PPoint::new(at.x, top), self.desk.text.get(key), ink);
            top += line + gap / 2.0;
        }
        match hold {
            Hold::Pill(list) => {
                let pill: Vec<(&str, &str, bool)> = list.iter().map(|(icon, word, on, _)| (icon.as_str(), word.as_str(), *on)).collect();
                let rect = pill.draw_full(paint, PPoint::new(at.x, top), wide);
                self.spot.borrow_mut().extend(rect.into_iter().zip(list.iter().map(|(.., act)| act.clone())));
                top + pill.high(paint)
            }
            Hold::Swatch(list) => self.swatches(paint, PPoint::new(at.x, top), wide, list),
        }
    }

    /// A row of round swatches across a width, each lit as a surface is
    /// and ringed in soft light, the chosen one in strong light; where
    /// the rows end.
    fn swatches(&self, paint: &mut dyn PPaint, at: PPoint, wide: f32, list: &[(PColor, bool, Act)]) -> f32 {
        let (gap, side, ring) = (paint.size(GAP), paint.size(SWATCH), paint.size(RING));
        // A little smaller, down to three quarters, where that keeps them
        // all on one row.
        let count = list.len().max(1) as f32;
        let fit = (wide + ring) / count - ring * 3.0;
        let side = match fit >= side * 0.75 {
            true => side.min(fit),
            false => side,
        };
        let cell = side + ring * 2.0;
        // As many on a row as fit with a ring's room between them.
        let across = ((wide + ring) / (cell + ring)).floor().clamp(1.0, list.len().max(1) as f32) as usize;
        let step = match across {
            1 => 0.0,
            n => (wide - cell) / (n - 1) as f32,
        };
        let rows = list.len().div_ceil(across);
        for (index, (colour, on, act)) in list.iter().enumerate() {
            let (x, y) = ((index % across) as f32, (index / across) as f32);
            let rect = PRect::new(at.x + ring + x * step, at.y + ring + y * (cell + gap / 4.0), side, side);
            let strength = match on {
                true => PLight::Strong,
                false => PLight::Soft,
            };
            light::label(paint, rect.inset(-ring), side / 2.0 + ring, strength, false, "");
            paint.round(rect, side / 2.0, *colour);
            light::surface(paint, rect, side / 2.0, PLight::Rest, "");
            self.spot.borrow_mut().push((rect, act.clone()));
        }
        at.y + rows as f32 * (cell + gap / 4.0) - gap / 4.0
    }

    /// Every group from a place, with so much room between them; where
    /// the last ends.
    fn groups_from(&self, paint: &mut dyn PPaint, at: PPoint, wide: f32, space: f32, group: &[(&str, Hold)]) -> f32 {
        let pad = paint.theme().size(PAD).unwrap_or_default();
        let (left, inner) = (at.x + pad, wide - pad * 2.0);
        let mut top = at.y + pad;
        for (key, hold) in group {
            let end = section(paint, PPoint::new(left, top), inner, &mut |paint| self.group(paint, PPoint::new(left, top), inner, key, hold));
            top = end + pad * 2.0 + space;
        }
        top - space - pad
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
        let top = origin.y + gap;
        paint.head(PPoint::new(left, top), text.get("title"), ink);
        let tool = [("undo", text.get("reset"), false), ("rotate", text.get("random"), false)];
        let icon: Vec<(&str, &str, bool)> = tool.iter().map(|(icon, _, on)| (*icon, "", *on)).collect();
        let rect = icon.draw_end(paint, PPoint::new(left + wide, top));
        self.spot.borrow_mut().extend(rect.into_iter().zip([Act::Reset, Act::Random]));
        let top = top + icon.high(paint) + gap / 2.0;
        // The groups, the room left in the pane shared out between them.
        let group = self.groups(&data, &look);
        let least = paint.theme().size(SPACE).unwrap_or(gap / 2.0);
        // The grounds stand in the title's column, what is on them inside.
        let end = self.groups_from(&mut PDry(&*paint), PPoint::new(left, top), wide, least, &group);
        let left_over = (origin.y + self.frame.size.height - gap - end).max(0.0);
        let space = least + left_over / group.len().saturating_sub(1).max(1) as f32;
        self.spot.borrow_mut().truncate(2);
        self.groups_from(paint, PPoint::new(left, top), wide, space, &group);
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
