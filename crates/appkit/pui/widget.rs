//! A widget: a description, not an object. What an app writes to say "a
//! button here, labelled so"; it builds views and controls. `PWidget`, with
//! `PStyle`, `PInput` and `PCard`.
//!
//! These four are puikit's traits (`old/pui/src/traits.rs`), with its
//! `NSView` and `NSColor` replaced by `PView` and `PColor`.

use crate::pui::core::{PColor, PView};

/// Anything that can be made into a view.
pub trait PWidget {
    /// Use the description up and give the view it describes.
    fn build(self) -> Box<dyn PView>;
}

/// How a widget looks, said in a chain: `label.color(c).padding(4.0)`.
pub trait PStyle: Sized {
    /// The colour of its text or mark.
    fn color(self, color: PColor) -> Self;

    /// The colour behind it.
    fn background(self, color: PColor) -> Self;

    /// How round its corners are.
    fn corner_radius(self, radius: f32) -> Self;

    /// The room around its content, the same on every side.
    fn padding(self, padding: f32) -> Self;

    fn width(self, width: f32) -> Self;

    fn height(self, height: f32) -> Self;
}

/// What a widget does when a hand uses it, said in a chain.
pub trait PInput: Sized {
    /// What to run when it is clicked.
    fn on_click<F>(self, callback: F) -> Self
    where
        F: FnMut() + 'static;

    /// Whether a hand can use it.
    fn enabled(self, enabled: bool) -> Self;
}

/// A widget that holds others of one kind: a row its cells, a form its
/// fields.
pub trait PCard<T>: Sized {
    /// Put one more in, after those already there.
    fn add(self, item: T) -> Self;

    /// Put several in, in their order.
    fn extend<I: IntoIterator<Item = T>>(self, item: I) -> Self {
        item.into_iter().fold(self, Self::add)
    }
}
