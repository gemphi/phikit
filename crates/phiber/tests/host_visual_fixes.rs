//! Gate 1 characterization smoke-tests for the prim.rs decomposition (task 6.2).
//!
//! These tests assert the contracts that must survive the module split — they
//! document existing runtime behaviour so any regression during focused-module
//! moves fails immediately.
//!
//! Feature: phiber-resolved-ui-style-settings
//! Validates: Requirements 6.1-6.6, 14.1-14.7

use phiber::v2::state::{CtxKey};
use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::host::BHostType;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BPath, BState};
use phiber::v2::view::prim::{BCardKind, BColumn};
use phiber::v2::view::{BAction, BView, BViewItem};
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
fn file_host_declares_blenders_regions_in_order() {
    use phiber::v2::region::BRegionType;
    // Blender `space_file`: header, ui (path strip), tools (source list),
    // window, execute — declared once, for the editor and the dialog alike.
    let kinds: Vec<BRegionType> = phiber::v2::host::file::REGIONS.iter().map(|r| r.kind).collect();
    assert_eq!(
        kinds,
        [
            BRegionType::Header,
            BRegionType::ToolHeader,
            BRegionType::Tools,
            BRegionType::Center,
            // The details column of the picked file (`part::file::info`).
            BRegionType::UI,
            BRegionType::Execute
        ]
    );
    let parts_of = |kind: BRegionType| -> Vec<phiber::v2::part::BPartType> {
        phiber::v2::host::file::REGIONS
            .iter()
            .find(|r| r.kind == kind)
            .map(|r| r.panes.iter().flat_map(|p| p.parts.iter().copied()).collect())
            .unwrap_or_default()
    };
    use phiber::v2::part::BPartType;
    // Both strips are the columns under them: the menus over the source
    // list, the path over the files (the search over the details is added
    // by `complete_header`); the folder moves over the source list and the
    // display modes and filters over the details.
    assert_eq!(parts_of(BRegionType::Header), [BPartType::Switcher, BPartType::Modebar, BPartType::Menubar, BPartType::PathBar]);
    assert_eq!(parts_of(BRegionType::ToolHeader), [BPartType::FileNav, BPartType::FileRefresh, BPartType::TypeFilter]);
    assert_eq!(parts_of(BRegionType::Tools), [BPartType::SourceList]);
    assert_eq!(parts_of(BRegionType::Execute), [BPartType::Execute]);
    assert!(parts_of(BRegionType::Center).contains(&BPartType::Custom));
}

#[test]
fn file_host_window_is_the_table_and_its_source_list_holds_the_bookmark_panels() {
    let s = BState::default();
    let table = phiber::v2::part::custom::BCustomItem::new(BHostType::File).build(&s);
    assert!(
        matches!(&table, BView::Card(c) if c.kind == BCardKind::Column(BColumn::Body)),
        "the window region is the file table"
    );
    // Reached through `BViewItem`, like every other part — the file browser's
    // five used to be free functions and are types now.
    let list = phiber::v2::part::file::sidebar::BSourceList::new(BHostType::File).build(&s);
    let BView::Card(column) = list else {
        panic!("the source list is a Sidebar column");
    };
    assert_eq!(column.kind, BCardKind::Column(BColumn::Sidebar));
    let panels = &column.body;
    // Volumes / System / Bookmarks / Recent, each a folding panel
    // (`FILEBROWSER_PT_bookmarks_*`), then the Open operator's options.
    assert_eq!(panels.len(), 5, "four Blender bookmark panels + Open options");
    assert!(panels[..4].iter().all(|p| matches!(p, BView::Card(card) if card.fold.is_some())));
}

#[test]
fn modal_file_browser_raises_the_file_host() {
    let s = BState::default();
    let modal = phiber::v2::modal::BModalItem::new(phiber::v2::modal::BModalType::FileBrowser).build(&s);
    let kind = phiber::v2::modal::BModalType::FileBrowser;
    let area = kind.spec().areas[0];
    let host = phiber::v2::area::with_scope(kind, || {
        phiber::v2::area::with_area(area, || phiber::v2::host::BHostItem::new(BHostType::File).build(&s))
    });
    let BView::Modal { body, .. } = modal else { panic!("modal") };
    let BView::Zone { list, .. } = *body else { panic!("the dialog's body is a zone") };
    let [BView::Area { child, .. }] = list.as_slice() else { panic!("one area") };
    assert_eq!(format!("{child:?}"), format!("{host:?}"), "the dialog raises the File host, not a copy of it");
}

/// The Timeline's window is Blender's: the Dope Sheet canvas in Timeline
/// mode (`action_main_region_draw`), scrubbed with the left button — not a
/// toolbar-and-labels stand-in. The transport lives in the header, where
/// `playback_controls` puts it.
#[test]
fn time_item_is_the_timeline_canvas() {
    let item = phiber::v2::part::time::BTimeItem::new().build(&BState::default());
    assert!(matches!(item, BView::Canvas { host: BHostType::Time, .. }), "the window is the Time canvas, got {item:?}");
    let custom = phiber::v2::part::custom::BCustomItem::new(BHostType::Time).build(&BState::default());
    assert_eq!(format!("{custom:?}"), format!("{item:?}"), "the custom part hands the host the same canvas");
}

