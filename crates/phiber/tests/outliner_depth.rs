//! The outliner's rows keep their height, its lines run the whole list, and
//! every level that holds something can be opened.

use phiber::v2::state::{BKey as _, OutlinerKey};
use phiber::v2::part::outliner::tree;
use phiber::v2::state::{BState, val::V_TRUE};
use phiber::v2::view::{BView, BViewItem};

fn rows(s: &BState) -> Vec<tree::BOutlineRow> {
    tree::flatten(s)
}

fn source(relative: &str) -> String {
    std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/v2").join(relative),
    )
    .unwrap_or_else(|error| panic!("{relative}: {error}"))
}

#[test]
fn a_data_row_can_be_opened_and_holds_what_the_mesh_has() {
    let mut s = BState::new();
    let object = rows(&s)
        .into_iter()
        .find(|r| r.depth == 2)
        .expect("an object row");
    s.set(&OutlinerKey::Expanded(object.id).path(), V_TRUE);

    let data = rows(&s)
        .into_iter()
        .find(|r| r.depth == 3)
        .expect("the object's data row");
    assert!(
        data.expanded.is_some(),
        "the third level must carry a disclosure arrow, not read as a leaf"
    );

    s.set(&OutlinerKey::Expanded(data.id).path(), V_TRUE);
    let held: Vec<String> = rows(&s)
        .into_iter()
        .filter(|r| r.depth == 4)
        .map(|r| r.label)
        .collect();
    assert_eq!(held, vec!["Vertex Groups", "UV Maps", "Material"]);
}

#[test]
fn a_row_never_shrinks_and_the_list_scrolls_instead() {
    // `keeps_size()` is the `BFlex` spelling of `.flex_shrink_0()`: it sets
    // `flex_shrink = 0.0`, which is exactly what GPUI's helper compiles to
    // (`flex/share.rs`, applied in `flex/apply.rs`).
    let tree_src = source("view/draw/tree/mod.rs");
    assert!(
        tree_src.contains(".keeps_size()"),
        "a tree row must keep its height when the host is short"
    );
    let share = source("view/draw/flex/share.rs");
    assert!(
        share.contains("self.shrink = Some(0.0);"),
        "keeps_size has to mean flex-shrink: 0, or the row would squash after all"
    );
    let column = source("view/draw/column.rs");
    assert!(
        column.contains(".overflow_y_scroll()"),
        "the list scrolls rather than squashing its rows"
    );
}

/// Each row draws its OWN hierarchy run, one line per indent unit of its own
/// depth, before its arrow slot — so a line exists exactly where a row is
/// nested and ends with the row it belongs to. It used to be one backdrop of
/// the deepest row's depth behind the whole column, in the accent and the text
/// tone, which put the same loud columns through every row and told nothing
/// about any one of them (`view/draw/tree/guide.rs` carries the reasoning).
#[test]
fn a_rows_hierarchy_lines_are_its_own_depth_in_the_other_bands_colour() {
    let row = source("view/draw/tree/mod.rs");
    assert!(row.contains("BTreeIndent"), "the depth is an indent run before the arrow slot");
    assert!(row.contains("BTreePart::IndentUnit"), "one unit per level of the row's own depth");
    assert!(row.contains("guide::guide_fill(shade, ctx)"), "and the line is filled from the row's band");
    // Not a backdrop: nothing paints guides behind the column any more.
    let column = source("view/draw/column.rs");
    assert!(!column.contains("guides_backdrop"), "no backdrop of the deepest depth behind the list");
    // The colour is the OPPOSITE zebra band over the editor's body, so a line
    // is subtle against the row it crosses and never a third colour.
    let guide = source("view/draw/tree/guide.rs");
    assert!(guide.contains("shade.opposite_band()"));
    assert!(guide.contains("host_body_surface(ctx.host)"));
    assert!(!guide.contains("colors.text_dim"), "the loud text tone is what this replaced");
}

#[test]
fn every_data_block_icon_has_an_asset() {
    let icons = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/icons");
    let mut s = BState::new();
    let object = rows(&s).into_iter().find(|r| r.depth == 2).unwrap();
    s.set(&OutlinerKey::Expanded(object.id).path(), V_TRUE);
    let data = rows(&s).into_iter().find(|r| r.depth == 3).unwrap();
    s.set(&OutlinerKey::Expanded(data.id).path(), V_TRUE);
    for row in rows(&s) {
        assert!(
            icons.join(format!("{}.svg", row.icon)).exists(),
            "outliner row {:?} has no icon asset ({})",
            row.label,
            row.icon
        );
    }
    let _ = BView::Empty;
    let _: fn(&BState) -> BView =
        |s| phiber::v2::part::outliner::BOutlinerItem::new().build(s);
}
