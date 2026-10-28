//! Which collection instances are drawn, against Blender 5.2's own
//! `depsgraph.object_instances` (`tests/fixtures/make_instance_visibility.py`,
//! `tests/truth/instance_visibility.txt`): the instancers Blender skips —
//! disabled in viewports, hidden, in an excluded or restricted collection,
//! in another scene — draw nothing; members disabled in viewports, or in a
//! child collection that is, are left out; an instancer in a collection
//! hidden only by the eye still draws (the depsgraph's
//! `BASE_ENABLED_AND_MAYBE_VISIBLE_IN_VIEWPORT` ignores it); and a member
//! that instances a collection that instances another draws them two levels
//! deep, at the composed matrices (`make_recursive_duplis`); and two
//! instancers standing at one place over a collection that is itself in the
//! scene draw its mesh three times there — nothing merges coincident
//! copies, `make_duplis_collection` runs once per instancer.

use phiphx::blend::BlendFile;

const FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/instance_visibility.blend");
const TRUTH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/truth/instance_visibility.txt");

/// (top instancer, drawn object, world matrix column by column).
type Copy = (String, String, Vec<f32>);

fn ours() -> Vec<Copy> {
    let file = BlendFile::parse_bytes(&std::fs::read(FILE).expect("fixture")).expect("parse");
    let scene = file.extract_scene().expect("scene");
    scene
        .objects
        .iter()
        .filter(|o| o.is_instance)
        .map(|o| {
            let top = o.name.split('/').next().unwrap_or_default().to_string();
            let object = o.name.rsplit('/').next().unwrap_or_default().to_string();
            (top, object, o.matrix_world.iter().flatten().copied().collect())
        })
        .collect()
}

fn blender() -> Vec<Copy> {
    std::fs::read_to_string(TRUTH)
        .expect("truth")
        .lines()
        .map(|line| {
            let mut parts = line.split('\t');
            let (top, object) = (parts.next().expect("instancer"), parts.next().expect("object"));
            let matrix = parts.next().expect("matrix").split(' ').map(|v| v.parse().expect("a number")).collect();
            (top.to_string(), object.to_string(), matrix)
        })
        .collect()
}

#[test]
fn the_instances_blender_draws_and_no_others() {
    let mut ours = ours();
    let truth = blender();
    for (top, object, want) in &truth {
        let close = |c: &Copy| &c.0 == top && &c.1 == object && c.2.iter().zip(want).all(|(a, b)| (a - b).abs() < 1e-4);
        let at = ours.iter().position(close).unwrap_or_else(|| panic!("{top}/…/{object} is drawn at {want:?}"));
        ours.remove(at);
    }
    let extra: Vec<String> = ours.iter().map(|c| format!("{}/…/{}", c.0, c.1)).collect();
    assert!(extra.is_empty(), "drawn, though Blender does not: {extra:?}");

    let by = |top: &str| truth.iter().filter(|c| c.0 == top).count();
    for skipped in ["Disabled", "Eyed", "Excluded", "Restricted", "Elsewhere"] {
        assert_eq!(by(skipped), 0, "{skipped} draws nothing in Blender");
    }
    let shown: Vec<&str> = truth.iter().filter(|c| c.0 == "Shown").map(|c| c.1.as_str()).collect();
    assert!(shown.contains(&"Bud"), "two levels deep: Shown → Inner → Stem → Bud");
    assert!(!shown.contains(&"Ghost") && !shown.contains(&"Nut"), "hidden members stay hidden");
    assert_eq!(by("Dimmed"), shown.len(), "the eye on a collection does not stop the depsgraph");

    // Coincident copies: Blender lists one instance of Tile per instancer at
    // the same matrix, and the matching above consumed one of ours for each.
    let atop: Vec<&Copy> = truth.iter().filter(|c| c.0.starts_with("Atop")).collect();
    assert_eq!(atop.len(), 2, "Atop and Atop.001 each draw Tile");
    assert_eq!(atop[0].2, atop[1].2, "at the same place");
    let file = BlendFile::parse_bytes(&std::fs::read(FILE).expect("fixture")).expect("parse");
    let scene = file.extract_scene().expect("scene");
    let tiles = scene.objects.iter().filter(|o| o.name.ends_with("Tile")).count();
    assert_eq!(tiles, 3, "Tile itself and its two coincident instances");
}
