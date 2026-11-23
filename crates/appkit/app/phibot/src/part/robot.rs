//! The robot (RobotView): the robot as the customiser made it, and Start
//! Planting at its top right. It is drawn from its look, flat, until the
//! window draws its meshes: a backpack behind, a body (legs for the
//! first, a wheel for the second, a glow under the third, as their
//! types are), a head with a screen and the eyes chosen on it, each part
//! in its material's colour and the lights in theirs.

use std::cell::RefCell;

use appkit::pui::block::PPill;
use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};
use phiace::bot::botanist::Data;
use phiace::bot::botanist::look::Look;

use super::{DIM, Desk, GAP, GARDEN, LINE, byte, light};

/// The theme's tokens: how high the robot is drawn at most, and each
/// part's height as a share of it.
pub const HEIGHT: &str = "bot.height";
pub const HEAD: &str = "bot.head";
pub const BODY: &str = "bot.body";
pub const PACK: &str = "bot.pack";
/// The colour of the head's screen, behind the eyes.
pub const SCREEN: &str = "bot.screen";

pub struct Robot {
    frame: PRect,
    desk: Desk,
    start: RefCell<PRect>,
}

impl Robot {
    pub fn new(desk: Desk) -> Robot {
        Robot { frame: PRect::default(), desk, start: RefCell::default() }
    }
}

/// The number at the end of a shape's name: `body2` is 2.
fn number(shape: &str) -> usize {
    shape.trim_start_matches(|c: char| !c.is_ascii_digit()).parse().unwrap_or(1)
}

/// Draw the robot standing with its middle foot at `foot`, `high` tall.
pub fn figure(paint: &mut dyn PPaint, look: &Look, data: &Data, foot: PPoint, high: f32) {
    let (head, body, pack) = (high * paint.size(HEAD), high * paint.size(BODY), high * paint.size(PACK));
    let shape = |part: &str| look.part.iter().find(|(n, _)| n == part).map_or(1, |(_, c)| number(&c.shape));
    let colour = |part: &str| byte(look.colour(part, data));
    let glow = light(data, look.light);
    let under = high - head - body;
    // What the body stands on: two legs, a wheel, or a glow.
    let wide = body * 0.9;
    let base = PRect::new(foot.x - wide / 2.0, foot.y - under, wide, under);
    match shape("body") {
        1 => [0.18, 0.62].iter().for_each(|x| paint.round(PRect::new(base.origin.x + wide * x, base.origin.y, wide * 0.2, under), wide * 0.1, colour("body"))),
        2 => paint.round(PRect::new(foot.x - under / 2.0, base.origin.y, under, under), under / 2.0, colour("body")),
        _ => paint.round(PRect::new(base.origin.x, foot.y - under * 0.35, wide, under * 0.3), under * 0.15, glow),
    }
    // The backpack behind the body, its size by its shape.
    let p = pack * (0.8 + 0.1 * shape("backpack") as f32);
    let body_top = foot.y - under - body;
    paint.round(PRect::new(foot.x + wide * 0.15, body_top + body * 0.08, p * 0.8, p), p * 0.25, colour("backpack"));
    paint.round(PRect::new(foot.x + wide * 0.15 + p * 0.55, body_top + body * 0.08 + p * 0.15, p * 0.12, p * 0.7), p * 0.06, glow);
    // The body, rounder for its later shapes.
    let b = PRect::new(foot.x - wide / 2.0, body_top, wide, body);
    paint.round(b, wide * (0.15 + 0.1 * shape("body") as f32), colour("body"));
    paint.round(PRect::new(b.origin.x + wide * 0.35, b.origin.y + body * 0.35, wide * 0.3, body * 0.12), body * 0.06, glow);
    // The head: its shape wide, a screen, the eyes in the light's colour.
    let hw = head * (1.0 + 0.12 * shape("head") as f32);
    let h = PRect::new(foot.x - hw / 2.0, body_top - head, hw, head * 0.95);
    paint.round(h, head * 0.3, colour("head"));
    let screen = PRect::new(h.origin.x + hw * 0.12, h.origin.y + head * 0.18, hw * 0.76, head * 0.55);
    let dark = paint.color(SCREEN);
    paint.round(screen, head * 0.18, dark);
    let eye = head * 0.26;
    let face = look.face_name(data);
    for x in [0.3, 0.7] {
        paint.icon(face, PRect::new(screen.origin.x + screen.size.width * x - eye / 2.0, screen.center().y - eye / 2.0, eye, eye), glow);
    }
}

impl PDraw for Robot {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, line, dim) = (paint.size(GAP), paint.size(LINE), paint.color(DIM));
        let size = self.frame.size;
        let (look, data, state) = {
            let bot = self.desk.bot.borrow();
            (bot.look(), bot.data().clone(), bot.state())
        };
        let high = paint.size(HEIGHT).min(size.height - gap * 4.0 - line * 2.0).max(0.0);
        let foot = PPoint::new(origin.x + size.width / 2.0, origin.y + (size.height + high) / 2.0);
        figure(paint, &look, &data, foot, high);
        // What it is doing, under it.
        let said = self.desk.text.get("state").replace("{state}", &state);
        paint.text(PPoint::new(foot.x - paint.measure(&said) / 2.0, foot.y + gap / 2.0), &said, dim);
        // Start Planting, at the top right.
        let pill = ("leaf", self.desk.text.get("start"), true);
        let wide = pill.size(paint).width;
        *self.start.borrow_mut() = pill.draw(paint, PPoint::new(origin.x + size.width - gap - wide, origin.y + gap), false);
    }
}

impl PView for Robot {
    fn hand(&self) -> Vec<PRect> {
        vec![*self.start.borrow()]
    }

    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        let PEvent::PointerDown(at) = *event else { return false };
        let pressed = self.start.borrow().contains(at.plus(self.frame.origin));
        pressed.then(|| self.desk.plant(GARDEN));
        pressed
    }
}
