//! What can and cannot be a project name.
//!
//! The refusals are the point: the version this replaces took
//! `file_stem().unwrap_or_default()`, so each of these produced a path
//! nobody meant and failed — or did not fail — two calls later.

use super::*;

fn name(path: &str) -> Result<String, BNameError> {
    BProjectName::of(Path::new(path)).map(|BProjectName(name)| name)
}

#[test]
fn a_name_is_the_file_stem() {
    assert_eq!(name("/assets/rig.blend").as_deref(), Ok("rig"));
    assert_eq!(name("rig.blend").as_deref(), Ok("rig"));
    // Several extensions: the stem is everything before the last.
    assert_eq!(name("/assets/scene.v2.usda").as_deref(), Ok("scene.v2"));
    // No extension at all is still a stem.
    assert_eq!(name("/assets/rig").as_deref(), Ok("rig"));
}

#[test]
fn surrounding_space_is_not_part_of_the_name() {
    // A directory called " rig " is a different directory from "rig" and
    // reads as the same one, which is the sort of thing a user never
    // intends.
    assert_eq!(name("/assets/ rig .blend").as_deref(), Ok("rig"));
    assert_eq!(name("/assets/   .blend"), Err(BNameError::NoStem));
}

#[test]
fn a_path_with_no_file_name_is_refused() {
    assert_eq!(name("/"), Err(BNameError::NoStem));
    assert_eq!(name(""), Err(BNameError::NoStem));
}

#[test]
fn a_relative_stem_is_refused_rather_than_turned_into_a_directory() {
    // `...phx` under the base would resolve *above* it, and `..phx` is the
    // base itself. Neither is reached the obvious way: `file_stem` of
    // `/assets/..` is `None`, because `..` is a parent component and not a
    // file name. These are the paths that really do give `.` and `..`.
    assert_eq!(name("/assets/..."), Err(BNameError::Relative("..".to_string())));
    assert_eq!(name("/assets/..  ."), Err(BNameError::Relative("..".to_string())));
    assert_eq!(name("/assets/..x"), Err(BNameError::Relative(".".to_string())));
    assert_eq!(name("/assets/. "), Err(BNameError::Relative(".".to_string())));
    // And the ones that give no stem at all.
    assert_eq!(name("/assets/.."), Err(BNameError::NoStem));
    assert_eq!(name("/assets/ . "), Err(BNameError::NoStem));
}

#[test]
fn a_stem_that_cannot_be_a_directory_name_is_refused_with_the_character() {
    // Written on one machine, opened on another, so the strict set wins.
    assert_eq!(name("/a/rig:1.blend"), Err(BNameError::Unusable("rig:1".to_string(), ':')));
    assert_eq!(name("/a/rig?.blend"), Err(BNameError::Unusable("rig?".to_string(), '?')));
    assert_eq!(name("/a/rig*.blend"), Err(BNameError::Unusable("rig*".to_string(), '*')));
    assert_eq!(name("/a/rig|x.blend"), Err(BNameError::Unusable("rig|x".to_string(), '|')));
}

#[test]
fn a_control_character_is_refused_too() {
    let Err(why) = name("/a/rig\u{7}.blend") else { panic!("a bell is not a directory name") };
    assert!(matches!(why, BNameError::Unusable(_, '\u{7}')), "{why:?}");
}

#[test]
fn every_refusal_says_what_is_wrong_in_words() {
    for why in [
        BNameError::NoStem,
        BNameError::Relative("..".to_string()),
        BNameError::Unusable("rig:1".to_string(), ':'),
    ] {
        let said = why.message();
        assert!(!said.is_empty() && !said.contains("NoStem"), "{why:?} reads as {said:?}");
    }
}
