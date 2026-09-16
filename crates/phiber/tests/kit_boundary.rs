//! Phiber UI-kit boundary guard.
//!
//! Feature: phiber-resolved-ui-style-settings
//! Validates: Requirements 10.1-10.6, 13.7
//!
//! Phiber composes views with BUIKit-compatible `B*` types. Shared AppKit and
//! service paths remain available for kit-neutral data and logic, but PUIKit
//! and SUIKit paths are forbidden everywhere under `crates/phiber/src`.

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug)]
struct Token<'a> {
    text: &'a str,
    line: usize,
}

fn walk_rust<P: AsRef<Path>>(root: P, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root.as_ref()) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_rust(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn phiber_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn raw_string_start(bytes: &[u8], at: usize) -> Option<(usize, usize)> {
    let mut cursor = at;
    if bytes.get(cursor) == Some(&b'b') {
        cursor += 1;
    }
    if bytes.get(cursor) != Some(&b'r') {
        return None;
    }
    cursor += 1;
    let hashes_start = cursor;
    while bytes.get(cursor) == Some(&b'#') {
        cursor += 1;
    }
    (bytes.get(cursor) == Some(&b'"')).then_some((cursor + 1, cursor - hashes_start))
}

fn rust_tokens(source: &str) -> Vec<Token<'_>> {
    let bytes = source.as_bytes();
    let (mut tokens, mut at, mut line) = (Vec::new(), 0, 1);
    while at < bytes.len() {
        if bytes[at] == b'\n' {
            line += 1;
            at += 1;
        } else if bytes[at..].starts_with(b"//") {
            at += bytes[at..].iter().position(|byte| *byte == b'\n').unwrap_or(bytes.len() - at);
        } else if bytes[at..].starts_with(b"/*") {
            let mut depth = 1;
            at += 2;
            while at < bytes.len() && depth > 0 {
                if bytes[at..].starts_with(b"/*") {
                    depth += 1;
                    at += 2;
                } else if bytes[at..].starts_with(b"*/") {
                    depth -= 1;
                    at += 2;
                } else {
                    line += usize::from(bytes[at] == b'\n');
                    at += 1;
                }
            }
        } else if let Some((mut cursor, hashes)) = raw_string_start(bytes, at) {
            loop {
                let Some(quote) = bytes[cursor..].iter().position(|byte| *byte == b'"') else {
                    at = bytes.len();
                    break;
                };
                let end_quote = cursor + quote;
                line += bytes[at..end_quote].iter().filter(|byte| **byte == b'\n').count();
                let end = end_quote + 1 + hashes;
                if end <= bytes.len() && bytes[end_quote + 1..end].iter().all(|byte| *byte == b'#') {
                    at = end;
                    break;
                }
                cursor = end_quote + 1;
            }
        } else if bytes[at] == b'"' {
            at += 1;
            while at < bytes.len() {
                if bytes[at] == b'\\' {
                    at = (at + 2).min(bytes.len());
                } else {
                    let byte = bytes[at];
                    at += 1;
                    line += usize::from(byte == b'\n');
                    if byte == b'"' {
                        break;
                    }
                }
            }
        } else if bytes[at].is_ascii_alphabetic() || bytes[at] == b'_' {
            let start = at;
            at += 1;
            while at < bytes.len() && (bytes[at].is_ascii_alphanumeric() || bytes[at] == b'_') {
                at += 1;
            }
            tokens.push(Token { text: &source[start..at], line });
        } else if bytes[at..].starts_with(b"::") {
            tokens.push(Token { text: "::", line });
            at += 2;
        } else if bytes[at] == b';' {
            tokens.push(Token { text: ";", line });
            at += 1;
        } else {
            at += 1;
        }
    }
    tokens
}

fn kit_violations(path: &Path, source: &str) -> Vec<String> {
    let tokens = rust_tokens(source);
    let (mut violations, mut in_use, mut in_extern_crate) = (Vec::new(), false, false);
    for (index, token) in tokens.iter().enumerate() {
        if token.text == "use" {
            in_use = true;
        } else if token.text == "extern"
            && tokens.get(index + 1).is_some_and(|next| next.text == "crate")
        {
            in_extern_crate = true;
        } else if token.text == ";" {
            in_use = false;
            in_extern_crate = false;
        } else if matches!(token.text, "puikit" | "suikit")
            && (in_use
                || in_extern_crate
                || tokens.get(index.wrapping_sub(1)).is_some_and(|previous| previous.text == "::")
                || tokens.get(index + 1).is_some_and(|next| next.text == "::"))
        {
            violations.push(format!(
                "{}:{}: forbidden `{}` UI-kit path",
                path.display(),
                token.line,
                token.text
            ));
        }
    }
    violations
}

#[test]
fn no_forbidden_kit_paths() {
    let mut files = Vec::new();
    walk_rust(phiber_src(), &mut files);
    files.sort();
    assert!(!files.is_empty(), "no .rs files found under phiber/src/");

    let violations: Vec<_> = files
        .iter()
        .filter_map(|path| fs::read_to_string(path).ok().map(|source| (path, source)))
        .flat_map(|(path, source)| kit_violations(path, &source))
        .collect();
    assert!(
        violations.is_empty(),
        "phiber references a forbidden UI kit:\n{}",
        violations.join("\n"),
    );
}

#[test]
fn allows_buikit_and_ui_neutral_shared_paths() {
    let fixture = r#"
use apple::buikit::{BButton, BView};
use apple::appkit::gpu::assets::MeshCache;
use apple::services::scene::SceneModel;
use crate::v2::view::BAtom;
// apple::puikit::PButton is documentation, not composition.
const NOTE: &str = "apple::suikit::SButton";
"#;
    assert!(kit_violations(Path::new("allowed.rs"), fixture).is_empty());
}

#[test]
fn rejects_aliases_and_direct_or_qualified_paths() {
    let fixtures = [
        ("use apple::puikit as ui;", "puikit", 1),
        ("use apple::{\n    suikit as ui,\n};", "suikit", 2),
        ("fn view() { ::apple :: puikit :: PButton::new(); }", "puikit", 1),
        ("use suikit::widgets::SView;", "suikit", 1),
        ("extern crate puikit as ui;", "puikit", 1),
    ];
    for (source, kit, line) in fixtures {
        let violations = kit_violations(Path::new("fixture.rs"), source);
        assert_eq!(violations.len(), 1, "fixture should reject {source:?}");
        assert_eq!(
            violations[0],
            format!("fixture.rs:{line}: forbidden `{kit}` UI-kit path")
        );
    }
}

#[test]
fn reports_each_violation_with_file_and_line() {
    let violations = kit_violations(
        Path::new("nested/view.rs"),
        "\nuse apple::puikit::PButton;\n\nfn view() { apple::suikit::SView::new(); }",
    );
    assert_eq!(
        violations,
        [
            "nested/view.rs:2: forbidden `puikit` UI-kit path",
            "nested/view.rs:4: forbidden `suikit` UI-kit path",
        ]
    );
}
