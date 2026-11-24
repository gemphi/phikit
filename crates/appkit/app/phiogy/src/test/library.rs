//! The asset library: its places, what each holds by kind and order, a
//! file brought in, a model read from USD and dressed, the kit's pressed
//! places, and the looks.

use std::path::{Path, PathBuf};

use appkit::pui::core::{PMark, PPaint, PRect, PSheet};
use appkit::pui::window::PDeed;
use appkit::pui::state::PState;
use appkit::pui::block::PPill;

use super::{DESK, drawn, staged};
use crate::library::{Dress, Library, Place, item};

/// A folder of its own under the system's, with these files in it.
fn folder(name: &str, file: &[(&str, &[u8])]) -> PathBuf {
    let at = std::env::temp_dir().join(format!("phiogy-library-{name}-{}", std::process::id()));
    _ = std::fs::create_dir_all(&at);
    file.iter().for_each(|(file, bytes)| _ = std::fs::write(at.join(file), bytes));
    at
}

#[test]
fn step_48_the_library_lists_a_place_by_kind_and_order_and_leaves_out_what_it_does_not_know() {
    let at = folder("list", &[("b.usda", b"#usda 1.0\n"), ("a.png", b"x"), ("clip.mp4", b"xyz"), ("notes.txt", b"t"), (".hidden.usda", b"")]);
    let library = Library::at(vec![Place::at("mine", at.clone(), true)]);
    let name = |kind: &str, order: &str| library.item("mine", kind, order).iter().map(|one| one.name.clone()).collect::<Vec<_>>();
    // Every kind it knows, by name; a hidden file and an unknown end are not listed.
    assert_eq!(name("all", "name"), ["a.png", "b.usda", "clip.mp4"]);
    assert_eq!(name("model", "name"), ["b.usda"]);
    assert_eq!(name("video", "name"), ["clip.mp4"]);
    // The largest first.
    assert_eq!(name("all", "large")[0], "b.usda");
    let one = library.get("mine:clip.mp4").expect("by its key");
    assert_eq!((one.kind.as_str(), one.icon.as_str(), one.size), ("video", "video", 3));
    // A name already there is never written over.
    assert_eq!(item::free(&at, "a.png"), at.join("a 2.png"));
    assert_eq!(item::size(1536, "B KB MB"), "1.5 KB");
}

#[test]
fn step_49_the_places_are_the_shapes_the_phiegg_space_and_the_person_s_own_folders() {
    let place = Place::own();
    let key: Vec<&str> = place.iter().map(|place| place.key.as_str()).collect();
    assert_eq!(key, ["shape", "phiegg", "download", "video", "picture", "document", "desktop"]);
    // The shapes are this program's own; the Phiegg space is the
    // person's own, made when something is brought in.
    let shape = place[0].folder.clone().expect("the shapes' folder");
    assert!(shape.ends_with(Path::new("asset").join("shape")) && !place[0].make);
    assert!(place[1].make && place[1].folder.as_ref().is_some_and(|folder| folder.ends_with("Phiegg")));
}

#[test]
fn step_50_every_shape_is_a_model_with_parts_and_the_dice_has_its_pips() {
    let library = Library::own();
    let shape = library.item("shape", "model", "name");
    assert!(shape.len() >= 12, "{:?}", shape.iter().map(|one| &one.name).collect::<Vec<_>>());
    for one in shape.iter() {
        let model = library.model(&one.path);
        assert!(!model.part.is_empty() && model.count > 0, "{}", one.name);
    }
    let dice = shape.iter().find(|one| one.name.starts_with("dice")).expect("a dice");
    let part: Vec<String> = library.model(&dice.path).part.iter().map(|part| part.name.clone()).collect();
    assert!(part.iter().any(|name| name.contains("body")) && part.iter().any(|name| name.contains("pip")), "{part:?}");
}

