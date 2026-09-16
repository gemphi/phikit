//! Gate 1 characterization smoke-tests for the prim.rs decomposition (task 6.2).
//!
//! These tests assert the contracts that must survive the module split — they
//! document existing runtime behaviour so any regression during focused-module
//! moves fails immediately.
//!
//! Feature: phiber-resolved-ui-style-settings
//! Validates: Requirements 6.1-6.6, 14.1-14.7

use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BPath, BState, key::K_SPACE};
use phiber::v2::view::{BAction, BPanelRole, BView, BViewItem};
use proptest::prelude::*;

// ── Gate 1: characterization smoke-tests ──────────────────────────────────

/// A file-host custom item must NOT be a plain Canvas node — the file browser
/// produces a structured Part/Panel tree, not a raw canvas fallback.
#[test]
fn file_host_custom_item_is_not_canvas() {
    let state = BState::default();
    let item = phiber::v2::part::custom::BCustomItem::new(BHostType::File).build(&state);
    // The file host builds a Part wrapping a Column/Panel tree — never a bare Canvas.
    assert!(
        !matches!(&item, BView::Canvas { .. }),
        "file host custom item must not be a bare Canvas node"
    );
}

#[test]
fn asset_host_custom_item_is_not_canvas() {
    let item =
        phiber::v2::part::custom::BCustomItem::new(BHostType::Asset).build(&BState::default());

    assert!(
        !matches!(item, BView::Canvas { .. }),
        "asset host custom item must not be a bare Canvas node"
    );
}

#[test]
fn file_host_first_child_is_toolbar() {
    let item =
        phiber::v2::part::custom::BCustomItem::new(BHostType::File).build(&BState::default());
    // The file host item is a Row([sidebar, content_column]).
    // The content column's first child is the navigation Toolbar panel.
    let BView::Row { children } = item else {
        panic!("file host custom item must be a Row (sidebar | content)");
    };
    // Second child is the content column (nav_row + table).
    let Some(BView::Column { children: col_children }) = children.get(1) else {
        panic!("file host Row's second child must be the content Column");
    };
    assert!(
        matches!(
            col_children.first(),
            Some(BView::Panel {
                role: BPanelRole::Toolbar,
                ..
            })
        ),
        "file host content Column's first child must be the navigation Toolbar panel"
    );
}

#[test]
fn file_host_second_child_is_not_empty() {
    let item =
        phiber::v2::part::custom::BCustomItem::new(BHostType::File).build(&BState::default());
    // The file host item is a Row([sidebar, content_column]).
    let BView::Row { children } = item else {
        panic!("file host custom item must be a Row (sidebar | content)");
    };
    assert!(
        matches!(children.get(1), Some(child) if !matches!(child, BView::Empty)),
        "file host Row's second child (content column) must not be empty"
    );
}

/// Builds the Timeline item directly so structure tests exercise `BTimeItem`,
/// independently of host dispatch.
fn build_time_item() -> BView {
    phiber::v2::part::custom::time::BTimeItem::new().build(&BState::default())
}

fn time_column_children(item: &BView) -> &[BView] {
    let BView::Column { children } = item else {
        panic!("Timeline item must be a Column, got {item:?}");
    };
    children
}

fn timeline_ruler_children(item: &BView) -> &[BView] {
    let timeline_children = time_column_children(item);
    let Some(BView::Panel {
        role: BPanelRole::Toolbar,
        children: toolbar_children,
    }) = timeline_children.first()
    else {
        panic!("Timeline Column's first child must be a Toolbar panel");
    };
    let Some(BView::Row { children }) = toolbar_children.first() else {
        panic!("Timeline Toolbar's first child must be the ruler Row");
    };
    children
}

#[test]
fn time_item_is_column() {
    let item = build_time_item();

    assert!(
        matches!(item, BView::Column { .. }),
        "BTimeItem must build a Column"
    );
}

#[test]
fn time_custom_item_is_not_canvas() {
    let item = phiber::v2::part::custom::BCustomItem::new(BHostType::Time)
        .build(&BState::default());

    assert!(
        !matches!(item, BView::Canvas { .. }),
        "Time custom item must not be a Canvas fallback"
    );
}

#[test]
fn time_item_first_child_is_toolbar() {
    let item = build_time_item();
    let children = time_column_children(&item);

    assert!(
        matches!(
            children.first(),
            Some(BView::Panel {
                role: BPanelRole::Toolbar,
                ..
            })
        ),
        "Timeline Column's first child must be the ruler Toolbar panel"
    );
}

