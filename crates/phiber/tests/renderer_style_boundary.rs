//! Structural boundary for resolved renderer style consumption.
//!
//! Feature: phiber-resolved-ui-style-settings
//! Validates: Requirements 8.1-8.7, 13.7
//!
//! Covered visual values must flow through `BRenderCtx::style`. The only
//! numeric allowlist is normalized GPUI/scene coordinate mechanics; backend
//! color conversion is isolated to `color.rs`.

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Violation {
    path: String,
    line: usize,
    rule: &'static str,
    detail: String,
}

impl Violation {
    fn diagnostic(&self) -> String {
        format!(
            "{}:{}: [{}] {}",
            self.path, self.line, self.rule, self.detail
        )
    }
}

fn gpui_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2/view/draw")
}

fn fixture(name: &str) -> String {
    fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/renderer_style_boundary")
            .join(name),
    )
    .unwrap_or_else(|error| panic!("failed to read style fixture {name}: {error}"))
}

fn rust_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            // A sibling `*_tests.rs` is a test module in a file of its own
            // (`#[path] mod …_tests`), not a renderer: a fixture's `px(12.0)`
            // is the number the assertion is ABOUT, so reading it as a design
            // literal asks the test to launder its own inputs through the
            // style system.
            let is_test_module =
                path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.ends_with("_tests.rs"));
            if !is_test_module {
                out.push(path);
            }
        }
    }
}

/// `source` with its `#[cfg(test)]` module blanked out, so a fixture's own
/// numbers are not read as design literals. A test's `px(200.0)` is the value
/// the assertion is ABOUT — a bounds it builds to drive a hit test — and
/// asking it to come from the style would mean the test could no longer state
/// the number it is checking. The same reason `rust_files` skips a sibling
/// `*_tests.rs`: the module is a test either way, in the file or beside it.
fn without_tests(source: &str) -> String {
    let Some(at) = source.find("#[cfg(test)]") else {
        return source.to_string();
    };
    let Some(brace) = source[at..].find('{').map(|offset| at + offset) else {
        return source.to_string();
    };
    let mut depth = 0usize;
    let mut out = source[..at].to_string();
    for (offset, byte) in source[brace..].bytes().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    // Keep the lines, lose the code: the violation line
                    // numbers below are counted in the ORIGINAL source.
                    out.extend(source[at..brace + offset + 1].chars().map(|c| if c == '\n' { '\n' } else { ' ' }));
                    out.push_str(&without_tests(&source[brace + offset + 1..]));
                    return out;
                }
            }
            _ => {}
        }
    }
    out
}

fn code_only(source: &str) -> String {
    let source = &without_tests(source);
    let bytes = source.as_bytes();
    let mut out = bytes.to_vec();
    let (mut at, mut block_depth) = (0, 0usize);
    while at < bytes.len() {
        if block_depth > 0 {
            if bytes[at..].starts_with(b"/*") {
                block_depth += 1;
                out[at] = b' ';
                out[at + 1] = b' ';
                at += 2;
            } else if bytes[at..].starts_with(b"*/") {
                block_depth -= 1;
                out[at] = b' ';
                out[at + 1] = b' ';
                at += 2;
            } else {
                if bytes[at] != b'\n' {
                    out[at] = b' ';
                }
                at += 1;
            }
        } else if bytes[at..].starts_with(b"//") {
            while at < bytes.len() && bytes[at] != b'\n' {
                out[at] = b' ';
                at += 1;
            }
        } else if bytes[at..].starts_with(b"/*") {
            block_depth = 1;
            out[at] = b' ';
            out[at + 1] = b' ';
            at += 2;
        } else if bytes[at] == b'"' {
            out[at] = b' ';
            at += 1;
            while at < bytes.len() {
                if bytes[at] == b'\\' {
                    out[at] = b' ';
                    if at + 1 < bytes.len() {
                        if bytes[at + 1] != b'\n' {
                            out[at + 1] = b' ';
                        }
                        at += 2;
                    } else {
                        at += 1;
                    }
                } else {
                    let done = bytes[at] == b'"';
                    if bytes[at] != b'\n' {
                        out[at] = b' ';
                    }
                    at += 1;
                    if done {
                        break;
                    }
                }
            }
        } else {
            at += 1;
        }
    }
    String::from_utf8(out).expect("replacement preserves UTF-8 ASCII boundaries")
}

