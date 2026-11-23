//! The greenhouse (ExplorationView), seen from above until the window
//! draws its meshes: its ground, its planters where Apple's scene puts
//! them (green once propagated), and the robot facing where it goes.
//! While the scene is read it says it is loading. The robot walks with
//! the keys, held as the system repeats them, or to where a press is.

use std::cell::{Cell, RefCell};
use std::time::Instant;

use appkit::pui::core::{PDraw, PEvent, PPaint, PPoint, PRect, PView};
use appkit::theme::PTheme;
use phiecs::comp::Pose;
use phiecs::comp::tree::matrix;

use super::{DIM, Desk, GAP, INK, LINE, Load};

/// The theme's tokens: how much room is shown round the floor, as a
/// share of its reach, and how large a planter, the robot and its facing mark are drawn.
pub const REACH: &str = "garden.reach";
pub const PLANTER: &str = "garden.planter";
pub const ROBOT: &str = "garden.robot";
pub const DOT: &str = "garden.dot";
/// Its colours.
pub const GROUND: &str = "garden.ground";
pub const PLANT: &str = "garden.planter";
pub const GROWN: &str = "garden.grown";
pub const BOT: &str = "garden.robot";
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
}

impl Ground {
    pub fn new(desk: Desk) -> Ground {
        Ground { frame: PRect::default(), desk, held: RefCell::default(), goal: Cell::default(), far: Cell::new(f32::MAX), stall: Cell::default(), last: Cell::default() }
    }

    /// The scale and middle the greenhouse is drawn at: pixels a metre.
    /// The floor fills the part, with the theme's room round it.
    fn view(&self) -> (f32, PPoint) {
        let size = self.frame.size;
        let floor = self.desk.bot.borrow().garden().map_or(1.0, |g| g.reach);
        let reach = floor * self.desk.shade.size(REACH).unwrap_or(1.0);
        (size.width.min(size.height) / 2.0 / reach.max(f32::EPSILON), PPoint::new(size.width / 2.0, size.height / 2.0))
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
        let (scale, mid) = self.view();
        let to = |p: [f32; 3]| PPoint::new(origin.x + mid.x + p[0] * scale, origin.y + mid.y + p[2] * scale);
        let ground = self.desk.bot.borrow().garden().map_or(0.0, |g| g.reach) * scale * 2.0;
        paint.round(PRect::new(origin.x + mid.x - ground / 2.0, origin.y + mid.y - ground / 2.0, ground, ground), ground / 2.0, paint.color(GROUND));
        let bot = self.desk.bot.borrow();
        let (side, robot, dot) = (paint.size(PLANTER), paint.size(ROBOT), paint.size(DOT));
        for planter in bot.garden().map(|g| g.planter.clone()).unwrap_or_default() {
            let at = to(matrix(&bot.world, planter.word).origin());
            let colour = match planter.grown {
                true => paint.color(GROWN),
                false => paint.color(PLANT),
            };
            paint.round(PRect::new(at.x - side / 2.0, at.y - side / 2.0, side, side), side / 2.0, colour);
            paint.text(PPoint::new(at.x - paint.measure(&planter.plant) / 2.0, at.y + side / 2.0 + gap / 4.0), &planter.plant, ink);
        }
        // The robot, and a mark where it faces: forward is -Z turned by its pose.
        let at = to(matrix(&bot.world, bot.robot).origin());
        let turn = bot.world.read::<Pose>(bot.robot).map(|p| p.orient).unwrap_or(Pose::default().orient);
        let ahead = Pose { orient: turn, ..Pose::default() }.matrix().apply([0.0, 0.0, -1.0]);
        paint.round(PRect::new(at.x - robot / 2.0, at.y - robot / 2.0, robot, robot), robot / 2.0, paint.color(BOT));
        let tip = PPoint::new(at.x + ahead[0] * robot * 0.6, at.y + ahead[2] * robot * 0.6);
        paint.round(PRect::new(tip.x - dot / 2.0, tip.y - dot / 2.0, dot, dot), dot / 2.0, ink);
        let said = self.desk.text.get("state").replace("{state}", &bot.state());
        paint.text(PPoint::new(at.x - paint.measure(&said) / 2.0, at.y + robot / 2.0 + gap / 4.0), &said, dim);
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
                let (scale, mid) = self.view();
                self.goal.set(Some([(at.x - mid.x) / scale, (at.y - mid.y) / scale]));
                self.far.set(f32::MAX);
                true
            }
            _ => false,
        }
    }
}
