//! The asset library, as a hand works it: the tree's places, a folder
//! listed and gone into, an item opened and its attributes set, a comment
//! written, a set made of a model, Phiegg's clubs, a file brought in, the
//! kit's folds and layouts, what can be pressed, and the looks.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use appkit::pui::block::PPill;
use appkit::pui::core::{PEvent, PMark, PMode, PPaint, PRect, PSheet};
use appkit::pui::form::PSource;
use appkit::pui::state::PState;
use appkit::pui::window::PDeed;
use appkit::theme::PTheme;

use super::{DESK, drawn, staged};
use crate::Phiogy;
use crate::library::{Attribute, Library, Place, Shelf, item};

/// A folder of its own under the system's, with these files in it.
fn folder(name: &str, file: &[(&str, &[u8])]) -> PathBuf {
    let at = std::env::temp_dir().join(format!("phiogy-library-{name}-{}", std::process::id()));
    _ = std::fs::create_dir_all(at.join("inner"));
    file.iter().for_each(|(file, bytes)| _ = std::fs::write(at.join(file), bytes));
    at
}

/// The place of a folder, as the library keys it.
fn dir(path: &Path) -> String {
    format!("dir:{}", path.to_string_lossy())
}

#[test]
fn step_48_the_library_lists_a_folder_its_folders_first_by_kind_and_order() {
    let at = folder("list", &[("b.usda", b"#usda 1.0\n"), ("a.png", b"x"), ("clip.mp4", b"xyz"), ("notes.txt", b"t"), (".hidden.usda", b"")]);
    let library = Library::at(vec![Place::at("mine", at.clone(), true)], Rc::default());
    let name = |kind: &str, order: &str| library.item(&dir(&at), kind, order).iter().map(|one| one.name.clone()).collect::<Vec<_>>();
    // Its folders first, then every file of a kind it knows; a hidden
    // file and an unknown end are not listed.
    assert_eq!(name("all", "name"), ["inner", "a.png", "b.usda", "clip.mp4"]);
    assert_eq!(name("model", "name"), ["b.usda"]);
    assert_eq!(name("all", "large")[1], "b.usda");
    let one = library.get(&dir(&at.join("clip.mp4"))).expect("by its key");
    assert_eq!((one.kind.as_str(), one.icon.as_str(), one.size), ("video", "video", 3));
    assert!(library.get(&dir(&at.join("a.png"))).is_some_and(|one| one.picture.starts_with("file:///")));
    // A name already there is never written over.
    assert_eq!(item::free(&at, "a.png"), at.join("a 2.png"));
    assert_eq!(item::size(1536, "B KB MB"), "1.5 KB");
}

#[test]
fn step_49_the_tree_has_the_drives_the_person_s_folders_and_phiogy_s_own() {
    let place = Place::own();
    let section = |name: &str| place.iter().filter(|place| place.section == name).map(|place| place.key.clone()).collect::<Vec<_>>();
    // The drives this system has, as Blender's Volumes.
    assert!(section("volume").iter().any(|key| key.starts_with('C')), "{:?}", section("volume"));
    assert_eq!(section("system"), ["home", "desktop", "document", "download", "picture", "video", "audio"]);
    assert_eq!(section("own"), ["shape", "once", "phiegg"]);
    let own: Vec<&Place> = place.iter().filter(|place| place.section == "own").collect();
    assert!(own[0].folder.as_ref().is_some_and(|folder| folder.ends_with(Path::new("asset").join("shape"))) && !own[0].make);
    assert!(own[1].folder.as_ref().is_some_and(|folder| folder.ends_with(Path::new("asset").join("once"))) && !own[1].make);
    assert!(own[2].make && own[2].folder.as_ref().is_some_and(|folder| folder.ends_with("Phiegg")));
}

#[test]
fn step_50_every_shape_is_a_model_with_parts_and_the_dice_has_its_pips() {
    let library = Library::own(Path::new(&Phiogy::source()), Rc::default());
    let shape = library.place.iter().find(|place| place.key == "shape").and_then(|place| place.folder.clone()).expect("the shapes");
    let list = library.item(&dir(&shape), "model", "name");
    assert!(list.len() >= 12, "{:?}", list.iter().map(|one| &one.name).collect::<Vec<_>>());
    for one in list.iter() {
        let model = library.model(&one.path);
        assert!(!model.part.is_empty() && model.count > 0, "{}", one.name);
    }
    let dice = list.iter().find(|one| one.name.starts_with("dice")).expect("a dice");
    let part: Vec<String> = library.model(&dice.path).part.iter().map(|part| part.name.clone()).collect();
    assert!(part.iter().any(|name| name.contains("body")) && part.iter().any(|name| name.contains("pip")), "{part:?}");
}

