//! The fields of the frame-range group, in Blender's order: `frame_current`
//! (unlabelled), `use_preview_range` as an icon toggle, then `frame_start` /
//! `frame_end` labelled Start / End — or `frame_preview_start` /
//! `frame_preview_end` while the preview range is on. Frames are clamped to
//! Blender's `MINAFRAME..MAXFRAME`.

use crate::v2::state::{BKey as _, BPath, BState, CtxKey, val::{BValue, V_TRUE}};
use crate::v2::view::prim::{BAction, BControl, BCtrl, BCtrlKind};
use crate::v2::view::BView;

/// `DNA_scene_types.h`: `MINAFRAME` and `MAXFRAME`.
pub const MIN_FRAME: i64 = -1_048_574;
pub const MAX_FRAME: i64 = 1_048_574;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BRangeField {
    Current,
    PreviewToggle,
    Start,
    End,
}

impl BRangeField {
    pub const ALL: [Self; 4] = [Self::Current, Self::PreviewToggle, Self::Start, Self::End];

    /// Whether the preview range is what the Start / End fields edit.
    pub fn previewing(s: &BState) -> bool {
        s.get(CtxKey::PreviewRange) == Some(V_TRUE)
    }

    /// The key a number field edits; the toggle has none.
    pub fn key(self, s: &BState) -> Option<BPath> {
        match (self, Self::previewing(s)) {
            (Self::Current, _) => Some(CtxKey::Frame.path()),
            (Self::PreviewToggle, _) => None,
            (Self::Start, false) => Some(CtxKey::FrameStart.path()),
            (Self::End, false) => Some(CtxKey::FrameEnd.path()),
            (Self::Start, true) => Some(CtxKey::PreviewStart.path()),
            (Self::End, true) => Some(CtxKey::PreviewEnd.path()),
        }
    }

    /// Blender's factory value when the state has none.
    pub fn default_value(self) -> i64 {
        match self {
            Self::Current | Self::Start => 1,
            Self::End => 250,
            Self::PreviewToggle => 0,
        }
    }

    pub fn label(self) -> Option<&'static str> {
        match self {
            Self::Start => Some("Start"),
            Self::End => Some("End"),
            Self::Current | Self::PreviewToggle => None,
        }
    }

    pub fn view(self, s: &BState) -> BView {
        let Some(key) = self.key(s) else {
            return BView::Button(
                BControl::new()
                    .with_icon("preview")
                    .with_tooltip("Use Preview Range")
                    .with_active(Self::previewing(s))
                    .with_action(BAction::Toggle(CtxKey::PreviewRange.path())),
            );
        };
        let kind = BCtrlKind::Int { min: MIN_FRAME, max: MAX_FRAME };
        let mut ctrl = BCtrl::read(kind, key, BValue::Int(self.default_value()), s);
        if let Some(label) = self.label() {
            ctrl = ctrl.with_label(label);
        }
        BView::Ctrl(Box::new(ctrl))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::action::apply_action;

    #[test]
    fn start_and_end_edit_the_preview_range_while_it_is_on() {
        let mut s = BState::default();
        assert_eq!(BRangeField::Start.key(&s), Some(CtxKey::FrameStart.path()));
        assert_eq!(BRangeField::End.key(&s), Some(CtxKey::FrameEnd.path()));
        apply_action(&mut s, &BAction::Toggle(CtxKey::PreviewRange.path()));
        assert_eq!(BRangeField::Start.key(&s), Some(CtxKey::PreviewStart.path()));
        assert_eq!(BRangeField::End.key(&s), Some(CtxKey::PreviewEnd.path()));
        assert_eq!(BRangeField::Current.key(&s), Some(CtxKey::Frame.path()));
        assert_eq!(BRangeField::PreviewToggle.key(&s), None);
    }
}
