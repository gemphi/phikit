//! `image.view_zoom_in` / `image.view_zoom_out` from the View ▸ Zoom menu:
//! the menu cannot know the zoom a fitted picture shows at, so it writes
//! `+` or `-` and this rule steps the zoom in force — the last ratio set,
//! or 1:1 when the picture was fitted.

use super::{ZOOM_MAX, ZOOM_MIN, ZOOM_STEP};
use crate::v2::state::{BKey as _, BPath, BState, ImageKey, rule::BRule};

pub struct BImageZoomStepRule;

impl BRule for BImageZoomStepRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == ImageKey::Zoom.path()
    }
    fn after(&self, _: &BPath, value: &str, state: &mut BState) {
        let factor = match value {
            "+" => ZOOM_STEP,
            "-" => 1.0 / ZOOM_STEP,
            _ => return,
        };
        let last = LAST.with(|cell| cell.get());
        let next = (last * factor).clamp(ZOOM_MIN, ZOOM_MAX);
        LAST.with(|cell| cell.set(next));
        state.set(ImageKey::Zoom, next.to_string());
    }
}

thread_local! {
    /// The last ratio the rule stepped to, so `+` then `-` lands back.
    static LAST: std::cell::Cell<f32> = const { std::cell::Cell::new(1.0) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::action::apply_action;
    use crate::v2::view::prim::BAction;

    #[test]
    fn plus_and_minus_step_the_ratio_and_a_ratio_stays_a_ratio() {
        let mut state = BState::new();
        apply_action(&mut state, &BAction::Set(ImageKey::Zoom.path(), "+".into()));
        let stepped: f32 = state.get(ImageKey::Zoom).unwrap().parse().unwrap();
        assert!((stepped - ZOOM_STEP).abs() < 1e-5);
        apply_action(&mut state, &BAction::Set(ImageKey::Zoom.path(), "-".into()));
        let back: f32 = state.get(ImageKey::Zoom).unwrap().parse().unwrap();
        assert!((back - 1.0).abs() < 1e-5);
        apply_action(&mut state, &BAction::Set(ImageKey::Zoom.path(), "0.5".into()));
        assert_eq!(state.get(ImageKey::Zoom), Some("0.5"));
    }
}
