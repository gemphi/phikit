//! Panel — accordion of property sections with RNA-backed props.

pub mod prop;
pub mod prop_view;
pub mod set;
pub mod active;
pub mod fold;
pub mod collection;
pub mod constraint;
pub mod data;
pub mod material;
pub mod modifier;
pub mod object;
pub mod options;
pub mod output;
pub mod particles;
pub mod physics;
pub mod pref;
pub mod render;
pub mod scene;
pub mod texture;
pub mod tool;
pub mod view_layer;
pub mod world;

use std::collections::HashMap;
use std::sync::Mutex;
use crate::v2::host::BHostType;
use crate::v2::part::navbar::set::tabs_for;
use crate::v2::state::BKey as _;
use crate::v2::state::BPath;
use crate::v2::state::BState;
use crate::v2::state::val::V_FALSE;
use crate::v2::state::val::V_TRUE;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;
use crate::v2::view::prim::BCard;
use crate::v2::view::prim::BColumn;
use crate::v2::view::prim::with_peers;
use self::prop_view::header_ctrl;
use self::prop_view::prop_row;
pub use prop::BEnumItem;
pub use prop::BPropKind;
pub use prop::BPropSpec;
pub use prop::BUnit;
pub use set::BSectionSpec;
pub use set::sections_for;

// `BPanelItem` — one Properties panel: a `BCard` headed by the section's
// title, its body one prop row per `BPropSpec`, then its sub-panels.
// Blender ref: a `bpy.types.Panel` in `properties_*.py` (and Cycles'
// `addon/ui.py`), drawn by `interface_panel.cc`.
// 
// What the declaration says, the card does: `DEFAULT_CLOSED` panels start
// closed, `HIDE_HEADER` ones have no header and never fold, `draw_header`
// properties sit in the header before the title, and sub-panels
// (`bl_parent_id`) are `Sub` cards at the foot of their parent's body.
#[derive(Clone, Copy, Debug)]
pub struct BPanelItem {
    pub host: BHostType,
    pub tab: &'static str,
    pub spec: BSectionSpec,
    pub expanded: bool,
}

impl BPanelItem {
    pub fn new(host: BHostType, tab: &'static str, spec: BSectionSpec, expanded: bool) -> Self {
        Self {
            host,
            tab,
            spec,
            expanded,
        }
    }

    /// State key for this section's expanded flag:
    /// `ctx.<host>.<tab>.<section>.expanded`. Including the active tab avoids
    /// collisions between common section names such as Surface and Preview.
    /// A sub-panel's `title` is its path from the top panel
    /// (`"Sampling Render Denoise"`), so the two Denoise panels differ.
    pub fn expand_key(host: BHostType, tab: &str, title: &str) -> BPath {
        BPath::owned(vec![host.slug(), slug(tab), slug(title), "expanded"])
    }

    /// Whether a panel is open: what the user last chose, or — until they
    /// choose — Blender's default for it (`DEFAULT_CLOSED` or not).
    pub fn is_open(s: &BState, key: &BPath, closed_by_default: bool) -> bool {
        match s.get(key) {
            Some(V_FALSE) => false,
            Some(V_TRUE) => true,
            _ => !closed_by_default,
        }
    }

    fn card(&self, spec: &BSectionSpec, path: &str, open: bool, s: &BState) -> BCard {
        let mut body: Vec<BView> = Vec::new();
        if open || spec.headless {
            body.extend(spec.props.iter().map(|p| prop_row(p, s)));
            for child in spec.list {
                let child_path = format!("{path} {}", child.title);
                let key = Self::expand_key(self.host, self.tab, &child_path);
                let child_open = Self::is_open(s, &key, child.closed);
                body.push(self.card(child, &child_path, child_open, s).sub().into());
            }
        }
        if spec.headless {
            return BCard::headless().body(body);
        }
        let key = Self::expand_key(self.host, self.tab, path);
        BCard::panel(spec.title)
            .fold(key, open)
            .controls(spec.header.iter().map(|p| header_ctrl(p, s)).collect())
            .body(body)
    }
}

