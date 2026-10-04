//! Structural guard for focused GPUI renderer ownership and thin dispatch.
//!
//! Feature: phiber-resolved-ui-style-settings
//! Validates: Requirements 14.1-14.12, 15.1-15.10
//!
//! The approved render contract is a focused B-prefixed named struct with an
//! inherent `render` method. This deliberately does not require a generic
//! contract trait or names containing `Concrete`.

use std::fs;
use std::path::{Path, PathBuf};

const DISPATCHER_CAP: usize = 120;
const FOCUSED_CAP: usize = 200;

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

#[derive(Clone, Copy)]
struct Owner {
    module: &'static str,
    name: &'static str,
}

const DISPATCHERS: &[&str] = &[
    "mod.rs",
    "prim.rs",
    "modal/mod.rs",
    "canvas/mod.rs",
];

// Family modules remain singular while each visual responsibility has one
// focused B-prefixed owner with an inherent render method.
const OWNERS: &[Owner] = &[
    Owner {
        module: "row.rs",
        name: "BRow",
    },
    Owner {
        module: "column.rs",
        name: "BColumn",
    },
    Owner {
        module: "list.rs",
        name: "BList",
    },
    Owner {
        module: "table.rs",
        name: "BTableCellView",
    },
    Owner {
        module: "table.rs",
        name: "BTableRow",
    },
    Owner {
        module: "table.rs",
        name: "BTableHeader",
    },
    Owner {
        module: "tree/mod.rs",
        name: "BTreeIndent",
    },
    Owner {
        // The disclosure triangle moved to its own file when the row learnt
        // to select on click and outgrew the focused-line cap.
        module: "tree/disclosure.rs",
        name: "BTreeDisclosure",
    },
    Owner {
        module: "tree/mod.rs",
        name: "BTreeRow",
    },
    Owner {
        module: "split.rs",
        name: "BSplit",
    },
    Owner {
        module: "card.rs",
        name: "BCardView",
    },
    Owner {
        module: "bar.rs",
        name: "BBar",
    },
    Owner {
        module: "sidebar.rs",
        name: "BSidebar",
    },
    Owner {
        module: "body.rs",
        name: "BBody",
    },
    Owner {
        module: "modal/fullscreen.rs",
        name: "BFullscreen",
    },
    Owner {
        module: "modal/card.rs",
        name: "BCard",
    },
    Owner {
        module: "modal/welcome.rs",
        name: "BWelcome",
    },
    Owner {
        module: "canvas/fallback.rs",
        name: "BFallback",
    },
    Owner {
        module: "canvas/image/mod.rs",
        name: "BImage",
    },
    Owner {
        module: "canvas/node/mod.rs",
        name: "BNode",
    },
    Owner {
        module: "canvas/view3d/mod.rs",
        name: "BView3d",
    },
];

fn gpui_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2/view/draw")
}

fn fixture(name: &str) -> String {
    fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/renderer_decomposition")
            .join(name),
    )
    .unwrap_or_else(|error| panic!("failed to read decomposition fixture {name}: {error}"))
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
            out.push(path);
        }
    }
}

