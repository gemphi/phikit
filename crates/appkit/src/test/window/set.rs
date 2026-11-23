use crate::pui::window::*;

/// What an app would keep in a file: a space a line.
const TABLE: &str = "# slug\tlabel\ticon\nlayout\tLayout\tgrid\n\nmodeling\tModeling\nlayout\tLayout Again\tother\n";

#[test]
fn a_set_is_read_from_a_table_an_entry_a_line() {
    let set = PSet::<PSpace>::read(TABLE);
    let entry: Vec<(&str, &str, &str)> = set.list().iter().map(|space| (space.slug(), space.label(), space.icon())).collect();
    // The comment and the empty line are skipped, the icon may be left
    // out, and a name given twice keeps its first entry.
    assert_eq!(entry, [("layout", "Layout", "grid"), ("modeling", "Modeling", "")]);
}

#[test]
fn an_entry_is_found_by_its_name() {
    let mut set = PSet::<PHost>::read("view3d\t3D Viewport\noutliner\tOutliner\n");
    assert_eq!(set.find("outliner").map(PHost::label), Some("Outliner"));
    assert!(set.find("node").is_none() && set.find("").is_none());
    set.find_mut("view3d").expect("in the table").entry.label = "View".to_string();
    assert_eq!(set.find("view3d").map(PHost::label), Some("View"));
}

#[test]
fn an_app_adds_an_entry_once_and_never_one_without_a_name() {
    let mut set = PSet::<PPart>::new();
    assert!(set.list().is_empty());
    assert!(set.add(PPart::of(PEntry::new("menubar", "Menu Bar", ""))));
    assert!(!set.add(PPart::of(PEntry::new("menubar", "Another", ""))));
    assert!(!set.add(PPart::of(PEntry::new("", "Nameless", ""))));
    assert_eq!(set.list().len(), 1);
    assert_eq!(set.find("menubar").map(PPart::label), Some("Menu Bar"));
}

#[test]
fn an_empty_table_is_an_empty_set() {
    assert_eq!(PSet::<PModal>::read(""), PSet::new());
    assert_eq!(PSet::<PModal>::read("# nothing\n\n").list().len(), 0);
}
