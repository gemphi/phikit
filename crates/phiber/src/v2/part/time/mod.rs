//! Timeline centre content — Blender's Dope Sheet window in Timeline mode.
//!
//! The window is a canvas (`view/draw/canvas/time`): the frame-number strip,
//! the grid, the scene range, the key columns of the shown objects and the
//! playhead, scrubbed with the left button. The transport and the frame
//! fields sit in the header (`part::playback`, `part::framerange`), where
//! `playback_controls` puts them; the frame rule that the buttons and the
//! ticker drive is [`step`].

pub mod range;
pub mod step;

use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;

#[derive(Clone, Copy, Debug, Default)]
pub struct BTimeItem;

impl BTimeItem {
    pub fn new() -> Self {
        Self
    }
}

impl BViewItem for BTimeItem {
    fn build(&self, _s: &BState) -> BView {
        BView::canvas(BHostType::Time)
    }
}

#[cfg(test)]
mod tests {
    use crate::v2::state::{BKey as _, BState, CtxKey, action::apply_action};
    use crate::v2::view::BAction;

    #[test]
    fn steps_wrap_inside_the_scene_range() {
        let mut s = BState::default();
        apply_action(&mut s, &BAction::Set(CtxKey::FrameStep.path(), "end".into()));
        assert_eq!(s.get(CtxKey::Frame), Some("250"));
        apply_action(&mut s, &BAction::Set(CtxKey::FrameStep.path(), "+1".into()));
        assert_eq!(s.get(CtxKey::Frame), Some("1"), "wraps to start");
        apply_action(&mut s, &BAction::Set(CtxKey::FrameStep.path(), "-1".into()));
        assert_eq!(s.get(CtxKey::Frame), Some("250"));
        assert_eq!(s.get(CtxKey::FrameStep), None);
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BTimeList;

impl BTimeList {
    pub fn new() -> Self {
        Self
    }
}

impl BViewList for BTimeList {
    fn items(&self, s: &BState) -> Vec<BView> {
        vec![BTimeItem::new().build(s)]
    }
}