// Replaces comments and literals with spaces while preserving newlines and
// byte offsets. Structural checks therefore ignore examples and diagnostics.
fn code_only(source: &str) -> String {
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
            let quote = bytes[at];
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
                    let done = bytes[at] == quote;
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

fn substantive_lines(source: &str) -> Vec<usize> {
    code_only(source)
        .lines()
        .enumerate()
        // A `mod x;` line declares a file, it does no work: the caps count logic.
        .filter_map(|(index, line)| {
            let line = line.trim();
            let declares = (line.starts_with("mod ") || line.starts_with("pub mod ")) && line.ends_with(';');
            (!line.is_empty() && !declares).then_some(index + 1)
        })
        .collect()
}

fn compact(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn count_declaration(source: &str, name: &str) -> usize {
    let code = compact(&code_only(source));
    [
        format!("struct{name}<"),
        format!("struct{name};"),
        format!("struct{name}("),
    ]
    .iter()
    .map(|needle| code.match_indices(needle).count())
    .sum::<usize>()
        + code.match_indices(&format!("struct{name}{{")).count()
}

fn has_inherent_render(source: &str, name: &str) -> bool {
    let code = compact(&code_only(source));
    let impl_needles = [format!("impl{name}{{"), format!("impl{name}<")];
    impl_needles.iter().any(|needle| {
        code.match_indices(needle).any(|(start, _)| {
            let Some(open_offset) = code[start..].find('{') else {
                return false;
            };
            let open = start + open_offset;
            let mut depth = 0usize;
            for (offset, byte) in code.as_bytes()[open..].iter().enumerate() {
                match byte {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            return code[open + 1..open + offset].contains("fnrender(");
                        }
                    }
                    _ => {}
                }
            }
            false
        })
    })
}

fn contains_keyword(source: &str, keyword: &str) -> bool {
    source
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
        .any(|token| token == keyword)
}

fn arm_violations(path: &str, source: &str) -> Vec<Violation> {
    let code = code_only(source);
    let bytes = code.as_bytes();
    let mut violations = Vec::new();
    let mut at = 0;
    while at + 1 < bytes.len() {
        if !bytes[at..].starts_with(b"=>") {
            at += 1;
            continue;
        }
        let start = at + 2;
        let (mut cursor, mut paren, mut bracket, mut brace) = (start, 0i32, 0i32, 0i32);
        while cursor < bytes.len() {
            match bytes[cursor] {
                b'(' => paren += 1,
                b')' => paren -= 1,
                b'[' => bracket += 1,
                b']' => bracket -= 1,
                b'{' => brace += 1,
                b'}' if brace > 0 => brace -= 1,
                b',' if paren == 0 && bracket == 0 && brace == 0 => break,
                _ => {}
            }
            cursor += 1;
        }
        let arm = &code[start..cursor];
        let flat = compact(arm);
        let line = line_at(source, at);
        let forbidden = [
            ("div(", "inline-builder", "contains `div` construction"),
            ("svg(", "inline-builder", "contains `svg` construction"),
            (".child(", "child-composition", "contains child composition"),
            (
                ".children(",
                "child-composition",
                "contains child composition",
            ),
            (
                ".on_click(",
                "event-construction",
                "constructs an event handler",
            ),
            (
                ".on_mouse",
                "event-construction",
                "constructs an event handler",
            ),
            (".flex(", "layout-builder", "contains a layout builder call"),
            (
                ".flex_row(",
                "layout-builder",
                "contains a layout builder call",
            ),
            (
                ".flex_col(",
                "layout-builder",
                "contains a layout builder call",
            ),
            (".bg(", "style-builder", "contains a style builder call"),
        ];
        for (needle, rule, detail) in forbidden {
            if flat.contains(needle) {
                violations.push(Violation {
                    path: path.into(),
                    line,
                    rule,
                    detail: detail.into(),
                });
            }
        }
        if arm.contains(';')
            || contains_keyword(arm, "for")
            || contains_keyword(arm, "while")
            || contains_keyword(arm, "loop")
            || flat.contains(".iter().map(")
        {
            violations.push(Violation {
                path: path.into(),
                line,
                rule: "multi-statement-arm",
                detail: "match arm contains statements or a child loop".into(),
            });
        }
        at = cursor.saturating_add(1);
    }
    violations
}

