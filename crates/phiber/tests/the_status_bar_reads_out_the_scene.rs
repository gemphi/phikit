//! The status bar shows the scene, or shows nothing — never its own labels.
//!
//! The foot of the window read "Scene 1 Stats Memory Version". Three of the
//! five segments had no value, and `unwrap_or(self.spec.label)` printed the
//! segment's NAME in its place, so the app announced the words "Stats",
//! "Memory" and "Version" where Blender shows counts.
//!
//! Two of the three can be answered from what the app already knows: the
//! visible counts, and the build. The third — memory — nothing measures, so
//! it shows nothing, which is what Blender's status bar does with a figure
//! it does not have.
//!
//! The counts come from `scene::stats_line`, the same function the
//! viewport's read-out draws, so the corner of the canvas and the foot of
//! the window cannot disagree about how big the scene is.

use phiber::v2::part::statusbar::{BStatusSegment, BStatusbarItem};
use phiber::v2::part::statusbar::set::SEGMENTS;
use phiber::v2::state::{BPath, BState};
use phiber::v2::view::{BView, BViewItem};

fn spec(kind: BStatusSegment) -> &'static phiber::v2::part::statusbar::BStatusSegmentSpec {
    SEGMENTS.iter().find(|s| s.kind == kind).expect("the segment")
}

fn text(kind: BStatusSegment, state: &BState) -> Option<String> {
    match BStatusbarItem::new(*spec(kind)).build(state) {
        BView::Label { text } => Some(text),
        _ => None,
    }
}

#[test]
fn no_segment_ever_prints_its_own_name() {
    // The exact bug: a segment with nothing to say saying what it is called.
    let state = BState::new();
    for segment in SEGMENTS {
        if let Some(shown) = text(segment.kind, &state) {
            assert_ne!(shown, segment.label, "{:?} printed its own label", segment.kind);
        }
    }
}

#[test]
fn a_segment_with_nothing_to_say_says_nothing() {
    // Memory: nothing measures it, so the foot of the window is quiet about
    // it rather than reading "Memory".
    assert_eq!(text(BStatusSegment::Memory, &BState::new()), None);
}

#[test]
fn the_stats_are_the_viewports_own_count() {
    // One function, two read-outs. A second copy here is how the corner of
    // the canvas and the foot of the window come to disagree.
    let state = BState::new();
    let shown = text(BStatusSegment::Stats, &state).expect("the stats segment");
    assert_eq!(shown, phiber::v2::scene::stats_line(&state));
    assert!(shown.starts_with("Objects: "), "Blender's wording: {shown}");
    assert!(shown.contains("Verts: ") && shown.contains("Faces: "));
}

#[test]
fn the_version_is_the_build() {
    let shown = text(BStatusSegment::Version, &BState::new()).expect("the version segment");
    assert_eq!(shown, format!("v{}", env!("CARGO_PKG_VERSION")));
    assert!(shown.starts_with('v'), "Blender ends its status bar with vX.Y.Z");
}

#[test]
fn a_written_value_still_wins() {
    // A render writes the stats line (`app::seed`); what it wrote is what
    // shows, not the count derived here.
    let mut state = BState::new();
    state.set(&BPath::owned(vec!["stats"]), "Objects: 9 | Verts: 9 | Faces: 9");
    assert_eq!(text(BStatusSegment::Stats, &state).as_deref(), Some("Objects: 9 | Verts: 9 | Faces: 9"));
}

#[test]
fn an_empty_written_value_does_not_blank_a_segment_that_can_answer() {
    // A key cleared to "" is not a value: falling through to the derived
    // count beats printing an empty segment.
    let mut state = BState::new();
    state.set(&BPath::owned(vec!["stats"]), "");
    assert_eq!(text(BStatusSegment::Stats, &state), Some(phiber::v2::scene::stats_line(&state)));
}
