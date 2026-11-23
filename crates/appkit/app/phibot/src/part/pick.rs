//! The customiser (RobotCustomizationView, SelectorViews), in the bar's
//! second column as Phiogy's rooms are: Reset and Randomize, then the part chosen (Head, Body, Backpack) as
//! tabs over what they choose: for that part its shape, its eyes (the
//! head's), its material and the material's colour, and the lights'
//! colour. Every name is phiace's tables; how it is laid out is the
//! kit's, the same for every app, in levels: the robot (its title, Reset
//! and Randomize); under it the parts as tabs over the ground of the
//! chosen part's groups (`tabbed`), and the robot's own lights on a
//! ground beside that one, for they are every part's (`stack`); in each,
//! a group under its heading, options in bands across the width, colours
//! as swatches.

use std::cell::RefCell;

use appkit::pui::block::{PBand, PSwatch, PTabMove};
use appkit::pui::core::{PColor, PDraw, PEvent, PPaint, PPoint, PRect, PView};
use appkit::pui::layout::{PDry, PGrouped, stack, tabbed};
use appkit::theme::PTheme;
use phiace::bot::botanist::Data;
use phiace::bot::botanist::look::Look;

use super::{Desk, GAP, byte};

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

/// What a group holds: options (an icon, words, whether on), or colours
/// (whether chosen); and what a press on each does.
enum Hold {
    Pill(Vec<(String, String, bool)>),
    Swatch(Vec<(PColor, bool)>),
}

pub struct Pick {
    frame: PRect,
    desk: Desk,
    spot: RefCell<Vec<(PRect, Act)>>,
    /// The light on the chosen part's tab, on its way to it.
    motion: PTabMove,
}

impl Pick {
    pub fn new(desk: Desk) -> Pick {
        Pick { frame: PRect::default(), desk, spot: RefCell::default(), motion: PTabMove::default() }
    }

    /// The groups the chosen part has, and the robot's own: a heading's
    /// key, what it holds, and what a press on each does.
    fn groups(&self, data: &Data, look: &Look) -> (Vec<Group>, Vec<Group>) {
        let chosen = self.desk.chosen.get();
        let name = data.part.row.get(chosen).and_then(|row| row.first()).cloned().unwrap_or_default();
        let count = data.part.rows(&name).next().and_then(|row| row.get(2)).and_then(|n| n.parse::<usize>().ok()).unwrap_or_default();
        let choice = look.part.iter().find(|(n, _)| *n == name).map(|(_, c)| c.clone()).unwrap_or_default();
        let shape: Vec<String> = (1..=count).map(|n| format!("{name}{n}")).collect();
        let material = materials(data);
        let colour: Vec<PColor> = data.material.rows(&choice.material).map(|row| byte([3, 4, 5].map(|c| row.get(c).and_then(|v| v.parse().ok()).unwrap_or_default()))).collect();
        let light = data.light.row.len();
        let eyes = match name.as_str() {
            "head" => vec![(
                "face",
                Hold::Pill(data.face.row.iter().enumerate().map(|(i, row)| (row.get(1).cloned().unwrap_or_default(), String::new(), i == look.face)).collect()),
                (0..data.face.row.len()).map(Act::Face).collect(),
            )],
            _ => Vec::new(),
        };
        let first = vec![("shape", Hold::Pill(shape.iter().map(|s| (String::new(), s.clone(), *s == choice.shape)).collect()), shape.iter().cloned().map(Act::Shape).collect())];
        let rest = vec![
            ("material", Hold::Pill(material.iter().map(|m| (String::new(), m.clone(), *m == choice.material)).collect()), material.iter().cloned().map(Act::Material).collect()),
            ("colour", Hold::Swatch(colour.iter().enumerate().map(|(i, c)| (*c, i == choice.colour)).collect()), (0..colour.len()).map(Act::Colour).collect()),
        ];
        let robot = vec![("light", Hold::Swatch((0..light).map(|i| (super::light(data, i), i == look.light)).collect()), (0..light).map(Act::Light).collect())];
        ([first, eyes, rest].into_iter().flatten().collect(), robot)
    }
}

