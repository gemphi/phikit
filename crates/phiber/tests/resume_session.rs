//! Resuming the last session: the layout is restored and the last file is
//! armed for reopen with Load UI off, so the file's own saved layout does not
//! overwrite the restored one. The persistence itself (durable vs transient,
//! the snapshot round-trip) is unit-tested in `v2::app::session`; this checks
//! the whole-app resume decision through `apply_session`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use phiber::v2::app::BPhiberApp;
use phiber::v2::state::{CtxKey, FileKey};

fn snapshot(pairs: &[(&str, &str)]) -> String {
    let map: BTreeMap<String, String> =
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    serde_json::to_string(&map).unwrap()
}

#[test]
fn resuming_restores_the_workspace_and_arms_the_reopen_with_load_ui_off() {
    let blend = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../phiphx/tests/fixtures/overlays.blend");
    assert!(blend.exists(), "fixture present: {}", blend.display());
    let path = blend.display().to_string();

    let mut app = BPhiberApp::new();
    let reopen = app.apply_session(&snapshot(&[
        ("ctx.space", "Shading"),
        ("ctx.scene.loaded_path", &path),
    ]));

    assert_eq!(reopen.as_deref(), Some(path.as_str()), "the last file is the one to reopen");
    assert_eq!(app.state.get(CtxKey::Space), Some("Shading"), "the workspace is restored");
    assert_eq!(app.state.get(FileKey::LoadUi), Some("false"), "the reopen is Load UI off");
    // The file is armed for a fresh read, not claimed already-loaded.
    assert_eq!(app.state.get(&phiber::v2::scene::loaded_path_key()), None);
}

#[test]
fn a_session_whose_file_is_gone_restores_the_layout_but_opens_nothing() {
    let mut app = BPhiberApp::new();
    let reopen = app.apply_session(&snapshot(&[
        ("ctx.space", "Shading"),
        ("ctx.scene.loaded_path", "/no/such/scene.blend"),
    ]));
    assert_eq!(reopen, None, "a missing file is not reopened");
    assert_eq!(app.state.get(CtxKey::Space), Some("Shading"), "the layout still resumes");
    assert_eq!(app.state.get(FileKey::LoadUi), None, "and Load UI is left alone");
}
