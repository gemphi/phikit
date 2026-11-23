//! Tests of `core`, one file for each file there. This file holds what the
//! others draw on and send events to.

mod asset;
mod event;
mod figure;
mod film;
mod icon;
mod light;
mod mode;
mod paint;
mod sheet;
mod view;

use std::cell::RefCell;
use std::rc::Rc;

use crate::pui::core::*;
use crate::theme::PTheme;

/// A theme for the tests: the tokens the controls are drawn with.
pub(crate) struct Plain;

impl PTheme for Plain {
    fn color(&self, token: &str) -> Option<PColor> {
        match token {
            "control.fill" => Some(PColor::new(0.8, 0.8, 0.8, 1.0)),
            "control.ink" => Some(PColor::new(0.0, 0.0, 0.0, 1.0)),
            "control.accent" => Some(PColor::new(0.0, 0.4, 1.0, 1.0)),
            _ => None,
        }
    }

    fn size(&self, token: &str) -> Option<f32> {
        match token {
            "control.inset" => Some(4.0),
            "control.line" => Some(2.0),
            _ => None,
        }
    }
}

/// A surface that remembers what was drawn on it, in order.
#[derive(Default)]
pub(crate) struct Board {
    pub fill: Vec<(PRect, PColor)>,
    pub text: Vec<(PPoint, String)>,
    pub icon: Vec<(String, PRect)>,
}

impl PPaint for Board {
    fn theme(&self) -> &dyn PTheme {
        &Plain
    }

    fn fill(&mut self, rect: PRect, color: PColor) {
        self.fill.push((rect, color));
    }

    fn text(&mut self, at: PPoint, text: &str, _color: PColor) {
        self.text.push((at, text.to_string()));
    }

    fn icon(&mut self, name: &str, rect: PRect, _color: PColor) {
        self.icon.push((name.to_string(), rect));
    }
}

/// What the tiles of one test were sent, in order: who, and what.
pub(crate) type Seen = Rc<RefCell<Vec<(&'static str, PEvent)>>>;

/// A view for the tests: a coloured rectangle that writes down every event
/// it is sent and handles it or not.
pub(crate) struct Tile {
    pub name: &'static str,
    pub frame: PRect,
    pub color: PColor,
    pub handle: bool,
    pub child: Vec<Box<dyn PView>>,
    pub seen: Seen,
}

impl Tile {
    pub fn new(name: &'static str, frame: PRect, handle: bool, seen: &Seen, child: Vec<Box<dyn PView>>) -> Box<Tile> {
        Box::new(Tile { name, frame, color: PColor::new(1.0, 0.0, 0.0, 1.0), handle, child, seen: Rc::clone(seen) })
    }
}

impl PDraw for Tile {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        paint.fill(PRect { origin, size: self.frame.size }, self.color);
    }
}

impl PView for Tile {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        self.seen.borrow_mut().push((self.name, *event));
        self.handle
    }

    fn child_list(&self) -> &[Box<dyn PView>] {
        &self.child
    }

    fn child_list_mut(&mut self) -> &mut [Box<dyn PView>] {
        &mut self.child
    }
}

/// The names of who was sent something, in order.
pub(crate) fn asked(seen: &Seen) -> Vec<&'static str> {
    seen.borrow().iter().map(|(name, _)| *name).collect()
}