/// A title or tab as a lowercase path segment, interned: the same name always
/// yields the same `&'static str`, and each distinct name is allocated once.
/// (Leaking a fresh string on every build grew memory every frame.)
pub(crate) fn slug(title: &str) -> &'static str {
    static POOL: Mutex<Option<HashMap<String, &'static str>>> = Mutex::new(None);
    let mut guard = POOL.lock().unwrap_or_else(|e| e.into_inner());
    let pool = guard.get_or_insert_with(HashMap::new);
    if let Some(&interned) = pool.get(title) {
        return interned;
    }
    let made: &'static str = Box::leak(title.to_ascii_lowercase().replace(' ', "_").into_boxed_str());
    pool.insert(title.to_string(), made);
    made
}

impl BViewItem for BPanelItem {
    fn build(&self, s: &BState) -> BView {
        self.card(&self.spec, self.spec.title, self.expanded, s).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::part::props::prop::BPropSpec;
    use crate::v2::part::props::set::sec;

    static EMPTY: &[BPropSpec] = &[];

    #[test]
    fn initially_expanded_header_first_click_sets_false() {
        use crate::v2::state::val::V_FALSE;
        use crate::v2::view::prim::BAction;
        let spec = sec("Surface", EMPTY);
        let BView::Card(card) = BPanelItem::new(BHostType::Buts, "Material", spec, true).build(&BState::new())
        else {
            panic!()
        };
        let BAction::Set(_, value) = card.fold.as_ref().unwrap().flip() else { panic!() };
        assert_eq!(value, V_FALSE);
        assert_eq!(card.title(), Some("Surface"));
    }
}

// `BPanelList` — reads `sections_for(host, tab)` + `host.<h>.panel.<title>.expanded`.

#[derive(Clone, Copy, Debug)]
pub struct BPanelList {
    pub host: BHostType,
}

impl BPanelList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BPanelList {
    fn items(&self, s: &BState) -> Vec<BView> {
        // Active context tab from state; fallback = navbar's first declared
        // tab for this host (no hardcoded name).
        let fallback = tabs_for(self.host).first().map(|t| t.label).unwrap_or("");
        let selected = s.host(self.host).get("navbar.tab").unwrap_or(fallback);
        // Resolve back to the static table entry. Besides avoiding unbounded
        // path-segment allocation, this guarantees the key uses the same tab
        // name that selected the section table.
        let tab = tabs_for(self.host)
            .iter()
            .find(|candidate| candidate.label == selected)
            .map(|candidate| candidate.label)
            .unwrap_or(fallback);
        // The header's search narrows the tab to the panels whose name
        // matches, and opens them: a match the user cannot see is no match
        // (`SpaceProperties.search_filter`). An empty search is not a
        // filter, so the tab reads exactly as it did before.
        let query = s
            .get(&crate::v2::state::ButsKey::Search.path())
            .unwrap_or("")
            .trim()
            .to_lowercase();
        let mut cards: Vec<BView> = sections_for(self.host, tab)
            .iter()
            .filter(|section| query.is_empty() || section.title.to_lowercase().contains(&query))
            .map(|section| {
                let key = BPanelItem::expand_key(self.host, tab, section.title);
                let expanded = if query.is_empty() { BPanelItem::is_open(s, &key, section.closed) } else { true };
                BPanelItem::new(self.host, tab, *section, expanded).build(s)
            })
            .collect();
        // `OBJECT_PT_animation`: the object's action, over the file's list.
        if tab == "Object" && query.is_empty() {
            cards.extend(crate::v2::part::action::template::BActionTemplate::new(self.host).panel(tab, s));
        }
        // One region's panels: Ctrl-click on a header closes the rest.
        with_peers(cards)
    }
}

// Per-context property tables.

#[derive(Clone, Copy, Debug)]
pub struct BPanel {
    pub host: BHostType,
}

impl BPanel {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewItem for BPanel {
    fn build(&self, s: &BState) -> BView {
        BView::from(BCard::column(BColumn::Body, BPanelList::new(self.host).items(s)))
    }
}
