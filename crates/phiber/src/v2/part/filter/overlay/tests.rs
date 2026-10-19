//! Tests for `mod` — split out so the module itself sits under the
//! 240-line read cap (`tests/level_structure.rs`).

use super::*;

/// Every host that draws a panel, and the family it draws it for.
const DRAWN_BY: [(BHostType, &str); 9] = [
    (BHostType::Shader, "node"),
    (BHostType::Node, "node"),
    (BHostType::GeoNode, "node"),
    (BHostType::Compositor, "node"),
    (BHostType::Seq, "seq"),
    (BHostType::Image, "image"),
    (BHostType::Uv, "image"),
    (BHostType::Clip, "clip"),
    (BHostType::Spreadsheet, "spreadsheet"),
];

#[test]
fn a_family_shares_one_table_and_one_scope() {
    for (host, family) in DRAWN_BY {
        let panel = panel_for(host).unwrap_or_else(|| panic!("{host:?}"));
        assert_eq!(panel.family, family, "{host:?}");
        // `encode` writes the scope it is read under, and every key has
        // one: `BPath::encode` opens with `ctx` (`state/path.rs`). The
        // expectation here never did, so it could not have matched.
        assert_eq!(
            key(host, &panel.boxes[0]).encode(),
            format!("ctx.overlay.{family}.{}", panel.boxes[0].key),
        );
    }
    // Two hosts in one family answer the same key; two families do not.
    assert_eq!(key(BHostType::Image, &IMAGE[0]), key(BHostType::Uv, &IMAGE[0]));
    assert_ne!(key(BHostType::Seq, &SEQ[0]), key(BHostType::Node, &NODE[0]));
}

#[test]
fn a_ticked_box_overrides_the_default_either_way() {
    let mut state = BState::new();
    state.set(&key(BHostType::Node, &NODE[1]), V_FALSE);
    assert!(!is_on(&state, BHostType::Node, &NODE[1]));
    // And the Sequencer's own box is untouched by it.
    assert!(is_on(&state, BHostType::Seq, &SEQ[1]));
    // A box Blender ships off comes on the same way.
    state.set(&key(BHostType::Clip, &CLIP[1]), V_TRUE);
    assert!(is_on(&state, BHostType::Clip, &CLIP[1]));
}

#[test]
fn a_host_with_no_overlays_builds_no_rows() {
    for host in [BHostType::Time, BHostType::Text, BHostType::Console, BHostType::Info] {
        assert!(!has_overlay(host), "{host:?}");
        assert!(body(&BState::new(), host).is_empty(), "{host:?}");
        assert!(matches!(view(&BState::new(), host), BView::Empty), "{host:?}");
    }
}

#[test]
fn no_two_boxes_in_a_table_share_a_key() {
    for table in [&NODE[..], &SEQ[..], &IMAGE[..], &CLIP[..], &SHEET[..]] {
        let mut keys: Vec<&str> = table.iter().map(|s| s.key).collect();
        let n = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), n, "two boxes would fight over one key");
    }
}

/// The rule `filter::fold` states, checked over every real table: a
/// table with two or more headings draws folding cards, a table with one
/// draws its rows. This is what stops a table that grows a second heading
/// from quietly staying a long flat list.
#[test]
fn a_table_with_two_headings_folds_and_a_table_with_one_does_not() {
    let state = BState::new();
    for host in DRAWN_BY.map(|(host, _)| host) {
        let headings = groups(&state, host).len();
        let drawn = body(&state, host);
        let cards = drawn.iter().filter(|v| matches!(v, BView::Card(_))).count();
        match headings {
            // `IMAGE_PT_overlay`'s four, `CLIP_PT_display`'s two.
            2.. => {
                assert_eq!(cards, headings, "{host:?}: a card per heading");
                assert_eq!(drawn.len(), headings, "{host:?}: cards only, no loose rows");
            }
            // `NODE_PT_overlay`, `SPREADSHEET_PT_filter`: one panel.
            _ => {
                assert_eq!(cards, 0, "{host:?}: one heading is the panel's own name");
                assert_eq!(drawn.len(), table_for(host).unwrap().len(), "{host:?}");
            }
        }
    }
}

/// The header and the sidebar fold as one: the Movie Clip editor's View
/// tab and its header popover draw the same groups, so a triangle shut in
/// one is shut in the other.
#[test]
fn the_header_and_the_sidebar_fold_on_one_set_of_keys() {
    use crate::v2::part::filter::fold;
    let mut state = BState::new();
    let key = fold::expand_key(&scope(BHostType::Clip), "Marker Display");
    assert_eq!(key.encode(), "ctx.overlay.clip.marker_display.expanded");
    state.set(&key, V_FALSE);
    let shut = |views: Vec<BView>| match views.first() {
        Some(BView::Card(card)) => card.body.is_empty(),
        _ => panic!("a card"),
    };
    assert!(shut(body(&state, BHostType::Clip)), "the header reads it");
    assert!(
        shut(fold::cards(&state, &scope(BHostType::Clip), groups(&state, BHostType::Clip), fold::all_open)),
        "and the sidebar's View tab reads the same key",
    );
    // Two families keep their own triangles.
    assert_ne!(key, fold::expand_key(&scope(BHostType::Image), "Marker Display"));
}

#[test]
fn every_table_opens_with_a_heading() {
    // A popover whose first row has no heading reads as a loose list; the
    // heading is what says which panel you opened.
    for host in DRAWN_BY.map(|(host, _)| host) {
        let table = table_for(host).unwrap();
        assert!(!table[0].group.is_empty(), "{host:?} opens with no heading");
    }
}
