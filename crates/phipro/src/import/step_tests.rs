//! The progress a bar is driven by.

use super::*;

#[test]
fn the_steps_only_ever_move_forward() {
    let step = [BImportStep::Read, BImportStep::Make, BImportStep::Manifest, BImportStep::Scene, BImportStep::Done];
    for pair in step.windows(2) {
        assert!(pair[0].part() < pair[1].part(), "{:?} comes before {:?}", pair[0], pair[1]);
    }
}

#[test]
fn the_bar_reaches_its_end() {
    // The version this replaces stopped reporting at 0.80 and returned, so
    // the bar sat at four fifths until the caller knew to finish it.
    assert_eq!(BImportStep::Done.part(), 1.0);
    assert!(BImportStep::Read.part() > 0.0, "and starts after zero, so something is showing");
}

#[test]
fn the_readers_own_progress_lands_inside_the_readers_share() {
    let (first, last) = (BImportStep::Read.part(), BImportStep::Make.part());
    assert_eq!(BImportStep::Read.within(0.0), first);
    assert_eq!(BImportStep::Read.within(1.0), last);
    let half = BImportStep::Read.within(0.5);
    assert!(half > first && half < last, "{half} sits between {first} and {last}");
}

#[test]
fn a_reader_that_reports_out_of_range_cannot_push_the_bar_past_its_step() {
    // A format importer that reports 1.5, or -1, must not make the bar
    // jump over the writing steps or go backwards.
    assert_eq!(BImportStep::Read.within(9.0), BImportStep::Make.part());
    assert_eq!(BImportStep::Read.within(-9.0), BImportStep::Read.part());
}

#[test]
fn every_step_says_what_it_is_doing() {
    for step in [BImportStep::Read, BImportStep::Make, BImportStep::Manifest, BImportStep::Scene, BImportStep::Done] {
        let said = step.note("rig");
        assert!(said.contains("rig") && !said.contains("Import"), "{step:?} reads as {said:?}");
    }
}

#[test]
fn a_bar_never_slides_backwards_even_when_the_reader_does() {
    // `phiphx`'s `.blend` reader restarts its count per data block, so it
    // really does report 0.125 and then 0.0875. A bar driven straight from
    // that is one the user watches go backwards.
    let seen = std::cell::RefCell::new(Vec::new());
    let note = |part: f32, what: &str| seen.borrow_mut().push((part, what.to_string()));
    let say = BProgress::new(&note);
    say.at(BImportStep::Read, "rig.blend");
    say.within(BImportStep::Read, 0.5, "half way");
    say.within(BImportStep::Read, 0.1, "and back to a tenth");
    say.at(BImportStep::Done, "rig");

    let seen = seen.into_inner();
    let mut last = 0.0;
    for (part, what) in &seen {
        assert!(*part >= last, "{part} comes after {last} ({what})");
        last = *part;
    }
    // The note still arrives for the backward report, so the line beside
    // the bar keeps moving while the fraction holds.
    assert_eq!(seen.len(), 4);
    assert_eq!(seen[2].0, seen[1].0, "held, not reversed");
    assert_eq!(seen[2].1, "and back to a tenth");
    assert_eq!(seen[3].0, 1.0);
}
