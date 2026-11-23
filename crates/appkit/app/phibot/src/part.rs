//! The parts of the botanist's window, and the desk they share: the
//! session (phiace's world), the words, the looks, where a part asks the
//! window for a deed, and the greenhouse while it loads.

mod ground;
mod pick;
mod robot;
mod score;

pub use ground::Ground;
pub use pick::Pick;
pub use robot::Robot;
pub use score::Score;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc::{Receiver, channel};

use appkit::pui::core::PColor;
use appkit::pui::window::{PDeed, PWish};
use appkit::service::PTable;
use appkit::theme::PShade;
use phiace::bot::botanist::{Botanist, Data, Phase, garden};
use phiecs::usd::Stage;

/// The theme's tokens the parts draw with.
pub const GAP: &str = "gap";
pub const LINE: &str = "text.line";
pub const INK: &str = "text.ink";
pub const DIM: &str = "text.dim";

/// The spaces: making the robot, and the greenhouse.
pub const MAKE: &str = "make";
pub const GARDEN: &str = "garden";

/// The greenhouse: not asked for, being read, in the world, or not on
/// this disk (where it was looked for).
#[derive(Default)]
pub enum Load {
    #[default]
    Idle,
    Reading(Receiver<Option<Stage>>),
    Ready,
    Missing(String),
}

/// What the parts share.
#[derive(Clone)]
pub struct Desk {
    pub bot: Rc<RefCell<Botanist>>,
    pub text: Rc<PTable>,
    pub shade: Rc<PShade>,
    pub wish: PWish,
    /// Which part the customiser shows, by its place in phiace's table.
    pub chosen: Rc<Cell<usize>>,
    pub load: Rc<RefCell<Load>>,
}

impl Desk {
    pub fn new(shade: Rc<PShade>, text: Rc<PTable>) -> Desk {
        let bot = Rc::new(RefCell::new(Botanist::new(Data::read())));
        Desk { bot, text, shade, wish: PWish::default(), chosen: Rc::default(), load: Rc::default() }
    }

    /// The botanist's tables.
    pub fn data(&self) -> Data {
        self.bot.borrow().data().clone()
    }

    /// "Start Planting": the greenhouse is read away from the window,
    /// while the greenhouse's screen says it is loading, and the window
    /// shows it. Asked for another space, nothing.
    pub fn plant(&self, space: &str) {
        if space != GARDEN {
            return;
        }
        let mut load = self.load.borrow_mut();
        match &*load {
            // In the world already: into it again.
            Load::Ready => {
                self.bot.borrow_mut().plant_in(&Stage::default());
            }
            Load::Idle | Load::Missing(_) => {
                let (send, take) = channel();
                let data = self.data();
                std::thread::spawn(move || _ = send.send(garden::read(&data)));
                *load = Load::Reading(take);
            }
            Load::Reading(_) => {}
        }
        self.wish.borrow_mut().push(PDeed::Show(GARDEN.to_string()));
    }

    /// Take the greenhouse once it is read. `true` while it is still
    /// being read.
    pub fn loading(&self) -> bool {
        let mut load = self.load.borrow_mut();
        let read = match &*load {
            Load::Reading(take) => take.try_recv().ok(),
            _ => return false,
        };
        match read {
            None => true,
            Some(Some(stage)) => {
                self.bot.borrow_mut().plant_in(&stage);
                *load = Load::Ready;
                false
            }
            Some(None) => {
                *load = Load::Missing(self.data().asset().display().to_string());
                false
            }
        }
    }

    /// "Exit Game": out of the greenhouse, back to making the robot.
    pub fn exit(&self) {
        self.bot.borrow_mut().exit();
        self.wish.borrow_mut().push(PDeed::Show(MAKE.to_string()));
    }

    pub fn exploring(&self) -> bool {
        self.bot.borrow().phase == Phase::Exploration
    }
}

/// A colour from 0-255 channels, as the robot's tables write them.
pub fn byte(rgb: [u8; 3]) -> PColor {
    PColor::new(rgb[0] as f32 / 255.0, rgb[1] as f32 / 255.0, rgb[2] as f32 / 255.0, 1.0)
}

/// A light's colour from phiace's light table, its 0-1 channels.
pub fn light(data: &Data, index: usize) -> PColor {
    let row = data.light.row.get(index);
    let cell = |i: usize| row.and_then(|row| row.get(i)).and_then(|v| v.parse::<f32>().ok()).unwrap_or_default();
    PColor::new(cell(2), cell(3), cell(4), 1.0)
}
