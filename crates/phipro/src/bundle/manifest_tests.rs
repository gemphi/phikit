//! The manifest, and the escaping its predecessors did not do.

use super::*;

#[test]
fn a_manifest_carries_what_the_app_reads_back() {
    let made = BManifest::new("rig", "/assets/rig.blend", 12, 3456);
    let text = made.text().expect("it serialises");
    let back: serde_json::Value = serde_json::from_str(&text).expect("and parses");
    assert_eq!(back["name"], "rig");
    assert_eq!(back["default_scene"], "rig");
    assert_eq!(back["version"], BManifest::VERSION);
    assert_eq!(back["object_count"], 12);
    assert_eq!(back["vertex_count"], 3456);
    assert_eq!(back["settings"]["grid_visible"], true);
    assert_eq!(back["settings"]["physics_enabled"], false);
    assert!(text.ends_with('\n'), "a text file ends in a newline");
}

#[test]
fn a_quotation_mark_in_the_name_still_gives_readable_json() {
    // The interpolated version wrote `"name": "He said "hi""`, which is a
    // manifest the app cannot open. Nothing is renamed to avoid it.
    let made = BManifest::new(r#"He said "hi""#, r#"C:\a\b.blend"#, 1, 1);
    let text = made.text().expect("it serialises");
    let back: serde_json::Value = serde_json::from_str(&text).expect("and parses");
    assert_eq!(back["name"], r#"He said "hi""#);
    assert_eq!(back["source"], r#"C:\a\b.blend"#);
}

#[test]
fn a_backslash_path_survives_the_round_trip() {
    // A Windows path is the common case, and `\a` is an escape in JSON.
    let made = BManifest::new("rig", r#"C:\Users\p\new\rig.blend"#, 0, 0);
    let back: serde_json::Value = serde_json::from_str(&made.text().expect("serialises")).expect("parses");
    assert_eq!(back["source"], r#"C:\Users\p\new\rig.blend"#);
}

#[test]
fn the_creation_time_is_a_time_or_null_and_never_a_misleading_zero() {
    let made = BManifest::new("rig", "rig.blend", 0, 0);
    // The epoch itself would read as 1970, so it is `None` rather than 0.
    assert!(made.created.is_some_and(|at| at > 1_600_000_000), "{:?}", made.created);
}
