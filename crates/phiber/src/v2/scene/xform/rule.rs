//! What the keys and the operator do while a transform runs — Blender's
//! `transform_modal_keymap`: X / Y / Z hold it to an axis, Shift X / Y / Z
//! to a plane, digits type a value, Return keeps it, Escape puts it back —
//! and `ctx.op.transform`, which starts one, switches its mode, and ends it.

use super::{BConstraint, BTransformModal, BTransformMode};
use crate::v2::scene::pick::BPickView;
use crate::v2::scene::transform::BTransforms;
use crate::v2::state::{BKey as _, BPath, BState, CtxKey, OpKey, SceneKey, View3dKey, log, rule::BRule};
use crate::v2::view::prim::BAction;

impl BTransformModal {
    /// What `key` (`shift` held) does while a transform is under way, as
    /// `(action, name)`; `None` when none is, or the key means nothing to it.
    pub fn key(key: &str, shift: bool, s: &BState) -> Option<(BAction, String)> {
        let modal = Self::now(s)?;
        let op = |value: String| BAction::Set(OpKey::Transform.path(), value);
        Some(match key {
            "x" | "y" | "z" if shift => (op(format!("plane:{key}")), format!("transform: lock {}", key.to_uppercase())),
            "x" | "y" | "z" => (op(format!("axis:{key}")), format!("transform: along {}", key.to_uppercase())),
            "enter" | "return" => (op("confirm".into()), "transform: confirm".to_string()),
            "escape" => (op("cancel".into()), "transform: cancel".to_string()),
            typed => {
                modal.typed.with(typed)?;
                (op(format!("type:{typed}")), format!("transform: type {typed}"))
            }
        })
    }

    /// The undo step and the log line an operator value makes, `before`
    /// the state it was set on: a kept transform is named for its mode.
    pub fn describe(value: &str, before: &BState) -> Option<String> {
        match (BTransformMode::from_slug(value), value) {
            (Some(mode), _) => Some(mode.label().to_string()),
            (None, "confirm") => Self::now(before).map(|modal| modal.mode.label().to_string()),
            _ => None,
        }
    }

    /// Write `self` and where it puts the objects now, in the view under the
    /// pointer — a new constraint or a typed number applies at once.
    fn write_applied(&self, s: &mut BState) {
        s.set(View3dKey::Transform, self.encode());
        let size = crate::v2::scene::camera::last_size();
        match self.applied(&BPickView::now(s), size) {
            Some(edits) => edits.write(s),
            None => s.set(SceneKey::Transformed, self.before.clone()),
        }
    }
}

/// `ctx.op.transform`: start, switch mode, hold to an axis or a plane, type,
/// keep or put back.
pub struct BTransformOpRule;

impl BRule for BTransformOpRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::Transform.path()
    }
    fn after(&self, _: &BPath, value: &str, s: &mut BState) {
        s.remove(OpKey::Transform);
        let modal = BTransformModal::now(s);
        match (value, modal) {
            // G during R: the same objects, the other mode, from the start.
            (mode, Some(modal)) if BTransformMode::from_slug(mode).is_some() => {
                s.set(SceneKey::Transformed, modal.before.clone());
                let mode = BTransformMode::from_slug(mode).unwrap_or(modal.mode);
                s.set(View3dKey::Transform, BTransformModal { mode, from: modal.at, typed: Default::default(), ..modal }.encode());
            }
            (mode, None) => {
                if let Some(mode) = BTransformMode::from_slug(mode) {
                    BTransformModal::start(s, mode);
                }
            }
            ("confirm", Some(modal)) => {
                s.remove(View3dKey::Transform);
                log::push(format!("{} {} object(s)", modal.mode.label(), modal.list.len()));
            }
            ("cancel", Some(modal)) => {
                s.set(SceneKey::Transformed, modal.before);
                s.remove(View3dKey::Transform);
            }
            (other, Some(modal)) => {
                let next = match other.split_once(':') {
                    Some(("axis", axis)) => BTransformModal { constraint: modal.constraint.pressed(axis, false), ..modal },
                    Some(("plane", axis)) => BTransformModal { constraint: modal.constraint.pressed(axis, true), ..modal },
                    Some(("type", key)) => match modal.typed.with(key) {
                        Some(typed) => BTransformModal { typed, ..modal },
                        None => modal,
                    },
                    _ => modal,
                };
                next.write_applied(s);
            }
        }
        // The panel shows what the active object now has.
        if let Some(active) = s.get(CtxKey::Object).map(str::to_string) {
            if let Some(t) = BTransforms::now(s).get(&active) {
                crate::v2::part::props::active::set::seed_edited(s, &t);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_constraint_is_pressed_once_and_freed_by_the_same_key() {
        assert_eq!(BConstraint::Free.pressed("y", false).pressed("y", false), BConstraint::Free);
    }
}
