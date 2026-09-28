//! The selection: which objects are selected, and the one that is active.
//!
//! Blender ref: `Base.flag & BASE_SELECTED` per object and
//! `ViewLayer.basact` for the active one; `object.select_all` with
//! `action = SELECT | DESELECT | INVERT`. Here the set is `ctx.selected`
//! (`|`-separated names) beside the active `ctx.object`. Until a selection
//! is made, the active object is the selection — what a freshly opened file
//! shows — and picking an object (`ctx.object`) selects just it, as a plain
//! click does in Blender; Shift-click extends through `ctx.op.select`.

use crate::v2::state::{BKey as _, BPath, BState, CtxKey, OpKey, View3dKey, rule::BRule, val::V_TRUE};

/// The names between the separators.
const SEP: char = '|';

/// The selected objects.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BSelection {
    pub list: Vec<String>,
}

impl BSelection {
    /// What is selected now: the set when one was made, else the active object.
    pub fn now(state: &BState) -> Self {
        match state.get(CtxKey::Selected) {
            Some(raw) => Self::parse(raw),
            None => Self { list: state.get(CtxKey::Object).map(|name| vec![name.to_string()]).unwrap_or_default() },
        }
    }

    pub fn parse(raw: &str) -> Self {
        Self { list: raw.split(SEP).filter(|name| !name.is_empty()).map(str::to_string).collect() }
    }

    pub fn encode(&self) -> String {
        self.list.join(&SEP.to_string())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.list.iter().any(|selected| selected == name)
    }

    /// Write the set; an empty set is kept as `""`, which is "nothing".
    pub fn write(&self, state: &mut BState) {
        state.set(CtxKey::Selected, self.encode());
    }
}

/// Picking an object (`ctx.object`) selects just it.
pub struct BPickSelectsRule;

impl BRule for BPickSelectsRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == CtxKey::Object.path()
    }
    fn after(&self, _: &BPath, name: &str, state: &mut BState) {
        if name.is_empty() {
            return;
        }
        // Shift-click asked to extend first (`ctx.op.select = extend:<name>`):
        // the name is already in the set, so the click only makes it active.
        let mut selection = BSelection::now(state);
        if !selection.contains(name) {
            selection = BSelection { list: vec![name.to_string()] };
        }
        selection.write(state);
    }
}

/// `ctx.op.select` → `all` / `none` / `invert` over the visible objects,
/// `extend:<name>` adding one, `box` arming the box gesture and
/// `box:<a>,<b>` / `box+:` / `box-:` applying its result (`scene::boxsel`).
pub struct BSelectOpRule;

/// The names a box op carries, and the selection it leaves.
fn boxed(op: &str, current: &BSelection) -> Option<BSelection> {
    let (mode, names) = ["box+:", "box-:", "box:"].iter().find_map(|head| op.strip_prefix(head).map(|rest| (*head, rest)))?;
    let names: Vec<String> = names.split(',').filter(|n| !n.is_empty()).map(str::to_string).collect();
    let mut list = match mode {
        "box:" => Vec::new(),
        _ => current.list.clone(),
    };
    match mode {
        "box-:" => list.retain(|name| !names.contains(name)),
        _ => {
            for name in names {
                if !list.contains(&name) {
                    list.push(name);
                }
            }
        }
    }
    Some(BSelection { list })
}

impl BRule for BSelectOpRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == OpKey::Select.path()
    }
    fn after(&self, _: &BPath, op: &str, state: &mut BState) {
        // B: the next drag is the box (`boxsel`); C: the brush (`circlesel`).
        if let Some((armed, hint)) = match op {
            "box" => Some((View3dKey::BoxSelect, "Box Select: drag over the objects")),
            "circle" => Some((View3dKey::CircleSelect, "Circle Select: paint over the objects, right click to end")),
            _ => None,
        } {
            state.set(armed, V_TRUE);
            state.remove(OpKey::Select);
            crate::v2::state::log::push(hint.to_string());
            return;
        }
        let visible = visible_names(state);
        let current = BSelection::now(state);
        let next = match op {
            _ if op.starts_with("box") => {
                state.remove(View3dKey::BoxSelect);
                state.remove(View3dKey::BoxRect);
                boxed(op, &current).unwrap_or(current)
            }
            "all" => BSelection { list: visible },
            "none" => BSelection::default(),
            "invert" => BSelection { list: visible.into_iter().filter(|name| !current.contains(name)).collect() },
            extend => match extend.strip_prefix("extend:") {
                Some(name) if !current.contains(name) => {
                    let mut list = current.list.clone();
                    list.push(name.to_string());
                    BSelection { list }
                }
                _ => current,
            },
        };
        crate::v2::state::log::push(match op {
            "all" => format!("Selected {} object(s)", next.list.len()),
            "none" => "Deselected all".to_string(),
            "invert" => format!("Inverted the selection: {} object(s)", next.list.len()),
            _ => format!("Selected {} object(s)", next.list.len()),
        });
        next.write(state);
        state.remove(OpKey::Select);
    }
}

#[cfg(test)]
mod box_tests {
    use super::*;
    use crate::v2::state::action::apply_action;
    use crate::v2::view::prim::BAction;

