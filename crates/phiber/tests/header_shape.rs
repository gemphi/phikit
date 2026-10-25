//! One header shape, everywhere: what the editor is on the left, what you are
//! looking for in the middle, how it is filtered on the right.
//!
//! The outliner used to build its own search inside the mode bar while the
//! shared header added one too, so its header carried two search fields.

use phiber::v2::state::{CtxKey};
use phiber::v2::space::{BSpaceList, BSpaceType};
use phiber::v2::state::{BState};
use phiber::v2::view::{BView, BViewList};
use std::collections::BTreeMap;

fn kids(v: &BView) -> Vec<&BView> {
    match v {
        BView::Space { list, .. }
        | BView::Zone { list, .. }
        | BView::Host { list, .. }
        | BView::Region { list, .. }
        | BView::Pane { list, .. }
        | BView::Row { list }
        | BView::Column { list } => list.iter().collect(),
        BView::Card(card) if matches!(card.kind, phiber::v2::view::prim::BCardKind::Column(_)) => card.body.iter().collect(),
        BView::Area { child, .. } | BView::Part { body: child, .. } => vec![child.as_ref()],
        BView::Popover { anchor, body, .. } => {
            let mut out = vec![anchor.as_ref()];
            out.extend(body.iter());
            out
        }
        _ => Vec::new(),
    }
}

/// Search fields inside one host's own subtree.
fn searches_in(v: &BView) -> usize {
    // A search field is one that prompts for a search; a path or name field
    // is an input of its own and is not counted here.
    let own = usize::from(
        matches!(v, BView::SearchField { placeholder, .. } if placeholder.starts_with("Search")),
    );
    own + kids(v)
        .into_iter()
        .filter(|child| !matches!(child, BView::Host { .. }))
        .map(searches_in)
        .sum::<usize>()
}

/// The most search fields any single instance of a host carries, per host.
fn searches_per_host() -> BTreeMap<String, usize> {
    fn walk(v: &BView, out: &mut BTreeMap<String, usize>) {
        if let BView::Host { kind, .. } = v {
            let count = searches_in(v);
            let slot = out.entry(format!("{kind:?}")).or_default();
            *slot = (*slot).max(count);
        }
        for child in kids(v) {
            walk(child, out);
        }
    }
    let mut out = BTreeMap::new();
    for space in BSpaceType::ALL.iter().copied() {
        let mut s = BState::new();
        s.set(CtxKey::Space, space.slug());
        for view in BSpaceList::new().items(&s) {
            walk(&view, &mut out);
        }
    }
    out
}

#[test]
fn no_host_shows_two_search_fields() {
    let doubled: Vec<(String, usize)> = searches_per_host()
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .collect();
    assert!(
        doubled.is_empty(),
        "a host header carries exactly one search field: {doubled:?}"
    );
}

#[test]
fn a_hosts_filters_sit_at_the_right_end_of_its_header() {
    // The outliner is the host that has filters; its popover must be placed by
    // the shared header rule, not built inline by the mode bar.
    let modebar = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/v2/part/modebar/mod.rs"),
    )
    .expect("modebar source");
    assert!(
        !modebar.contains("BView::search_field("),
        "the mode bar must not build a search field of its own"
    );
    let region = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2/region/mod.rs"),
    )
    .expect("region item source");
    assert!(
        region.contains("BPaneType::Right, BPartType::Filter"),
        "filters are placed at the right end of the header"
    );
    // The search is the header's middle, except in a browser, whose header
    // is the columns under it: there it stands over the details column.
    assert!(
        region.contains("if self.host.heads_its_columns() { BPaneType::Right } else { BPaneType::Center }")
            && region.contains("Self::ensure_part(panes, pane, BPartType::Search"),
        "search is placed in the middle of the header, or over a browser's details"
    );
}