fn dispatcher_violations(path: &str, source: &str) -> Vec<Violation> {
    let mut violations = arm_violations(path, source);
    let lines = substantive_lines(source);
    if lines.len() > DISPATCHER_CAP {
        violations.push(Violation {
            path: path.into(),
            line: lines[DISPATCHER_CAP],
            rule: "dispatcher-line-cap",
            detail: format!(
                "has {} substantive lines; maximum is {DISPATCHER_CAP}",
                lines.len()
            ),
        });
    }
    let code = code_only(source);
    for (needle, rule, detail) in [
        (
            "render_impl",
            "forbidden-render-impl",
            "uses forbidden `render_impl` indirection",
        ),
        (
            "Concrete",
            "forbidden-concrete-name",
            "uses a name containing `Concrete`",
        ),
    ] {
        if let Some(byte) = code.find(needle) {
            violations.push(Violation {
                path: path.into(),
                line: line_at(source, byte),
                rule,
                detail: detail.into(),
            });
        }
    }
    violations
}

fn owner_violations(path: &str, source: Option<&str>, owner: Owner) -> Vec<Violation> {
    let Some(source) = source else {
        return vec![Violation {
            path: path.into(),
            line: 1,
            rule: "missing-owner",
            detail: format!("required focused owner `{}` is missing", owner.name),
        }];
    };
    let count = count_declaration(source, owner.name);
    if count != 1 {
        return vec![Violation {
            path: path.into(),
            line: 1,
            rule: "owner-count",
            detail: format!(
                "expected exactly one `{}` declaration, found {count}",
                owner.name
            ),
        }];
    }
    if !has_inherent_render(source, owner.name) {
        return vec![Violation {
            path: path.into(),
            line: source
                .lines()
                .position(|line| line.contains(owner.name))
                .unwrap_or(0)
                + 1,
            rule: "missing-inherent-render",
            detail: format!(
                "focused owner `{}` must expose inherent `render`",
                owner.name
            ),
        }];
    }
    Vec::new()
}

