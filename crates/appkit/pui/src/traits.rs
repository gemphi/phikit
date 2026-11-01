//! Core PUI traits.
//!
//! Canonical names: `PWidget`, `PStyle`, `PInteractive`, `PContainer`, `PTerse`.
//! Short aliases (`Pwgt`, `Psty`, `Pint`, `Pcnt`, `Ptrs`) live in `aliases::traits`.

use rust_appkit::prelude::*;
use std::sync::Arc;

/// Anything that can be turned into an `NSView`. Swift: `View` protocol.
pub trait PWidget {
    /// Consumes the widget and produces a final `NSView`. Swift: `body`
    fn build(self) -> NSView;
}

/// Chainable styling. Swift: `.foregroundColor`, `.background`, `.padding` modifiers.
pub trait PStyle: Sized {
    /// Foreground / text color. Swift: `.foregroundColor(_:)`
    fn color(self, color: NSColor) -> Self;
    
    /// Background fill color. Swift: `.background(_:)`
    fn background(self, color: NSColor) -> Self;
    
    /// Corner radius for rounded shapes. Swift: `.cornerRadius(_:)`
    fn corner_radius(self, radius: f32) -> Self;
    
    /// Symmetric padding around the content. Swift: `.padding(_:)`
    fn padding(self, padding: f32) -> Self;
    
    /// Explicit width. Swift: `.frame(width:)`
    fn width(self, width: f32) -> Self;
    
    /// Explicit height. Swift: `.frame(height:)`
    fn height(self, height: f32) -> Self;
}

/// Chainable interaction handlers. Swift: `Button { action }`.
pub trait PInteractive: Sized {
    /// Click / activation callback. Swift: `Button(action:)` closure
    fn on_click<F>(self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static;
    
    /// Enable / disable the widget. Swift: `.disabled(_:)` (inverse)
    fn enabled(self, enabled: bool) -> Self;
}

/// A widget that can hold typed children. Swift: `VStack<Content>` generic content.
pub trait PContainer<T>: Sized {
    /// Adds a child item. Swift: `@ViewBuilder` content append
    fn add(self, item: T) -> Self;
    
    /// Adds many children at once.
    fn extend<I: IntoIterator<Item = T>>(self, items: I) -> Self;
}

/// Compact 3-letter ops shared across PUI generics. Swift: short modifier closures.
///
/// | Op       | Meaning                                        |
/// |----------|------------------------------------------------|
/// | `var`    | Variable / value identifier (string key)       |
/// | `r#let`  | Constant / literal value                       |
/// | `has`    | Add child / content                            |
/// | `sub`    | Add subtitle / secondary text                  |
/// | `r#pub`  | Publish callback                               |
/// | `ops`    | Inline mutator block                           |
pub trait PTerse: Sized {
    /// Sets the variable / value key. Swift: `binding.id`
    fn var(self, name: impl Into<String>) -> Self;
    
    /// Sets a literal / fixed value. Swift: `let value = ...`
    fn r#let(self, value: impl Into<String>) -> Self;
    
    /// Adds child content. Requires `W: PWidget`. Swift: `@ViewBuilder content`
    fn has<W: PWidget>(self, widget: W) -> Self;
    
    /// Sets a subtitle / secondary text. Swift: `Text` placed below primary text
    fn sub(self, text: impl Into<String>) -> Self;
    
    /// Publishes a callback. Swift: action closure
    fn r#pub<F>(self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static;
    
    /// Inline mutator — applies an arbitrary block to self. Swift: `.modifier(...)`
    fn ops<F>(self, block: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        block(self)
    }
}

/// Type-erased boxed callback used by `r#pub`.
pub type PCallback = Arc<dyn Fn() + Send + Sync>;
