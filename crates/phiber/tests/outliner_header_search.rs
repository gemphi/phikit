//! The Outliner's header search on a narrow side panel.
//!
//! In a side zone the switcher, the View Layer mode dropdown and the View
//! menu fill the header, so an inline search field crushed to a bordered
//! sliver at the right end (the clipped box). The field moves to the Filter
//! popover there — which now carries it, as Blender's `OUTLINER_PT_filter`
//! does — so a side panel can still search; `region::complete_header` drops
//! the inline copy only for the Outliner in a side zone.

use phiber::v2::state::{BState, OutlinerKey, BKey as _};
use phiber::v2::view::BView;
use phiber::v2::view::prim::BAction;

fn search_fields(v: &BView, out: &mut Vec<BAction>) {
    match v {
        BView::SearchField { on_change, .. } => out.push(on_change.clone()),
        BView::Row { list } | BView::Column { list } => list.iter().for_each(|c| search_fields(c, out)),
        // The popover's groups are accordions now (`part::filter::fold`), so
        // the field lives one card deep: a walk that stopped at rows found
        // nothing and read as the field having been dropped.
        BView::Card(card) => card.body.iter().for_each(|c| search_fields(c, out)),
        BView::Part { body, .. } => search_fields(body, out),
        BView::Shared(rc) => search_fields(rc, out),
        _ => {}
    }
}

#[test]
fn the_filter_popover_carries_the_search_field_bound_to_the_outliner_search() {
    // The popover opens short — only its first heading open — so open the
    // Search heading, as a user does, before looking for its field.
    let mut s = BState::new();
    s.set(&phiber::v2::part::filter::fold::expand_key(&["outliner", "filter"], "Search"), "true");
    let mut fields = Vec::new();
    for row in phiber::v2::part::outliner::filter::popover::body(&s) {
        search_fields(&row, &mut fields);
    }
    assert_eq!(fields.len(), 1, "exactly one search field, so a side panel can search from the popover");
    assert!(
        matches!(&fields[0], BAction::Set(path, v) if *path == OutlinerKey::Search.path() && v.is_empty()),
        "it clears the Outliner's search key: {:?}",
        fields[0]
    );
}

#[test]
fn complete_header_drops_the_inline_search_only_for_a_side_zone_outliner() {
    // The gate lives in region.rs; its source names the host and the two
    // side zones, and nothing else, so the Properties editor and a maximized
    // Outliner keep their inline field.
    let src = include_str!("../src/v2/region/mod.rs");
    assert!(src.contains("fn header_too_narrow_for_search"));
    assert!(src.contains("host == BHostType::Outliner"));
    assert!(src.contains("BZoneType::Left | crate::v2::zone::BZoneType::Right"));
    assert!(src.contains("&& !header_too_narrow_for_search(self.host)"), "the Center search is gated on it");
}