#[test]
fn step_51_an_item_is_opened_beside_the_inspector_its_attributes_set_and_commented_on() {
    let (app, mut stage, look) = staged(DESK);
    let desk = app.desk();
    assert!(stage.window.apply(&PDeed::Show("library".to_string())));
    stage.lay(DESK, PMode::of_width(DESK.width));
    // Two columns: the tree and what the place holds.
    let part = super::part(&stage);
    assert!(part.contains(&"tree") && part.contains(&"library"), "{part:?}");
    // The shapes' tiles are pictures the kit's raster drew.
    let tile = drawn(&stage, &look).iter().filter(|mark| matches!(mark, PMark::Picture { name, .. } if name.starts_with("library/dir:"))).count();
    assert!(tile >= 12, "{tile}");
    let shape = desk.library.place.iter().find(|place| place.key == "shape").and_then(|place| place.folder.clone()).expect("the shapes");
    let cube = dir(&shape.join("cube.usda"));
    let source = crate::part::library::source(desk);
    source.set("ui/item", &cube);
    assert_eq!(desk.shelf.get("ui/item"), cube);
    assert_eq!(source.list("part").len(), 1);
    // A colour for its part, a finish, kept; and a comment written.
    let attribute = Attribute::new(desk.choice.clone());
    source.set(&format!("look/{cube}/0"), "ruby");
    assert_eq!(attribute.colour(&cube, 0), Some([196.0 / 255.0, 40.0 / 255.0, 58.0 / 255.0]));
    let matte = attribute.sun(&cube, &desk.library.set).shine;
    source.set(&format!("look/{cube}/finish"), "metal");
    assert!(attribute.sun(&cube, &desk.library.set).shine > matte);
    source.set(&format!("note/{cube}"), "Rounder corners next time.");
    source.set(&format!("note/{cube}"), "Yes.");
    assert_eq!(source.list("comment").iter().map(|c| c.label.clone()).collect::<Vec<_>>(), ["Rounder corners next time.", "Yes."]);
    // A set made of it: its variants, each part in a colour drawn by
    // weight, the same each time.
    source.set("ui/do", "set");
    let at = desk.shelf.get("ui/folder");
    assert!(at.starts_with("set:"), "{at}");
    let variant = desk.library.item(&at, "all", "name");
    assert_eq!(variant.len(), desk.library.own.get("count").parse::<usize>().unwrap_or_default());
    assert!(variant.iter().all(|one| one.kind == "model" && one.variant.len() == 1));
    desk.library.again();
    let again = desk.library.item(&at, "all", "name");
    assert_eq!(variant.iter().map(|one| one.variant.clone()).collect::<Vec<_>>(), again.iter().map(|one| one.variant.clone()).collect::<Vec<_>>());
    assert!(variant.iter().map(|one| one.variant[0].clone()).collect::<std::collections::HashSet<_>>().len() > 3);
}

#[test]
fn step_52_phiegg_s_collections_and_clubs_are_in_the_tree_with_their_items() {
    let source = Path::new(&Phiogy::source()).with_file_name("shelf.json");
    let shelf = Shelf::read(&source, "users");
    assert!(shelf.collection.len() > 10 && shelf.club.len() > 10, "{}", source.display());
    // Bold Pups: what it is, in Phiegg's words, and its pups with their
    // portraits and the traits they were made of.
    let pup = shelf.club("c_bp").expect("Bold Pups");
    assert!(pup.supply >= 10000 && pup.attribute.iter().any(|(name, _)| name == "Chain"));
    let one = &pup.item[0];
    assert!(one.face.is_some() && one.attribute.iter().any(|(name, _)| name == "Outfit"), "{one:?}");
    // A collection holds its clubs.
    let collection = shelf.collection(&pup.collection).expect("its collection");
    assert!(collection.club.contains(&"c_bp".to_string()));
}

