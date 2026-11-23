use crate::pui::core::PRect;
use crate::pui::window::*;
use crate::theme::PLook;

fn laid(slot: &[PSlot]) -> Vec<(&str, PRect)> {
    slot.iter().map(|slot| (slot.part.as_str(), slot.rect)).collect()
}

#[test]
fn a_region_the_theme_gives_a_height_takes_it_and_side_panes_hug_their_edges() {
    let mut set = PSet::<PHost>::read("chat\tChat\n");
    PHost::declare(&mut set, "chat\theader\tleft\tmenu\nchat\theader\tcenter\ttitle\nchat\theader\tright\tuser\nchat\tcenter\tcenter\ttalk\nchat\tfooter\tcenter\task\n");
    let look = PLook::read("size\tregion.header\t40\nsize\tregion.footer\t60\nsize\tpane.side\t100\n");
    let slot = set.find("chat").expect("in the set").lay(PRect::new(200.0, 0.0, 600.0, 500.0), &look);
    let want = [
        ("menu", PRect::new(200.0, 0.0, 100.0, 40.0)),
        ("title", PRect::new(300.0, 0.0, 400.0, 40.0)),
        ("user", PRect::new(700.0, 0.0, 100.0, 40.0)),
        ("talk", PRect::new(200.0, 40.0, 600.0, 400.0)),
        ("ask", PRect::new(200.0, 440.0, 600.0, 60.0)),
    ];
    assert_eq!(laid(&slot), want);
    assert_eq!((slot[0].host.as_str(), slot[0].region.as_str()), ("chat", "header"));
}

#[test]
fn with_no_sizes_from_the_theme_regions_and_panes_share_and_a_pane_s_parts_are_stacked() {
    let mut set = PSet::<PHost>::read("chat\tChat\n");
    PHost::declare(&mut set, "chat\theader\tleft\tmenu\nchat\theader\tright\tmodel\nchat\theader\tright\tuser\nchat\tcenter\tcenter\ttalk\n");
    let slot = set.find("chat").expect("in the set").lay(PRect::new(0.0, 0.0, 400.0, 200.0), &PLook::default());
    let want = [
        ("menu", PRect::new(0.0, 0.0, 200.0, 100.0)),
        ("model", PRect::new(200.0, 0.0, 200.0, 50.0)),
        ("user", PRect::new(200.0, 50.0, 200.0, 50.0)),
        ("talk", PRect::new(0.0, 100.0, 400.0, 100.0)),
    ];
    assert_eq!(laid(&slot), want);
}

#[test]
fn a_host_that_declares_nothing_shows_nothing() {
    let set = PSet::<PHost>::read("chat\tChat\n");
    assert!(set.find("chat").expect("in the set").lay(PRect::new(0.0, 0.0, 400.0, 200.0), &PLook::default()).is_empty());
}
