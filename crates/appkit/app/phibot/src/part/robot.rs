//! The robot (RobotView): the robot as the customiser made it, in 3D
//! from its parts' own USD meshes, turned and tipped by dragging it and
//! brought nearer with the wheel, and Start Planting at its top right.
//! It is drawn again only when its look, the view or the part's size
//! changes.

use std::cell::RefCell;

use appkit::pui::block::PPill;
use appkit::pui::core::{PDraw, PEvent, PFit, PPaint, PPoint, PRaster, PRect, PView};
use appkit::theme::PTheme;
use phiace::bot::botanist::figure::figure;
use phiace::bot::botanist::look::Look;

use super::view::{Orbit, ROOM, around, draw, ground, smooth, sun};
use super::{DIM, Desk, GAP, GARDEN};

/// The picture the robot is drawn into.
pub const FILM: &str = "bot/robot";

/// What the robot was drawn for: its look, the view, its size.
type Drawn = (Look, [f32; 3], (u32, u32));

pub struct Robot {
    frame: PRect,
    desk: Desk,
    start: RefCell<PRect>,
    orbit: Orbit,
    drawn: RefCell<Option<Drawn>>,
}

impl Robot {
    pub fn new(desk: Desk) -> Robot {
        Robot { frame: PRect::default(), desk, start: RefCell::default(), orbit: Orbit::default(), drawn: RefCell::default() }
    }

    /// Draw the robot into its picture, where anything changed: made
    /// larger than shown, and made smaller, so no edge is jagged.
    fn shoot(&self, size: (u32, u32)) {
        let bot = self.desk.bot.borrow();
        let look = bot.look();
        let now = (look.clone(), self.orbit.now(), size);
        if self.drawn.borrow().as_ref() == Some(&now) || size.0 == 0 || size.1 == 0 {
            return;
        }
        let data = bot.data().clone();
        let piece = figure(&bot.world, &look, &data, &mut self.desk.library.borrow_mut());
        let theme = self.desk.shade.as_ref();
        let by = smooth(theme);
        let first = around(&piece, 0.0, data.game.num("camera.fov"), theme.size(ROOM).unwrap_or(1.0), size.0 as f32 / size.1 as f32);
        let mut raster = PRaster::new(size.0 * by, size.1 * by, ground(theme));
        draw(&mut raster, &self.orbit.camera(&first), &piece, &sun(&data));
        let (w, h, pixel) = raster.smooth(by);
        self.desk.film.show(FILM, w, h, pixel);
        *self.drawn.borrow_mut() = Some(now);
    }
}

impl PDraw for Robot {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let (gap, dim) = (paint.size(GAP), paint.color(DIM));
        let size = self.frame.size;
        let state = self.desk.bot.borrow().state();
        self.shoot((size.width.max(0.0) as u32, size.height.max(0.0) as u32));
        paint.picture(FILM, PRect { origin, size }, 0.0, PFit::Whole);
        // What it is doing, at the foot.
        let said = self.desk.text.get("state").replace("{state}", &state);
        paint.text(PPoint::new(origin.x + (size.width - paint.measure(&said)) / 2.0, origin.y + size.height - gap * 2.0), &said, dim);
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
        let theme = self.desk.shade.as_ref();
        match *event {
            PEvent::PointerDown(at) => {
                let pressed = self.start.borrow().contains(at.plus(self.frame.origin));
                match pressed {
                    true => self.desk.plant(GARDEN),
                    false => self.orbit.down(at),
                }
                pressed
            }
            PEvent::PointerMove(at) => self.orbit.moved(at, theme),
            PEvent::PointerUp(_) => {
                self.orbit.up();
                false
            }
            PEvent::Scroll(_, by) => self.orbit.wheel(by.y, theme),
            _ => false,
        }
    }
}