#[test]
fn step_53_a_file_brought_in_lands_in_the_phiegg_space_under_a_name_of_its_own() {
    let from = folder("from", &[("look.png", b"png")]);
    let home = folder("home", &[("look.png", b"old")]);
    let (app, _, _) = staged(DESK);
    let mut desk = app.desk().clone();
    desk.library = Rc::new(Library::at(vec![Place::at("phiegg", home.clone(), true)], Rc::default()));
    let browse = crate::part::Browse::new("library", desk.clone());
    browse.import(&from.join("look.png"));
    assert_eq!(std::fs::read(home.join("look.png")).ok(), Some(b"old".to_vec()));
    assert_eq!(std::fs::read(home.join("look 2.png")).ok(), Some(b"png".to_vec()));
    assert_eq!(desk.shelf.get("ui/folder"), dir(&home));
    assert_eq!(desk.shelf.get("ui/said"), desk.text.fill("import.done", "name", "look.png"));
}

#[test]
fn step_54_the_items_heading_has_a_drop_of_three_layouts_at_its_left_and_a_filter_and_an_order_at_its_right() {
    let (app, mut stage, look) = staged(DESK);
    assert!(stage.window.apply(&PDeed::Show("library".to_string())));
    stage.lay(DESK, PMode::of_width(DESK.width));
    let text = |stage: &appkit::pui::window::PStage, want: &str| drawn(stage, &look).iter().filter_map(|mark| match mark {
        PMark::Text { at, text, .. } if text == want => Some(*at),
        _ => None,
    }).max_by(|a, b| a.x.total_cmp(&b.x));
    let icon = |stage: &appkit::pui::window::PStage, name: &str| drawn(stage, &look).iter().filter_map(|mark| match mark {
        PMark::Icon { name: known, rect, .. } if known == name => Some(*rect),
        _ => None,
    }).max_by(|a, b| a.origin.x.total_cmp(&b.origin.x));
    let word = |name: &str| app.desk().text.get(name).to_string();
    // The heading, the layout's drop just after its words, the filter
    // and the order at its right; no line of pills for them any more.
    let head = drawn(&stage, &look).iter().filter_map(|mark| match mark {
        PMark::Text { at, text, .. } if *text == word("library.item") => Some(*at),
        _ => None,
    }).min_by(|a, b| a.y.total_cmp(&b.y)).expect("the heading");
    let layout = icon(&stage, "grid").expect("the layout's drop");
    let (filter, sort) = (icon(&stage, "filter").expect("the filter"), icon(&stage, "sort").expect("the order"));
    assert!(layout.origin.x > head.x && layout.origin.x < DESK.width / 2.0 && (layout.center().y - head.y).abs() < 24.0, "{head:?} {layout:?}");
    assert!(filter.origin.x > DESK.width / 2.0 && sort.origin.x > filter.origin.x);
    assert!(text(&stage, &word("filter.source")).is_none() && text(&stage, &word("sort.large")).is_none());
    // The layout's drop opens with its three; Large tiles chosen, it is
    // put away and the tiles are larger.
    let tile = |stage: &appkit::pui::window::PStage| drawn(stage, &look).iter().filter_map(|mark| match mark {
        PMark::Picture { name, rect, .. } if name.starts_with("library/dir:") => Some(rect.size.width),
        _ => None,
    }).fold(0.0, f32::max);
    let small = tile(&stage);
    assert!(stage.send(&PEvent::PointerDown(layout.center())));
    stage.lay(DESK, PMode::of_width(DESK.width));
    let large = text(&stage, &word("layout.large")).expect("the three");
    assert!(text(&stage, &word("layout.list")).is_some());
    assert!(stage.send(&PEvent::PointerDown(appkit::pui::core::PPoint::new(large.x + 4.0, large.y + 4.0))));
    assert_eq!((app.desk().shelf.get("ui/layout/item").as_str(), app.desk().shelf.get("ui/drop").as_str()), ("large", ""));
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert!(tile(&stage) > small * 1.3, "{small} {}", tile(&stage));
    // The order's drop chooses the order.
    let sort = icon(&stage, "sort").expect("the order");
    assert!(stage.send(&PEvent::PointerDown(sort.center())));
    stage.lay(DESK, PMode::of_width(DESK.width));
    let newest = text(&stage, &word("sort.new")).expect("the orders");
    assert!(stage.send(&PEvent::PointerDown(appkit::pui::core::PPoint::new(newest.x + 4.0, newest.y + 4.0))));
    assert_eq!(app.desk().shelf.get("ui/sort"), "new");
    // A fold closes a section to its heading.
    stage.lay(DESK, PMode::of_width(DESK.width));
    let before = super::shown(&stage, &look).len();
    let fold = drawn(&stage, &look).iter().find_map(|mark| match mark {
        PMark::Icon { name, rect, .. } if name == "chevron-down" && rect.size.width >= look.size("icon.size").unwrap_or_default() - 1.0 => Some(rect.center()),
        _ => None,
    }).expect("a fold");
    assert!(stage.send(&PEvent::PointerDown(fold)));
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert!(super::shown(&stage, &look).len() < before);
}

