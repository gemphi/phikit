//! # ViewBuilder Trait
//!
//! The core abstraction for declarative UI. Any screen, panel, or component
//! implements `ViewBuilder` to describe its view hierarchy as a `ViewNode` tree.
//! The layout engine then resolves that tree into absolute frames.

use super::view_node::ViewNode;

/// Declarative view description trait — the SwiftUI `View` protocol equivalent.
///
/// Implementors return a `ViewNode` tree from `body()`. The framework calls
/// `body()` whenever the view needs to be rebuilt (e.g. after state changes),
/// then feeds the result into the layout engine to produce absolute frames.
///
/// # Example
///
/// ```ignore
/// use rust_appkit::layout_engine::*;
///
/// struct WelcomeScreen {
///     title: String,
/// }
///
/// impl ViewBuilder for WelcomeScreen {
///     fn body(&self) -> ViewNode {
///         ViewNode::vstack(12.0, vec![
///             ViewNode::text(&self.title, TextStyle::title()),
///             ViewNode::text("A 3D content creation tool", TextStyle::caption()),
///             ViewNode::spacer(20.0),
///             ViewNode::button("Get Started", ButtonStyle::accent_filled(), None),
///         ])
///         .with_padding(Edges::uniform(24.0))
///     }
/// }
/// ```
pub trait ViewBuilder {
    /// Produce the declarative view tree for this component.
    fn body(&self) -> ViewNode;
}

/// Blanket implementation: a closure that returns a ViewNode is a ViewBuilder.
impl<F> ViewBuilder for F
where
    F: Fn() -> ViewNode,
{
    fn body(&self) -> ViewNode {
        (self)()
    }
}
