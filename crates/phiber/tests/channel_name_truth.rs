//! Every channel name Blender prints, phiber prints too
//! (`truth/blender_channel_name.py`, `truth/channel_name.txt`).
//!
//! A channel row's label is not stored in a `.blend`. `getname_anim_fcurve`
//! builds it, every draw, from the array item's letter and
//! `RNA_property_ui_name(prop)` — and that second piece is a **declared**
//! string, not a transformation of the identifier. `energy` displays as
//! "Power". `spot_size` as "Beam Angle". `mid_level` as "Midlevel". Nothing
//! derives those, so the only way to be right about them is to read them
//! out of Blender and hold the implementation to what came back.
//!
//! Which is the point of this file. `row::name` used to carry an
//! eighteen-entry hand table and title-case everything else, so a light's
//! power read "Energy", a Displace modifier's midlevel read "Mid Level",
//! and the whole channel list was plausible-looking and wrong. The tables
//! are generated now; this is what keeps them generated.
//!
//! The dump's columns are `owner, identifier, name, letters, length`. Every
//! row is checked: either phiber prints Blender's name for it, or the
//! identifier is one of the four in
//! [`row::name::AMBIGUOUS`](phiber::v2::view::draw::canvas::time::row::name::AMBIGUOUS)
//! whose owner the path does not carry — and then the recorded spelling has
//! to be exactly what phiber prints and exactly what Blender showed for one
//! of the owners. There is no third outcome and nothing is skipped.

use phiber::phiphx::blend::BFCurve;
use phiber::v2::view::draw::canvas::time::row::name::{AMBIGUOUS, OWNED, fcurve_name};
use std::collections::BTreeSet;
use std::path::PathBuf;

/// One row of the dump.
struct Property {
    owner: String,
    identifier: String,
    name: String,
    letters: String,
}

fn truth() -> Vec<Property> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/truth/channel_name.txt");
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let field: Vec<&str> = line.split('\t').collect();
            (field.len() >= 5).then(|| Property {
                owner: field[0].to_string(),
                identifier: field[1].to_string(),
                name: field[2].to_string(),
                letters: field[3].to_string(),
            })
        })
        .collect()
}

