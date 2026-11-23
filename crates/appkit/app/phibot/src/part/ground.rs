//! The greenhouse (ExplorationView) in 3D (`scene.rs`), its planters
//! marked (green once propagated). While the scene is read it says it is
//! loading. The robot walks with the keys, held as the system repeats
//! them, or to where a press on the floor is; a drag turns and tips the
//! view and the wheel brings it nearer.

use std::cell::{Cell, RefCell};
use std::time::Instant;

use appkit::pui::core::{PDraw, PEvent, PFit, PPaint, PPoint, PRect, PView};
use appkit::theme::PTheme;
use phiecs::comp::tree::matrix;

use super::scene::{FILM, Scene};
use super::{DIM, Desk, GAP, INK, LINE, Load};

/// The theme's tokens: how large a planter's mark is, and its colours.
pub const PLANTER: &str = "garden.planter";
pub const PLANT: &str = "garden.planter";
pub const GROWN: &str = "garden.grown";
/// How long a key is held after the system last said it, in seconds.
pub const HOLD: &str = "garden.hold";
/// How long the robot may come no nearer to where a press sent it
/// before it gives up, in seconds.
pub const STALL: &str = "garden.stall";

pub struct Ground {
    frame: PRect,
    desk: Desk,
    /// The keys held, and how long each is still held.
    held: RefCell<Vec<(String, f32)>>,
    /// Where a press asked the robot to go, on the ground, and how far
    /// it still was the step before: a goal it comes no nearer to (past
    /// the floor's edge) is given up.
    goal: Cell<Option<[f32; 2]>>,
    far: Cell<f32>,
    stall: Cell<f32>,
    last: Cell<Option<Instant>>,
    scene: Scene,
    /// Where the last press went down, to walk there if it did not drag.
    down: Cell<Option<PPoint>>,
}

impl Ground {
    pub fn new(desk: Desk) -> Ground {
        let (goal, far, stall, last) = (Cell::default(), Cell::new(f32::MAX), Cell::default(), Cell::default());
        Ground { frame: PRect::default(), desk, held: RefCell::default(), goal, far, stall, last, scene: Scene::default(), down: Cell::default() }
    }

    fn seen(&self, p: [f32; 3]) -> Option<PPoint> {
        self.scene.seen(p, self.frame.size)
    }

    /// The name phiace's key table gives a key the window sends.
    fn named(key: char) -> String {
        match key {
            PEvent::UP => "up".to_string(),
            PEvent::DOWN => "down".to_string(),
            PEvent::LEFT => "left".to_string(),
            PEvent::RIGHT => "right".to_string(),
            other => other.to_ascii_lowercase().to_string(),
        }
    }

    /// Steer toward the goal with the keys, as a person would.
    fn steer(&self, dt: f32) {
        let Some(goal) = self.goal.get() else { return };
        let mut bot = self.desk.bot.borrow_mut();
        let at = matrix(&bot.world, bot.robot).origin();
        let near = bot.data().game.num("touch.reach") / 4.0;
        let (dx, dz) = (goal[0] - at[0], goal[1] - at[2]);
        let far = dx.hypot(dz);
        // Given up once it has come no nearer for as long as the theme says.
        let nearer = far < self.far.replace(far) - f32::EPSILON;
        self.stall.set(match nearer {
            true => 0.0,
            false => self.stall.get() + dt,
        });
        let stuck = self.stall.get() >= self.desk.shade.size(STALL).unwrap_or_default();
        let want: Vec<&str> = [(dx > near, "d"), (dx < -near, "a"), (dz > near, "s"), (dz < -near, "w")].into_iter().filter(|(on, _)| *on).map(|(_, k)| k).collect();
        ["d", "a", "s", "w"].iter().filter(|k| !want.contains(k)).for_each(|k| bot.lift(k));
        let done = want.is_empty() || stuck;
        match done {
            true => {
                ["d", "a", "s", "w"].iter().for_each(|k| bot.lift(k));
                self.goal.set(None);
                self.far.set(f32::MAX);
                self.stall.set(0.0);
            }
            false => want.iter().for_each(|k| bot.press(k)),
        }
    }
}

