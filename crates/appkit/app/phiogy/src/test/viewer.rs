//! The viewer as Google Photos' and the web's: no head of a panel's, one
//! row of the way back, who and when, the tools and Save; the picture
//! on its own haze; a picture shown larger and whole again; what share,
//! save and open ask the system for; what a control is called, shown
//! under the pointer; marks sketched and set on a picture, taken back,
//! put back and kept; and a picture of the library looked at alone.

use std::path::Path;

use appkit::pui::core::{PEvent, PFit, PMark, PMode, PPoint, PRect, PSheet, PSize};
use appkit::pui::window::{PDeed, PStage};
use appkit::theme::{PLook, PTheme};

use super::{DESK, HAND, shown, staged};
use crate::Phiogy;
use crate::hosted::Hosted;

/// How large a picture is, for a sheet that is told; and the second
/// picture posted in the small site's lobby.
const NATURAL: PSize = PSize { width: 640.0, height: 360.0 };
const KEY: &str = "phiegg:worlds/w_carousel.png";

/// The viewer open on a picture of the lobby, in a window of that size.
fn opened(size: PSize, number: usize) -> (Phiogy, PStage, std::rc::Rc<PLook>) {
    let (app, mut stage, look) = staged(size);
    assert!(app.desk().show("lobby"));
    *app.desk().now.album.borrow_mut() = app.desk().shot();
    app.desk().view.set(Some(number));
    assert!(stage.window.apply(&PDeed::Open("view".to_string())));
    stage.lay(size, PMode::of_width(size.width));
    (app, stage, look)
}

/// Everything drawn, on a sheet that knows how large the pictures are.
fn sized(stage: &PStage, look: &PLook) -> Vec<PMark> {
    let known = |_: &str| Some(NATURAL);
    let mut sheet = PSheet::new(look).sized(&known);
    stage.draw(&mut sheet);
    sheet.mark
}

/// Where the icon of that name was drawn last: the viewer's, which is
/// drawn over everything.
fn icon(mark: &[PMark], wanted: &str) -> Option<PRect> {
    mark.iter().rev().find_map(|mark| match mark {
        PMark::Icon { name, rect, .. } if name == wanted => Some(*rect),
        _ => None,
    })
}

/// The picture looked at: where it is drawn and how it lies. It is
/// drawn after the room's own, and is no picture of the strip.
fn looked(stage: &PStage, look: &PLook) -> (PRect, PFit) {
    let thumb = look.size("view.thumb").unwrap_or_default();
    let one = |mark: PMark| match mark {
        PMark::Picture { name, rect, fit, .. } if name == KEY && rect.size.width > thumb => Some((rect, fit)),
        _ => None,
    };
    sized(stage, look).into_iter().filter_map(one).next_back().expect("the picture is drawn")
}

/// A press on the viewer's control with that icon.
fn tool(stage: &mut PStage, look: &PLook, name: &str) -> bool {
    let at = icon(&sized(stage, look), name).unwrap_or_else(|| panic!("the control {name}")).center();
    stage.send(&PEvent::PointerDown(at))
}

#[test]
fn step_17_the_viewer_has_no_head_and_one_row_of_the_way_back_who_and_when_its_tools_and_save() {
    let (_, stage, look) = opened(DESK, 1);
    // Its panel is bare: no title of a modal's, no button of appkit's.
    let panel = stage.panel().expect("the viewer is open");
    assert!(panel.bare && panel.body == panel.rect);
    let (mark, text) = (sized(&stage, &look), shown(&stage, &look));
    assert!(!text.contains(&"Picture".to_string()) && icon(&mark, "maximize").is_none());
    // The tools from the left, then Save, on one line at the right; the
    // way back at the left, who posted it beside it.
    let row: Vec<PRect> = ["zoom-in", "share-2", "download", "external-link", "undo", "redo", "check"].iter().map(|name| icon(&mark, name).unwrap_or_else(|| panic!("{name}"))).collect();
    assert!(row.windows(2).all(|pair| pair[0].origin.x < pair[1].origin.x && (pair[0].center().y - pair[1].center().y).abs() < 1.0), "{row:?}");
    assert!(row[0].origin.x > DESK.width / 2.0 && row[6].origin.x + row[6].size.width < DESK.width);
    let back = icon(&mark, "arrow-left").expect("the way back");
    assert!(back.origin.x < DESK.width / 8.0 && (back.center().y - row[0].center().y).abs() < 1.0);
    let who = mark.iter().rev().find_map(|mark| match mark {
        PMark::Text { at, text, .. } if text == "You" => Some(*at),
        _ => None,
    });
    let who = who.expect("who posted it");
    assert!(who.x > back.origin.x + back.size.width && who.x < DESK.width / 2.0 && (who.y - row[0].center().y).abs() < look.size("view.control").unwrap_or_default(), "{who:?} {row:?}");
    // It is the last picture: one before it, none after.
    let after = mark.iter().any(|mark| matches!(mark, PMark::Icon { name, rect, .. } if name == "chevron-right" && rect.origin.x > DESK.width * 0.8 && rect.center().y > DESK.height / 4.0));
    assert!(icon(&mark, "chevron-left").is_some() && !after);
    // The inks and the pens are under the picture.
    let (picture, _) = looked(&stage, &look);
    let sketch = icon(&mark, "pencil").expect("Sketch");
    assert!(sketch.origin.y > picture.origin.y + picture.size.height && icon(&mark, "type").is_some());
}

