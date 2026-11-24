//! Things as Once City's tables make them (Phiogy's `data/once`), and the
//! files Phiogy keeps of them.

use std::path::{Path, PathBuf};

use super::{ThingForm, ThingNode, ThingSet};

fn tables() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../appkit/app/phiogy/data/once")
}

fn once() -> ThingSet {
    ThingSet::read(&tables())
}

#[test]
fn every_part_has_a_known_look_and_every_thing_inside_is_a_thing() {
    let set = once();
    assert!(set.thing.len() >= 50, "{} things", set.thing.len());
    for thing in &set.thing {
        for row in &thing.row {
            match row.form {
                ThingForm::Thing => assert!(set.named(&row.from).is_some(), "{}/{}: no thing {}", thing.name, row.name, row.from),
                _ => assert!(set.look.contains_key(&row.look), "{}/{}: no look {}", thing.name, row.name, row.look),
            }
        }
    }
}

#[test]
fn every_thing_builds_with_triangles_in_every_part() {
    let set = once();
    for thing in &set.thing {
        let tree = set.build(&thing.name).expect("built");
        assert!(tree.triangle_count() > 0, "{} is empty", thing.name);
        for (part, _) in tree.mesh() {
            assert!(part.triangle_count() > 0, "{}/{} is empty", thing.name, part.name);
            assert_eq!(part.normal.len(), part.index.len(), "{}/{}: a normal a corner", thing.name, part.name);
        }
    }
}

#[test]
fn sixteen_couches_and_sixteen_trees() {
    let set = once();
    let city = set.named("once-city").expect("the city");
    let placed = |row: &str| city.row.iter().filter(|r| r.name == row).map(|r| r.from.clone()).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(placed("couch").len(), 16, "{:?}", placed("couch"));
    assert_eq!(placed("tree").len(), 16, "{:?}", placed("tree"));
}

#[test]
fn a_letter_is_fitted_to_its_size() {
    let set = once();
    let gate = set.build("letter-gate").expect("the gate");
    let o = gate.mesh().into_iter().find(|(p, _)| p.name == "O").expect("the O").0.clone();
    let [lo, hi] = o.extent();
    assert!(((hi[1] - lo[1]) - 5.5).abs() < 1e-3, "{lo:?} {hi:?}");
    assert!(((hi[2] - lo[2]) - 0.8).abs() < 1e-3, "{lo:?} {hi:?}");
    // As wide as the O is for its height (Phi-Pill Bold: 600 of 635).
    assert!(((hi[0] - lo[0]) / 5.5 - 600.0 / 635.0).abs() < 0.02, "{lo:?} {hi:?}");
    // Its counter is a door: a ray through its middle meets nothing.
    let centre = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
    let hit = o.point.iter().any(|p| (p[0] - centre[0]).abs() < 0.3 && (p[1] - centre[1]).abs() < 0.3);
    assert!(!hit, "the O has no door");
}

#[test]
fn the_bird_is_drawn_from_phiogy_s_mark() {
    let set = once();
    let tree = set.build("tree-bird").expect("the bird tree");
    let hedge = tree.mesh().into_iter().find(|(p, _)| p.name == "hedge").expect("the hedge").0.clone();
    let [lo, hi] = hedge.extent();
    assert!(((hi[1] - lo[1]) - 2.2).abs() < 1e-3);
    // The mark is higher than wide: its box (446.9 by 565.15).
    assert!((hi[0] - lo[0]) / (hi[1] - lo[1]) < 0.85, "{lo:?} {hi:?}");
}

#[test]
fn a_swapped_thing_is_written_out_and_a_plain_one_refers_to_its_file() {
    let set = once();
    let gate = set.build("letter-gate").expect("the gate");
    let amber = gate.node.iter().find_map(|n| match n {
        ThingNode::Group { name, from, .. } if name == "amber" => Some(from.clone()),
        _ => None,
    });
    assert_eq!(amber, Some(Some("lioness-amber".to_string())));
    let onyx = set.build("lioness-onyx").expect("Onyx");
    let ThingNode::Group { from, node, .. } = &onyx.node[0] else { panic!("Onyx holds the lioness") };
    assert!(from.is_none());
    assert!(node.iter().any(|n| matches!(n, ThingNode::Mesh { look, .. } if look.name == "onyx")));
    let text = set.build("once-city").expect("the city").to_usda();
    assert!(text.contains("prepend references = @./letter-gate.usda@"));
    assert!(text.contains("prepend references = @./lotus-dome-lilac.usda@"));
}

