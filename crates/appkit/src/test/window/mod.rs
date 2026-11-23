//! Tests of `window`, one file for each file there. This file tests
//! `mod.rs`, the window and the levels it is built from.

mod deed;
mod frame;
mod host;
mod panel;
mod part;
mod place;
mod set;
mod space;
mod stage;

use crate::pui::window::*;

/// A space of that name with one zone whose areas hold these hosts.
pub(crate) fn space(slug: &str, host: &[&str]) -> PSpace {
    let zone = PZone { area: host.iter().map(|name| PArea::holding(name)).collect(), ..PZone::of(PEntry::new("main", "Main", "")) };
    PSpace { zone: vec![zone], ..PSpace::of(PEntry::new(slug, slug, "")) }
}

/// A window with a layout space (a viewport and an outliner) and a shading
/// space (a node editor).
pub(crate) fn window() -> PWindow {
    PWindow::new("Book", vec![space("layout", &["view3d", "outliner"]), space("shading", &["node"])])
}

/// The hosts an app has: a viewport with a header and a centre, and an
/// outliner with nothing declared.
pub(crate) fn host_set() -> PSet<PHost> {
    let mut set = PSet::<PHost>::read("view3d\t3D Viewport\tview3d\noutliner\tOutliner\toutliner\n");
    let view3d = set.find_mut("view3d").expect("just read");
    view3d.region = vec![PRegion::new("header", vec![PPane::new(PSide::Left, &["menubar"])]), PRegion::new("center", vec![PPane::new(PSide::Center, &["canvas"])])];
    set
}

#[test]
fn a_level_says_its_word_and_an_entry_its_name_label_and_icon() {
    assert_eq!([PSpace::WORD, PZone::WORD, PHost::WORD, PPart::WORD, PModal::WORD], ["space", "zone", "host", "part", "modal"]);
    let host = PHost::of(PEntry::new("view3d", "3D Viewport", "cube"));
    assert_eq!((host.slug(), host.label(), host.icon()), ("view3d", "3D Viewport", "cube"));
    assert!(host.region.is_empty());
}

#[test]
fn a_window_shows_its_first_space_and_any_other_by_name() {
    let mut window = window();
    assert_eq!(window.current().map(PSpace::slug), Some("layout"));
    assert!(window.show("shading"));
    assert_eq!(window.current().map(PSpace::slug), Some("shading"));
    // A space it does not have changes nothing.
    assert!(!window.show("sculpting"));
    assert_eq!(window.current().map(PSpace::slug), Some("shading"));
    assert_eq!(PWindow::new("Empty", Vec::new()).current(), None);
}

#[test]
fn a_modal_lies_over_the_space_until_it_is_closed() {
    let mut window = window();
    assert_eq!(window.modal(), None);
    window.open("about");
    assert_eq!((window.modal(), window.current().map(PSpace::slug)), (Some("about"), Some("layout")));
    window.close();
    assert_eq!(window.modal(), None);
}

#[test]
fn a_window_names_what_the_app_s_sets_do_not_have() {
    let mut window = window();
    let (host, mut part, mut modal) = (host_set(), PSet::<PPart>::read("menubar\tMenu Bar\n"), PSet::<PModal>::new());
    // The shading space holds a node editor the app has not supplied, and
    // the viewport's centre a part it has not.
    assert_eq!(window.missing(&host, &part, &modal), ["host node", "part canvas"]);
    window.open("about");
    assert_eq!(window.missing(&host, &part, &modal), ["host node", "modal about", "part canvas"]);
    // Once the app supplies them, the window can be built whole.
    let mut host = host;
    assert!(host.add(PHost::of(PEntry::new("node", "Node Editor", ""))));
    assert!(part.add(PPart::of(PEntry::new("canvas", "Canvas", ""))));
    assert!(modal.add(PModal::of(PEntry::new("about", "About", ""))));
    assert!(window.missing(&host, &part, &modal).is_empty());
}

#[test]
fn a_host_named_by_two_areas_is_missing_once() {
    let window = PWindow::new("Twice", vec![space("layout", &["view3d", "view3d"]), space("other", &["view3d"])]);
    let empty = (PSet::<PHost>::new(), PSet::<PPart>::new(), PSet::<PModal>::new());
    assert_eq!(window.missing(&empty.0, &empty.1, &empty.2), ["host view3d"]);
}

#[test]
fn a_pane_sits_in_the_centre_unless_told_and_keeps_its_parts_in_order() {
    assert_eq!(PPane::default().side, PSide::Center);
    let pane = PPane::new(PSide::Right, &["search", "account"]);
    assert_eq!((pane.side, pane.part.as_slice()), (PSide::Right, ["search".to_string(), "account".to_string()].as_slice()));
    assert_eq!(PRegion::new("header", vec![pane.clone()]).pane, [pane]);
}