/// A group: its heading's key, what it holds, what a press on each does.
type Group = (&'static str, Hold, Vec<Act>);

/// The kit's groups for some of ours.
fn grouped<'a>(group: &'a [Group], text: &appkit::service::PTable) -> Vec<PGrouped<'a>> {
    group.iter().map(|(key, hold, _)| PGrouped { head: text.get(key).to_string(), draw: Box::new(move |paint: &mut dyn PPaint, at: PPoint, wide: f32| drawn(hold, paint, at, wide)) }).collect()
}

/// What a group draws: its options across the width, or its colours.
fn drawn(hold: &Hold, paint: &mut dyn PPaint, at: PPoint, wide: f32) -> (f32, Vec<PRect>) {
    match hold {
        Hold::Pill(list) => {
            let pill: Vec<(&str, &str, bool)> = list.iter().map(|(icon, word, on)| (icon.as_str(), word.as_str(), *on)).collect();
            let rect = pill.draw_full(paint, at, wide);
            (at.y + pill.high(paint), rect)
        }
        Hold::Swatch(list) => {
            let (rect, end) = list.draw_in(paint, at, wide);
            (end, rect)
        }
    }
}

impl PDraw for Pick {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        self.spot.borrow_mut().clear();
        let gap = paint.size(GAP);
        let data = self.desk.data();
        let look = self.desk.bot.borrow().look();
        let (left, wide) = (origin.x + gap, self.frame.size.width - gap * 2.0);
        let text = &self.desk.text;
        // Reset and Randomize at the top right: the app's name is the
        // window's, in its top row.
        let top = origin.y + gap;
        let tool = [("undo", text.get("reset"), false), ("rotate", text.get("random"), false)];
        let icon: Vec<(&str, &str, bool)> = tool.iter().map(|(icon, _, on)| (*icon, "", *on)).collect();
        let rect = icon.draw_end(paint, PPoint::new(left + wide, top));
        self.spot.borrow_mut().extend(rect.into_iter().zip([Act::Reset, Act::Random]));
        let top = top + icon.high(paint) + gap / 2.0;
        // The robot's own groups at the foot, and the parts as tabs over
        // the chosen part's groups, down to them.
        let part: Vec<(&str, &str, bool)> = data.part.row.iter().enumerate().map(|(i, row)| ("", row.get(3).map_or("", String::as_str), i == self.desk.chosen.get())).collect();
        let (group, robot) = self.groups(&data, &look);
        let (mut chosen, mut own) = (grouped(&group, text), grouped(&robot, text));
        let foot = origin.y + self.frame.size.height - gap;
        let space = paint.theme().size(appkit::pui::layout::SPACE).unwrap_or(gap / 2.0);
        let high = stack(&mut PDry(&*paint), PPoint::new(left, 0.0), wide, 0.0, &mut own).0;
        let (tabs, pressed) = tabbed(paint, PPoint::new(left, top), wide, foot - high - space, part.as_slice(), &self.motion, &mut chosen);
        let (_, held) = stack(paint, PPoint::new(left, foot - high), wide, foot, &mut own);
        let mut spot = self.spot.borrow_mut();
        spot.extend(tabs.into_iter().zip((0..part.len()).map(Act::Part)));
        for (rect, (.., act)) in pressed.into_iter().zip(&group).chain(held.into_iter().zip(&robot)) {
            spot.extend(rect.into_iter().zip(act.iter().cloned()));
        }
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

    /// The tab's light moves while it is on its way.
    fn tick(&mut self) -> bool {
        self.motion.tick(self.desk.shade.as_ref() as &dyn PTheme)
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        let PEvent::PointerDown(at) = *event else { return false };
        let place = at.plus(self.frame.origin);
        let act = self.spot.borrow().iter().find(|(rect, _)| rect.contains(place)).map(|(_, act)| act.clone());
        act.is_some_and(|act| {
            let tab = match act {
                Act::Part(i) => Some(i),
                _ => None,
            };
            tab.into_iter().for_each(|i| self.motion.to(i));
            apply(&self.desk, act);
            true
        })
    }
}
