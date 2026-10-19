//! The manual's shape is checked, not trusted: a tree declared by hand
//! drifts — two pages under one slug, a Blender path that moved, a figure
//! nobody will ever take.

use super::*;
use std::collections::HashSet;

#[test]
fn every_chapter_is_reachable_and_the_tree_is_not_empty() {
    let all = every();
    assert!(all.len() > 80, "the hierarchy is declared whole up front, got {}", all.len());
    assert_eq!(MANUAL.len(), 10, "ten chapters, in Blender's own order");
}

#[test]
fn a_slug_is_unique_among_its_siblings() {
    fn check(nodes: &'static [BManualNode], path: &str) {
        let mut seen = HashSet::new();
        for node in nodes {
            assert!(seen.insert(node.slug), "duplicate slug {}/{} ", path, node.slug);
            check(node.children, &format!("{path}/{}", node.slug));
        }
    }
    check(MANUAL, "");
}

#[test]
fn a_page_either_answers_to_a_blender_page_or_is_declared_as_our_own() {
    // `blender: ""` is the deliberate divergence marker. Anything else must
    // look like a path under `docs/blender-manual/manual/`, so a typo shows
    // up here rather than as a dead parity check later.
    for node in every() {
        if node.blender.is_empty() {
            continue;
        }
        assert!(
            node.blender.ends_with(".rst") && !node.blender.starts_with('/'),
            "{}: {:?} is not a manual-relative .rst path",
            node.slug,
            node.blender
        );
    }
}

#[test]
fn the_pages_we_have_not_written_are_the_gap_list() {
    let (done, total) = progress();
    let stubs = stubs();
    assert!(stubs.len() < total, "not everything is a stub");
    assert!(done <= total);
    // The point of declaring the tree: the gap list is a query, and it is
    // the majority of the manual today.
    assert!(stubs.len() > total / 2, "most of the manual is still to write: {} of {total}", stubs.len());
}

#[test]
fn every_figure_is_listed_once_with_the_pages_that_want_it() {
    let figures = figures();
    let wanted: usize = every().iter().filter(|n| n.figure.is_some()).count();
    let listed: usize = figures.iter().map(|(_, pages)| pages.len()).sum();
    assert_eq!(wanted, listed, "every page wanting a figure is accounted for");
    // Distinct screens are fewer than the pages that show them — which is
    // the reason to gather them: one pass over the app, not one per page.
    assert!(figures.len() < wanted, "{} screens for {wanted} pages", figures.len());
}

#[test]
fn the_rendering_chapter_is_the_one_furthest_along() {
    let render = MANUAL.iter().find(|c| c.slug == "render").expect("a Rendering chapter");
    let written = render.walk().iter().filter(|n| n.state != BManualState::Stub).count();
    assert!(written >= 6, "the rendering work so far is reflected in the tree, got {written}");
}