#[test]
fn a_repeat_moves_then_turns_each_copy() {
    let set = once();
    let palm = set.build("tree-palm").expect("the palm");
    let frond = palm.mesh().into_iter().find(|(p, _)| p.name == "frond").expect("fronds").0.clone();
    assert!(frond.point.len() >= 7 * 10, "{}", frond.point.len());
    // Seven fronds round the trunk: some lean each way.
    let [lo, hi] = frond.extent();
    assert!(lo[0] < -1.0 && hi[0] > 1.0 && lo[2] < -1.0 && hi[2] > 1.0, "{lo:?} {hi:?}");
}

#[test]
fn a_bloom_has_its_lobes_and_its_hole() {
    let set = once();
    let ring = set.build("wave-ring").expect("the wavy ring");
    let band = ring.mesh().into_iter().find(|(p, _)| p.name == "band").expect("its band").0.clone();
    let radius: Vec<f32> = band.point.iter().map(|p| (p[0] * p[0] + p[2] * p[2]).sqrt()).collect();
    let (near, far) = radius.iter().fold((f32::MAX, 0.0f32), |(a, b), r| (a.min(*r), b.max(*r)));
    // Nine lobes 0.07 deep, reaching 60 m at the farthest from its own
    // middle; the hole 0.82 as large, its nearest between two lobes, a
    // little less for the rounded edge.
    assert!((far - 60.0).abs() < 0.1, "{far}");
    let hole = 60.0 * 0.82 * 0.93 / 1.07;
    assert!(near < hole && near > hole - 1.5, "{near} {hole}");
    assert!((band.extent()[1][1] - band.extent()[0][1] - 1.6).abs() < 1e-3);
}

#[test]
fn a_turned_form_scalloped_swells_at_each_lobe() {
    let set = once();
    let tower = set.build("cake-tower").expect("the tower");
    let tier = tower.mesh().into_iter().find(|(p, _)| p.name == "tier").expect("its tier").0.clone();
    // Eight lobes 0.08 deep on a 3 m radius: out along x, in between.
    let at = |angle: f32| tier.point.iter().filter(|p| (p[2].atan2(p[0]) - angle).abs() < 0.01).map(|p| (p[0] * p[0] + p[2] * p[2]).sqrt()).fold(0.0f32, f32::max);
    assert!((at(0.0) - 3.0 * 1.08).abs() < 0.02, "{}", at(0.0));
    assert!((at(std::f32::consts::PI / 8.0) - 3.0 * 0.92).abs() < 0.02, "{}", at(std::f32::consts::PI / 8.0));
}

#[test]
fn a_look_of_two_colours_turns_from_the_back_to_the_front() {
    let set = once();
    let jet = set.build("jet-long").expect("the long jet");
    let (part, shade) = jet.node.iter().find_map(|n| match n {
        ThingNode::Mesh { part, shade, .. } if part.name == "body" => Some((part, shade)),
        _ => None,
    }).expect("its body");
    assert_eq!(shade.len(), part.point.len());
    let back = part.point.iter().zip(shade).min_by(|a, b| a.0[0].total_cmp(&b.0[0])).expect("a point").1;
    let front = part.point.iter().zip(shade).max_by(|a, b| a.0[0].total_cmp(&b.0[0])).expect("a point").1;
    assert!(front[1] < back[1] - 0.2, "pearl at the back to rose at the front: {back:?} {front:?}");
    assert!(set.build("jet-long").expect("built").to_usda().contains("interpolation = \"vertex\""));
}

#[test]
fn phiogy_once_files_match_their_tables() {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("../appkit/app/phiogy/asset/once");
    let set = once();
    for thing in &set.thing {
        let file = folder.join(format!("{}.usda", thing.name));
        let on_disk = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{}: {e} - run `cargo run -p phiphx --example thing_usd -- {} {}`", file.display(), tables().display(), folder.display()));
        let made = set.build(&thing.name).expect("built").to_usda();
        assert!(on_disk.replace('\r', "") == made, "{} drifted from data/once; regenerate with the thing_usd example", file.display());
    }
}
