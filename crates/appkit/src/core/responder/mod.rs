use crate::event::NSEvent;

pub mod undo;
pub use undo::*;

pub trait NSResponder: Send + Sync {
    fn mouse_down(&mut self, _event: &NSEvent) {}
    fn mouse_up(&mut self, _event: &NSEvent) {}
    fn mouse_moved(&mut self, _event: &NSEvent) {}
    fn key_down(&mut self, _event: &NSEvent) {}
    fn key_up(&mut self, _event: &NSEvent) {}
}

pub struct NSResponderChain {
    pub(crate) next_responder: Option<Box<dyn NSResponder>>,
}

impl NSResponderChain {
    pub fn new() -> Self {
        Self {
            next_responder: None,
        }
    }
}
