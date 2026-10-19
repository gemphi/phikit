//! Seven more headers end with the popover Blender gives them.
//!
//! Every host presents itself the same way: what it is on the left, what you
//! are looking for in the middle, how it is filtered or displayed on the
//! right. The four node editors, the Sequencer, the Image and UV editors, the
//! Movie Clip editor and the Spreadsheet had nothing on the right at all —
//! and Blender ends each of those headers with a popover of boxes:
//!
//! - `NODE_PT_overlay` over `SpaceNodeEditor.overlay`;
//! - `SEQUENCER_PT_overlay` over `SpaceSequenceEditor.timeline_overlay`;
//! - `IMAGE_PT_overlay` over `SpaceImageEditor.overlay`;
//! - `CLIP_PT_display`, which Blender calls Display because half of it is
//!   about the clip rather than the marks drawn over it;
//! - `SPREADSHEET_PT_filter`, a filter rather than an overlay but the same
//!   control on the same end of the header.
//!
//! Two rules run through all of them. A FAMILY shares one table and one
//! scope, because four editors that draw the same overlays are how they come
//! to disagree about which boxes exist. And the DEFAULTS are Blender's box by
//! box — mostly on, since an overlay you have to go and switch on is not an
//! overlay, but off where Blender ships it off, because a fresh file that
//! draws more than Blender's is as wrong as one that draws less.
//!
//! Two of these panels are also a sidebar tab (`CLIP_PT_display` in the Clip
//! editor's View tab, `SPREADSHEET_PT_filter` in the Spreadsheet's Filter
//! tab). They read the same table, so a box cannot be ticked in one place and
//! clear in the other.

use phiber::v2::host::BHostType;
use phiber::v2::part::filter::overlay::{self, CLIP, IMAGE, NODE, SEQ, SHEET};
use phiber::v2::part::filter::has_filter;
use phiber::v2::part::sidebar::panel::BSidebarPanels;
use phiber::v2::part::sidebar::set::{BSidebarBody, tabs_for};
use phiber::v2::part::sidebar::show::BSidebarShown;
use phiber::v2::state::val::{V_FALSE, V_TRUE};
use phiber::v2::state::{BKey as _, BState, MenuKey};
use phiber::v2::view::BView;

const NODE_EDITORS: [BHostType; 4] =
    [BHostType::Shader, BHostType::Node, BHostType::GeoNode, BHostType::Compositor];

/// Every host that gained a header popover, and the panel's title.
const CARRIERS: [(BHostType, &str); 9] = [
    (BHostType::Shader, "Overlays"),
    (BHostType::Node, "Overlays"),
    (BHostType::GeoNode, "Overlays"),
    (BHostType::Compositor, "Overlays"),
    (BHostType::Seq, "Overlays"),
    (BHostType::Image, "Overlays"),
    (BHostType::Uv, "Overlays"),
    (BHostType::Clip, "Display"),
    (BHostType::Spreadsheet, "Filters"),
];

/// The popover the header's control opens, when it is open.
fn opened(state: &BState, host: BHostType) -> Option<Vec<BView>> {
    match overlay::view(state, host) {
        BView::Popover { body, open, .. } => open.then_some(body),
        _ => None,
    }
}

fn open_for(host: BHostType) -> BState {
    let title = CARRIERS.iter().find(|(h, _)| *h == host).expect("a carrier").1;
    let mut state = BState::new();
    state.set(&MenuKey::Panel(host.slug(), title).path(), V_TRUE);
    state
}

#[test]
fn all_nine_carry_a_box_popover_now() {
    for (host, _) in CARRIERS {
        assert!(overlay::has_overlay(host), "{host:?} has a box popover");
    }
}

