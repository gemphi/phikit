//! A view: something drawn that has a rectangle, takes events and holds
//! other views. The earlier appkit's `NSView` was one struct every view
//! shared; here it is a trait, so that a stack, a table and a button are
//! each their own type.

use super::{PDraw, PEvent, PPaint, PPoint, PRect};

/// Something drawn that has a rectangle, takes events and holds other
/// views.
pub trait PView: PDraw {
    /// Where the view is, in the coordinates of the view that holds it.
    fn frame(&self) -> PRect;

    /// Put the view somewhere else, or make it another size.
    fn set_frame(&mut self, frame: PRect);

    /// Take an event, its place in the view's own coordinates. `true` when
    /// it was handled, and nobody else is asked; `false` passes it up to
    /// the view that holds this one. A view that handles nothing need not
    /// write this.
    fn respond(&mut self, _event: &PEvent) -> bool {
        false
    }

    /// Called once a frame, before the window is drawn: the moment to take
    /// in what has arrived since, such as more of an answer a model is
    /// writing. `true` while there is more to come, which has the screen
    /// draw another frame. A view that changes only when a hand does
    /// something need not write this.
    fn tick(&mut self) -> bool {
        false
    }

    /// Take a word from the window's own furniture, such as `begin` from
    /// a row of the activity bar: something only the app can do. `true`
    /// when this view did it. A view that does nothing of the kind need
    /// not write this.
    fn tell(&mut self, _word: &str) -> bool {
        false
    }

    /// Where a press on it does something, in the window: the pointer
    /// is a hand there. A view that says nothing has the pointer as it
    /// is everywhere.
    fn hand(&self) -> Vec<PRect> {
        Vec::new()
    }

    /// The views it holds, the lowest first: a later one is drawn over an
    /// earlier one, and is asked first when the pointer is over both.
    fn child_list(&self) -> &[Box<dyn PView>] {
        &[]
    }

    fn child_list_mut(&mut self) -> &mut [Box<dyn PView>] {
        &mut []
    }
}

/// Draw a view and everything in it. `origin` is where the view's parent
/// has its top left in the window.
pub fn show(view: &dyn PView, paint: &mut dyn PPaint, origin: PPoint) {
    let own = origin.plus(view.frame().origin);
    view.draw(paint, own);
    view.child_list().iter().for_each(|child| show(child.as_ref(), paint, own));
}

/// Send an event to a view. An event with a place goes to the innermost
/// view under it, and is passed up from there until somebody handles it;
/// a view that the pointer is not over is not asked. A key goes to the
/// view itself, since nothing here says yet who is listening. `true` when
/// it was handled.
///
/// The event's place is in the coordinates of the view's parent, as the
/// view's frame is; `respond` gets it in the view's own.
pub fn send(view: &mut dyn PView, event: &PEvent) -> bool {
    let frame = view.frame();
    let Some(at) = event.at() else {
        return view.respond(event);
    };
    if !frame.contains(at) {
        return false;
    }
    let own = event.seen_from(frame.origin);
    let taken = view.child_list_mut().iter_mut().rev().any(|child| send(child.as_mut(), &own));
    taken || view.respond(&own)
}
