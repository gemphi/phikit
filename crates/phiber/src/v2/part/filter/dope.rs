//! The animation editors' filter — `bl_ui/space_dopesheet.py`.
//!
//! Blender declares this once and four editors draw it: the Dope Sheet
//! (`DOPESHEET_PT_filters`), the Graph editor (`GRAPH_PT_filters`), the
//! Drivers editor (the same panel) and the NLA (`NLA_PT_filters`). They all
//! read one `DopeSheet` struct, so the table is shared and each host's header
//! reaches it through `part::filter`.
//!
//! The four had no filter and no search at all, which is the whole right end
//! and the whole middle of four headers — and the Dope Sheet is the editor
//! where filtering by name is how you find a channel in a rig.
//!
//! Each row is one RNA name, its label as Blender writes it, and the value
//! Blender ships with. The type boxes all ship on: the editor lists every
//! kind of channel until it is asked not to, and defaulting them off would
//! empty the tree for anyone who never opens the popover.

use crate::v2::state::{BKey as _, BPath, BState, DopeKey};
use crate::v2::state::val::{V_FALSE, V_TRUE};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BControl, BPopoverSide};

/// One box in the popover: the RNA name, its label, and Blender's default.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BDopeFilterSpec {
    pub label: &'static str,
    /// The RNA name on `DopeSheet`, which is also the key under
    /// `ctx.dopesheet`.
    pub key: &'static str,
    /// What Blender ships with, used until the box is ticked either way.
    pub default_on: bool,
}

const fn f(key: &'static str, label: &'static str, default_on: bool) -> BDopeFilterSpec {
    BDopeFilterSpec { label, key, default_on }
}

/// The three boxes `dopesheet_filter` draws inline in the header, before the
/// popover. `show_missing_nla` is the NLA's in place of `show_only_errors`,
/// which is why the set is asked for by host.
pub static INLINE: [BDopeFilterSpec; 2] = [
    f("show_only_selected", "Only Show Selected", true),
    f("show_hidden", "Show Hidden", false),
];

/// `show_only_errors` everywhere but the NLA, `show_missing_nla` there.
pub const ERRORS: BDopeFilterSpec = f("show_only_errors", "Only Show Errors", false);
pub const MISSING_NLA: BDopeFilterSpec = f("show_missing_nla", "Include Missing NLA", false);

/// `filter_text`'s one modifier: match each word anywhere in the name.
pub const MULTI_WORD: BDopeFilterSpec = f("use_multi_word_filter", "Multi-Word Match", false);

/// How the channels are ordered and whether the filter is inverted.
pub static ORDER: [BDopeFilterSpec; 2] = [
    f("use_datablock_sort", "Sort Data-Blocks", true),
    f("use_filter_invert", "Invert Filter", false),
];

/// The datablock type boxes, in the order `space_dopesheet.py` lays them
/// out. Every one ships on.
pub static TYPES: [BDopeFilterSpec; 23] = [
    f("show_transforms", "Transforms", true),
    f("show_modifiers", "Modifiers", true),
    f("show_meshes", "Meshes", true),
    f("show_shapekeys", "Shape Keys", true),
    f("show_materials", "Materials", true),
    f("show_lattices", "Lattices", true),
    f("show_cameras", "Cameras", true),
    f("show_curves", "Curves", true),
    f("show_armatures", "Armatures", true),
    f("show_particles", "Particle Systems", true),
    f("show_lights", "Lights", true),
    f("show_speakers", "Speakers", true),
    f("show_linestyles", "Line Styles", true),
    f("show_gpencil", "Grease Pencil", true),
    f("show_movieclips", "Movie Clips", true),
    f("show_worlds", "Worlds", true),
    f("show_cache_files", "Cache Files", true),
    f("show_nodes", "Node Trees", true),
    f("show_scenes", "Scenes", true),
    f("show_textures", "Textures", true),
    f("show_volumes", "Volumes", true),
    f("show_hair_curves", "Hair Curves", true),
    f("show_pointclouds", "Point Clouds", true),
];