#[test]
fn seven_of_them_reach_it_through_the_header_part_and_two_build_their_own() {
    // `BFilter` is what puts a control on a header's right end, so a host that
    // has a popover normally declares a filter …
    for (host, _) in CARRIERS {
        if matches!(host, BHostType::Image | BHostType::Uv) {
            continue;
        }
        assert!(has_filter(host), "{host:?}'s header right end is no longer empty");
    }
    // … except the Image and UV editors, which build their own right end and
    // end it with the popover themselves (`IMAGE_HT_header` puts the render
    // slot, view layer, pass and channels in front of it). A filter here too
    // would be a SECOND overlay button on the same header.
    for host in [BHostType::Image, BHostType::Uv] {
        assert!(!has_filter(host), "{host:?} would draw the popover twice");
    }
    let header = include_str!("../src/v2/part/image/header/mod.rs");
    assert!(
        header.contains("part::filter::overlay::view(s, self.host)"),
        "the Image header must open the shared popover, not a toggle of its own"
    );
    assert!(
        !header.contains("ImageKey::Overlay"),
        "the toggle it replaced was a key nothing ever read"
    );
}

#[test]
fn each_panel_wears_blenders_own_name_and_an_icon_that_is_not_the_overlay_one() {
    // Calling the Clip editor's Display panel "Overlays" would be a small
    // lie that spreads: the sidebar tab reads the same title.
    for (host, title) in CARRIERS {
        let panel = overlay::panel_for(host).expect("the panel");
        assert_eq!(panel.title, title, "{host:?}");
        assert!(!panel.icon.is_empty(), "{host:?} draws a blank button");
    }
    assert_eq!(overlay::panel_for(BHostType::Clip).unwrap().title, "Display");
    assert_eq!(overlay::panel_for(BHostType::Spreadsheet).unwrap().title, "Filters");
}

#[test]
fn a_family_shares_one_table_and_the_families_do_not_share_with_each_other() {
    for host in NODE_EDITORS {
        assert_eq!(overlay::table_for(host).map(<[_]>::len), Some(NODE.len()), "{host:?}");
    }
    for host in [BHostType::Image, BHostType::Uv] {
        assert_eq!(overlay::table_for(host).map(<[_]>::len), Some(IMAGE.len()), "{host:?}");
    }
    assert_eq!(overlay::table_for(BHostType::Seq).map(<[_]>::len), Some(SEQ.len()));
    assert_eq!(overlay::table_for(BHostType::Clip).map(<[_]>::len), Some(CLIP.len()));
    assert_eq!(overlay::table_for(BHostType::Spreadsheet).map(<[_]>::len), Some(SHEET.len()));
    // Five distinct panels, not one shared one wearing five names.
    let mut families: Vec<&str> =
        CARRIERS.iter().map(|(host, _)| overlay::panel_for(*host).unwrap().family).collect();
    families.sort_unstable();
    families.dedup();
    assert_eq!(families.len(), 5, "families: {families:?}");
}

#[test]
fn the_hosts_of_one_family_share_one_setting_too() {
    // Turning wire colours off in the Shader editor turns them off in the
    // Geometry Nodes editor: one `SpaceNodeEditor.overlay` per space in
    // Blender, but the same struct and the same panel.
    let mut state = BState::new();
    state.set(&overlay::key(BHostType::Shader, &NODE[1]), V_FALSE);
    for host in NODE_EDITORS {
        assert!(!overlay::is_on(&state, host, &NODE[1]), "{host:?}");
    }
    // The Image and UV editors are one space in Blender, so they share too.
    state.set(&overlay::key(BHostType::Image, &IMAGE[1]), V_FALSE);
    assert!(!overlay::is_on(&state, BHostType::Uv, &IMAGE[1]));
    // And nothing here reached the Sequencer or the Clip editor.
    assert!(overlay::is_on(&state, BHostType::Seq, &SEQ[1]));
    assert!(overlay::is_on(&state, BHostType::Clip, &CLIP[0]));
}

#[test]
fn the_control_is_shut_until_it_is_clicked() {
    for (host, _) in CARRIERS {
        assert!(opened(&BState::new(), host).is_none(), "{host:?} starts shut");
        assert!(opened(&open_for(host), host).is_some(), "{host:?} opens");
    }
}

/// Every box in the table is a row in the popover — counted through the
/// accordions, since a table with more than one heading is sub-panels now
/// (`part::filter::fold`) and its rows sit one card deep.
fn rows_in(views: &[BView]) -> usize {
    views
        .iter()
        .map(|view| match view {
            BView::Card(card) => rows_in(&card.body),
            _ => 1,
        })
        .sum()
}