#[test]
fn step_55_the_toolbar_has_back_and_forward_and_the_way_in_crumbs_an_arrow_between_each_step() {
    // A screen as wide as most: the way whole.
    let wide = appkit::pui::core::PSize { width: 1920.0, height: 1032.0 };
    let (app, mut stage, look) = staged(wide);
    assert!(stage.window.apply(&PDeed::Show("library".to_string())));
    stage.lay(wide, PMode::of_width(wide.width));
    let mark = drawn(&stage, &look);
    // The rightmost of a text: the crumbs', not the tree's.
    let text = |want: &str| mark.iter().filter_map(|mark| match mark {
        PMark::Text { at, text, .. } if text == want => Some(*at),
        _ => None,
    }).max_by(|a, b| a.x.total_cmp(&b.x));
    let shape = text("shape").expect("where it is");
    let way: Vec<PRect> = mark.iter().filter_map(|mark| match mark {
        PMark::Icon { name, rect, .. } if name == "chevron-right" && (rect.center().y - shape.y).abs() < 24.0 => Some(*rect),
        _ => None,
    }).collect();
    assert!(way.len() >= 3, "{way:?}");
    assert!(text(app.desk().text.get("library.head")).is_none());
    // A step pressed goes there; back goes to where it was, forward again.
    let shown = app.desk().library.home();
    let up = text("asset").expect("a step back");
    assert!(stage.send(&PEvent::PointerDown(appkit::pui::core::PPoint::new(up.x + 4.0, up.y + 4.0))));
    let asset = app.desk().shelf.get("ui/folder");
    assert!(asset.ends_with("asset") && asset != shown, "{asset}");
    let source = crate::part::library::source(app.desk());
    source.set("ui/do", "past");
    assert_eq!(app.desk().shelf.get("ui/folder"), shown);
    source.set("ui/do", "ahead");
    assert_eq!(app.desk().shelf.get("ui/folder"), asset);
}

#[test]
fn step_56_the_tree_has_an_arrow_at_each_depth_and_lines_down_the_gaps_under_an_open_one() {
    let (app, mut stage, look) = staged(DESK);
    // In Phiogy's own folder: its folders can open.
    app.desk().shelf.set("ui/folder", &dir(Path::new(env!("CARGO_MANIFEST_DIR"))));
    assert!(stage.window.apply(&PDeed::Show("library".to_string())));
    stage.lay(DESK, PMode::of_width(DESK.width));
    let mark = drawn(&stage, &look);
    let tree = super::part(&stage).iter().position(|part| *part == "tree").map(|_| DESK.width / 3.0).unwrap_or_default();
    // A line's arrow is smaller than an icon: a heading's fold is not one.
    let small = look.size("icon.size").unwrap_or_default() - 1.0;
    let arrow = |name: &str| mark.iter().filter_map(|mark| match mark {
        PMark::Icon { name: known, rect, .. } if known == name && rect.origin.x < tree && rect.size.width < small => Some(*rect),
        _ => None,
    }).collect::<Vec<_>>();
    // The folders down to the shapes are open, one step further in each.
    let open = arrow("chevron-down");
    let mut x: Vec<f32> = open.iter().map(|rect| rect.origin.x).collect();
    x.dedup();
    assert!(x.len() >= 4 && x.windows(2).filter(|pair| pair[1] > pair[0]).count() >= 3, "{x:?}");
    // A thin line runs down from the middle of an open arrow.
    let rule: Vec<PRect> = mark.iter().filter_map(|mark| match mark {
        PMark::Fill { rect, .. } if rect.size.width <= 1.0 && rect.origin.x < tree && rect.size.height > 4.0 => Some(*rect),
        _ => None,
    }).collect();
    assert!(open.iter().any(|arrow| rule.iter().any(|line| (line.center().x - arrow.center().x).abs() < 1.0 && line.origin.y >= arrow.center().y)), "{rule:?}");
    // The folders in it are not open, and can be.
    assert!(!arrow("chevron-right").is_empty());
}

