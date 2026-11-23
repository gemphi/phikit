mod lay;

use crate::pui::window::*;

/// What an app would keep in a file: a part a line.
const REGION: &str = "# host\tregion\tside\tpart\nchat\theader\tleft\ttitle\nchat\theader\tright\tmodel\nchat\theader\tright\taccount\nchat\tcenter\tcenter\ttalk\nnone\tcenter\tcenter\ttalk\nchat\tfooter\tmiddle\tstatus\n";

#[test]
fn a_host_s_regions_are_declared_from_a_table_a_part_a_line() {
    let mut set = PSet::<PHost>::read("chat\tChat\n");
    // The lines for a host the set does not have, and for a side that is
    // none, are skipped.
    assert_eq!(PHost::declare(&mut set, REGION), 4);
    let chat = set.find("chat").expect("in the set");
    let region: Vec<&str> = chat.region.iter().map(|region| region.name.as_str()).collect();
    assert_eq!(region, ["header", "center"]);
    let header = &chat.region[0];
    assert_eq!(header.pane, [PPane::new(PSide::Left, &["title"]), PPane::new(PSide::Right, &["model", "account"])]);
}

#[test]
fn a_side_is_named_left_center_or_right() {
    assert_eq!([PSide::named("left"), PSide::named("center"), PSide::named("right"), PSide::named("top")], [Some(PSide::Left), Some(PSide::Center), Some(PSide::Right), None]);
}
