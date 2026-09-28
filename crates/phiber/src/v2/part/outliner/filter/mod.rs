//! The Outliner's filter — what the header's search text and the Filter
//! popover (`OUTLINER_PT_filter`) let into the View Layer tree. Read once
//! per flatten into a [`BOutlineFilter`], which the tree asks per row, so
//! the popover's boxes and the rows they hide have one owner.
//!
//! Blender ref: `outliner_filter_tree` / `outliner_element_visible_get`
//! (`space_outliner/outliner_tree.cc`): search matches by `fnmatch`,
//! `*text*` unless Exact Match; the object filters drop object rows by
//! type and by `filter_state`, flipped by `filter_invert`; with Collections
//! off the objects hang straight off the Scene Collection.

pub mod popover;
pub mod set;

pub use set::{BOutlineFilterSpec, FILTERS};

use super::key::{BHidden, search_text};
use crate::v2::scene::select::BSelection;
use crate::v2::state::{BKey as _, BState, CtxKey, OutlinerKey, val::{BValue, BValueKind}};

/// The filter as it stands, read from the state once.
pub struct BOutlineFilter {
    query: String,
    case: bool,
    exact: bool,
    pub collections: bool,
    pub objects: bool,
    pub contents: bool,
    pub children: bool,
    state: String,
    invert: bool,
    types: Vec<(bool, &'static [&'static str])>,
    selection: BSelection,
    active: Option<String>,
    hidden: Option<BHidden>,
}

impl BOutlineFilter {
    /// A box's value, its default when never ticked.
    pub fn is_on(s: &BState, spec: &BOutlineFilterSpec) -> bool {
        match s.value(&OutlinerKey::Filter(spec.key).path(), BValueKind::Bool) {
            Some(BValue::Bool(on)) => on,
            _ => spec.default_on,
        }
    }

    /// `filter_state`, `ALL` until picked.
    pub fn state_of(s: &BState) -> String {
        match s.value(&OutlinerKey::Filter(set::STATE_KEY).path(), BValueKind::Enum) {
            Some(BValue::Enum(state)) => state,
            _ => set::STATES[0].0.to_string(),
        }
    }

    pub fn now(s: &BState) -> Self {
        let state = Self::state_of(s);
        let hidden = match state.as_str() {
            "VISIBLE" => Some(BHidden::now(s)),
            "SELECTABLE" => Some(BHidden::for_select(s)),
            _ => None,
        };
        let case = Self::is_on(s, &set::CASE);
        let raw = s.get(OutlinerKey::Search).unwrap_or("").trim().to_string();
        Self {
            query: if case { raw } else { search_text(s) },
            case,
            exact: Self::is_on(s, &set::EXACT),
            collections: Self::is_on(s, &set::COLLECTIONS),
            objects: Self::is_on(s, &set::OBJECTS),
            contents: Self::is_on(s, &set::CONTENTS),
            children: Self::is_on(s, &set::CHILDREN),
            invert: Self::is_on(s, &set::INVERT),
            types: set::TYPES.iter().map(|(spec, names)| (Self::is_on(s, spec), *names)).collect(),
            selection: BSelection::now(s),
            active: s.get(CtxKey::Object).map(str::to_string),
            hidden,
            state,
        }
    }

    /// Does the open file hold objects of these types — for Others (no
    /// names), of a type no box names? Blender lists a type box only then,
    /// and Empties always; with no file open every box is listed.
    pub fn file_has(names: &[&str]) -> bool {
        if names.contains(&"EMPTY") {
            return true;
        }
        crate::v2::scene::source::with_loaded(|m| {
            let Some(m) = m else { return true };
            let cameras = m.cameras.iter().map(|_| "CAMERA");
            let mut types = m.object_types.iter().map(String::as_str).chain(cameras).chain(m.lights.iter().map(|_| "LIGHT"));
            if names.is_empty() {
                types.any(|t| !set::TYPES.iter().any(|(_, named)| named.contains(&t)))
            } else {
                types.any(|t| names.contains(&t))
            }
        })
    }

    /// Is a search typed?
    pub fn searching(&self) -> bool {
        !self.query.is_empty()
    }

    /// Does `name` match the search? Everything does while none is typed.
    pub fn hits(&self, name: &str) -> bool {
        let name = if self.case { name.to_string() } else { name.to_lowercase() };
        if self.exact { name == self.query } else { name.contains(&self.query) }
    }

    /// Does an object of type `ty` (`Object.type`, `MESH`, `CAMERA`, …)
    /// get a row?
    pub fn shows(&self, s: &BState, name: &str, ty: &str) -> bool {
        let named = self.types.iter().find(|(_, names)| names.contains(&ty));
        let others = self.types.iter().find(|(_, names)| names.is_empty());
        let by_type = named.or(others).is_none_or(|(on, _)| *on);
        let by_state = match self.state.as_str() {
            "SELECTED" => self.selection.contains(name),
            "ACTIVE" => self.active.as_deref() == Some(name),
            "VISIBLE" | "SELECTABLE" => !self.hidden.as_ref().is_some_and(|h| h.hides(s, name)),
            _ => true,
        };
        let by_state = if self.state == "ALL" { true } else { by_state != self.invert };
        self.objects && by_type && by_state
    }
}