#[test]
fn every_block_a_press_acts_on_says_so_as_it_is_drawn_inside_what_is_in_sight() {
    let (_, _, look) = staged(DESK);
    let mut sheet = PSheet::new(look.as_ref());
    sheet.clip(PRect::new(0.0, 0.0, 100.0, 100.0));
    let shown = ("", "In sight", false).draw(&mut sheet, appkit::pui::core::PPoint::new(10.0, 10.0), false);
    let hidden = ("", "Gone", false).draw(&mut sheet, appkit::pui::core::PPoint::new(10.0, 400.0), false);
    sheet.unclip();
    assert_eq!(sheet.pressed(), [appkit::pui::core::within(shown, PRect::new(0.0, 0.0, 100.0, 100.0)).expect("in sight")]);
    assert!(hidden.origin.y > 100.0);
}

#[test]
fn eight_looks_and_the_plain_ones_draw_a_faint_line_down_the_sidebar() {
    let (app, stage, _) = staged(DESK);
    let list = app.desk().shade.list();
    // Eight looks, Silver, and sixteen of glass (the user, 25 November 2026).
    assert_eq!(list.len(), 25, "{list:?}");
    assert!(["plain", "plain-dark", "silver", "vision", "horizon", "graphite"].iter().all(|name| list.contains(name)));
    let shade = app.desk().shade.clone();
    // The sidebar's line: down its edge, where the page begins (a
    // section's own thin level line stands inside the page).
    let line = |stage: &appkit::pui::window::PStage| {
        let mut sheet = PSheet::new(shade.as_ref());
        stage.draw(&mut sheet);
        let edge = stage.frame().body.origin.x + 2.0;
        sheet.mark.iter().any(|mark| matches!(mark, PMark::Fill { rect, .. } if rect.size.width <= 1.0 && rect.size.height > 100.0 && rect.center().x <= edge))
    };
    assert!(!line(&stage));
    app.desk().shade.turn("plain");
    assert!(line(&stage));
}

/// The dice opened in the 3D view of a window of that size.
fn dice(size: appkit::pui::core::PSize) -> (Phiogy, appkit::pui::window::PStage, Rc<appkit::theme::PLook>) {
    let (app, mut stage, look) = staged(size);
    let desk = app.desk();
    let shape = desk.library.place.iter().find(|place| place.key == "shape").and_then(|place| place.folder.clone()).expect("the shapes");
    crate::part::library::source(desk).set("ui/item", &dir(&shape.join("dice.usda")));
    assert!(stage.window.apply(&PDeed::Show("model".to_string())));
    stage.lay(size, PMode::of_width(size.width));
    (app, stage, look)
}