/// An RNA path that puts `identifier` on the struct the dump names it for,
/// and the `" (struct)"` suffix `owner()` will add for that path.
///
/// The owner matters for exactly the rows in `OWNED`, where it is the whole
/// question; for everything else a bare identifier is the object's own
/// property and `owner()` finds nothing to append. The suffix is returned
/// rather than stripped off the result afterwards, because a UI name can
/// carry its own bracket — "Delta Rotation (Euler)" and "Lock Rotation (4D
/// Angle)" are both real, and cutting at the first `" ("` eats half of each.
fn path_for(owner: &str, identifier: &str) -> (String, &'static str) {
    if !OWNED.iter().any(|row| row.property == identifier) {
        return (identifier.to_string(), "");
    }
    match owner {
        "PoseBone" => (format!(r#"pose.bones["B"].{identifier}"#), " (B)"),
        "ShapeKey" => (format!(r#"key_blocks["K"].{identifier}"#), " (K)"),
        "ConstraintTargetBone" => (format!(r#"constraints["C"].targets[0].{identifier}"#), " (C)"),
        owner if owner.starts_with("Con:") => (format!(r#"constraints["C"].{identifier}"#), " (C)"),
        owner if owner.starts_with("Mod:") => (format!(r#"modifiers["M"].{identifier}"#), " (M)"),
        _ => (identifier.to_string(), ""),
    }
}

/// What `fcurve_name` makes of that path, less the suffix the path forced.
/// The suffix is `owner()`'s business and `row::name`'s own tests pin it.
fn printed(owner: &str, identifier: &str, index: u32) -> String {
    let (path, suffix) = path_for(owner, identifier);
    let name = fcurve_name(&BFCurve::new(path, index, Vec::new()));
    name.strip_suffix(suffix).unwrap_or(&name).to_string()
}

#[test]
fn every_property_blender_names_phiber_names_the_same_way() {
    let truth = truth();
    assert!(truth.len() > 600, "the dump is there and parses: {} rows", truth.len());

    let mut wrong = Vec::new();
    for row in &truth {
        let have = printed(&row.owner, &row.identifier, 0);
        let want = match row.letters.chars().next() {
            Some(letter) => format!("{letter} {}", row.name),
            None => row.name.clone(),
        };
        if have == want {
            continue;
        }
        // The only permitted difference: an identifier whose owner the path
        // does not carry, printing the spelling `AMBIGUOUS` records.
        let recorded = AMBIGUOUS
            .iter()
            .find(|recorded| recorded.property == row.identifier)
            .is_some_and(|recorded| have == recorded.printed.0);
        if !recorded {
            wrong.push(format!("{} {}: Blender shows {want:?}, phiber prints {have:?}", row.owner, row.identifier));
        }
    }
    assert!(wrong.is_empty(), "{} of {} names differ:\n  {}", wrong.len(), truth.len(), wrong.join("\n  "));
}

#[test]
fn every_letter_blender_gives_an_item_phiber_gives_too() {
    // `RNA_property_array_item_char` bounds the index at 4 rather than at
    // the array's length, so a three-component vector still answers 'W' for
    // index 3 — and index 4 falls through to the bracketed form for every
    // subtype. Both are easy to get wrong and neither shows up at index 0.
    let mut wrong = Vec::new();
    for row in truth().iter().filter(|row| !row.letters.is_empty()) {
        for (index, letter) in row.letters.chars().enumerate() {
            let have = printed(&row.owner, &row.identifier, index as u32);
            let want = format!("{letter} {}", row.name);
            if have != want && !AMBIGUOUS.iter().any(|recorded| recorded.property == row.identifier) {
                wrong.push(format!("{} {}[{index}]: want {want:?}, got {have:?}", row.owner, row.identifier));
            }
        }
        let past = printed(&row.owner, &row.identifier, 4);
        let want = format!("[4]{}", row.name);
        if past != want && !AMBIGUOUS.iter().any(|recorded| recorded.property == row.identifier) {
            wrong.push(format!("{} {}[4]: want {want:?}, got {past:?}", row.owner, row.identifier));
        }
    }
    assert!(wrong.is_empty(), "{} letters differ:\n  {}", wrong.len(), wrong.join("\n  "));
}

#[test]
fn every_recorded_divergence_is_one_blender_actually_has() {
    // `AMBIGUOUS` is a record of four real differences, not a licence to
    // print anything. Each row has to name an identifier the dump carries
    // under more than one spelling, and the spelling it claims the odd
    // owner shows has to be one of them.
    let truth = truth();
    for recorded in AMBIGUOUS {
        let property = recorded.property;
        // `(name, letters)`, because `offset` is the row where the name
        // agrees across owners and only the letters part company.
        let shown: BTreeSet<(&str, &str)> = truth
            .iter()
            .filter(|row| row.identifier == property)
            .map(|row| (row.name.as_str(), row.letters.as_str()))
            .collect();
        assert!(shown.len() > 1, "{property} is recorded as ambiguous, but the dump gives it only {shown:?}");
        for (want, whose) in [(recorded.printed, "the spelling it prints"), (recorded.odd_shows, recorded.odd)] {
            assert!(
                shown.contains(&want),
                "{property}: {whose} is {want:?}, which Blender shows for no owner: {shown:?}"
            );
        }
    }
}

#[test]
fn every_owner_sensitive_name_is_one_blender_really_splits() {
    // And `OWNED` the same way round: a row claiming two spellings has to
    // be a property the dump really carries under both.
    let truth = truth();
    for row in OWNED {
        let spelling: BTreeSet<&str> = truth
            .iter()
            .filter(|property| property.identifier == row.property)
            .map(|property| property.name.as_str())
            .collect();
        for want in [row.named.0, row.other.0] {
            assert!(
                spelling.contains(want),
                "{} is recorded as reading {want:?}, which Blender shows for no owner: {spelling:?}",
                row.property
            );
        }
    }
}

#[test]
fn the_dump_covers_the_properties_a_rig_actually_drives() {
    // The first generated pass walked objects and datablocks only, and so
    // lost `influence`, `mute`, `value` and `weight` — the four most
    // commonly driven properties in any rig, none of which belong to the
    // object. This is what stops that happening twice.
    let truth = truth();
    for property in ["influence", "mute", "value", "weight", "energy", "spot_size", "mid_level"] {
        assert!(
            truth.iter().any(|row| row.identifier == property),
            "the dump has no {property}, so no test here covers how it reads"
        );
    }
}