    #[test]
    fn b_arms_the_box_and_its_result_sets_adds_or_subtracts() {
        let mut state = BState::new();
        state.set(CtxKey::Object, "Cube");
        apply_action(&mut state, &BAction::Set(OpKey::Select.path(), "box".into()));
        assert!(crate::v2::scene::boxsel::armed(&state) && state.get(OpKey::Select).is_none());
        state.set(View3dKey::BoxRect, "1,1,2,2");
        apply_action(&mut state, &BAction::Set(OpKey::Select.path(), "box:Sphere,Gem".into()));
        assert_eq!(BSelection::now(&state).list, ["Sphere", "Gem"], "the box sets the selection");
        assert!(!crate::v2::scene::boxsel::armed(&state) && state.get(View3dKey::BoxRect).is_none(), "the gesture is over");
        apply_action(&mut state, &BAction::Set(OpKey::Select.path(), "box+:Cube,Gem".into()));
        assert_eq!(BSelection::now(&state).list, ["Sphere", "Gem", "Cube"], "Shift adds");
        apply_action(&mut state, &BAction::Set(OpKey::Select.path(), "box-:Gem".into()));
        assert_eq!(BSelection::now(&state).list, ["Sphere", "Cube"], "Ctrl subtracts");
        apply_action(&mut state, &BAction::Set(OpKey::Select.path(), "box:".into()));
        assert!(BSelection::now(&state).list.is_empty(), "an empty box deselects");
    }
}

/// Every object the viewport shows that can be selected, by name — what
/// `object.select_all` runs over (`BASE_SELECTABLE`).
pub fn visible_names(state: &BState) -> Vec<String> {
    let hidden = crate::v2::part::outliner::key::BHidden::now(state);
    let locked = crate::v2::part::outliner::key::BHidden::for_select(state);
    super::source::scene_objects(state)
        .iter()
        .map(|object| object.name().to_string())
        .filter(|name| !hidden.hides(state, name) && !locked.hides(state, name))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::action::apply_action;
    use crate::v2::view::prim::BAction;

    #[test]
    fn the_active_object_is_the_selection_until_one_is_made() {
        let mut state = BState::new();
        state.set(CtxKey::Object, "Sphere");
        assert_eq!(BSelection::now(&state).list, ["Sphere"]);
        apply_action(&mut state, &BAction::Set(OpKey::Select.path(), "none".into()));
        assert!(BSelection::now(&state).list.is_empty(), "none selects nothing, the active stays");
        assert_eq!(state.get(CtxKey::Object), Some("Sphere"));
    }

    #[test]
    fn a_pick_selects_just_that_object_and_extend_adds_to_it() {
        let mut state = BState::new();
        apply_action(&mut state, &BAction::Set(CtxKey::Object.path(), "Cube".into()));
        assert_eq!(BSelection::now(&state).list, ["Cube"]);
        apply_action(&mut state, &BAction::Set(OpKey::Select.path(), "extend:Gem".into()));
        apply_action(&mut state, &BAction::Set(CtxKey::Object.path(), "Gem".into()));
        assert_eq!(BSelection::now(&state).list, ["Cube", "Gem"], "the shift-click kept the Cube");
        apply_action(&mut state, &BAction::Set(CtxKey::Object.path(), "Sphere".into()));
        assert_eq!(BSelection::now(&state).list, ["Sphere"], "a plain click starts over");
    }

    #[test]
    fn all_and_invert_run_over_the_visible_scene() {
        crate::v2::scene::source::clear_loaded_scene();
        let mut state = BState::new();
        state.set(CtxKey::Object, "Cube");
        apply_action(&mut state, &BAction::Set(OpKey::Select.path(), "invert".into()));
        let inverted = BSelection::now(&state);
        assert!(!inverted.contains("Cube") && inverted.contains("Sphere") && inverted.contains("Gem"));
        apply_action(&mut state, &BAction::Set(OpKey::Select.path(), "all".into()));
        assert_eq!(BSelection::now(&state).list.len(), crate::v2::scene::OBJECTS.len());
        assert!(state.get(OpKey::Select).is_none());
    }

    #[test]
    fn the_arrow_toggle_keeps_an_object_out_of_select_all_and_the_camera_out_of_renders() {
        use crate::v2::part::outliner::key::{BHidden, k_restrict};
        use crate::v2::state::val::V_TRUE;
        crate::v2::scene::source::clear_loaded_scene();
        let mut state = BState::new();
        state.set(&k_restrict("Gem", "hide_select"), V_TRUE);
        state.set(&k_restrict("Sphere", "hide_render"), V_TRUE);
        state.set(&k_restrict("Cube", "hide_get"), V_TRUE);
        assert_eq!(visible_names(&state), ["Sphere"], "the eye hides the Cube, the arrow keeps the Gem from being selected");
        let render = BHidden::for_render(&state);
        assert!(render.hides(&state, "Sphere") && !render.hides(&state, "Cube"), "a render honours the camera, not the eye");
        let view = BHidden::now(&state);
        assert!(view.hides(&state, "Cube") && !view.hides(&state, "Sphere"));
        assert!(BHidden::for_select(&state).hides(&state, "Gem"));
    }
}