impl PDraw for Ground {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, line, ink, dim) = (paint.size(GAP), paint.size(LINE), paint.color(INK), paint.color(DIM));
        let size = self.frame.size;
        let middle = |text: &str, paint: &mut dyn PPaint, y: f32| paint.text(PPoint::new(origin.x + (size.width - paint.measure(text)) / 2.0, y), text, dim);
        match &*self.desk.load.borrow() {
            Load::Reading(_) | Load::Idle => return middle(self.desk.text.get("loading"), paint, origin.y + size.height / 2.0),
            Load::Missing(path) => return middle(&self.desk.text.get("missing").replace("{path}", path), paint, origin.y + size.height / 2.0),
            Load::Ready => {}
        }
        self.scene.shoot(&self.desk, (size.width.max(0.0) as u32, size.height.max(0.0) as u32));
        paint.picture(FILM, PRect { origin, size }, 0.0, PFit::Whole);
        let bot = self.desk.bot.borrow();
        let side = paint.size(PLANTER);
        for planter in bot.garden().map(|g| g.planter.clone()).unwrap_or_default() {
            let Some(at) = self.seen(matrix(&bot.world, planter.word).origin()).map(|p| p.plus(origin)) else { continue };
            let colour = match planter.grown {
                true => paint.color(GROWN),
                false => paint.color(PLANT),
            };
            paint.round(PRect::new(at.x - side / 2.0, at.y - side / 2.0, side, side), side / 2.0, colour);
            paint.text(PPoint::new(at.x - paint.measure(&planter.plant) / 2.0, at.y + side / 2.0 + gap / 4.0), &planter.plant, ink);
        }
        // What the robot is doing, at its feet.
        let said = self.desk.text.get("state").replace("{state}", &bot.state());
        // On the floor under it, not over its middle.
        let floor = bot.world.lexicon.get(phiace::bot::botanist::garden::GREENHOUSE).map_or(0.0, |root| matrix(&bot.world, root).origin()[1]);
        let [x, _, z] = matrix(&bot.world, bot.robot).origin();
        let feet = self.seen([x, floor, z]).map(|p| p.plus(origin));
        feet.into_iter().for_each(|at| paint.text(PPoint::new(at.x - paint.measure(&said) / 2.0, at.y + gap / 2.0), &said, dim));
        middle(self.desk.text.get("walk"), paint, origin.y + size.height - gap - line);
    }
}

impl PView for Ground {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    /// Time passes in the greenhouse while it is read, while the robot
    /// walks or a key is held, and while it celebrates.
    fn tick(&mut self) -> bool {
        let loading = self.desk.loading();
        let now = Instant::now();
        let dt = self.last.replace(Some(now)).map_or(0.0, |then| (now - then).as_secs_f32()).min(0.1);
        let exploring = self.desk.exploring() && matches!(*self.desk.load.borrow(), Load::Ready);
        if !exploring {
            self.last.set(None);
            return loading;
        }
        // Keys the system has stopped repeating are let go.
        let mut held = self.held.borrow_mut();
        held.iter_mut().for_each(|(_, left)| *left -= dt);
        let gone: Vec<String> = held.iter().filter(|(_, left)| *left <= 0.0).map(|(k, _)| k.clone()).collect();
        held.retain(|(_, left)| *left > 0.0);
        drop(held);
        gone.iter().for_each(|key| self.desk.bot.borrow_mut().lift(key));
        self.steer(dt);
        self.desk.bot.borrow_mut().step(dt);
        let state = self.desk.bot.borrow().state();
        let busy = !self.held.borrow().is_empty() || self.goal.get().is_some() || state != phiace::bot::botanist::machine::IDLE;
        (!busy).then(|| self.last.set(None));
        busy
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        match *event {
            PEvent::KeyDown(key) => {
                let name = Ground::named(key);
                let hold = self.desk.shade.size(HOLD).unwrap_or_default();
                let known = self.desk.data().key.rows(&name).next().is_some();
                if known {
                    let mut held = self.held.borrow_mut();
                    match held.iter_mut().find(|(k, _)| *k == name) {
                        Some((_, left)) => *left = hold,
                        None => held.push((name.clone(), hold)),
                    }
                    drop(held);
                    self.desk.bot.borrow_mut().press(&name);
                }
                known
            }
            PEvent::PointerDown(at) => {
                self.scene.orbit.down(at);
                self.down.set(Some(at));
                true
            }
            PEvent::PointerMove(at) => self.scene.orbit.moved(at, self.desk.shade.as_ref()),
            // Let go without dragging: a press on the floor, and there
            // the robot goes. After a drag, the greenhouse drawn whole.
            PEvent::PointerUp(_) => {
                let pressed = self.scene.orbit.up();
                let floor = self.down.take().filter(|_| pressed).and_then(|at| self.scene.floor(&self.desk, at, self.frame.size));
                floor.into_iter().for_each(|p| self.goal.set(Some([p[0], p[2]])));
                self.far.set(f32::MAX);
                true
            }
            PEvent::Scroll(_, by) => self.scene.orbit.wheel(by.y, self.desk.shade.as_ref()),
            _ => false,
        }
    }
}
