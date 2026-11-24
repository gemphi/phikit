//! A model opened from the library, in 3D: its parts as the person
//! dressed them, turned and tipped by a drag and brought nearer by the
//! wheel (the kit's orbit), standing at the top of the part. It is drawn
//! again only when its dress, the view or the part's size changes.

use std::cell::RefCell;

use appkit::pui::core::orbit::{around, shot};
use appkit::pui::core::{PDraw, PEvent, PFit, POrbit, PPaint, PPoint, PRect, PView};
use appkit::pui::state::PState;
use appkit::theme::PTheme;

use super::{DIM, Desk, GAP};
use crate::library::Dress;

/// The picture the model is drawn into.
pub const FILM: &str = "library/figure";

/// What it was drawn for: the model and its dress, the view, its size.
type Drawn = (String, [f32; 3], (u32, u32));

pub struct Figure {
    frame: PRect,
    desk: Desk,
    orbit: POrbit,
    drawn: RefCell<Option<Drawn>>,
}

impl Figure {
    pub fn new(desk: Desk) -> Figure {
        Figure { frame: PRect::default(), desk, orbit: POrbit::default(), drawn: RefCell::default() }
    }

    /// Draw the model into its picture where anything changed. `false`
    /// where nothing is opened.
    fn shoot(&self, size: (u32, u32)) -> bool {
        let library = &self.desk.library;
        let Some(item) = library.get(&self.desk.shelf.get("ui/item")) else { return false };
        let dress = Dress::new(self.desk.choice.clone());
        let parts = library.model(&item.path).part.len();
        let now = (format!("{}|{}", item.key, dress.mark(&item.key, parts)), self.orbit.now(), size);
        if self.drawn.borrow().as_ref() == Some(&now) || size.0 == 0 || size.1 == 0 {
            return true;
        }
        let theme = self.desk.shade.as_ref();
        let piece = library.dressed(&item, &dress);
        let fov = library.set.get("camera.fov").parse().unwrap_or(30.0);
        let room = theme.size(appkit::pui::core::orbit::ROOM).unwrap_or(1.0);
        let first = around(&piece, 0.6, fov, room, size.0 as f32 / size.1 as f32);
        let (wide, high, pixel) = shot(&piece, &self.orbit.camera(&first), &library.sun(&item, &dress), size, theme);
        library.film.show(FILM, wide, high, pixel);
        *self.drawn.borrow_mut() = Some(now);
        true
    }
}

impl PDraw for Figure {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        let size = self.frame.size;
        match self.shoot((size.width.max(0.0) as u32, size.height.max(0.0) as u32)) {
            true => paint.picture(FILM, PRect { origin, size }, 0.0, PFit::Top),
            false => {
                let said = self.desk.text.get("library.none");
                paint.text(PPoint::new(origin.x + paint.size(GAP), origin.y + paint.size(GAP)), said, paint.color(DIM));
            }
        }
    }
}

impl PView for Figure {
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
                self.orbit.down(at);
                false
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
