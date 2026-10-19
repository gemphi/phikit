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

/// The scope this popover's fold keys sit under, beside the overlay
/// families' (`overlay::scope`).
const SCOPE: [&str; 2] = ["dope", "filter"];

/// Blender keeps `DOPESHEET_PT_filters` short two ways that phiber does not
/// have: `draw_standard_filters` draws a type's box only when the file holds
/// data of that type (`if bpy.data.armatures:` …), and lays the survivors out
/// `grid_flow(columns=2)`. phiber draws all twenty-three in one column, so
/// the popover ran to twenty-nine rows. Recorded divergence: the four
/// headings fold instead, and Filter by Type ships shut — the same twenty-odd
/// boxes an empty scene would never have shown. Polling the file would be the
/// closer port and is the better fix when the type counts are to hand; this
/// one costs nothing and survives a file that does hold every type.
fn closed(group: &str) -> bool {
    group == "Filters"
}

/// The popover's boxes split at Blender's headings, headless — what the
/// cards fold over.
pub fn groups(state: &BState, is_nla: bool) -> Vec<super::fold::BGroup> {
    let mut show = group(state, "", &INLINE);
    show.push(check(state, "", if is_nla { &MISSING_NLA } else { &ERRORS }));
    vec![
        ("Show", show),
        ("Search", group(state, "", [&MULTI_WORD])),
        ("Order", group(state, "", &ORDER)),
        ("Filters", group(state, "", &TYPES)),
    ]
}

/// The popover's body, in Blender's order: four folding cards
/// (`filter::fold`), `is_nla` swapping the one box that differs there.
pub fn body(state: &BState, is_nla: bool) -> Vec<BView> {
    super::fold::cards(state, &SCOPE, groups(state, is_nla), closed)
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

impl BDopeFilterSpec {
    /// The key one box reads and writes.
    pub fn key(&self) -> BPath {
        DopeKey::Filter(self.key).path()
    }
}

#[cfg(test)]
#[path = "dope_tests.rs"]
mod tests;
