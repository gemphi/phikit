//! An animation editor's visible frame range — Blender's `View2D.cur`
//! (`xmin`, `xmax`) of its window region — under `ctx.time_view.<editor>`,
//! one per editor kind: the Timeline, the Dope Sheet, the Graph editor and
//! the NLA editor each keep their own, as each region does in Blender.
//! Unset until the view is zoomed, scrolled or framed: the editor opens on
//! its `*_create` default (`canvas/time/view/set.rs`).

use super::BKey;
use crate::v2::host::BHostType;
use crate::v2::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimeViewKey(pub BHostType);

impl BKey for TimeViewKey {
    fn path(&self) -> BPath {
        BPath::owned(vec!["time_view", self.0.slug()])
    }
}