fn line_at(source: &str, byte: usize) -> usize {
    source.as_bytes()[..byte.min(source.len())]
        .iter()
        .filter(|value| **value == b'\n')
        .count()
        + 1
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn identifier_occurrences(source: &str, needle: &str) -> Vec<usize> {
    source
        .match_indices(needle)
        .filter_map(|(index, _)| {
            let before = index
                .checked_sub(1)
                .and_then(|at| source.as_bytes().get(at));
            let after = source.as_bytes().get(index + needle.len());
            (!before.is_some_and(|byte| is_identifier_byte(*byte))
                && !after.is_some_and(|byte| is_identifier_byte(*byte)))
            .then_some(index)
        })
        .collect()
}

fn call_with_numeric_argument(source: &str, call: &str) -> Vec<usize> {
    let bytes = source.as_bytes();
    let mut hits = Vec::new();
    for start in identifier_occurrences(source, call) {
        let mut at = start + call.len();
        while bytes.get(at).is_some_and(|byte| byte.is_ascii_whitespace()) {
            at += 1;
        }
        if bytes.get(at) != Some(&b'(') {
            continue;
        }
        at += 1;
        while bytes.get(at).is_some_and(|byte| byte.is_ascii_whitespace()) {
            at += 1;
        }
        if bytes.get(at) == Some(&b'-') {
            at += 1;
        }
        if bytes.get(at).is_some_and(|byte| byte.is_ascii_digit()) {
            hits.push(start);
        }
    }
    hits
}

fn method_with_numeric_argument(source: &str, method: &str) -> Vec<usize> {
    let needle = format!(".{method}");
    source
        .match_indices(&needle)
        .filter_map(|(start, _)| {
            let suffix = &source[start + needle.len()..];
            call_with_numeric_argument(suffix, "")
                .first()
                .is_some_and(|offset| *offset == 0)
                .then_some(start)
        })
        .collect()
}

fn relative_mechanics_allowed(path: &str) -> bool {
    // `flex/apply.rs` and its split-out tail `flex/place.rs` are where a
    // `BFlex` becomes GPUI style — the flex-basis and full-length mechanics
    // that used to be written out in `zone.rs`.
    matches!(
        path,
        "flex/apply.rs"
            | "flex/place.rs"
            // A gpui `Element` laying out its own line box: `relative(1.)` is
            // "fill the width you are given", the mechanics this allows for.
            | "text_input.rs"
            | "canvas/image.rs"
            | "canvas/node.rs"
            | "canvas/view3d/mod.rs"
            | "canvas/view3d/hit.rs"
    )
}

fn style_violations(path: &str, source: &str) -> Vec<Violation> {
    let code = code_only(source);
    let mut violations = Vec::new();

    for (needle, rule, detail) in [
        (
            "METRICS",
            "legacy-metrics",
            "accesses legacy global `METRICS`",
        ),
        ("BMetrics", "legacy-metrics", "accesses legacy `BMetrics`"),
        (
            "DEFAULT_RADII",
            "palette-radius",
            "accesses default/palette radii directly",
        ),
        (
            "BRadiusScale",
            "palette-radius",
            "accesses a radius scale directly from renderer code",
        ),
        (
            ".radii.",
            "palette-radius",
            "accesses radius fields directly instead of a semantic role",
        ),
    ] {
        for byte in code.match_indices(needle).map(|(byte, _)| byte) {
            violations.push(Violation {
                path: path.into(),
                line: line_at(source, byte),
                rule,
                detail: detail.into(),
            });
        }
    }
    for byte in code.match_indices("metric::").map(|(byte, _)| byte) {
        violations.push(Violation {
            path: path.into(),
            line: line_at(source, byte),
            rule: "legacy-metrics",
            detail: "accesses a legacy metrics namespace".into(),
        });
    }

    for byte in call_with_numeric_argument(&code, "px") {
        violations.push(Violation {
            path: path.into(),
            line: line_at(source, byte),
            rule: "hardcoded-dimension",
            detail: "passes a numeric design literal directly to `px`".into(),
        });
    }

    if !relative_mechanics_allowed(path) {
        for byte in call_with_numeric_argument(&code, "relative") {
            violations.push(Violation {
                path: path.into(),
                line: line_at(source, byte),
                rule: "hardcoded-geometry",
                detail:
                    "uses a numeric relative geometry literal outside the GPUI mechanics allowlist"
                        .into(),
            });
        }
    }

    for method in [
        "gap",
        "p",
        "px",
        "py",
        "m",
        "mx",
        "my",
        "w",
        "h",
        "min_w",
        "min_h",
        "max_w",
        "max_h",
        "rounded",
        "border",
        "text_size",
        "opacity",
    ] {
        for byte in method_with_numeric_argument(&code, method) {
            violations.push(Violation {
                path: path.into(),
                line: line_at(source, byte),
                rule: "hardcoded-style",
                detail: format!("passes a numeric design literal directly to `.{method}`"),
            });
        }
    }

    for call in ["hsla", "rgba", "rgb", "BColor"] {
        for byte in call_with_numeric_argument(&code, call) {
            violations.push(Violation {
                path: path.into(),
                line: line_at(source, byte),
                rule: "hardcoded-color",
                detail: format!("constructs `{call}` from a numeric color literal"),
            });
        }
    }

    violations
}

#[test]
fn production_renderers_use_resolved_style_without_covered_literals() {
    let root = gpui_root();
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    files.sort();
    assert!(!files.is_empty(), "no GPUI renderer source files found");

    let violations: Vec<_> = files
        .iter()
        .flat_map(|path| {
            let source = fs::read_to_string(path).expect("renderer source is readable");
            let relative = path
                .strip_prefix(&root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            style_violations(&relative, &source)
        })
        .collect();

    assert!(
        violations.is_empty(),
        "renderer style boundary violations:\n{}",
        violations
            .iter()
            .map(Violation::diagnostic)
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn resolved_style_fixture_passes() {
    let violations = style_violations("pass/resolved_style.rs", &fixture("resolved_style.rs"));
    assert!(
        violations.is_empty(),
        "{}",
        violations
            .iter()
            .map(Violation::diagnostic)
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn negative_fixtures_report_file_line_and_rule() {
    let cases = [
        ("legacy_metrics.rs", "legacy-metrics", 2),
        ("palette_radius.rs", "palette-radius", 2),
        ("hardcoded_size.rs", "hardcoded-dimension", 2),
        ("hardcoded_color.rs", "hardcoded-color", 2),
    ];
    for (name, rule, line) in cases {
        let path = format!("fail/{name}");
        let violations = style_violations(&path, &fixture(name));
        assert!(
            violations.iter().any(|violation| violation.path == path
                && violation.line == line
                && violation.rule == rule),
            "fixture {name} did not produce expected diagnostic; got: {}",
            violations
                .iter()
                .map(Violation::diagnostic)
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}

#[test]
fn comments_and_strings_do_not_trigger_style_rules() {
    let source = r#"
// let width = px(12.0); METRICS.radius
const NOTE: &str = "BRadiusScale hsla(0.0, 0.0, 0.0, 1.0)";
fn view(c: &BRenderCtx) { div().w(px(c.style.dimensions.panel_width)); }
"#;
    assert!(style_violations("comments.rs", source).is_empty());
}
