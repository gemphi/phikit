//! A function whose first argument is a type's own value is a method of it.
//!
//! The sibling of `a_renderer_hangs_off_its_type.rs`, which makes the same claim
//! about the two canvas view types. That one is about PAINTING; this one is
//! about the rest — formatting a row, naming a key, answering a question about
//! an entry. The convention is the codebase's own, written on `BCtrlView`:
//! *"Each kind is a method of this type rather than a free function beside it,
//! so the widget a kind draws as is reached through the control, not from
//! anywhere."*
//!
//! Four types had behaviour sitting beside them as free functions:
//! `format_size`/`format_time` took a `&BFileRow`, `key` took a
//! `&BDopeFilterSpec`, `is_grouping` took a `&BActivityEntry`, and
//! `value_key`/`open_key` took a `&BPropSpec` — the last two imported into two
//! other modules by name, so a question about a property's key was asked from
//! three places that each had to know the function existed.
//!
//! Narrow BY NAME, deliberately, exactly as the renderer guard is. A
//! receiver-first function is not wrong in general: `add_bookmark(state, path)`
//! takes a `&mut BState` first and is emphatically not behaviour of the store,
//! and `chrome::paint` takes an element and wraps it. A guard that could not
//! tell those apart would be noise. What it can say precisely is: these four
//! types own their own behaviour.

use std::collections::BTreeSet;

/// Every `.rs` under `src/v2`.
fn sources() -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut stack = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    out.push((path.display().to_string(), text));
                }
            }
        }
    }
    out
}

/// Module-level `fn NAME(<anything>: &TYPE, …)` — a method written outside its
/// type. A method is indented inside its `impl`; a free function starts at
/// column zero, which is the whole distinction and it is exact.
fn loose(ty: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (path, text) in sources() {
        for line in text.lines() {
            let starts = line.starts_with("fn ") || line.starts_with("pub fn ") || line.starts_with("pub(");
            if !starts || !line.contains("fn ") {
                continue;
            }
            let Some(args) = line.split_once('(').map(|(_, rest)| rest) else { continue };
            let first = args.split(',').next().unwrap_or("");
            let is_receiver = first
                .split_once(": &")
                .is_some_and(|(_, t)| t.trim_end_matches([')', ' ']).trim_start_matches("mut ") == ty);
            if is_receiver {
                let name = line.split("fn ").nth(1).and_then(|r| r.split('(').next()).unwrap_or(line);
                found.push(format!("{path}: {name}"));
            }
        }
    }
    found
}

/// Receiver-first functions that are deliberately NOT methods, and why.
///
/// The line this guard draws is not "everything that takes a `&T` first". It is:
/// **a type owns its keys and its questions; a presentation module owns how the
/// type is drawn.**
///
/// So `BPropSpec::value_key` and `open_key` moved onto the spec — which state
/// key a property reads is the property's own business, and both were imported
/// by name into `filter/view3d.rs` and `scene/overlay.rs`, so the question was
/// asked from three places that each had to know the function existed. But
/// `prop_view`'s `value`, `control`, `prop_row`, `header_ctrl` and `default_of`
/// stay free: that whole file is the spec's VIEW, and moving it onto the spec
/// would make the data know how it is rendered. Same for `table::row` and
/// `thumb::tile`, which build a `BView` of a file row one way for the details
/// table and another for the thumbnail grid — a `BFileRow` is what the
/// filesystem said, and how a table draws it is the table's business.
///
/// The guard found every one of these and made the call explicit, which is the
/// point of writing them down rather than narrowing the search until they
/// disappeared.
const BY_DESIGN: &[&str] = &[
    // Presentations of a row, owned by the presentations.
    "file/table.rs: row",
    "file/thumb.rs: tile",
    // `prop_view.rs` is the spec's view, start to finish.
    "props/prop_view.rs: default_of",
    "props/prop_view.rs: control",
    "props/prop_view.rs: value",
    "props/prop_view.rs: prop_row",
    "props/prop_view.rs: header_ctrl",
];

#[test]
fn these_types_own_their_own_behaviour() {
    for ty in ["BFileRow", "BDopeFilterSpec", "BActivityEntry", "BPropSpec"] {
        let loose: Vec<String> = loose(ty)
            .into_iter()
            .filter(|found| {
                let tail = found.replace('\\', "/");
                !BY_DESIGN.iter().any(|ok| tail.ends_with(ok))
            })
            .collect();
        assert!(
            loose.is_empty(),
            "these take a `&{ty}` first and are not methods of it:\n  {}\n\
             Move each into an `impl {ty}` block.",
            loose.join("\n  "),
        );
    }
}

#[test]
fn and_still_have_the_methods_they_gained() {
    // A guard that passes because the functions were deleted is no guard.
    let wanted: &[(&str, &str, &[&str])] = &[
        ("src/v2/part/file/listing.rs", "BFileRow", &["format_size", "format_time"]),
        ("src/v2/part/filter/dope.rs", "BDopeFilterSpec", &["key"]),
        ("src/v2/part/activity/option.rs", "BActivityEntry", &["is_grouping"]),
        ("src/v2/part/props/prop_view.rs", "BPropSpec", &["value_key", "open_key"]),
    ];
    for (file, ty, names) in wanted {
        let at = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(file);
        let text = std::fs::read_to_string(&at).unwrap_or_else(|_| panic!("{file} is missing"));
        assert!(text.contains(&format!("impl {ty} {{")), "{file} has no `impl {ty}`");
        let methods: BTreeSet<&str> = text
            .lines()
            .filter(|l| l.contains("fn ") && l.contains("&self"))
            .filter_map(|l| l.split("fn ").nth(1)?.split('(').next())
            .collect();
        for name in *names {
            assert!(methods.contains(name), "`{ty}::{name}` is missing from {file}: {methods:?}");
        }
    }
}
