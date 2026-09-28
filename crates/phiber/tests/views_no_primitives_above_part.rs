//! Coordinator purity lint.
//!
//! Feature: phiber-blender-space-parity
//! Validates: Requirements 50.3, 52.14
//!
//! Inside `v2/{space,zone,area,host,region,pane}/**`, no source line may
//! construct a `BView` primitive (`BView::Row`, `BView::Column`,
//! `BView::Panel`, `BView::Icon`, `BView::Label`, `BView::Button`,
//! `BView::Popover`, `BView::Field`, `BView::VLabel`, `BView::MenuItem`,
//! `BView::MenuSep`, `BView::Tab`, `BView::Canvas`).
//! Only the level's own coordinator variant (`Space`, `Zone`, `Area`,
//! `Host`, `Region`, `Pane`) and `BView::Empty` are allowed.

use std::fs;
use std::path::{Path, PathBuf};

fn walk_rust<P: AsRef<Path>>(root: P, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root.as_ref()) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk_rust(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

fn phiber_src() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

const FORBIDDEN_PRIMITIVES: &[&str] = &[
    "BView::Row",
    "BView::Column",
    "BView::Panel",
    "BView::Icon",
    "BView::Label",
    "BView::Button",
    "BView::SortHeader",
    "BView::Card",
    "BView::Popover",
    "BView::Field",
    "BView::VLabel",
    "BView::MenuItem",
    "BView::MenuSep",
    "BView::Tab",
    "BView::Canvas",
];

const COORDINATOR_LEVELS: &[&str] = &["space", "zone", "area", "host", "region", "pane"];

fn is_coordinator_path(path: &Path) -> bool {
    // A file is a coordinator file if its path contains one of the
    // COORDINATOR_LEVELS as an ancestor directory AND does not sit
    // inside `part/` (Parts are allowed to construct primitives).
    let components: Vec<String> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str().map(|s| s.to_string()))
        .collect();
    if components.iter().any(|c| c == "part") {
        return false;
    }
    components
        .iter()
        .any(|c| COORDINATOR_LEVELS.contains(&c.as_str()))
}

#[test]
fn no_primitives_above_part() {
    let mut files = Vec::new();
    walk_rust(phiber_src(), &mut files);
    assert!(!files.is_empty());

    let mut violations: Vec<String> = Vec::new();
    for path in &files {
        if !is_coordinator_path(path) {
            continue;
        }
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        for (lineno, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            for prim in FORBIDDEN_PRIMITIVES {
                if trimmed.contains(prim) {
                    violations.push(format!(
                        "{}:{}: coordinator emits primitive {prim}: {}",
                        path.display(),
                        lineno + 1,
                        line.trim(),
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "coordinator files emit view primitives outside `part/`:\n{}",
        violations.join("\n"),
    );
}
