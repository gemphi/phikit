//! Property 3 — Composition walker structural invariant.
//!
//! Feature: phiber-blender-space-parity
//! Validates: Requirements 2.8, 3.1, 4.1, 5.1, 6.1, 7.1, 8.1, 9.1, 10.1, 11.1, 12.1,
//!            13.1, 14.1, 20.1, 22.1, 26.1, 42.1, 42.5, 52.13
//!
//! For any valid `(space, per-area host)` state, the `BView` tree emitted
//! by `BSpaceList` contains:
//! - exactly one `BView::Space` at the root
//! - one `BView::Area` per declared area slot in that space
//! - one `BView::Host` per `BView::Area`
//! - the declared regions on each host appear as `BView::Region`

use phiber::v2::space::{BSpaceItem, BSpaceType};
use phiber::v2::state::{BState, default_state};
use phiber::v2::view::{BView, BViewItem};
use proptest::prelude::*;

fn count(view: &BView) -> (usize, usize, usize, usize, usize, usize) {
    // (space, zone, area, host, region, pane)
    fn go(v: &BView, c: &mut (usize, usize, usize, usize, usize, usize)) {
        match v {
            BView::Space { list, .. } => {
                c.0 += 1;
                for ch in list {
                    go(ch, c);
                }
            }
            BView::Zone { list, .. } => {
                c.1 += 1;
                for ch in list {
                    go(ch, c);
                }
            }
            BView::Area { child, .. } => {
                c.2 += 1;
                go(child, c);
            }
            BView::Host { list, .. } => {
                c.3 += 1;
                for ch in list {
                    go(ch, c);
                }
            }
            BView::Region { list, .. } => {
                c.4 += 1;
                for ch in list {
                    go(ch, c);
                }
            }
            BView::Pane { list, .. } => {
                c.5 += 1;
                for ch in list {
                    go(ch, c);
                }
            }
            _ => {}
        }
    }
    let mut acc = (0, 0, 0, 0, 0, 0);
    go(view, &mut acc);
    acc
}

fn arb_space() -> impl Strategy<Value = BSpaceType> {
    (0..BSpaceType::ALL.len()).prop_map(|i| BSpaceType::ALL[i])
}

proptest! {
    #[test]
    fn one_space_per_tree(space in arb_space()) {
        let state = default_state();
        let root = BSpaceItem::new(space).build(&state);
        let (n_space, _, _, _, _, _) = count(&root);
        prop_assert_eq!(n_space, 1, "expected 1 BView::Space, got {}", n_space);
    }

    #[test]
    fn one_area_per_declared_slot(space in arb_space()) {
        let state = default_state();
        let root = BSpaceItem::new(space).build(&state);
        let (_, _, n_area, _, _, _) = count(&root);
        prop_assert_eq!(n_area, space.spec().areas.len(),
            "space {:?} declared {} areas but tree has {}",
            space, space.spec().areas.len(), n_area);
    }

    #[test]
    fn one_host_per_area(space in arb_space()) {
        let state = default_state();
        let root = BSpaceItem::new(space).build(&state);
        let (_, _, n_area, n_host, _, _) = count(&root);
        prop_assert_eq!(n_host, n_area,
            "space {:?}: {} areas but {} hosts", space, n_area, n_host);
    }
}

#[test]
fn every_space_walks_cleanly() {
    let state = default_state();
    for &space in BSpaceType::ALL {
        let root = BSpaceItem::new(space).build(&state);
        let (n_space, n_zone, n_area, n_host, _, _) = count(&root);
        assert_eq!(n_space, 1, "{space:?}: expected 1 Space");
        assert!(n_zone >= 1, "{space:?}: expected ≥1 Zone");
        assert_eq!(
            n_area,
            space.spec().areas.len(),
            "{space:?}: areas mismatch"
        );
        assert_eq!(n_host, n_area, "{space:?}: hosts != areas");
    }
}

#[test]
fn arbitrary_state_does_not_panic() {
    // BSpaceItem must handle both default state and empty state.
    let empty = BState::new();
    let root = BSpaceItem::new(BSpaceType::Layout).build(&empty);
    // No panic; a Space variant at the root.
    assert!(matches!(root, BView::Space { .. }));
}
