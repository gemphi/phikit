//! A header row and a vertical rail are the same element.
//!
//! The activity bar, the tool strip, the properties navbar and the editor side
//! rail are a strip with three groups, exactly as a header is — only the axis
//! differs. Both go through `strip` rather than each growing its own
//! flex rules.

fn source(relative: &str) -> String {
    std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2").join(relative),
    )
    .unwrap_or_else(|error| panic!("{relative}: {error}"))
}

#[test]
fn a_row_and_a_rail_share_one_strip() {
    for (name, relative) in [
        ("row", "view/draw/row.rs"),
        ("rail", "view/draw/rail.rs"),
    ] {
        let src = source(relative);
        assert!(
            src.contains("strip::place("),
            "the {name} must place its groups through the shared strip"
        );
    }
}

#[test]
fn the_strip_differs_only_by_axis_and_fit() {
    let strip = source("view/draw/strip.rs");
    for arm in [
        "BStripAxis::Row => BAxis::Row",
        "BStripAxis::Column => BAxis::Column",
    ] {
        assert!(strip.contains(arm), "the strip lost {arm}");
    }
    // One placement rule, whichever axis it runs along: first at the start,
    // last at the end, the rest centred between them.
    for rule in [
        "(0, 2) => element.fills().justify(BJustify::Start)",
        "(0, _) => element.keeps_size().justify(BJustify::Start)",
        "element.keeps_size().justify(BJustify::End)",
        "_ => element.fills().justify(BJustify::Center)",
    ] {
        assert!(strip.contains(rule), "the strip lost {rule}");
    }
}

#[test]
fn neither_the_row_nor_the_rail_writes_its_own_flex_rules() {
    for (name, relative) in [
        ("row", "view/draw/row.rs"),
        ("rail", "view/draw/rail.rs"),
    ] {
        let src = source(relative);
        assert!(
            !src.contains("justify_center()"),
            "{name} places a group itself instead of using the strip"
        );
    }
}
