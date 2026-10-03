//! The objects the viewport shows this frame, and those of them a click may
//! not pick — kept between frames.
//!
//! Both lists come from the scene's objects (`scene_objects`, kept per
//! model) through the hidden sets (`BHidden`, kept per scene revision), so
//! while those two are the ones of the last frame the lists are too. An
//! orbit over five hundred objects was filtering and cloning every object,
//! and reading its arrow toggle into a fresh set of names, every frame.

use crate::v2::part::outliner::key::BHidden;
use crate::v2::scene::{BObject3D, object_name, scene_objects};
use crate::v2::state::BState;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

/// What the viewport draws and what a click may pick from it.
#[derive(Clone)]
pub struct BShown {
    /// The visible objects, the shared list the overlay, the picks and the
    /// painter read this pass.
    pub scene: Rc<Vec<BObject3D>>,
    /// The shown objects the arrow toggle keeps from being picked.
    pub unselectable: Rc<HashSet<String>>,
}

/// What a `BShown` was built from, held so the pointers it compares stay
/// taken while it is kept.
struct BFrom {
    objects: Rc<Vec<BObject3D>>,
    hidden: BHidden,
    unselectable: BHidden,
}

impl BFrom {
    fn same(&self, objects: &Rc<Vec<BObject3D>>, hidden: &BHidden, unselectable: &BHidden) -> bool {
        Rc::ptr_eq(&self.objects, objects) && self.hidden.id() == hidden.id() && self.unselectable.id() == unselectable.id()
    }
}

thread_local! {
    static KEPT: RefCell<Option<(BFrom, BShown)>> = const { RefCell::new(None) };
}

impl BShown {
    /// This frame's lists: the last frame's when nothing they read changed.
    pub fn now(s: &BState) -> Self {
        let objects = scene_objects(s);
        let (hidden, unselectable) = (BHidden::now(s), BHidden::for_select(s));
        if let Some(hit) = KEPT.with(|kept| kept.borrow().as_ref().filter(|(from, _)| from.same(&objects, &hidden, &unselectable)).map(|(_, shown)| shown.clone())) {
            return hit;
        }
        let scene: Rc<Vec<_>> = Rc::new(objects.iter().filter(|object| !hidden.hides(s, object_name(object))).cloned().collect());
        let kept_out = scene.iter().map(object_name).filter(|name| unselectable.hides(s, name)).map(str::to_string).collect();
        let shown = Self { scene, unselectable: Rc::new(kept_out) };
        KEPT.with(|kept| *kept.borrow_mut() = Some((BFrom { objects, hidden, unselectable }, shown.clone())));
        shown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::part::outliner::key::k_restrict;
    use crate::v2::state::{View3dKey, val::V_TRUE};

    #[test]
    fn the_lists_are_kept_across_an_orbit_and_follow_a_toggle() {
        let mut s = BState::new();
        let first = BShown::now(&s);
        assert!(first.scene.iter().any(|o| object_name(o) == "Cube"));
        s.set(View3dKey::Yaw, "31");
        assert!(Rc::ptr_eq(&first.scene, &BShown::now(&s).scene), "an orbit: the same list");
        s.set(&k_restrict("Cube", "hide_viewport"), V_TRUE);
        let hidden = BShown::now(&s);
        assert!(!hidden.scene.iter().any(|o| object_name(o) == "Cube"), "the eye: Cube is out");
        s.set(&k_restrict("Sphere", "hide_select"), V_TRUE);
        let shown = BShown::now(&s);
        assert!(shown.unselectable.contains("Sphere"), "the arrow: Sphere is shown but not picked");
        assert!(shown.scene.iter().any(|o| object_name(o) == "Sphere"));
    }
}