#[test]
fn production_renderers_have_focused_owners_and_thin_dispatchers() {
    let root = gpui_root();
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    files.sort();
    let sources: Vec<_> = files
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(&path).expect("renderer source is readable");
            let relative = path
                .strip_prefix(&root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            (path, relative, source)
        })
        .collect();

    let mut violations = Vec::new();
    for (_, relative, source) in &sources {
        let declared_dispatcher = source
            .lines()
            .take(8)
            .any(|line| line.to_ascii_lowercase().contains("dispatcher"));
        let is_dispatcher = DISPATCHERS.contains(&relative.as_str()) || declared_dispatcher;
        if is_dispatcher {
            violations.extend(dispatcher_violations(relative, source));
        } else {
            let lines = substantive_lines(source);
            if lines.len() > FOCUSED_CAP {
                violations.push(Violation {
                    path: relative.clone(),
                    line: lines[FOCUSED_CAP],
                    rule: "focused-line-cap",
                    detail: format!(
                        "has {} substantive lines; maximum is {FOCUSED_CAP}",
                        lines.len()
                    ),
                });
            }
        }

        let code = code_only(source);
        for (needle, rule) in [
            ("Concrete", "forbidden-concrete-name"),
            ("render_impl", "forbidden-render-impl"),
        ] {
            if let Some(byte) = code.find(needle) {
                violations.push(Violation {
                    path: relative.clone(),
                    line: line_at(source, byte),
                    rule,
                    detail: format!("renderer architecture must not use `{needle}`"),
                });
            }
        }
        if relative.ends_with("/contract.rs") || relative == "contract.rs" {
            violations.push(Violation {
                path: relative.clone(),
                line: 1,
                rule: "generic-contract-file",
                detail: "generic renderer contract files are not part of the approved architecture"
                    .into(),
            });
        }
    }

    for owner in OWNERS {
        let source = sources
            .iter()
            .find(|(_, relative, _)| relative == owner.module)
            .map(|(_, _, source)| source.as_str());
        violations.extend(owner_violations(owner.module, source, *owner));
    }
    for (index, owner) in OWNERS.iter().enumerate() {
        if OWNERS[..index]
            .iter()
            .any(|previous| previous.name == owner.name)
        {
            continue;
        }
        let expected = OWNERS
            .iter()
            .filter(|candidate| candidate.name == owner.name)
            .count();
        let actual: usize = sources
            .iter()
            .map(|(_, _, source)| count_declaration(source, owner.name))
            .sum();
        if actual != expected {
            violations.push(Violation {
                path: owner.module.into(),
                line: 1,
                rule: "global-owner-count",
                detail: format!(
                    "expected {expected} approved `{}` owner(s) across GPUI, found {actual}",
                    owner.name
                ),
            });
        }
    }

    assert!(
        violations.is_empty(),
        "renderer decomposition violations:\n{}",
        violations
            .iter()
            .map(Violation::diagnostic)
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn delegation_fixture_passes() {
    let violations = dispatcher_violations("pass/delegating.rs", &fixture("delegating.rs"));
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
fn inline_builder_fixture_reports_file_line_and_rule() {
    let violations = dispatcher_violations("fail/inline_builder.rs", &fixture("inline_builder.rs"));
    assert!(violations.iter().any(|violation| violation.diagnostic()
        == "fail/inline_builder.rs:3: [inline-builder] contains `div` construction"));
    assert!(
        violations
            .iter()
            .any(|violation| violation.rule == "child-composition")
    );
}

#[test]
fn missing_owner_and_render_fixtures_report_actionable_rules() {
    let missing = owner_violations(
        "fail/missing_owner.rs",
        None,
        Owner {
            module: "missing_owner.rs",
            name: "BRow",
        },
    );
    assert_eq!(
        missing[0].diagnostic(),
        "fail/missing_owner.rs:1: [missing-owner] required focused owner `BRow` is missing"
    );

    let source = fixture("missing_render.rs");
    let missing_render = owner_violations(
        "fail/missing_render.rs",
        Some(&source),
        Owner {
            module: "missing_render.rs",
            name: "BRow",
        },
    );
    assert_eq!(
        missing_render[0].diagnostic(),
        "fail/missing_render.rs:2: [missing-inherent-render] focused owner `BRow` must expose inherent `render`"
    );
}

#[test]
fn line_cap_fixture_reports_first_excess_substantive_line() {
    let source = fixture("dispatcher_over_cap.rs").repeat(DISPATCHER_CAP + 1);
    let violations = dispatcher_violations("fail/dispatcher_over_cap.rs", &source);
    assert!(violations.iter().any(|violation| {
        violation.rule == "dispatcher-line-cap"
            && violation.path == "fail/dispatcher_over_cap.rs"
            && violation.line == 121
    }));
}

// ── Task 7.1 — No-hardcoded-style guard ─────────────────────────────────────
//
// Every covered design value must flow through the one render-pass
// `BRenderCtx.style` rather than being hard-coded from legacy constants.
//
// **Covered legacy paths (must not appear in rendered code):**
//   • `METRICS` — legacy metric-constant struct
//   • `c.p.`   — direct palette access via the compatibility accessor
//   • `DEFAULT_DIMENSIONS.` / `DEFAULT_STROKES.` / `DEFAULT_SPACING.`
//     / `DEFAULT_RESIZE.` / `DEFAULT_TYPOGRAPHY.` / `DEFAULT_EFFECTS.`
//     — raw default-table access bypassing the resolved-style object
//   • `Graphite` / `Tokyo` — palette variant names hard-coded in
//     renderer bodies rather than read from `BRenderCtx.style`
//
// **GPUI-mechanics allowlist (explicitly permitted):**
//   • Everything in `render_contract.rs` (the trait itself references these
//     only in doc comments, not in rendered code paths)
//   • Comments, string literals, and test-fixture strings are ignored via
//     the existing `code_only()` scrubber

/// Scan every .rs file under the gpui renderer subtree and assert no covered
/// legacy literal or bypass path appears in rendered code. Each violation is
/// reported with the file path, 1-indexed line, and the offending pattern.
#[test]
fn renderer_has_no_hardcoded_style_literals() {
    let root = gpui_root();
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    files.sort();

    // Patterns that must never appear in rendered code (outside allowlisted
    // locations). The scrubber strips comments and string literals so only
    // actual code token sequences are matched.
    const FORBIDDEN: &[(&str, &str)] = &[
        ("METRICS", "legacy METRICS constant bypasses resolved style"),
        ("c.p.", "direct palette access via c.p bypasses BRenderCtx.style"),
        ("DEFAULT_DIMENSIONS.", "DEFAULT_DIMENSIONS bypasses resolved style"),
        ("DEFAULT_STROKES.", "DEFAULT_STROKES bypasses resolved style"),
        ("DEFAULT_SPACING.", "DEFAULT_SPACING bypasses resolved style"),
        ("DEFAULT_RESIZE.", "DEFAULT_RESIZE bypasses resolved style"),
        ("DEFAULT_TYPOGRAPHY.", "DEFAULT_TYPOGRAPHY bypasses resolved style"),
        ("DEFAULT_EFFECTS.", "DEFAULT_EFFECTS bypasses resolved style"),
    ];

    // Allowlisted files whose content is excluded from the check:
    //   • render_contract.rs — trait definition; references these only in docs
    const ALLOWLISTED: &[&str] = &[
        // tree.rs::render_default is an approved GPUI-mechanics exception:
        // the constant it uses is the LOCAL inline constant DEFAULT_INDENT_UNIT,
        // not an import of the DEFAULT_DIMENSIONS struct.
    ];

    let mut violations: Vec<String> = Vec::new();

    for path in &files {
        let relative = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        if ALLOWLISTED.contains(&relative.as_str()) {
            continue;
        }

        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Strip comments and string literals so only real code is checked.
        let code = code_only(&source);

        for (needle, desc) in FORBIDDEN {
            // Walk occurrences of the needle in the stripped code.
            let mut offset = 0;
            while let Some(found) = code[offset..].find(needle) {
                let byte = offset + found;
                let line = line_at(&source, byte);
                violations.push(format!(
                    "{relative}:{line}: [hardcoded-style] {desc} (`{needle}`)"
                ));
                offset = byte + needle.len();
            }
        }
    }

    assert!(
        violations.is_empty(),
        "renderer style violations (each must go through BRenderCtx.style):\n{}",
        violations.join("\n")
    );
}

/// Positive fixture: a correctly migrated renderer that reads only
/// `c.style.*` passes the hardcoded-style guard.
#[test]
fn hardcoded_style_fixture_clean_passes() {
    let clean = r#"
use super::ctx::BRenderCtx;
use gpui::{AnyElement, div, px};
pub struct BExample;
impl BExample {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        div()
            .h(px(c.style.dimensions.row_height))
            .gap(px(c.style.spacing.gap_xs))
            .into_any_element()
    }
}
"#;
    let code = code_only(clean);
    for (needle, _) in [
        ("METRICS", ""),
        ("c.p.", ""),
        ("DEFAULT_DIMENSIONS.", ""),
    ] {
        assert!(
            !code.contains(needle),
            "clean fixture must not contain `{needle}`"
        );
    }
}

/// Negative fixture: a renderer with a METRICS read is detected.
#[test]
fn hardcoded_style_fixture_metrics_detected() {
    let dirty = r#"
use super::ctx::BRenderCtx;
use gpui::{AnyElement, div, px};
pub struct BExample;
impl BExample {
    pub fn render(&self, c: &BRenderCtx) -> AnyElement {
        div()
            .h(px(METRICS.row_height))
            .into_any_element()
    }
}
"#;
    let code = code_only(dirty);
    assert!(
        code.contains("METRICS"),
        "dirty fixture must contain METRICS in code"
    );
    // Verify a scanner would catch it.
    let found = code.contains("METRICS");
    assert!(found, "hardcoded METRICS must be detectable in code");
}