#[test]
fn step_51_a_model_is_opened_from_the_library_and_dressed_part_by_part() {
    let (app, mut stage, look) = staged(DESK);
    let desk = app.desk();
    assert!(stage.window.apply(&PDeed::Show("library".to_string())));
    stage.lay(DESK, appkit::pui::core::PMode::of_width(DESK.width));
    // The shapes' tiles are pictures the kit's raster drew.
    let tile = drawn(&stage, &look).iter().filter(|mark| matches!(mark, PMark::Picture { name, .. } if name.starts_with("library/shape:"))).count();
    assert!(tile >= 12, "{tile}");
    // A press on a model opens it beside its dress.
    let source = crate::part::library::source(desk);
    source.set("ui/item", "shape:cube.usda");
    assert_eq!(desk.shelf.get("ui/item"), "shape:cube.usda");
    // Dressed: a colour for its first part, and a finish, kept.
    let dress = Dress::new(desk.choice.clone());
    source.set("look/shape:cube.usda/0", "ruby");
    assert_eq!(dress.colour("shape:cube.usda", 0), Some([196.0 / 255.0, 40.0 / 255.0, 58.0 / 255.0]));
    let matte = dress.sun("shape:cube.usda", &desk.library.set).shine;
    source.set("look/shape:cube.usda/finish", "metal");
    assert!(dress.sun("shape:cube.usda", &desk.library.set).shine > matte);
}

#[test]
fn step_52_a_file_brought_in_lands_in_the_phiegg_space_under_a_name_of_its_own() {
    let from = folder("from", &[("look.png", b"png")]);
    let home = folder("home", &[("look.png", b"old")]);
    let (app, _, _) = staged(DESK);
    let mut desk = app.desk().clone();
    desk.library = std::rc::Rc::new(Library::at(vec![Place::at("phiegg", home.clone(), true)]));
    let shelf = crate::part::Shelf::new("library", desk.clone());
    shelf.import(&from.join("look.png"));
    // What was there is kept; the new one has a name of its own.
    assert_eq!(std::fs::read(home.join("look.png")).ok(), Some(b"old".to_vec()));
    assert_eq!(std::fs::read(home.join("look 2.png")).ok(), Some(b"png".to_vec()));
    assert_eq!(desk.shelf.get("ui/place"), "phiegg");
    assert_eq!(desk.shelf.get("ui/said"), desk.text.fill("import.done", "name", "look.png"));
}

#[test]
fn every_block_a_press_acts_on_says_so_as_it_is_drawn_inside_what_is_in_sight() {
    let (_, _, look) = staged(DESK);
    let mut sheet = PSheet::new(look.as_ref());
    sheet.clip(PRect::new(0.0, 0.0, 100.0, 100.0));
    let shown = ("", "In sight", false).draw(&mut sheet, appkit::pui::core::PPoint::new(10.0, 10.0), false);
    let hidden = ("", "Gone", false).draw(&mut sheet, appkit::pui::core::PPoint::new(10.0, 400.0), false);
    sheet.unclip();
    // The one in sight is a hand; the one scrolled out of sight is not.
    assert_eq!(sheet.pressed(), [appkit::pui::core::within(shown, PRect::new(0.0, 0.0, 100.0, 100.0)).expect("in sight")]);
    assert!(hidden.origin.y > 100.0);
}

#[test]
fn eight_looks_and_the_plain_ones_draw_a_faint_line_down_the_sidebar() {
    let (app, stage, look) = staged(DESK);
    let list = app.desk().shade.list();
    assert_eq!(list.len(), 8, "{list:?}");
    assert!(list.contains(&"plain") && list.contains(&"plain-dark"));
    // Drawn in the look the window wears now.
    let shade = app.desk().shade.clone();
    let line = |stage: &appkit::pui::window::PStage| {
        let mut sheet = PSheet::new(shade.as_ref());
        stage.draw(&mut sheet);
        sheet.mark.iter().any(|mark| matches!(mark, PMark::Fill { rect, .. } if rect.size.width <= 1.0 && rect.size.height > 100.0))
    };
    let _ = &look;
    assert!(!line(&stage));
    app.desk().shade.turn("plain");
    assert!(line(&stage));
}
