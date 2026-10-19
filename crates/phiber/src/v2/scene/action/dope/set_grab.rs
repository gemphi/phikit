//! The grab arms of the Dope Sheet operator match — `grab:start`,
//! `grab:at:<frame>`, `grab:confirm`, `grab:cancel`: the four moments of a
//! time-translation gesture. Split from `set_edit.rs` so each file sits
//! under the 240-line read cap (`tests/level_structure.rs`); the dispatcher
//! in `set.rs` routes every non-pick arm through `set_edit`, which in turn
//! routes the grab arms here.

use super::super::BSceneActions;
use super::{BKeyGrab, BKeyPick, BKeySelection, BKeyTarget};
use crate::phiphx::scene::SceneModel;
use crate::v2::state::{BKey as _, BState, CtxKey, log};
use std::sync::Arc;

/// Handle one grab sub-op. `Some(picks)` means the caller should write those
/// picks back to state; `None` means the arm handled everything itself (and
/// may have logged a reason there was nothing to do, or scheduled its own
/// state write via `BKeyGrab::write`).
pub(super) fn grab(
    head: (&str, Option<&str>, Option<&str>),
    s: &mut BState,
    model: &Arc<SceneModel>,
    picks: BKeySelection,
) -> Option<BKeySelection> {
    match head {
        ("grab", Some("start"), from) => {
            if picks.0.is_empty() {
                log::push("No keyframes selected to move".to_string());
                return None;
            }
            let base = BSceneActions::now(s).keyed.len();
            BKeyGrab { from: from.and_then(|f| f.parse().ok()), base, picks: picks.clone() }.write(s);
            None
        }
        ("grab", Some("at"), Some(frame)) => {
            let (Some(mut grab), Ok(frame)) = (BKeyGrab::now(s), frame.parse::<i32>()) else { return None };
            let Some(from) = grab.from else {
                grab.from = Some(frame);
                grab.write(s);
                return None;
            };
            let by = (frame - from) as f32;
            let mut edits = BSceneActions::now(s);
            edits.keyed.truncate(grab.base);
            if by != 0.0 {
                // The keys as they stood when the grab began.
                let before = edits.keyed_actions(&super::super::set::BActionScene::raw(model.clone()));
                edits.keyed.extend(BKeyTarget::steps(&before, &grab.picks, Some(by)));
            }
            edits.write(s);
            Some(BKeySelection(
                grab.picks.0.iter().map(|p| BKeyPick { object: p.object.clone(), frame: p.frame + by }).collect(),
            ))
        }
        ("grab", Some("confirm"), _) => {
            let Some(grab) = BKeyGrab::now(s) else { return None };
            s.remove(CtxKey::KeyGrab);
            let by = picks.0.first().zip(grab.picks.0.first()).map_or(0.0, |(a, b)| a.frame - b.frame);
            log::push(format!("Moved {} keyframe column(s) by {by} frame(s)", picks.0.len()));
            None
        }
        ("grab", Some("cancel"), _) => {
            let Some(grab) = BKeyGrab::now(s) else { return None };
            s.remove(CtxKey::KeyGrab);
            let mut edits = BSceneActions::now(s);
            edits.keyed.truncate(grab.base);
            edits.write(s);
            Some(grab.picks)
        }
        _ => None,
    }
}
