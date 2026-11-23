use crate::pui::window::*;

/// What an app would keep in a file: an area a line.
const LAYOUT: &str = "# space\tzone\thost\nlayout\tmain\tview3d\nlayout\tside\toutliner\nlayout\tside\tbuts\n\nshading\tmain\tnode\nsculpting\tmain\tview3d\n";

#[test]
fn spaces_are_laid_out_from_a_table_an_area_a_line() {
    let mut set = PSet::<PSpace>::read("layout\tLayout\nshading\tShading\n");
    // The line for a space the set does not have is skipped.
    assert_eq!(PSpace::arrange(&mut set, LAYOUT), 4);
    let layout = set.find("layout").expect("in the set");
    let zone: Vec<(&str, Vec<&str>)> = layout.zone.iter().map(|zone| (zone.slug(), zone.area.iter().map(|area| area.host.as_str()).collect())).collect();
    assert_eq!(zone, [("main", vec!["view3d"]), ("side", vec!["outliner", "buts"])]);
    assert_eq!(set.find("shading").map(|space| space.zone.len()), Some(1));
}

#[test]
fn an_empty_table_places_nothing() {
    let mut set = PSet::<PSpace>::read("layout\tLayout\n");
    assert_eq!(PSpace::arrange(&mut set, "# nothing\n\n"), 0);
    assert!(set.find("layout").expect("in the set").zone.is_empty());
}

#[test]
fn a_zone_takes_its_share_of_the_width_from_the_table_and_one_when_it_gives_none() {
    let mut set = PSet::<PSpace>::read("chat\tChat\n");
    PSpace::arrange(&mut set, "chat\tside\thistory\nchat\tmain\tchat\t3\n");
    let share: Vec<f32> = set.find("chat").expect("in the set").zone.iter().map(|zone| zone.share).collect();
    assert_eq!(share, [1.0, 3.0]);
}
