//! An item opened from the library, as Reality Composer Pro's viewport
//! shows one: a model in 3D, its parts as the person set them, turned and
//! tipped by a drag and brought nearer by the wheel (the kit's orbit);
//! in its corner the orientation ball, a press on whose balls looks along
//! that way; at its foot the viewport's tools (`data/viewport.tsv`): the
//! view as it began, four views, and the model as wire. With four views
//! the model is seen from the top, the front and the right as well, each
//! from a camera of its own, nearly flat, as wire. A picture is drawn
//! again only when what it shows changes. Any other item is its own
//! picture or face, at the top.

mod view;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use appkit::pui::core::{PDraw, PEvent, PFit, POrbit, PPaint, PPoint, PRect, PView};
use appkit::pui::state::PState;

use super::{DIM, Desk, GAP};
use crate::library::Item;

/// The picture the model is drawn into, and each side's.
pub const FILM: &str = "library/figure";
/// Where the viewport's switches are kept: four views, and wire.
pub const FOUR: &str = "ui/view/four";
pub const WIRE: &str = "ui/view/wire";

/// What a picture was drawn for: what it shows and its size.
type Drawn = (String, (u32, u32));

pub struct Figure {
    frame: PRect,
    desk: Desk,
    orbit: POrbit,
    /// The way the main view looks along, where a ball was pressed.
    way: Cell<Option<[f32; 3]>>,
    drawn: RefCell<HashMap<String, Drawn>>,
    /// Where it was drawn, the ball's balls and the tools, last.
    origin: Cell<PPoint>,
    ball: RefCell<Vec<(PRect, [f32; 3])>>,
    tool: RefCell<Vec<(PRect, String)>>,
}

impl Figure {
    pub fn new(desk: Desk) -> Figure {
        Figure { frame: PRect::default(), desk, orbit: POrbit::default(), way: Cell::default(), drawn: RefCell::default(), origin: Cell::default(), ball: RefCell::default(), tool: RefCell::default() }
    }

    /// Do what a tool does.
    fn act(&self, name: &str) {
        let flip = |path: &str| {
            let now = self.desk.shelf.get(path);
            self.desk.shelf.set(path, if now.is_empty() { "1" } else { "" });
        };
        match name {
            "reset" => {
                self.orbit.reset();
                self.way.set(None);
            }
            "four" => flip(FOUR),
            "wire" => flip(WIRE),
            _ => {}
        }
    }
}

impl PDraw for Figure {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        self.origin.set(origin);
        let (size, room) = (self.frame.size, PRect { origin, size: self.frame.size });
        let item = self.desk.library.get(&self.desk.shelf.get("ui/item"));
        match (item, room.size.width.min(room.size.height)) {
            (Some(item), _) if item.kind == "model" => self.model(paint, &item, room),
            (Some(Item { face: Some((sheet, column, index)), .. }), side) => paint.picture(&sheet, PRect::new(origin.x + (size.width - side) / 2.0, origin.y, side, side), 0.0, PFit::Cell { column, index }),
            (Some(item), _) if !item.picture.is_empty() => paint.picture(&item.picture, room, 0.0, PFit::Top),
            _ => {
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

    fn hand(&self) -> Vec<PRect> {
        let ball = self.ball.borrow().iter().map(|(rect, _)| *rect).collect::<Vec<_>>();
        ball.into_iter().chain(self.tool.borrow().iter().map(|(rect, _)| *rect)).collect()
    }

    /// A press on a tool does what it does; a drag turns the model, and a
    /// press on a ball that did not drag looks along its way.
    fn respond(&mut self, event: &PEvent) -> bool {
        let theme = self.desk.shade.as_ref();
        let place = |at: PPoint| at.plus(self.origin.get());
        match *event {
            PEvent::PointerDown(at) => {
                let tool = self.tool.borrow().iter().find(|(rect, _)| rect.contains(place(at))).map(|(_, name)| name.clone());
                match tool {
                    Some(name) => {
                        self.act(&name);
                        true
                    }
                    None => {
                        self.orbit.down(at);
                        false
                    }
                }
            }
            PEvent::PointerMove(at) => self.orbit.moved(at, theme),
            PEvent::PointerUp(at) => {
                let pressed = self.orbit.up();
                let way = self.ball.borrow().iter().find(|(rect, _)| rect.contains(place(at))).map(|(_, way)| *way);
                match (pressed, way) {
                    (true, Some(way)) => {
                        self.way.set(Some(way));
                        self.orbit.reset();
                        true
                    }
                    _ => false,
                }
            }
            PEvent::Scroll(_, by) => self.orbit.wheel(by.y, theme),
            _ => false,
        }
    }
}
