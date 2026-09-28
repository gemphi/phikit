//! How a `BFlex` takes space in its parent: whether it grows, shrinks, keeps
//! its size, measures by its content, and whether its own children wrap.

use super::element::BFlex;
use super::token::BBasis;

impl BFlex {
    /// Takes the free space along its parent's axis, and may shrink below its
    /// content (`flex_1`).
    pub const fn fills(self) -> Self {
        self.shares(1.0)
    }

    /// Takes `fraction` of the free space along its parent's axis.
    pub const fn shares(mut self, fraction: f32) -> Self {
        self.grow = Some(fraction);
        self.shrink = Some(1.0);
        self.basis = Some(BBasis::NoShare);
        self
    }

    /// One of a set of equal cells that divide the row by grow alone.
    pub const fn equal_cell(mut self) -> Self {
        self.grow = Some(1.0);
        self.basis = Some(BBasis::NoPixels);
        self
    }

    /// Keeps its size: never shrinks below it.
    pub const fn keeps_size(mut self) -> Self {
        self.shrink = Some(0.0);
        self
    }

    /// Takes the free space along its parent's axis, measured by its own
    /// content and never below it — in a wrapping row, the child that moves
    /// to the next line rather than squeezing.
    pub const fn grows(mut self) -> Self {
        self.grow = Some(1.0);
        self.shrink = Some(0.0);
        self.basis = Some(BBasis::Content);
        self
    }

    /// Lines its children up along the axis and starts a new line when they
    /// no longer fit.
    pub const fn wraps(mut self) -> Self {
        self.wrap = true;
        self
    }

    /// May shrink below its share.
    pub const fn yields(mut self) -> Self {
        self.shrink = Some(1.0);
        self
    }
}
