//! What the journal promises: it writes what was logged, in order, with the
//! timings; it writes nothing unarmed; and a journal dropped while lines are
//! still arriving leaves a file that reads to the end.
//!
//! The journal is one file per process, so these tests take a gate: two of
//! them arming at once would each see the other's lines.

use crate as philog;
use std::sync::Mutex;

/// One test at the journal at a time.
static GATE: Mutex<()> = Mutex::new(());

/// A path of this test's own, in the temp directory.
fn probe(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("philog-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("session.log")
}

#[test]
fn a_line_carries_the_time_from_the_start_and_the_gap_since_the_last() {
    let line = philog::line(std::time::Duration::from_millis(12_345), std::time::Duration::from_micros(2_500), "Opened Cube.blend");
    assert_eq!(line, "   12.345s  +     2.5ms  Opened Cube.blend");
}

#[test]
fn a_duration_reads_as_milliseconds_under_a_second_and_seconds_above() {
    assert_eq!(philog::took(std::time::Duration::from_micros(61_040)), "61.0ms");
    assert_eq!(philog::took(std::time::Duration::from_millis(126_312)), "126.312s");
}

#[test]
fn an_armed_journal_writes_every_note_once_in_order_with_its_timings() {
    let _gate = GATE.lock().unwrap();
    let path = probe("in-order");
    assert!(philog::arm_at(&path));
    philog::note(philog::header("Probe session journal"));
    philog::note("Opening Probe.blend");
    philog::timed("Reading Probe.blend", || std::thread::sleep(std::time::Duration::from_millis(5)));
    philog::note("Opened Probe.blend: 3 objects");
    philog::disarm();

    let text = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 4, "{text}");
    assert!(lines[0].contains("Probe session journal"), "{text}");
    assert!(lines[1].ends_with("Opening Probe.blend"), "{text}");
    assert!(lines[2].contains("Reading Probe.blend took "), "{text}");
    assert!(lines[3].ends_with("Opened Probe.blend: 3 objects"), "{text}");
    // Every line carries the time from the start and the gap since the last,
    // and the timing of the read is the 5 ms it slept, not a round zero.
    for line in &lines {
        assert!(line.contains("s  +") && line.contains("ms  "), "{line}");
    }
    let took: f64 = lines[2].rsplit_once("took ").unwrap().1.trim_end_matches("ms").parse().unwrap();
    assert!(took >= 5.0, "{}", lines[2]);
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn an_unarmed_journal_writes_nothing_and_opens_no_file() {
    let _gate = GATE.lock().unwrap();
    let path = probe("unarmed");
    let _ = std::fs::remove_file(&path);
    assert!(!philog::armed());
    philog::note("a line nobody armed a journal for");
    philog::note_took("work nobody is timing", std::time::Duration::from_secs(1));
    {
        let _span = philog::BSpan::open("a span nobody armed a journal for");
    }
    log::info!("a record nobody armed a journal for");
    assert!(philog::feed().is_none(), "no journal to feed");
    assert!(!path.exists(), "no file beside an unarmed journal");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn the_log_facade_is_the_front_door_a_records_target_and_level_and_all() {
    let _gate = GATE.lock().unwrap();
    let path = probe("facade");
    // Best effort: another test binary's logger may already hold the slot.
    // Either way the sink is the one philog installs, so the records land.
    let _ = philog::install();
    assert!(philog::arm_at(&path));
    log::info!("RPC Server listening on 0.0.0.0:41451");
    log::debug!("a debug record the Info sink drops");
    philog::disarm();

    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("[INFO] philog::tests: RPC Server listening on 0.0.0.0:41451"), "{text}");
    assert!(!text.contains("drops"), "below the installed level: {text}");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn a_feed_files_a_callers_own_records_once_each_in_order() {
    let _gate = GATE.lock().unwrap();
    let path = probe("feed");
    assert!(philog::arm_at(&path));
    let records = [(1u64, "Opening Feed.blend"), (2, "Opened Feed.blend: 3 objects")];
    // Twice over, as a per-frame feeder calls it: the cursor keeps the
    // second pass from writing the same records again.
    for _ in 0..2 {
        let mut feed = philog::feed().unwrap();
        let written = feed.cursor();
        for (seq, text) in records.iter().filter(|(seq, _)| *seq > written) {
            feed.note_at(web_time::Instant::now(), text);
            feed.set_cursor(*seq);
        }
    }
    philog::disarm();

    let text = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2, "each record once: {text}");
    assert!(lines[0].ends_with("Opening Feed.blend") && lines[1].ends_with("Opened Feed.blend: 3 objects"), "{text}");
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn a_journal_dropped_mid_write_leaves_a_file_that_reads_to_the_end() {
    let _gate = GATE.lock().unwrap();
    let path = probe("mid-write");
    assert!(philog::arm_at(&path));
    // A thread still noting when the journal closes under it: the notes
    // after the drop go nowhere, and no line is left half-written.
    let writer = std::thread::spawn(|| {
        for n in 0..5_000 {
            philog::note(format!("line {n:04} of a session that is about to end"));
        }
    });
    std::thread::sleep(std::time::Duration::from_millis(2));
    philog::disarm();
    writer.join().unwrap();
    assert!(!philog::armed());

    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.ends_with('\n'), "the last line is whole");
    for line in text.lines() {
        let (head, tail) = line.split_once("ms  ").unwrap_or_else(|| panic!("a whole line: {line:?}"));
        assert!(head.contains("s  +"), "{line}");
        assert!(tail.starts_with("line ") && tail.ends_with("is about to end"), "{line}");
    }
    // The lines that did land are in order, and they stop at the drop.
    let seen: Vec<&str> = text.lines().map(|l| l.split_once("ms  ").unwrap().1).collect();
    let mut sorted = seen.clone();
    sorted.sort_unstable();
    assert_eq!(seen, sorted, "in order");
    assert!(seen.len() < 5_000, "the journal closed before the thread finished: {}", seen.len());
    let _ = std::fs::remove_dir_all(path.parent().unwrap());
}