#[test]
fn step_60_the_3d_view_has_the_ball_in_its_corner_its_tools_four_views_and_wire() {
    let (app, mut stage, look) = dice(DESK);
    let desk = app.desk();
    let colour = |token: &str| desk.shade.color(token).expect(token);
    let ball = |stage: &appkit::pui::window::PStage| drawn(stage, &look).iter().filter_map(|mark| match mark {
        PMark::Fill { rect, color, .. } if *color == colour("scene.x") && rect.size.width > 10.0 => Some(*rect),
        _ => None,
    }).collect::<Vec<_>>();
    // The ball, in the view's lower right; its across-way pressed looks
    // along it.
    let x = ball(&stage);
    assert_eq!(x.len(), 1, "{x:?}");
    assert!(x[0].origin.x > DESK.width / 3.0 && x[0].origin.y > DESK.height / 2.0);
    let picture = |stage: &appkit::pui::window::PStage| drawn(stage, &look).iter().filter_map(|mark| match mark {
        PMark::Picture { name, .. } if name.starts_with(crate::part::figure::FILM) => Some(name.clone()),
        _ => None,
    }).collect::<Vec<_>>();
    let before = desk.library.film.frame(crate::part::figure::FILM).expect("drawn").pixel;
    stage.send(&PEvent::PointerDown(x[0].center()));
    assert!(stage.send(&PEvent::PointerUp(x[0].center())));
    stage.lay(DESK, PMode::of_width(DESK.width));
    drawn(&stage, &look);
    assert_ne!(desk.library.film.frame(crate::part::figure::FILM).expect("drawn").pixel, before);
    // Four views: the main one and the top, the front and the right.
    let tool = |stage: &appkit::pui::window::PStage, name: &str| drawn(stage, &look).iter().rev().find_map(|mark| match mark {
        PMark::Icon { name: known, rect, .. } if known == name => Some(rect.center()),
        _ => None,
    }).expect(name);
    assert!(stage.send(&PEvent::PointerDown(tool(&stage, "layout-grid"))));
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert_eq!(picture(&stage).len(), 4, "{:?}", picture(&stage));
    assert!(super::shown(&stage, &look).contains(&desk.text.get("view3d.top").to_string()));
    // Wire: the model drawn as its edges, in the wire's ink.
    assert!(stage.send(&PEvent::PointerDown(tool(&stage, "box"))));
    stage.lay(DESK, PMode::of_width(DESK.width));
    drawn(&stage, &look);
    let frame = desk.library.film.frame(crate::part::figure::FILM).expect("drawn");
    let ink = colour("scene.wire");
    let near = |at: usize| (frame.pixel[at + 2] as f32 / 255.0 - ink.r).abs() < 0.1 && (frame.pixel[at + 1] as f32 / 255.0 - ink.g).abs() < 0.1;
    assert!((0..frame.pixel.len()).step_by(4).filter(|at| near(*at)).count() > 100);
}

#[test]
fn step_61_the_inspector_has_the_scene_as_a_tree_what_holds_parts_and_the_parts_in_it() {
    let (app, _, _) = dice(DESK);
    let source = crate::part::library::source(app.desk());
    let scene = source.list("scene");
    // The dice holds its body and its pips, a step in.
    let line: Vec<(String, u32, Option<bool>)> = scene.iter().map(|c| (c.label.clone(), c.depth, c.fold)).collect();
    assert_eq!(line[0], ("dice".to_string(), 0, Some(true)));
    assert!(line[1..].iter().all(|(_, depth, fold)| *depth == 1 && fold.is_none()), "{line:?}");
    assert!(line.iter().any(|(name, ..)| name == "body") && line.iter().any(|(name, ..)| name == "pip"));
    // A part pressed is the part chosen.
    source.set("ui/part", &scene[2].value);
    assert_eq!(app.desk().shelf.get("ui/part"), scene[2].value);
}

#[test]
fn the_dice_s_edges_are_as_round_as_the_phi_fonts_corners() {
    let shape = Path::new(env!("CARGO_MANIFEST_DIR")).join("asset").join("shape");
    let library = Library::own(Path::new(&Phiogy::source()), Rc::default());
    let bounds = |name: &str| {
        let model = library.model(&shape.join(name));
        appkit::pui::core::orbit::bounds(&model.part.iter().map(|part| part.piece.clone()).collect::<Vec<_>>())
    };
    // Both are as large; the Phi-Soft one rounder than the Phiber one,
    // and each is a dice of 16 mm.
    let (soft, hard) = (bounds("dice.usda"), bounds("dice-phiber.usda"));
    assert!(((soft.1[0] - soft.0[0]) - 0.016).abs() < 0.002 && ((hard.1[0] - hard.0[0]) - 0.016).abs() < 0.002, "{soft:?} {hard:?}");
    let rows = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../phiphx/data/shape.tsv")).unwrap_or_default();
    let bevel = |name: &str| rows.lines().find(|line| line.starts_with(&format!("{name}\t"))).and_then(|line| line.split('\t').nth(6)).and_then(|v| v.parse::<f32>().ok());
    assert_eq!((bevel("dice"), bevel("dice-phiber")), (Some(0.00092), Some(0.00080)));
}
