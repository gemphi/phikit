//! Outliner state keys and the small predicates that read them
//! (expansion, search, display mode, sort, per-object restrictions).

use super::set::BOutlineMode;
use super::tree::intern_name;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::v2::state::{
    BKey as _, BPath, BState, OutlinerKey,
    val::{V_FALSE, V_TRUE},
};


/// Per-object restriction flag, e.g. `ctx.outliner.Cube.hide_render` —
/// the object's name interned so the key is `'static`.
pub fn k_restrict(object: &str, field: &'static str) -> BPath {
    OutlinerKey::Restrict(intern_name(object), field).path()
}

/// The restriction fields a row can carry (`k_restrict`'s `field`): the eye,
/// the monitor, the arrow, the camera, the checkbox, holdout and indirect only.
pub const RESTRICT_FIELDS: &[&str] = &["hide_get", "hide_viewport", "hide_select", "hide_render", "exclude", "holdout", "indirect_only"];

/// Is `encoded` a restriction flag of some row (`ctx.outliner.<row>.<field>`)?
///
/// These are FILE data — Blender keeps them in the `.blend`, on the object
/// (`Object.visibility_flag`), the collection (`Collection.flag`) and the view
/// layer (`LayerCollection.flag`) — so they are seeded from the file on every
/// open (`app::seed`) and never resumed from a session. A session that carried
/// them put one file's "camera off" on the master collection of every file
/// opened after it (every file's master is `col/Scene Collection`), which is
/// how every render of pose 26 came to be handed an empty scene. The column
/// toggles (`ctx.outliner.col.<field>`: which columns the Outliner shows) are
/// UI and are not restrictions.
pub fn is_restriction(encoded: &str) -> bool {
    let Some(rest) = encoded.strip_prefix("ctx.outliner.") else { return false };
    RESTRICT_FIELDS.iter().any(|field| rest.strip_suffix(field).is_some_and(|row| row.ends_with('.') && row != "col."))
}

/// Is `object` hidden in the viewport? Read by the 3D viewport, which is what
/// turns the eye icon from decoration into a working control.
pub fn is_hidden(s: &BState, object: &str) -> bool {
    s.get(&k_restrict(object, "hide_get")) == Some(V_TRUE)
        || s.get(&k_restrict(object, "hide_viewport")) == Some(V_TRUE)
}

/// Is the collection row `id` unchecked (`LAYER_COLLECTION_EXCLUDE`)? An
/// excluded collection is out of the view layer altogether: not drawn,
/// not rendered, not selectable.
pub fn is_excluded(s: &BState, id: &str) -> bool {
    s.get(&k_restrict(id, "exclude")) == Some(V_TRUE)
}

/// Is `object` left out of renders (the camera toggle, `OB_HIDE_RENDER`)?
/// The eye does not decide this: Blender renders what the viewport hides.
pub fn is_render_hidden(s: &BState, object: &str) -> bool {
    s.get(&k_restrict(object, "hide_render")) == Some(V_TRUE)
}

/// Can `object` not be picked (the arrow toggle, `OB_HIDE_SELECT`)?
pub fn is_unselectable(s: &BState, object: &str) -> bool {
    s.get(&k_restrict(object, "hide_select")) == Some(V_TRUE)
}

/// What one pass leaves out: objects with their own toggle set, plus
/// everything under a collection whose toggle is set. Built once and asked
/// per object, so the walk over the collection tree is not repeated for each
/// of a few hundred objects.
///
/// Inside a render pass (`BHidden::begin_pass` … `end_pass`) every reader
/// shares one set per kind, answers included: the viewport, its overlay, its
/// extras and its picks each asked for the viewport's set, and each walked
/// the collection tree and read two keys per object again.
///
/// Across frames a set is kept until what it reads changes — a write outside
/// the viewport's camera (`BState::scene_revision`) or another loaded model.
/// An orbit over five hundred objects was building each kind's set again
/// every frame: the tree walked, two keys encoded and read per object.
#[derive(Clone)]
pub struct BHidden(Rc<BHiddenSet>);

struct BHiddenSet {
    under: HashSet<String>,
    own: fn(&BState, &str) -> bool,
    /// Each object's answer, once asked.
    asked: RefCell<HashMap<String, bool>>,
}

/// What a set was built from: the scene revision and the loaded model.
type BFrom = (u64, usize);

thread_local! {
    /// The pass's sets by kind (`own`), while a pass is drawing.
    static PASS: RefCell<Option<Vec<(usize, BHidden)>>> = const { RefCell::new(None) };
    /// Each kind's set as last built, with what it was built from.
    static KEPT: RefCell<Vec<(usize, BFrom, BHidden)>> = const { RefCell::new(Vec::new()) };
}

impl BHidden {
    /// The viewport's: the eye, and the monitor.
    pub fn now(s: &BState) -> Self {
        Self::by(s, is_hidden)
    }