/// What a box answers: what the user ticked, or what Blender ships with.
pub fn is_on(state: &BState, spec: &BDopeFilterSpec) -> bool {
    match state.get(&spec.key()) {
        Some(V_TRUE) => true,
        Some(V_FALSE) => false,
        _ => spec.default_on,
    }
}

/// One box as a row, under `heading` when it leads a group.
fn check(state: &BState, heading: &str, spec: &BDopeFilterSpec) -> BView {
    BView::check_row(state, heading, spec.key(), spec.default_on, spec.label)
}

fn group<'a>(
    state: &BState,
    heading: &str,
    specs: impl IntoIterator<Item = &'a BDopeFilterSpec>,
) -> Vec<BView> {
    specs
        .into_iter()
        .enumerate()
        .map(|(n, spec)| check(state, if n == 0 { heading } else { "" }, spec))
        .collect()
}

/// The popover's rows, in Blender's order. `is_nla` swaps the one box that
/// differs there.
pub fn body(state: &BState, is_nla: bool) -> Vec<BView> {
    let mut rows = group(state, "Show", &INLINE);
    rows.push(check(state, "", if is_nla { &MISSING_NLA } else { &ERRORS }));
    rows.extend(group(state, "Search", [&MULTI_WORD]));
    rows.extend(group(state, "Order", &ORDER));
    rows.extend(group(state, "Filters", &TYPES));
    rows
}

/// The header's right-hand control: the icon that opens the popover.
pub fn view(state: &BState, is_nla: bool) -> BView {
    let k = DopeKey::FilterOpen.path();
    let open = state.get(&k) == Some(V_TRUE);
    let toggle = BAction::Set(k.clone(), if open { V_FALSE } else { V_TRUE }.to_string());
    let close = BAction::Set(k, V_FALSE.to_string());
    let anchor = BView::Button(BControl::new().with_icon("filter").with_active(open).with_action(toggle));
    BView::popover(anchor, body(state, is_nla), open, close, BPopoverSide::BelowEnd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_type_box_ships_on() {
        // Defaulting them off empties the channel list for anyone who never
        // opens the popover.
        for spec in &TYPES {
            assert!(spec.default_on, "{} ships off", spec.key);
        }
    }

    #[test]
    fn an_untouched_box_answers_the_way_a_fresh_blender_does() {
        let state = BState::new();
        assert!(is_on(&state, &TYPES[0]), "Transforms is on");
        assert!(is_on(&state, &INLINE[0]), "Only Show Selected is on");
        assert!(!is_on(&state, &INLINE[1]), "Show Hidden is off");
        assert!(!is_on(&state, &ERRORS), "Only Show Errors is off");
    }

    #[test]
    fn a_ticked_box_overrides_the_default_either_way() {
        let mut state = BState::new();
        state.set(&TYPES[0].key(), V_FALSE);
        assert!(!is_on(&state, &TYPES[0]));
        state.set(&INLINE[1].key(), V_TRUE);
        assert!(is_on(&state, &INLINE[1]));
    }

    #[test]
    fn the_nla_swaps_one_box_and_nothing_else() {
        let state = BState::new();
        let (dope, nla) = (body(&state, false), body(&state, true));
        assert_eq!(dope.len(), nla.len(), "one box swapped, not added");
        assert_eq!(dope.len(), INLINE.len() + 1 + 1 + ORDER.len() + TYPES.len());
    }

    #[test]
    fn no_two_boxes_share_a_key() {
        let mut keys: Vec<&str> = INLINE.iter().chain(&ORDER).chain(&TYPES).map(|s| s.key).collect();
        keys.push(ERRORS.key);
        keys.push(MISSING_NLA.key);
        keys.push(MULTI_WORD.key);
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), keys.len(), "two boxes would fight over one key");
    }
}

impl BDopeFilterSpec {
    /// The key one box reads and writes.
    pub fn key(&self) -> BPath {
        DopeKey::Filter(self.key).path()
    }
}
