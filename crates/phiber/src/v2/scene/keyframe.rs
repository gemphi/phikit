//! The key columns the Timeline shows.
//!
//! Blender's Timeline is a Dope Sheet with its summary collapsed and
//! "Only Show Selected" on (`space_action.cc`, `action_create`): one row of
//! diamonds, one per frame that carries a key on any channel of a selected
//! object. The active object counts as shown, as it is nearly always
//! selected too and is what the header's keyframe jump is for.

use crate::phiphx::scene::SceneAction;
use crate::v2::state::{BState, CtxKey};

/// The merged, sorted key columns of the objects the editor shows.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BShownKeys(Vec<f32>);

impl BShownKeys {
    /// Built from the loaded scene's actions and view-layer selection.
    pub fn now(s: &BState) -> Self {
        let active = s.get(CtxKey::Object).map(str::to_string);
        super::source::with_loaded(|loaded| {
            let Some(model) = loaded else { return Self::default() };
            let mut shown: Vec<&str> = model.selection.selected.iter().map(String::as_str).collect();
            shown.extend(active.as_deref());
            Self::of(&model.actions, &shown)
        })
    }

    /// The columns of every action one of `objects` uses.
    pub fn of(actions: &[SceneAction], objects: &[&str]) -> Self {
        let mut columns: Vec<f32> = actions
            .iter()
            .filter(|action| objects.iter().any(|object| action.used_by(object)))
            .flat_map(|action| action.keys.iter().copied())
            .collect();
        columns.sort_by(f32::total_cmp);
        columns.dedup();
        Self(columns)
    }

    pub fn columns(&self) -> &[f32] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(name: &str, user: &str, keys: &[f32]) -> SceneAction {
        SceneAction {
            name: name.into(),
            keys: keys.to_vec(),
            users: vec![user.into()],
            ..Default::default()
        }
    }

    #[test]
    fn only_the_shown_objects_actions_contribute_and_columns_merge() {
        let list = [action("Walk", "rig", &[10.0, 1.0]), action("Spin", "Cube", &[5.0, 10.0]), action("Idle", "Lamp", &[99.0])];
        assert_eq!(BShownKeys::of(&list, &["rig", "Cube"]).columns(), &[1.0, 5.0, 10.0]);
        assert_eq!(BShownKeys::of(&list, &["Camera"]).columns(), &[] as &[f32]);
    }
}