    /// A render's: the camera.
    pub fn for_render(s: &BState) -> Self {
        Self::by(s, is_render_hidden)
    }

    /// A pick's: the arrow.
    pub fn for_select(s: &BState) -> Self {
        Self::by(s, is_unselectable)
    }

    /// A render pass starts: the state holds still until `end_pass`.
    pub fn begin_pass() {
        PASS.with(|pass| *pass.borrow_mut() = Some(Vec::new()));
    }

    /// The pass is drawn; what runs after it (a click, a paint) reads afresh.
    pub fn end_pass() {
        PASS.with(|pass| *pass.borrow_mut() = None);
    }

    fn by(s: &BState, own: fn(&BState, &str) -> bool) -> Self {
        let kind = own as usize;
        if let Some(hit) = PASS.with(|pass| pass.borrow().as_ref().and_then(|sets| sets.iter().find(|(k, _)| *k == kind).map(|(_, set)| set.clone()))) {
            return hit;
        }
        let from = Self::from(s);
        let set = KEPT.with(|kept| kept.borrow().iter().find(|(k, f, _)| *k == kind && *f == from).map(|(_, _, set)| set.clone()));
        let set = set.unwrap_or_else(|| {
            let under = crate::v2::scene::source::with_loaded(|m| {
                m.map(|m| super::layer::hidden_by_collections(s, m, own)).unwrap_or_default()
            });
            let set = Self(Rc::new(BHiddenSet { under, own, asked: RefCell::new(HashMap::new()) }));
            KEPT.with(|kept| {
                let mut kept = kept.borrow_mut();
                kept.retain(|(k, _, _)| *k != kind);
                kept.push((kind, from, set.clone()));
            });
            set
        });
        PASS.with(|pass| {
            if let Some(sets) = pass.borrow_mut().as_mut() {
                sets.push((kind, set.clone()));
            }
        });
        set
    }

    /// What a set is built from.
    fn from(s: &BState) -> BFrom {
        let model = crate::v2::scene::source::loaded_shared().map_or(0, |m| std::sync::Arc::as_ptr(&m) as usize);
        (s.scene_revision(), model)
    }

    /// Which set this is: two readers holding the same number hold the same
    /// set, so what they built from it can be kept too.
    pub fn id(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }

    pub fn hides(&self, s: &BState, object: &str) -> bool {
        if let Some(&known) = self.0.asked.borrow().get(object) {
            return known;
        }
        let hides = (self.0.own)(s, object) || self.0.under.contains(object);
        self.0.asked.borrow_mut().insert(object.to_string(), hides);
        hides
    }
}

/// Expansion state with a per-row default.
///
/// Blender opens the Scene Collection and its collections but leaves objects
/// closed, so their data-blocks (mesh, materials, modifiers) stay out of the
/// way until asked for. A single "default open" rule would show a Mesh row
/// under every object on first paint.
pub fn is_expanded_or(s: &BState, id: &'static str, default_open: bool) -> bool {
    match s.get(OutlinerKey::Expanded(id)) {
        Some(V_FALSE) => false,
        Some(V_TRUE) => true,
        _ => default_open,
    }
}

/// Rows default to expanded; only an explicit `"false"` collapses one.
pub fn is_expanded(s: &BState, id: &'static str) -> bool {
    is_expanded_or(s, id, true)
}

pub fn search_text(s: &BState) -> String {
    s.get(OutlinerKey::Search).unwrap_or("").trim().to_lowercase()
}

pub fn display_mode(s: &BState) -> BOutlineMode {
    s.get(OutlinerKey::DisplayMode)
        .and_then(BOutlineMode::from_slug)
        .unwrap_or(BOutlineMode::ViewLayer)
}

/// Blender sorts alphabetically by default; the UI flag is the inverted
/// `SO_SKIP_SORT_ALPHA`, so only an explicit `"false"` turns sorting off.
pub fn sort_alpha(s: &BState) -> bool {
    s.get(OutlinerKey::SortAlpha) != Some(V_FALSE)
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::View3dKey;

    /// An orbit keeps the set; a toggle makes a new one that says so.
    #[test]
    fn the_set_is_kept_across_an_orbit_and_rebuilt_by_a_toggle() {
        let mut s = BState::new();
        let first = BHidden::now(&s);
        assert!(!first.hides(&s, "Cube"));
        s.set(View3dKey::Yaw, "31");
        assert_eq!(BHidden::now(&s).id(), first.id(), "an orbit: the same set");
        s.set(&k_restrict("Cube", "hide_viewport"), V_TRUE);
        let toggled = BHidden::now(&s);
        assert_ne!(toggled.id(), first.id(), "a toggle: a new set");
        assert!(toggled.hides(&s, "Cube"));
        assert!(!BHidden::for_select(&s).hides(&s, "Cube"), "each kind its own set");
    }
}
