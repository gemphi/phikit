//! Scrolling: how far a content longer than its room is moved, and no
//! further than its ends. A view keeps one, tells it how long its content
//! and its room are when it draws, and turns it by what the wheel gives.

/// How far a content is scrolled in its room.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PScroll {
    at: f32,
    far: f32,
}

impl PScroll {
    /// Say how long the content and the room are. What is scrolled stays
    /// within what does not fit.
    pub fn fit(&mut self, content: f32, room: f32) {
        self.far = (content - room).max(0.0);
        self.at = self.at.clamp(0.0, self.far);
    }

    /// Scroll by `by`, no further than either end. `true` when there is
    /// anything to scroll: the content is longer than the room.
    pub fn turn(&mut self, by: f32) -> bool {
        self.at = (self.at + by).clamp(0.0, self.far);
        self.far > 0.0
    }

    /// Go back to where the content begins.
    pub fn home(&mut self) {
        self.at = 0.0;
    }

    /// How far it is scrolled.
    pub fn at(&self) -> f32 {
        self.at
    }
}
