//! Against Blender 5.0 (`truth/blender_overlays.py`): what the viewport
//! draws over `fixtures/overlays.blend` — the Layout view's overlay and
//! gizmo switches, each set away from its default, the 3D cursor turned in
//! ZXY order, and the relationship lines at three frames: children of an
//! object, of a bone's tail and of a bone's deform (relative parenting),
//! and the lines of an object target, a bone target at `head_tail`, a Follow
//! Path curve, with none for a folded constraint or Clamp To.

use crate::blend::file::types::BlendFile;
use crate::scene::SceneRelation;

const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests");

fn lines() -> Vec<Vec<String>> {
    std::fs::read_to_string(format!("{DIR}/truth/overlays.txt"))
        .expect("truth file")
        .lines()
        .map(|l| l.split('\t').map(str::to_string).collect())
        .collect()
}

fn floats(s: &str) -> Vec<f32> {
    s.split(' ').filter_map(|x| x.parse().ok()).collect()
}

fn near(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-4)
}

fn scene() -> crate::blend::BlendScene {
    let file = BlendFile::parse_bytes(&std::fs::read(format!("{DIR}/fixtures/overlays.blend")).expect("fixture")).expect("parse");
    file.extract_scene().expect("scene")
}

#[test]
fn the_view_s_switches_and_the_cursor_are_blender_s() {
    let scene = scene();
    let view = scene.props.ui.view3d.as_ref().expect("the Layout view");
    let mut checked = 0;
    for l in lines() {
        match l[0].as_str() {
            "OVL" => {
                assert_eq!(view.overlay.get(&l[1]), Some(l[2] == "1"), "{}", l[1]);
                checked += 1;
            }
            "CURSOR" => {
                let ours: Vec<f32> = scene.props.cursor.matrix.iter().flatten().copied().collect();
                assert!(near(&ours, &floats(&l[1])), "cursor {ours:?} against {}", l[1]);
                checked += 1;
            }
            _ => {}
        }
    }
    assert_eq!(checked, 22, "21 switches and the cursor");
}

#[test]
fn every_relationship_line_runs_where_blender_s_does() {
    let scene = scene();
    let motion = &scene.props.motion;
    let name = |i: usize| motion.list[i].name.as_str();
    let truth = lines();
    let mut frames: Vec<&str> = truth.iter().filter(|l| l.len() > 5).map(|l| l[0].as_str()).collect();
    frames.dedup();
    assert_eq!(frames, ["1", "12", "24"]);
    for frame in frames {
        let at = motion.solve(frame.parse().unwrap(), &scene.props.rigs.rigs, false);
        let ours = SceneRelation::list(&at, |i| motion.list[i].parent);
        let want: Vec<&Vec<String>> = truth.iter().filter(|l| l[0] == frame).collect();
        assert_eq!(ours.len(), want.len(), "frame {frame}: {:?}", ours.iter().map(|r| name(r.child)).collect::<Vec<_>>());
        for l in want {
            let found = match l[1].as_str() {
                // PARENT child parent child-place parent-origin
                "PARENT" => ours.iter().any(|r| {
                    name(r.child) == l[2] && r.parent.map(name) == Some(l[3].as_str()) && near(&r.from, &floats(&l[4])) && near(&r.to, &floats(&l[5]))
                }),
                // TARGET owner constraint target-place owner-place
                _ => ours.iter().any(|r| name(r.child) == l[2] && r.parent.is_none() && near(&r.from, &floats(&l[4])) && near(&r.to, &floats(&l[5]))),
            };
            assert!(found, "frame {frame}: {l:?} in {ours:?}");
        }
    }
}
