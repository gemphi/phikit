//! Every host header reads the same way, whichever area the host sits in:
//! its icon and name first (that is the switch trigger), its search in the
//! middle when it has one, everything else after.

use phiber::v2::host::BHostType;
use phiber::v2::part::BPartType;
use phiber::v2::part::search::search_key;
use phiber::v2::pane::BPaneType;
use phiber::v2::region::{BRegionItem, BRegionType};
use phiber::v2::state::BState;
use phiber::v2::view::{BView, BViewItem};

/// The parts of `pane` in the host's header, in order.
fn parts(host: BHostType, s: &BState, want: BPaneType) -> Vec<BPartType> {
    // A host without a header (the status bar) simply has no parts to check.
    let Some(header) = host
        .spec()
        .regions
        .iter()
        .find(|r| r.kind == BRegionType::Header)
    else {
        return Vec::new();
    };
    let BView::Region { list, .. } = BRegionItem::new(host, *header).build(s) else {
        return Vec::new();
    };
    list
        .iter()
        .filter_map(|pane| match pane {
            BView::Pane { kind, list } if *kind == want => Some(list),
            _ => None,
        })
        .flatten()
        .filter_map(|child| match child {
            BView::Part { kind, .. } => Some(*kind),
            _ => None,
        })
        .collect()
}

#[test]
fn every_host_leads_with_its_own_icon_and_name() {
    let s = BState::new();
    for &host in BHostType::ALL {
        let left = parts(host, &s, BPaneType::Left);
        // A host with no header content at all, or one of the window's own
        // bars (top bar, status bar): those are not editors and have no
        // editor type to switch to.
        if left.is_empty() || host.is_fixed_bar() {
            continue;
        }
        assert_eq!(
            left.first(),
            Some(&BPartType::Switcher),
            "{host:?} leads its header with the switcher"
        );
        assert_eq!(
            left.iter().filter(|p| **p == BPartType::Switcher).count(),
            1,
            "{host:?} has exactly one switcher"
        );
    }
}

#[test]
fn a_host_with_a_search_puts_it_in_the_middle_and_others_leave_it_empty() {
    let s = BState::new();
    for &host in BHostType::ALL {
        let centre = parts(host, &s, BPaneType::Center);
        let has_search = centre.contains(&BPartType::Search);
        // The file browsers' header is the columns under it: the search
        // stands over the details column, at the header's end.
        let at_end = parts(host, &s, BPaneType::Right).contains(&BPartType::Search);
        assert_eq!(at_end, search_key(host).is_some() && host.heads_its_columns(), "{host:?}: the search ends the header only in a browser");
        assert_eq!(
            has_search,
            search_key(host).is_some() && !host.heads_its_columns(),
            "{host:?}: the middle holds a search only when the host searches from its header"
        );
    }
}
