//! The Filter popover's rows, laid out like every other header popover
//! (`BView::check_row` / `select_row`): the restriction columns, sorting,
//! then Blender's Search and Filter sub-panels — Filter only in View Layer
//! mode, as `OUTLINER_PT_options_filter.poll` has it.

use super::set::{self, BOutlineFilterSpec};
use super::BOutlineFilter;
use crate::v2::part::outliner::set::{BOutlineMode, TOGGLES};
use crate::v2::part::outliner::tree::display_mode;
use crate::v2::state::{BKey as _, BState, OutlinerKey, edit};
use crate::v2::view::BView;

/// A box keyed by its RNA name under `ctx.outliner.filter`.
fn check(s: &BState, heading: &str, spec: &BOutlineFilterSpec) -> BView {
    BView::check_row(s, heading, OutlinerKey::Filter(spec.key).path(), spec.default_on, spec.label)
}

fn group<'a>(s: &BState, heading: &str, specs: impl IntoIterator<Item = &'a BOutlineFilterSpec>) -> Vec<BView> {
    specs.into_iter().enumerate().map(|(n, spec)| check(s, if n == 0 { heading } else { "" }, spec)).collect()
}

/// The popover's rows.
pub fn body(s: &BState) -> Vec<BView> {
    let mut rows: Vec<BView> = TOGGLES
        .iter()
        .enumerate()
        .map(|(n, t)| BView::check_row(s, if n == 0 { "Columns" } else { "" }, OutlinerKey::Column(t.state_key).path(), t.default_on, t.tooltip))
        .collect();
    rows.push(BView::check_row(s, "Sort", OutlinerKey::SortAlpha.path(), true, "Sort Alphabetically"));
    // `OUTLINER_PT_filter` opens with the search field (`filter_text`), then
    // its two modifiers. Carrying the field here is what lets a narrow
    // side-panel header drop its crushed inline copy (`region::complete_header`)
    // and still search — the popover is self-sufficient.
    let search = OutlinerKey::Search.path();
    rows.push(BView::text_field("Search", s.get(&search).unwrap_or("").to_string(), edit::is_editing(s, &search), crate::v2::view::prim::BAction::Set(search, String::new())));
    rows.extend(group(s, "Search", &set::SEARCH));
    if display_mode(s) != BOutlineMode::ViewLayer {
        return rows;
    }
    rows.extend(group(s, "Filter", &set::FILTERS));
    rows.push(BView::select_row(s, "Objects", OutlinerKey::Filter(set::STATE_KEY).path(), set::STATE_KEY, &set::STATES, set::STATES[0].0));
    if BOutlineFilter::state_of(s) != set::STATES[0].0 {
        rows.push(check(s, "", &set::INVERT));
    }
    let present = set::TYPES.iter().filter(|(_, names)| BOutlineFilter::file_has(names));
    rows.extend(group(s, "Types", present.map(|(spec, _)| spec)));
    rows
}
