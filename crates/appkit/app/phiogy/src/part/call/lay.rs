//! Where the tiles of a call stand, and how high the call is. A row of
//! tiles folds where the next does not fit. The call takes no more of
//! the stream's height than the theme allows: where its tiles would
//! need more, all of them are made smaller, a step at a time, so what
//! stands under them, the pills and what the call says, is never cut.

use appkit::pui::core::{PPoint, PRect};

use super::{Calling, FACE, HIGH, LOW, SCENE, SHARE, Shown, Tile, WIDE};
use crate::part::present::BAR;
use crate::part::{GAP, LINE};

/// The theme's tokens: the share of its size a tile keeps at each step
/// smaller, and the least share it is made.
pub const LESS: &str = "call.less";
pub const LEAST: &str = "call.least";
/// How wide the room for a name under a tile is at least: a face made
/// small still has its person's name under it.
pub const NAME: &str = "call.name";

impl Calling {
    /// How wide and how high a tile's picture is where the part is that
    /// wide, at that share of its full size, and how high the lines
    /// under it are.
    fn size(&self, tile: &Tile, width: f32, share: f32) -> (f32, f32, f32) {
        let size = |token: &str| self.desk.size(token);
        let under = [&tile.name, &tile.said].iter().filter(|said| !said.is_empty()).count() as f32 * size(LINE);
        let wide = |token: &str| (size(token) * share).min(width);
        match (&tile.shown, tile.scene) {
            (Shown::Face(..), _) => (wide(FACE), wide(FACE), under),
            (Shown::Film(_), true) => (wide(SCENE), wide(SCENE) * size(LOW), under),
            (Shown::Film(_), false) => (wide(WIDE), wide(WIDE) * size(HIGH), under),
        }
    }

    /// Where each tile's picture stands in a part that wide, from its
    /// own top left, a row folding where the next does not fit; and how
    /// high they are together.
    pub fn lay(&self, tile: &[Tile], width: f32, share: f32) -> (Vec<PRect>, f32) {
        let gap = self.desk.size(GAP);
        let inside = (width - gap * 2.0).max(0.0);
        let (mut at, mut row, mut rect) = (PPoint::new(gap, 0.0), 0.0f32, Vec::new());
        for one in tile {
            let (wide, high, under) = self.size(one, inside, share);
            // The picture stands in the middle of the room its name has.
            let room = wide.max(self.desk.size(NAME).min(inside));
            if at.x > gap && at.x + room > gap + inside {
                at = PPoint::new(gap, at.y + row + gap);
                row = 0.0;
            }
            rect.push(PRect::new(at.x + (room - wide) / 2.0, at.y, wide, high));
            at.x += room + gap;
            row = row.max(high + under);
        }
        (rect, at.y + row)
    }

    /// How high the call is with its tiles at that share of their size:
    /// the head, the tiles, the bar, the pills and what is said.
    fn whole(&self, tile: &[Tile], width: f32, share: f32) -> f32 {
        let (gap, line) = (self.desk.size(GAP), self.desk.size(LINE));
        let (_, tiles) = self.lay(tile, width - gap, share);
        gap + line + gap / 2.0 + tiles + gap + self.desk.size(BAR) + gap / 2.0 + (line + gap + gap / 2.0) * self.row.get().max(1.0) + line * self.line.get().max(1.0) + gap
    }

    /// How high the call stands in a room that wide, with that much
    /// height to share: nothing when this window is in no call. The
    /// share of their size the tiles are drawn at is kept for the draw.
    pub fn high(&self, width: f32, room: f32) -> f32 {
        let tile = self.tile();
        // A voice room has no tiles: who is in it is the room itself.
        if tile.is_empty() || self.desk.voice() {
            return 0.0;
        }
        let most = room * self.desk.size(SHARE);
        let (less, least) = (self.desk.size(LESS).clamp(0.1, 0.99), self.desk.size(LEAST));
        let mut share = 1.0;
        while self.whole(&tile, width, share) > most && share * less >= least {
            share *= less;
        }
        self.share.set(share);
        self.whole(&tile, width, share).min(most)
    }
}