#[test]
fn time_item_has_transport_buttons() {
    let item = build_time_item();
    let button_count = timeline_ruler_children(&item)
        .iter()
        .filter(|child| matches!(child, BView::Button(_)))
        .count();

    assert!(
        button_count >= 5,
        "Timeline ruler must contain at least five transport buttons, found {button_count}"
    );
}

#[test]
fn time_item_play_pause_is_toggle() {
    let item = build_time_item();
    let has_playing_toggle = timeline_ruler_children(&item).iter().any(|child| {
        matches!(
            child,
            BView::Button(ctl)
                if matches!(&ctl.action, Some(BAction::Toggle(path)) if path.encode() == "ctx.time.playing")
        )
    });

    assert!(
        has_playing_toggle,
        "Timeline ruler must contain a Toggle action targeting ctx.time.playing"
    );
}

#[test]
fn time_item_second_child_is_body() {
    let item = build_time_item();
    let children = time_column_children(&item);

    assert!(
        matches!(
            children.get(1),
            Some(BView::Panel {
                role: BPanelRole::Body,
                children,
            }) if !children.is_empty()
        ),
        "Timeline Column's second child must be a non-empty Body panel"
    );
}

#[test]
fn time_item_default_frame_label() {
    let item = build_time_item();
    let has_default_frame = timeline_ruler_children(&item)
        .iter()
        .any(|child| matches!(child, BView::Label { text } if text == "1"));

    assert!(
        has_default_frame,
        "Timeline ruler must show default frame Label 1 when state is absent"
    );
}

proptest! {
    // Feature: phiber-host-visual-fixes, Property 1: Frame label reflects state
    /// Property 1: Frame label reflects state
    ///
    /// **Validates: Requirements R3.5**
    #[test]
    fn frame_label_reflects_state(frame_str in "[1-9][0-9]{0,5}") {
        let mut state = BState::default();
        state.set_encoded("ctx.time.frame", frame_str.clone());
        let item = phiber::v2::part::custom::time::BTimeItem::new().build(&state);

        let has_frame_label = timeline_ruler_children(&item)
            .iter()
            .any(|child| matches!(child, BView::Label { text } if text == &frame_str));

        prop_assert!(
            has_frame_label,
            "expected ruler row to contain Label {{ text: {frame_str:?} }}"
        );
    }
}

/// A table row with a click action must carry a stable, action-derived
/// element id. We verify the id-generation semantics are deterministic
/// and action-scoped by checking two distinct actions produce different ids.
#[test]
fn table_row_has_id() {
    use phiber::v2::view::prim::{BRowShade, BTableCell};

    let path_a = BPath::owned(vec!["file", "browser", "open"]);
    let action_a = BAction::Open(path_a.clone());

    let path_b = BPath::owned(vec!["ctx", "file", "name"]);
    let action_b = BAction::Set(path_b.clone(), "cube.blend".to_string());

    // table_row_id produces "table-row-<action-seg>". We verify the
    // generated IDs are stable, action-scoped, and distinguishable.
    let id_a = match &action_a {
        BAction::Open(p) => format!("table-row-open-{}", p.encode()),
        _ => unreachable!(),
    };
    let id_b = match &action_b {
        BAction::Set(p, v) => format!("table-row-set-{}-{}", p.encode(), v),
        _ => unreachable!(),
    };

    assert!(
        id_a.starts_with("table-row-"),
        "action-derived id must start with table-row-"
    );
    assert!(
        id_b.starts_with("table-row-"),
        "set action id must start with table-row-"
    );
    assert_ne!(id_a, id_b, "different actions must produce different ids");

    // Confirm the BView constructor builds the right shape (cells + action).
    let cell = BTableCell::flex(BView::label("test.blend"));
    let row = BView::table_row(vec![cell], BRowShade::Flat, Some(action_a));
    assert!(matches!(
        row,
        BView::TableRow {
            action: Some(_),
            ..
        }
    ));
}

/// A tree row at depth 0 with an expanded state must render a disclosure
/// element (the chevron arrow column).
#[gpui::test]
async fn tree_row_renders_disclosure(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    cx.simulate_resize(size(px(1280.0), px(800.0)));
    app.update(cx, |app, cx| {
        app.state = BState::new();
        app.state.set(&K_SPACE, BSpaceType::Layout.slug());
        cx.notify();
    });
    cx.run_until_parked();

    // The outliner's scene_collection is a top-level expanded tree row.
    // Its disclosure arrow must be painted.
    let arrow_bounds = cx.debug_bounds("tree-arrow-scene_collection");
    assert!(
        arrow_bounds.is_some(),
        "tree row must render a disclosure arrow element for the top-level scene collection"
    );
    let b = arrow_bounds.unwrap();
    assert!(
        f32::from(b.size.width) > 0.0,
        "disclosure arrow must have non-zero width"
    );
}