#[test]
fn step_18_a_picture_is_shown_larger_moved_with_the_wheel_and_whole_again() {
    let (_, mut stage, look) = opened(DESK, 1);
    // Whole, and never larger than it is, in the room the controls leave.
    let (room, fit) = looked(&stage, &look);
    assert_eq!(fit, PFit::Whole);
    assert!(tool(&mut stage, &look, "zoom-in"));
    let mark = sized(&stage, &look);
    assert!(icon(&mark, "zoom-out").is_some() && icon(&mark, "zoom-in").is_none());
    // Larger: twice as wide as its room, as high as its shape makes
    // it, from its left, and nothing of it outside its room.
    let (large, fit) = looked(&stage, &look);
    let high = room.size.width * 2.0 * NATURAL.height / NATURAL.width;
    assert_eq!((fit, large.size.width, large.size.height, large.origin.x), (PFit::Cover, room.size.width * 2.0, high, room.origin.x));
    let before = mark.iter().position(|mark| matches!(mark, PMark::Picture { name, fit: PFit::Cover, rect, .. } if name == KEY && *rect == large)).expect("drawn");
    assert_eq!(mark[before - 1], PMark::Clip(room));
    // The wheel moves it, and no farther than it goes.
    assert!(stage.send(&PEvent::Scroll(room.center(), PPoint::new(-120.0, 0.0))));
    assert_eq!(looked(&stage, &look).0.origin.x, room.origin.x - 120.0);
    assert!(stage.send(&PEvent::Scroll(room.center(), PPoint::new(-9999.0, 0.0))));
    assert_eq!(looked(&stage, &look).0.origin.x, room.origin.x - room.size.width);
    // A press on the picture shows it whole again, and another larger.
    assert!(stage.send(&PEvent::PointerDown(room.center())));
    assert_eq!(looked(&stage, &look), (room, PFit::Whole));
    assert!(stage.send(&PEvent::PointerDown(room.center())));
    assert_eq!(looked(&stage, &look).1, PFit::Cover);
    // Another picture is seen whole first.
    assert!(stage.send(&PEvent::KeyDown(PEvent::LEFT)));
    assert!(icon(&sized(&stage, &look), "zoom-in").is_some());
    // Whole, it is 640 wide in a wider room: a press in its room and
    // beside it puts the viewer away.
    assert!(stage.send(&PEvent::PointerDown(room.origin.plus(PPoint::new(2.0, 2.0)))));
    assert_eq!(stage.window.modal(), None);
}

#[test]
fn step_19_share_save_and_open_ask_the_system_for_the_picture_s_own_file() {
    let (app, mut stage, look) = opened(DESK, 1);
    // The web's own file, where it is on this disk, with no step back
    // up left in it.
    let file = Hosted::own().place(KEY).expect("the picture's file");
    assert!(Path::new(&file).is_file() && !file.contains(".."), "{file}");
    let said = app.desk().text.get("view.shared").to_string();
    assert!(!shown(&stage, &look).contains(&said));
    // Share copies where it is, and says so by the tool.
    assert!(tool(&mut stage, &look, "share-2"));
    assert_eq!(stage.errand(), [PDeed::Copy(file.clone())]);
    assert!(shown(&stage, &look).contains(&said));
    // Save asks where a copy is to be kept; open opens it.
    assert!(tool(&mut stage, &look, "download"));
    assert_eq!(stage.errand(), [PDeed::Save(file.clone())]);
    assert!(tool(&mut stage, &look, "external-link"));
    assert_eq!(stage.errand(), [PDeed::Launch(file)]);
    // The viewer stays; another picture was not shared yet.
    assert_eq!(stage.window.modal(), Some("view"));
    assert!(stage.send(&PEvent::KeyDown(PEvent::LEFT)));
    assert!(!shown(&stage, &look).contains(&said));
    // Its way back is its own.
    assert!(tool(&mut stage, &look, "arrow-left"));
    assert_eq!(stage.window.modal(), None);
}

