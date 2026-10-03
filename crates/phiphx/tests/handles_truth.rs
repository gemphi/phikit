//! Keys inserted the way Blender inserts them carry Blender's handles and
//! evaluate to Blender's values between them — against `truth/handles.txt`
//! (made by `truth/blender_handles.py`: `keyframe_insert`, auto-clamped
//! handles, Continuous Acceleration smoothing, Blender 5.0).

use phiphx::blend::BFCurve;
use std::collections::BTreeMap;

const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/truth/handles.txt");

fn pair(s: &str) -> [f32; 2] {
    let (a, b) = s.split_once(',').unwrap();
    [a.parse().unwrap(), b.parse().unwrap()]
}

#[test]
fn inserted_keys_get_blenders_handles_and_values() {
    let text = std::fs::read_to_string(TRUTH).expect("truth");
    let mut keys: BTreeMap<usize, Vec<([f32; 2], [f32; 2], [f32; 2])>> = BTreeMap::new();
    let mut evals: BTreeMap<usize, Vec<f32>> = BTreeMap::new();
    for line in text.lines() {
        let words: Vec<&str> = line.split_whitespace().collect();
        let case: usize = words[1].parse().unwrap();
        match words[2] {
            "key" => keys.entry(case).or_default().push((pair(words[3]), pair(words[4]), pair(words[5]))),
            _ => evals.insert(case, words[3..].iter().map(|v| v.parse().unwrap()).collect()).map_or((), |_| ()),
        }
    }
    let mut wrong = Vec::new();
    for (case, want) in &keys {
        let mut curve = BFCurve::new("location", 0, Vec::new());
        for (co, _, _) in want {
            curve.insert(co[0], co[1]);
        }
        for (key, (co, h1, h2)) in curve.keys.iter().zip(want) {
            let close = |a: [f32; 2], b: [f32; 2]| (a[0] - b[0]).abs() < 2e-4 && (a[1] - b[1]).abs() < 2e-4;
            if !(close(key.co, *co) && close(key.h1, *h1) && close(key.h2, *h2)) {
                wrong.push(format!("case {case} key {co:?}: ours {:?} {:?}, Blender's {h1:?} {h2:?}", key.h1, key.h2));
            }
        }
        let first = want[0].0[0] as i32;
        let frames = (first - 2..=want.last().unwrap().0[0] as i32 + 2).step_by(3);
        for (frame, value) in frames.zip(&evals[case]) {
            let ours = curve.evaluate(frame as f32);
            if (ours - value).abs() > 2e-4 {
                wrong.push(format!("case {case} frame {frame}: ours {ours}, Blender's {value}"));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