#[test]
fn each_popover_draws_one_row_per_box() {
    for (host, _) in CARRIERS {
        let rows = opened(&open_for(host), host).expect("the open popover");
        assert_eq!(rows_in(&rows), overlay::table_for(host).unwrap().len(), "{host:?}");
    }
}

#[test]
fn an_untouched_box_answers_the_way_a_fresh_blender_does() {
    let state = BState::new();
    for (host, _) in CARRIERS {
        for spec in overlay::table_for(host).unwrap() {
            assert_eq!(
                overlay::is_on(&state, host, spec),
                spec.default_on,
                "{host:?} {} answers against its own default",
                spec.key
            );
        }
    }
    // Every table's master box is on: an editor whose overlays start off
    // draws nothing over its content in a file nobody has touched.
    for table in [&NODE[..], &SEQ[..], &IMAGE[..], &CLIP[..], &SHEET[..]] {
        assert!(table[0].default_on, "{} ships off", table[0].key);
    }
    // And the three Blender ships OFF are off, or a fresh file would draw
    // more than Blender's: stretch recolours every face, the search area
    // covers the footage, and the metadata read-out covers the corner.
    for (table, key) in [(&IMAGE[..], "show_stretch"), (&CLIP[..], "show_marker_search"), (&IMAGE[..], "show_metadata")] {
        let spec = table.iter().find(|s| s.key == key).expect(key);
        assert!(!spec.default_on, "{key} should ship off");
    }
}

#[test]
fn opening_one_editors_panel_does_not_open_anothers() {
    // The popover's open state is per host (`MenuKey::Panel`), even where the
    // settings behind it are shared.
    let state = open_for(BHostType::Node);
    assert!(opened(&state, BHostType::Node).is_some());
    for host in [BHostType::Seq, BHostType::Shader, BHostType::Clip] {
        assert!(opened(&state, host).is_none(), "{host:?} opened along with it");
    }
}

#[test]
fn the_groups_are_the_same_boxes_split_at_their_headings() {
    // The sidebar draws one card per group and the header draws one list; a
    // box in one and not the other is the drift this rules out.
    let state = BState::new();
    for (host, _) in CARRIERS {
        let groups = overlay::groups(&state, host);
        let rows: usize = groups.iter().map(|(_, rows)| rows.len()).sum();
        assert_eq!(rows, overlay::table_for(host).unwrap().len(), "{host:?}");
        assert!(groups.iter().all(|(title, _)| !title.is_empty()), "{host:?} has an unnamed card");
    }
    // Blender's two sub-panels under `CLIP_PT_display`, in its order.
    let clip: Vec<&str> = overlay::groups(&state, BHostType::Clip).iter().map(|(t, _)| *t).collect();
    assert_eq!(clip, vec!["Marker Display", "Clip Display"]);
}

#[test]
fn the_two_sidebar_tabs_that_draw_a_panel_draw_this_one() {
    // `CLIP_PT_display` is the Clip editor's View tab and
    // `SPREADSHEET_PT_filter` is the Spreadsheet's only tab. Both were empty.
    for (host, tab) in [(BHostType::Clip, "View"), (BHostType::Spreadsheet, "Filter")] {
        let spec = tabs_for(host).iter().find(|t| t.label == tab).expect(tab);
        assert_eq!(spec.body, BSidebarBody::Overlay, "{host:?} {tab}");
        let mut state = BState::new();
        state.set(&BSidebarShown(host).tab_key(), tab);
        let cards = BSidebarPanels::new(host).views(&state);
        assert_eq!(cards.len(), overlay::groups(&state, host).len(), "{host:?} {tab}");
        assert!(!cards.is_empty(), "{host:?} {tab} still opens nothing");
    }
}

#[test]
fn a_host_blender_leaves_empty_is_left_empty() {
    // The Text editor keeps its display toggles in `TEXT_PT_properties` (its
    // sidebar), and the Console and Info have none. Inventing a popover for
    // them would be a worse kind of missing.
    for host in [BHostType::Text, BHostType::Console, BHostType::Info, BHostType::Time] {
        assert!(!overlay::has_overlay(host), "{host:?}");
        assert!(overlay::body(&BState::new(), host).is_empty(), "{host:?}");
        assert!(overlay::groups(&BState::new(), host).is_empty(), "{host:?}");
    }
}