#[test]
fn what_a_control_is_called_is_shown_by_it_while_the_pointer_is_over_it() {
    let (app, mut stage, look) = opened(DESK, 1);
    let word = |name: &str| app.desk().text.get(name).to_string();
    let over = |stage: &mut PStage, name: &str| {
        let at = icon(&sized(stage, &look), name).expect("the control").center();
        stage.send(&PEvent::PointerMove(at))
    };
    // The window is drawn again when the pointer comes over a control,
    // and not while it stays there.
    assert!(over(&mut stage, "arrow-left") && !over(&mut stage, "arrow-left"));
    assert!(shown(&stage, &look).contains(&word("view.close")));
    assert!(over(&mut stage, "zoom-in"));
    let text = shown(&stage, &look);
    assert!(text.contains(&word("view.zoom")) && !text.contains(&word("view.close")), "{text:?}");
    // A tool that is on is called by what a press on it does now.
    assert!(tool(&mut stage, &look, "zoom-in"));
    assert!(shown(&stage, &look).contains(&word("view.fit")));
    assert!(over(&mut stage, "chevron-left"));
    assert!(shown(&stage, &look).contains(&word("view.prev")));
    // Off every control, nothing is said.
    assert!(stage.send(&PEvent::PointerMove(PPoint::new(DESK.width / 2.0, DESK.height / 2.0))));
    let text = shown(&stage, &look);
    assert!(["view.close", "view.zoom", "view.fit", "view.prev"].iter().all(|name| !text.contains(&word(name))), "{text:?}");
}

#[test]
fn in_a_hand_the_picture_has_the_width_and_the_arrows_lie_over_its_edges() {
    let (_, stage, look) = opened(HAND, 1);
    let mark = sized(&stage, &look);
    let room = mark.iter().rev().find_map(|mark| match mark {
        PMark::Picture { name, rect, fit: PFit::Whole, .. } if name == KEY => Some(*rect),
        _ => None,
    });
    let room = room.expect("the picture");
    let gap = look.size("gap").unwrap_or_default() / 2.0;
    assert_eq!((room.origin.x, room.size.width), (gap, HAND.width - gap * 2.0));
    assert!(room.contains(icon(&mark, "chevron-left").expect("the one before").center()));
    // The tools keep the top row; who and when stand under the picture.
    assert!(icon(&mark, "zoom-in").is_some_and(|tool| tool.center().y < room.origin.y));
    let who = mark.iter().rev().find_map(|mark| match mark {
        PMark::Text { at, text, .. } if text == "You" => Some(*at),
        _ => None,
    });
    assert!(who.is_some_and(|who| who.y > room.origin.y + room.size.height));
}

#[test]
fn put_away_and_opened_again_a_picture_is_whole_and_was_not_shared() {
    let (app, mut stage, look) = opened(DESK, 1);
    assert!(tool(&mut stage, &look, "zoom-in") && tool(&mut stage, &look, "share-2"));
    assert!(icon(&sized(&stage, &look), "zoom-out").is_some());
    // Put away: a frame goes by in which it is not drawn.
    assert!(stage.send(&PEvent::KeyDown(PEvent::ESCAPE)));
    stage.tick();
    // Opened on the same picture, in the frame that follows the press.
    assert!(stage.window.apply(&PDeed::Open("view".to_string())));
    stage.lay(DESK, PMode::of_width(DESK.width));
    stage.tick();
    assert!(icon(&sized(&stage, &look), "zoom-in").is_some());
    assert!(!shown(&stage, &look).contains(&app.desk().text.get("view.shared").to_string()));
    // A tool told as a word before the first frame holds through it.
    let (app, mut stage, look) = opened(DESK, 0);
    app.desk().wish.borrow_mut().extend([PDeed::Tell("zoom".to_string()), PDeed::Tell("share".to_string())]);
    stage.send(&PEvent::KeyUp(' '));
    stage.tick();
    assert!(icon(&sized(&stage, &look), "zoom-out").is_some());
    assert!(shown(&stage, &look).contains(&app.desk().text.get("view.shared").to_string()));
}

