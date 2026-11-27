//! Once City in the library: its place, its things (phiphx's
//! `thing_usd` writes them from `data/once`), and the city read whole,
//! each thing in it through its own file, as deep as they nest.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::Phiogy;
use crate::library::{Library, Place};

fn once() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("asset").join("once")
}

fn library() -> Library {
    Library::own(Path::new(&Phiogy::source()), Rc::default())
}

#[test]
fn step_62_once_city_is_a_place_of_the_library_with_its_things() {
    let place = Place::own();
    let city = place.iter().find(|place| place.key == "once").expect("Once City's place");
    assert_eq!((city.section.as_str(), city.icon.as_str()), ("own", "letter-o"));
    let file: Vec<String> = std::fs::read_dir(once()).expect("its folder").filter_map(|f| f.ok()).map(|f| f.file_name().to_string_lossy().to_string()).collect();
    for name in ["once-city.usda", "letter-gate.usda", "lotus-dome.usda", "petal-dome.usda", "cake-tower.usda", "pad-bubble.usda", "wave-ring.usda", "lioness-amber.usda", "lioness-onyx.usda", "couch-bird.usda", "zebra-rose.usda", "tree-bird.usda"] {
        assert!(file.iter().any(|f| f == name), "no {name} in {file:?}");
    }
    // Thirty-two couches: sixteen of letters, sixteen of the user's scroll
    // drawing (their posts and legs are parts, not couches); eight steel
    // lounge chairs of the photo, and their table.
    let part = ["couch-post.usda", "couch-leg.usda"];
    let couch = file.iter().filter(|f| (f.starts_with("couch-") || f.starts_with("ottoman-") || f.starts_with("pouf-") || f.starts_with("stool-")) && !part.contains(&f.as_str())).count();
    let tree = file.iter().filter(|f| f.starts_with("tree-")).count();
    let lounge = file.iter().filter(|f| f.strip_prefix("lounge-").is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_alphabetic()))).count();
    assert_eq!((couch, craft_of(&file, "couch-scroll-"), lounge, tree), (32, 16, 8, 16), "{file:?}");
    assert!(file.iter().any(|f| f == "table-tulip.usda"));
    // The craft: eight jetskis, four Phijets small to large, four rockets.
    let craft = |lead: &str| craft_of(&file, lead);
    assert_eq!((craft("jetski-"), craft("phijet-long-"), craft("phijet-short-"), craft("rocket-"), craft("flycar-")), (10, 4, 4, 4, 4), "{file:?}");
    // The electric jets of the user's cyan sketch, their cabin and its seats.
    for name in ["jet-long.usda", "jet-short.usda", "jet-inside.usda", "club-seat.usda", "fly-car.usda"] {
        assert!(file.iter().any(|f| f == name), "no {name}");
    }
    // Sixteen lands, ice to the tropics, and their map.
    assert_eq!(craft("land-"), 16, "{file:?}");
    assert!(file.iter().any(|f| f == "lands.usda"));
}

#[test]
fn the_city_is_read_through_each_thing_s_own_file() {
    let model = library().model(&once().join("once-city.usda"));
    // The gate refers to the lioness Amber, Amber to the lioness: her
    // parts stand three things deep in the city.
    let deep = model.part.iter().find(|part| part.path.contains("/gate/amber/lioness/")).expect("Amber in the gate");
    assert!(deep.path.ends_with("/head") || model.part.iter().any(|part| part.path.ends_with("/gate/amber/lioness/head")), "{}", deep.path);
    // Every part once, and only the city's: its things make 1,049,248
    // triangles (the salon 155,160 of them in each of its four lotus
    // domes), and no file it refers to stands at the root beside it.
    let mut path: Vec<&str> = model.part.iter().map(|part| part.path.as_str()).collect();
    path.sort();
    let twice: Vec<&&str> = path.windows(2).filter(|w| w[0] == w[1]).map(|w| &w[0]).take(8).collect();
    assert!(twice.is_empty(), "read twice: {twice:?}");
    assert!(model.part.iter().all(|part| part.path.starts_with("/once_city/")), "a root of another file");
    assert_eq!(model.count, 1_049_248, "{} parts", model.part.len());
    // The tree shows it: the city, the gate, Amber, the lioness, her parts.
    let tree = model.tree();
    assert!(tree.iter().any(|node| node.name == "amber" && node.depth == 2 && node.holds), "{:?}", tree.iter().take(12).collect::<Vec<_>>());
    // Each thing stands where the city puts it: the gate's lionesses by
    // the way in, the gate 30 metres out.
    let head = model.part.iter().find(|part| part.path.ends_with("/gate/amber/lioness/head")).expect("Amber's head");
    let z: f32 = head.piece.triangle.iter().map(|t| t[0][2]).sum::<f32>() / head.piece.triangle.len() as f32;
    assert!(z > 30.0 && z < 38.0, "Amber's head at z {z}");
}

#[test]
fn onyx_wears_black_and_amber_her_own_gold() {
    let library = library();
    let coat = |file: &str| library.model(&once().join(file)).part.iter().find(|part| part.name == "body").map(|part| part.piece.colour).expect("a body");
    let (amber, onyx) = (coat("lioness-amber.usda"), coat("lioness-onyx.usda"));
    assert!(onyx.iter().all(|c| *c < 0.2), "{onyx:?}");
    // Shown as the look says it is seen (#de9f3c), not as USD keeps it.
    let seen = [0xde, 0x9f, 0x3c].map(|c| c as f32 / 255.0);
    assert!(amber.iter().zip(seen).all(|(a, b)| (a - b).abs() < 0.01), "{amber:?}");
}

/// How many of the files are a set's: their name the set's lead and then
/// a letter or a number.
fn craft_of(file: &[String], lead: &str) -> usize {
    file.iter().filter(|f| f.starts_with(lead) && f[lead.len()..].starts_with(|c: char| c.is_ascii_alphanumeric())).count()
}
