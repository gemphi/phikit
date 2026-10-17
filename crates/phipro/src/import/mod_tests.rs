//! The import service, over the parts of it that need no asset file.
//!
//! What it does with a real `.blend` is `tests/writes_a_bundle.rs`, which
//! has a fixture to read; these are the refusals and the order they happen
//! in, which is the part that was wrong before.

use super::*;

#[test]
fn an_unusable_name_is_refused_before_the_asset_is_read() {
    // The old order parsed the whole asset and then failed on the name, so
    // a mistyped path cost a full parse. There is no file at this path at
    // all, and the error is still about the name.
    let write = BProjectWrite::new(Path::new("/nowhere/rig:1.blend"), Path::new("/out"));
    let why = write.run(&|_, _| {}).expect_err("a colon is not a directory name");
    assert!(matches!(why, BWriteError::Name(BNameError::Unusable(_, ':'))), "{why:?}");
}

#[test]
fn a_missing_asset_is_reported_as_a_read() {
    let base = std::env::temp_dir().join(format!("phipro-missing-{}", std::process::id()));
    let write = BProjectWrite::new(Path::new("/nowhere/rig.blend"), &base);
    let why = write.run(&|_, _| {}).expect_err("there is no such file");
    assert!(matches!(why, BWriteError::Read(_)), "{why:?}");
    // And nothing was created for a project that cannot be made.
    assert!(!base.join("rig.phx").exists(), "a failed import leaves no directory");
    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn every_refusal_says_what_is_wrong_in_words() {
    for why in [
        BWriteError::Name(BNameError::NoStem),
        BWriteError::Read("not a .blend".to_string()),
        BWriteError::Write(BBundlePart::Manifest, "read-only".to_string()),
        BWriteError::Encode("a float that is not a number".to_string()),
    ] {
        let said = why.message();
        assert!(said.len() > 8 && !said.starts_with('B'), "{why:?} reads as {said:?}");
    }
}
