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

#[test]
fn the_hierarchy_lines_run_the_whole_list() {
    let guide = source("view/draw/tree/guide.rs");
    // A backdrop spanning the container, not a segment inside each row —
    // per-row guides stop where the content does.
    assert!(guide.contains(".absolute().inset_0()"));
    assert!(guide.contains("colors.text_dim"), "the lines are the theme's opposite shade");
    let column = source("view/draw/column.rs");
    assert!(
        column.contains("guides_backdrop"),
        "the list must paint the guides behind its rows"
    );
    assert!(column.contains(".max()"), "one line per indent unit of the deepest row, the whole column down");
    let row = source("view/draw/tree/mod.rs");
    assert!(!row.contains("BTreeGuide"), "no row draws a guide of its own: the depth is an indent before the arrow slot");
    assert!(row.contains("BTreeIndent(self.depth)"));
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
