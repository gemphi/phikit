//! Focused tests for the import-progress staging vocabulary. Split out of
//! `keys.rs` so that file stays under the 200-line cap.

use super::key::{LoadStage, load_progress_key, load_stage_key, view_load_progress, view_load_stage};
use crate::v2::state::BState;

#[test]
fn load_stages_have_monotonic_increasing_progress() {
    // The staged bar only ever advances — each declared stage's progress is
    // strictly greater than the previous one, ending at a full 1.0.
    let stages = [
        LoadStage::Reading,
        LoadStage::ParsingMeshes,
        LoadStage::BuildingMaterials,
        LoadStage::Done,
    ];
    for pair in stages.windows(2) {
        assert!(
            pair[0].progress() < pair[1].progress(),
            "{:?} ({}) must precede {:?} ({})",
            pair[0],
            pair[0].progress(),
            pair[1],
            pair[1].progress(),
        );
    }
    assert_eq!(LoadStage::Reading.progress(), 0.15);
    assert_eq!(LoadStage::Done.progress(), 1.0);
    // Every stage carries a non-empty label.
    for stage in stages {
        assert!(!stage.label().is_empty());
    }
}

#[test]
fn view_load_progress_reads_and_clamps() {
    let mut s = BState::new();
    // Absent => no import in flight.
    assert_eq!(view_load_progress(&s), None);
    assert_eq!(view_load_stage(&s), None);

    s.set(&load_progress_key(), LoadStage::Reading.progress().to_string());
    s.set(&load_stage_key(), LoadStage::Reading.label());
    assert_eq!(view_load_progress(&s), Some(0.15));
    assert_eq!(view_load_stage(&s), Some("Reading file"));

    // A corrupt over-range value is clamped into 0..=1.
    s.set(&load_progress_key(), "5.0");
    assert_eq!(view_load_progress(&s), Some(1.0));
    s.set(&load_progress_key(), "-2.0");
    assert_eq!(view_load_progress(&s), Some(0.0));
}
