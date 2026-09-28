//! Focused source audit for task 4.2 hierarchy-surface migration.
//!
//! Feature: phiber-resolved-ui-style-settings
//! Validates: Requirements 3.4, 3.5, 5.1-5.7, 11.2, 11.3, 14.9

use std::fs;
use std::path::PathBuf;

fn renderer(name: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2/view/draw");
    let path = module_file(root.join(format!("{name}.rs")));
    let mut source =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    // A level whose layout outgrew its walker keeps its `BFlexed` impl in
    // `<level>/part.rs`; it is the same level, so read them together.
    if let Ok(part) = fs::read_to_string(module_file(root.join(name).join("part.rs"))) {
        source.push_str(&part);
    }
    source
}

#[test]
fn hierarchy_surfaces_use_resolved_semantic_roles() {
    for (name, role) in [
        ("space", "Workspace"),
        ("zone", "Zone"),
        ("area", "Area"),
        ("host", "Host"),
        ("region", "Region"),
    ] {
        let source = renderer(name);
        assert!(
            source.contains(&format!("BStyleRole::{role}")),
            "{name} does not resolve its semantic surface role"
        );
        assert!(
            !source.contains("c.p."),
            "{name} still reads covered palette fields directly"
        );
        assert!(
            !source.contains("METRICS"),
            "{name} still reads covered legacy metrics directly"
        );
    }
}

#[test]
fn opaque_descendants_receive_corners_without_rectangular_clipping() {
    let area = renderer("area");
    let host = renderer("host");
    let region = renderer("region");

    assert!(!area.contains(".overflow_hidden()"));
    assert!(area.contains("with_corners(BCorners::ALL)"));
    assert!(host.contains("assign_row(BRowPosition::at"));
    assert!(host.contains("assign_cell("));
    assert!(host.contains("region::surface_opacity"));
    assert!(region.contains("child_surface_plan("));
    assert!(region.contains("compatible_inner_radius("));
}

#[test]
fn hierarchy_composition_and_viewport_fill_contracts_remain_present() {
    let space = renderer("space");
    let host = renderer("host");
    let region = renderer("region");

    // The space renderer decomposes its children into top / middle / bottom
    // bands (see `BSpaceComposition`) and lays the middle band out as a flex
    // row of zones. Assert the composition fields and the row axis remain
    // present rather than pinning to specific local-variable names.
    for band in ["top", "middle", "bottom"] {
        assert!(
            space.contains(&format!("{band}:")) || space.contains(&format!("{band} ")),
            "space lost {band} composition"
        );
    }
    // The middle band is a row of zones — a `BSpacePart` decision now.
    assert!(space.contains("Self::Middle => BFlex::row()"));
    for band in [
        "let mut top",
        "let mut left",
        "let mut center",
        "let mut right",
        "let mut bottom",
    ] {
        assert!(host.contains(band), "host lost {band} composition");
    }
    // A region without a fixed size fills its host and may shrink: the
    // viewport-fill contract, as the region's `BFlexed` decision.
    assert!(region.contains("(None, _) => {\n                let flex = flex.fills().min_height(BLen::Zero).full();"));
}

/// A module is a file or a folder; read whichever spelling exists.
fn module_file(path: std::path::PathBuf) -> std::path::PathBuf {
    if path.exists() {
        return path;
    }
    // `x.rs` asked for, `x/mod.rs` exists.
    let folder = path.with_extension("").join("mod.rs");
    if folder.exists() {
        return folder;
    }
    // `x/mod.rs` asked for, `x.rs` exists.
    if path.file_name().is_some_and(|n| n == "mod.rs") {
        if let Some(parent) = path.parent() {
            let file = parent.with_extension("rs");
            if file.exists() {
                return file;
            }
        }
    }
    path
}
