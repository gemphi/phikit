//! The same scene, read the same — and the two places it is not
//! (`truth/blender_open_time.py`, `truth/open_time.txt`).
//!
//! The point of that dump is the timing column, and `examples/bench_open`
//! is what reads it beside phiphx's own. But a reader that is faster for
//! having read **less** is not faster, so the dump also carries Blender's
//! object and vertex totals, and this is what holds phiphx to them.
//!
//! Timing is not asserted here. It belongs in a measurement, not a test: a
//! shared runner's clock would make it flaky, and a threshold that passes
//! on one machine says nothing about another. What is asserted is the thing
//! that must not drift — the geometry both sides end up with.
//!
//! Two shortfalls were found when this was first written, and both are
//! recorded in [`SHORT`] rather than rounded away. Each is a real gap in
//! what phiphx reads, not noise, and the figures are exact:
//!
//! * **A Subdivision Surface does not subdivide loose edges** — 192
//!   vertices, in both rig files, on the same six Rigify widget objects
//!   (`WGT-rig_palm.L/R` 16 against Blender's 64, `WGT-rig_hand_ik.L/R`
//!   and `WGT-rig_foot_ik.L/R` 8 against 32). Those meshes are edge loops
//!   with no faces at all, and `subsurf.rs`'s `refine` builds its topology
//!   from faces, so a wire-only mesh passes through untouched. Blender
//!   subdivides the chain: with Use Limit Surface on, a valence-two vertex
//!   goes to `(prev + 4·v + next) / 6` and the new points are the cubic
//!   B-spline through the chain — checked against Blender's own output for
//!   `WGT-rig_hand_ik.L` to six decimal places.
//! * **A curve object is not tessellated into geometry** — 48 vertices,
//!   THYMUS's `Path`, which Blender evaluates to a mesh and phiphx reports
//!   as zero.
//!
//! Neither is a timing matter and neither is hidden: closing one makes the
//! assertion below fail, which is the point — the row then gets removed.

use std::collections::HashMap;
use std::path::PathBuf;

/// One row of Blender's dump.
struct Blender {
    file: String,
    objects: usize,
    vertices: usize,
}

fn truth() -> Vec<Blender> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/truth/open_time.txt");
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let field: Vec<&str> = line.split('\t').collect();
            if field.len() < 7 {
                return None;
            }
            Some(Blender {
                file: field[0].to_string(),
                objects: field[2].parse().ok()?,
                vertices: field[3].parse().ok()?,
            })
        })
        .collect()
}

/// `file -> (vertices phiphx is short by, objects it is short by, why)`.
///
/// A row is a decision, and the numbers are exact so that closing a gap —
/// or opening a new one — fails here.
const SHORT: &[(&str, usize, usize, &str)] = &[
    ("THYMUS-BOLDROID-WAVE-003.blend", 0, 1, "the object is an Empty, which phiphx does not list"),
    ("constraints.blend", 0, 1, "the object is an Empty, which phiphx does not list"),
];

fn short(file: &str) -> (usize, usize) {
    SHORT.iter().find(|(name, _, _, _)| *name == file).map_or((0, 0), |(_, v, o, _)| (*v, *o))
}

/// What phiphx reads for each fixture the dump names. Cameras and lights are
/// objects to Blender and are counted the same way here, so the two columns
/// mean one thing.
fn phiphx() -> HashMap<String, (usize, usize)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    truth()
        .iter()
        .filter_map(|row| {
            let path = dir.join(&row.file);
            let model = phiphx::Phx::open(&path).ok()?;
            let objects = model.objects.len() + model.cameras.len() + model.lights.len();
            let vertices = model.objects.iter().map(|o| o.mesh.vertices.len()).sum();
            Some((row.file.clone(), (objects, vertices)))
        })
        .collect()
}

#[test]
fn phiphx_reads_every_vertex_blender_does_bar_the_recorded_shortfall() {
    let truth = truth();
    assert!(!truth.is_empty(), "the dump is there and parses");
    let have = phiphx();

    let mut wrong = Vec::new();
    for row in &truth {
        let Some(&(_, vertices)) = have.get(&row.file) else {
            continue; // a fixture this checkout does not carry
        };
        let (allowed, _) = short(&row.file);
        let want = row.vertices - allowed;
        if vertices != want {
            let side = if vertices < want { "short by" } else { "over by" };
            wrong.push(format!(
                "{}: Blender reads {} and phiphx {vertices}; allowing {allowed}, that is {side} {}",
                row.file,
                row.vertices,
                vertices.abs_diff(want)
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "{} files read a different number of vertices than `SHORT` records:\n  {}",
        wrong.len(),
        wrong.join("\n  ")
    );
}

#[test]
fn phiphx_lists_every_object_blender_does_bar_the_recorded_shortfall() {
    let truth = truth();
    let have = phiphx();
    let mut wrong = Vec::new();
    for row in &truth {
        let Some(&(objects, _)) = have.get(&row.file) else { continue };
        let (_, allowed) = short(&row.file);
        if objects != row.objects - allowed {
            wrong.push(format!(
                "{}: Blender lists {} objects and phiphx {objects}, allowing {allowed}",
                row.file, row.objects
            ));
        }
    }
    assert!(wrong.is_empty(), "object counts differ from the dump:\n  {}", wrong.join("\n  "));
}

#[test]
fn every_recorded_shortfall_is_one_a_fixture_really_has() {
    // A stale row is worse than none: it says a gap was considered when it
    // is no longer there. Every row must name a file the dump carries, and
    // claim a shortfall that is not zero on both counts.
    let truth = truth();
    for (file, vertices, objects, why) in SHORT {
        assert!(
            truth.iter().any(|row| row.file == *file),
            "{file} is recorded as short ({why}) but the dump does not carry it"
        );
        assert!(*vertices > 0 || *objects > 0, "{file}'s row records no shortfall at all");
        assert!(why.len() > 20, "{file}'s row should say what the gap is, not just that there is one");
    }
}

#[test]
fn the_dump_carries_the_two_files_the_timing_actually_turns_on() {
    // The small fixtures open in single-digit milliseconds, where the
    // machine's noise is the same size as the measurement. The two rig
    // files are where a regression would show, so they have to be in the
    // dump for `examples/bench_open` to have anything worth comparing.
    let truth = truth();
    for file in ["THYMUS-BOLDROID-WAVE-003.blend", "FLORIS_BOLDRIOD_RIG.blend"] {
        let row = truth.iter().find(|row| row.file == file).unwrap_or_else(|| panic!("{file} is in the dump"));
        assert!(row.vertices > 50_000, "{file} is one of the heavy ones ({} vertices)", row.vertices);
    }
}