#[test]
fn step_57_a_picture_is_marked_a_line_sketched_and_words_set_taken_back_put_back_and_kept() {
    let (app, mut stage, look) = opened(DESK, 1);
    let (room, _) = looked(&stage, &look);
    // The ink: small dots (a section's thin level line behind is long).
    let ink = |stage: &PStage| sized(stage, &look).iter().filter(|mark| matches!(mark, PMark::Fill { rect, .. } if room.contains(rect.center()) && rect.size.width <= 8.0 && rect.size.height <= 8.0)).count();
    // The red ink, and Sketch: a press on the picture begins a line that
    // follows the pointer while it is held.
    let dot: Vec<PRect> = sized(&stage, &look).iter().filter_map(|mark| match mark {
        PMark::Fill { rect, .. } if rect.size.width == look.size("view.dot").unwrap_or_default() => Some(*rect),
        _ => None,
    }).collect();
    assert_eq!(dot.len(), 7, "{dot:?}");
    assert!(stage.send(&PEvent::PointerDown(dot[1].center())));
    assert!(tool(&mut stage, &look, "pencil"));
    let from = room.center();
    assert!(stage.send(&PEvent::PointerDown(from)));
    assert!(stage.send(&PEvent::PointerMove(from.plus(PPoint::new(80.0, 20.0)))));
    assert!(stage.send(&PEvent::PointerUp(from.plus(PPoint::new(80.0, 20.0)))));
    let drawn = ink(&stage);
    assert!(drawn > 10, "{drawn}");
    // Taken back, put back.
    assert!(tool(&mut stage, &look, "undo"));
    assert_eq!(ink(&stage), 0);
    assert!(tool(&mut stage, &look, "redo"));
    assert_eq!(ink(&stage), drawn);
    // Text: a press sets words where it was, the keys write them.
    assert!(tool(&mut stage, &look, "type"));
    assert!(stage.send(&PEvent::PointerDown(room.origin.plus(PPoint::new(room.size.width / 4.0, room.size.height / 4.0)))));
    "Nice".chars().for_each(|key| _ = stage.send(&PEvent::KeyDown(key)));
    assert!(stage.send(&PEvent::KeyDown(PEvent::ENTER)));
    assert!(shown(&stage, &look).contains(&"Nice".to_string()));
    // Kept: put away and opened again, the picture has its marks.
    let saved = app.desk().text.get("view.kept").to_string();
    assert!(!shown(&stage, &look).contains(&saved));
    assert!(tool(&mut stage, &look, "check"));
    assert!(shown(&stage, &look).contains(&saved));
    assert!(app.desk().choice.get(&format!("mark/{KEY}")).contains("Nice"));
    assert!(stage.send(&PEvent::KeyDown(PEvent::ESCAPE)));
    stage.tick();
    assert!(stage.window.apply(&PDeed::Open("view".to_string())));
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert!(shown(&stage, &look).contains(&"Nice".to_string()));
}

#[test]
fn the_viewer_stands_on_the_look_s_own_ground_not_the_picture_s_colours() {
    let (app, stage, look) = opened(DESK, 1);
    let name = format!("haze/{KEY}");
    // As every page: the look's own pane over the whole window, no haze.
    let mark = sized(&stage, &look);
    assert!(!mark.iter().any(|mark| matches!(mark, PMark::Picture { name: known, .. } if *known == name)));
    assert!(mark.iter().any(|mark| matches!(mark, PMark::Fill { rect, .. } | PMark::Blend { rect, .. } if rect.size.width >= DESK.width && rect.size.height >= DESK.height - 1.0)));
    // (A theme with `view.haze` more than nothing has the picture's haze
    // under the ground, as before: viewer/lay.rs.)
    drop(app);
}

#[test]
fn step_58_a_picture_of_the_library_is_looked_at_alone_among_the_place_s_others() {
    let at = std::env::temp_dir().join(format!("phiogy-viewer-{}", std::process::id()));
    _ = std::fs::create_dir_all(&at);
    ["a.png", "b.png", "c.png"].iter().for_each(|name| _ = std::fs::write(at.join(name), b"png"));
    let (app, _, _) = staged(DESK);
    let mut desk = app.desk().clone();
    desk.library = std::rc::Rc::new(crate::library::Library::at(vec![crate::library::Place::at("mine", at.clone(), true)], std::rc::Rc::default()));
    let source = crate::part::library::source(&desk);
    use appkit::pui::state::PState;
    source.set("ui/folder", &format!("dir:{}", at.to_string_lossy()));
    source.set("ui/item", &format!("dir:{}", at.join("b.png").to_string_lossy()));
    // The viewer opens on it, the place's pictures in its strip.
    assert_eq!(desk.wish.borrow().last(), Some(&PDeed::Open("view".to_string())));
    let album = desk.now.album.borrow().clone();
    assert_eq!(album.iter().map(|shot| shot.by.clone()).collect::<Vec<_>>(), ["a.png", "b.png", "c.png"]);
    assert_eq!(desk.view.get(), Some(1));
    // Its file is the one on the disk: share copies where it is.
    assert_eq!(album[1].file.as_deref(), Some(at.join("b.png").as_path()));
    assert!(album[1].key.starts_with("file:///"));
}