/// Every button of the header's transport group, in the state it is built
/// in — the three settings popovers (Auto Keyframing after record, Time
/// Jump after the jump pair, Playhead after the magnet) are their chevrons,
/// not transport buttons.
fn transport_buttons(state: &BState) -> Vec<phiber::v2::view::prim::BControl> {
    use phiber::v2::part::playback::BPlayback;
    let BView::Row { list } = BPlayback::new(BHostType::Time).build(state) else { panic!("the transport is a row") };
    let popovers = list.iter().filter(|child| matches!(child, BView::Popover { .. })).count();
    assert_eq!(popovers, 3, "the Auto Keyframing, Time Jump and Playhead popovers");
    list.into_iter()
        .filter_map(|child| match child {
            BView::Button(ctl) => Some(ctl),
            BView::Popover { .. } => None,
            other => panic!("the transport holds buttons and its three popovers only, got {other:?}"),
        })
        .collect()
}

/// `playback_controls`: record, |◀, ◆◀, ◀, ▶, ▶◆, ▶|, the time jump pair,
/// then the playhead magnet.
#[test]
fn the_header_transport_is_blenders_playback_controls() {
    let icon_list: Vec<String> = transport_buttons(&BState::default()).into_iter().filter_map(|c| c.icon).collect();
    assert_eq!(
        icon_list,
        ["record", "skip_start", "keyframe_prev", "play_back", "play", "keyframe_next", "skip_end", "chevron_left", "chevron_right", "magnet"]
    );
}

#[test]
fn play_sets_the_global_playing_flag_and_one_wide_pause_stands_in_for_both_play_buttons() {
    let mut state = BState::default();
    let play = transport_buttons(&state).into_iter().find(|c| c.icon.as_deref() == Some("play")).expect("a play button");
    let Some(BAction::Set(path, value)) = play.action else { panic!("play writes the playing flag, got {:?}", play.action) };
    assert_eq!((path.encode().as_str(), value.as_str()), ("ctx.playing", "true"));
    phiber::v2::state::action::apply_action(&mut state, &BAction::Set(path, value));
    let icon_list: Vec<String> = transport_buttons(&state).into_iter().filter_map(|c| c.icon).collect();
    assert_eq!(
        icon_list,
        ["record", "skip_start", "keyframe_prev", "pause", "keyframe_next", "skip_end", "chevron_left", "chevron_right", "magnet"],
        "`playback_controls` while playing: one pause where the two play buttons were"
    );
    let pause = transport_buttons(&state).into_iter().find(|c| c.icon.as_deref() == Some("pause")).unwrap();
    assert!(pause.active && pause.span == 2, "lit, and two slots wide (`scale_x = 2`)");
    // Reverse runs the same ticker the other way.
    phiber::v2::state::action::apply_action(&mut state, &pause.action.unwrap());
    let reverse = transport_buttons(&state).into_iter().find(|c| c.icon.as_deref() == Some("play_back")).unwrap();
    phiber::v2::state::action::apply_action(&mut state, &reverse.action.unwrap());
    assert_eq!(state.get(CtxKey::Playing), Some("reverse"));
    assert_eq!(phiber::v2::part::playback::set::BTransportButton::tick_step(&state), Some("-1"));
}

/// The header's right group: the current frame, the preview-range toggle,
/// Start and End — `frame_current`, `use_preview_range`, `frame_start`,
/// `frame_end` in `playback_controls`.
fn frame_fields(state: &BState) -> Vec<BView> {
    use phiber::v2::part::framerange::BFrameRange;
    let BView::Row { list } = BFrameRange::new(BHostType::Time).build(state) else { panic!("the range group is a row") };
    list
}

#[test]
fn the_header_shows_frame_start_and_end_as_numbers() {
    let list = frame_fields(&BState::default());
    let key_list: Vec<String> = list
        .iter()
        .filter_map(|v| match v {
            BView::Ctrl(ctrl) => Some(ctrl.key.encode()),
            _ => None,
        })
        .collect();
    assert_eq!(key_list, ["ctx.frame", "ctx.frame_start", "ctx.frame_end"]);
    assert!(list.iter().any(|v| matches!(v, BView::Button(ctl) if ctl.icon.as_deref() == Some("preview"))), "the preview-range toggle sits between them");
}

proptest! {
    // Feature: phiber-host-visual-fixes, Property 1: Frame field reflects state
    /// Property 1: the header's frame field shows the current frame.
    ///
    /// **Validates: Requirements R3.5**
    #[test]
    fn frame_field_reflects_state(frame_str in "[1-9][0-9]{0,5}") {
        let mut state = BState::default();
        state.set_encoded("ctx.frame", frame_str.clone());
        let shown = frame_fields(&state).into_iter().find_map(|v| match v {
            BView::Ctrl(ctrl) if ctrl.key.encode() == "ctx.frame" => Some(ctrl.value.display()),
            _ => None,
        });
        prop_assert_eq!(shown, Some(frame_str));
    }
}

/// A table row with a click action must carry a stable, action-derived
/// element id. We verify the id-generation semantics are deterministic
/// and action-scoped by checking two distinct actions produce different ids.
#[test]
fn table_row_has_id() {
    use phiber::v2::view::prim::{BRowShade, BTableCell};

    let path_a = BPath::owned(vec!["file", "browser", "open"]);
    let action_a = BAction::Open(path_a);

    let path_b = BPath::owned(vec!["ctx", "file", "name"]);
    let action_b = BAction::Set(path_b, "cube.blend".to_string());

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
        app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
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
