//! All eight states, each reached from the signals that produce it.

use super::*;
use crate::v2::scene::render_status::BRenderStatus;

/// A status carrying `taken`/`target`, a picture or not, done or not.
fn status(taken: u32, target: u32, has_image: bool, done: bool) -> BRenderStatus {
    BRenderStatus {
        w: 1920,
        h: 1080,
        taken,
        target,
        done,
        has_image,
        depth: false,
        normal: false,
        albedo: false,
        bytes: 1920 * 1080 * 4,
        peak: 1920 * 1080 * 4,
        seq: 1,
    }
}

/// `phase(status, running, paused, cancelled, camera, saved)`.
#[test]
fn every_one_of_the_nine_is_reachable_and_distinct() {
    let cases = [
        (BRenderPhase::Empty, phase(None, false, false, false, true, false)),
        (BRenderPhase::NoCamera, phase(None, false, false, false, false, false)),
        (BRenderPhase::Preparing, phase(Some(status(0, 128, false, false)), true, false, false, true, false)),
        (BRenderPhase::Rendering, phase(Some(status(12, 128, true, false)), true, false, false, true, false)),
        (BRenderPhase::Paused, phase(Some(status(12, 128, true, false)), true, true, false, true, false)),
        (BRenderPhase::Cancelled, phase(Some(status(12, 128, true, false)), false, false, true, true, false)),
        (BRenderPhase::Finished, phase(Some(status(128, 128, true, true)), false, false, false, true, false)),
        (BRenderPhase::Saved, phase(Some(status(128, 128, true, true)), false, false, false, true, true)),
        // F12 on pose 26: the job ended, done, with no picture at all.
        (BRenderPhase::Failed, phase(Some(status(0, 128, false, true)), false, false, false, true, false)),
    ];
    for (want, got) in cases {
        assert_eq!(got, want, "{want:?} is not what those signals mean");
    }
    // Nine distinct answers, not seven with two spellings.
    let mut seen: Vec<BRenderPhase> = cases.iter().map(|(p, _)| *p).collect();
    let n = seen.len();
    seen.sort_by_key(|p| format!("{p:?}"));
    seen.dedup();
    assert_eq!(seen.len(), n, "two of the nine are the same state");
}

#[test]
fn the_narrower_question_is_asked_first() {
    // A paused render is also unfinished and also has a picture; held wins.
    assert_eq!(phase(Some(status(5, 64, true, false)), true, true, false, true, false), BRenderPhase::Paused);
    // A cancelled render is also not running and also not done.
    assert_eq!(phase(Some(status(5, 64, true, false)), false, false, true, true, false), BRenderPhase::Cancelled);
    // A saved render is also finished.
    assert_eq!(phase(Some(status(64, 64, true, true)), false, false, false, true, true), BRenderPhase::Saved);
    // No camera wins over empty, because it is the REASON the slot is empty.
    assert_eq!(phase(None, false, false, false, false, false), BRenderPhase::NoCamera);
    // A job that went away mid-flight leaves what it reached, as a cancel does.
    assert_eq!(phase(Some(status(7, 64, true, false)), false, false, false, true, false), BRenderPhase::Cancelled);
}

#[test]
fn each_phase_answers_what_the_ui_asks_of_it() {
    use BRenderPhase::*;
    // A picture is up for exactly the five that have one.
    for p in [Rendering, Paused, Cancelled, Finished, Saved] {
        assert!(p.has_picture(), "{p:?} has a picture");
        assert!(p.placeholder().is_none(), "{p:?} needs no placeholder over it");
    }
    for p in [Empty, NoCamera, Preparing, Failed] {
        assert!(!p.has_picture(), "{p:?} has none");
        assert!(p.placeholder().is_some(), "{p:?} should say why it is blank");
    }
    // The job is going for exactly the three that are going.
    for p in [Preparing, Rendering, Paused] {
        assert!(p.is_running(), "{p:?} is under way");
    }
    for p in [Empty, NoCamera, Cancelled, Finished, Saved, Failed] {
        assert!(!p.is_running(), "{p:?} is not");
    }
    // `render_save::save_to` refuses a running render, so only the settled
    // three can be written out.
    for p in [Cancelled, Finished, Saved] {
        assert!(p.can_save(), "{p:?} can be written out");
    }
    for p in [Empty, NoCamera, Preparing, Rendering, Paused, Failed] {
        assert!(!p.can_save(), "{p:?} cannot");
    }
}

#[test]
fn the_info_line_says_the_state_and_the_count_where_a_count_means_something() {
    use BRenderPhase::*;
    assert_eq!(Rendering.info(12, 128), "Sample 12/128");
    assert_eq!(Paused.info(12, 128), "Paused at sample 12/128");
    assert_eq!(Cancelled.info(12, 128), "Cancelled at sample 12/128");
    assert_eq!(Finished.info(128, 128), "Finished");
    assert_eq!(Saved.info(128, 128), "Saved");
    assert_eq!(Empty.info(0, 0), "No render");
    assert_eq!(NoCamera.info(0, 0), "No camera found in scene");
    assert_eq!(Preparing.info(0, 128), "Preparing…");
    assert!(Failed.info(0, 128).starts_with("Render failed"), "{}", Failed.info(0, 128));
    // Nine lines, nine different things to read.
    let mut lines: Vec<String> = [Empty, NoCamera, Preparing, Rendering, Paused, Cancelled, Finished, Saved, Failed]
        .iter()
        .map(|p| p.info(12, 128))
        .collect();
    let n = lines.len();
    lines.sort();
    lines.dedup();
    assert_eq!(lines.len(), n, "two states read the same on the bar");
}
