//! Pictures: the web's own files found by their keys, faces and marks
//! as cells of a sheet, a picture posted in a room, and the viewer.

use appkit::pui::core::{PAsset, PEvent, PFit, PMark, PMode, PPoint, PRect};
use appkit::pui::window::{PDeed, PStage};
use appkit::theme::{PLook, PTheme};

use super::{DESK, HAND, drawn, shown, site, staged};
use crate::hosted::Hosted;

/// Every picture the stage draws: its name, where, and how it lies.
fn pictured(stage: &PStage, look: &PLook) -> Vec<(String, PRect, PFit)> {
    let one = |mark: PMark| match mark {
        PMark::Picture { name, rect, fit, .. } => Some((name, rect, fit)),
        _ => None,
    };
    drawn(stage, look).into_iter().filter_map(one).collect()
}

#[test]
fn a_picture_s_key_names_a_file_of_the_web_s_own() {
    let hosted = Hosted::own();
    // The web's own file, as it is on disk: a PNG.
    let bytes = hosted.load("phiegg:worlds/w_brocade.png").expect("the web's picture is on this disk");
    assert_eq!(&bytes[1..4], b"PNG");
    assert!(hosted.load("phiegg:portraits/sheet-00.webp").is_some());
    // A key of no app here, one that is no key, and one that would
    // leave its app's folder name nothing.
    assert!(hosted.load("nowhere:worlds/w_brocade.png").is_none() && hosted.load("worlds/w_brocade.png").is_none());
    assert!(hosted.path("phiegg:../../README.md").is_none() && hosted.load("phiegg:worlds/none.png").is_none());
}

#[test]
fn the_site_reads_faces_and_marks_as_cells_of_a_sheet() {
    let site = site();
    let cell = |id: &str| site.face(id).map(|face| (face.sheet.as_str(), face.column, face.index));
    // A person's face is on the sheet all share, or on one of their own.
    assert_eq!(cell("lora"), Some(("phiegg:portraits/sheet-00.webp", 8, 1)));
    assert_eq!(cell("tavi"), Some(("phiegg:portraits/sheet-06.webp", 8, 9)));
    assert_eq!(cell("nobody"), None);
    let mark = |index: usize| site.venue[index].mark.as_ref().map(|mark| (mark.sheet.as_str(), mark.index));
    assert_eq!((mark(2), mark(0)), (Some(("phiegg:portraits/sheet-06.webp", 2)), None));
    assert_eq!(site.plate("lora").map(|(_, plate)| plate.with.as_str()), Some("lora"));
}

#[test]
fn step_14_faces_marks_and_posted_pictures_are_drawn_from_the_web_s_files() {
    let (app, mut stage, look) = staged(DESK);
    // A direct room has its person's face in the title and in the rail.
    let face = PFit::Cell { column: 8, index: 1 };
    let lora = pictured(&stage, &look).into_iter().filter(|(name, _, fit)| name == "phiegg:portraits/sheet-00.webp" && *fit == face).count();
    assert_eq!(lora, 2);
    // In a room each message has the face of who said it, and a picture
    // posted with it covers a frame under its words.
    assert!(app.desk().show("lobby"));
    let seen = pictured(&stage, &look);
    assert!(seen.iter().any(|(name, _, fit)| name == "phiegg:portraits/sheet-06.webp" && *fit == PFit::Cell { column: 8, index: 9 }), "{seen:?}");
    let shot: Vec<&(String, PRect, PFit)> = seen.iter().filter(|(_, _, fit)| *fit == PFit::Cover).collect();
    assert_eq!(shot.iter().map(|(name, ..)| name.as_str()).collect::<Vec<_>>(), ["phiegg:worlds/w_brocade.png", "phiegg:worlds/w_carousel.png"]);
    assert_eq!((shot[0].1.size.width, shot[0].1.size.height), (256.0, 160.0));
    assert!(!shown(&stage, &look).contains(&"w_brocade.png".to_string()));
    // A place with a mark shows it, a cell of its sheet.
    assert!(stage.window.apply(&PDeed::Show("venue".to_string())));
    stage.lay(DESK, PMode::of_width(DESK.width));
    assert!(pictured(&stage, &look).iter().any(|(name, _, fit)| name == "phiegg:portraits/sheet-06.webp" && *fit == PFit::Cell { column: 8, index: 2 }));
}

#[test]
fn step_15_a_press_on_a_posted_picture_opens_it_in_the_viewer() {
    let (app, mut stage, look) = staged(DESK);
    assert!(app.desk().show("lobby"));
    let cover = |stage: &PStage| pictured(stage, &look).into_iter().filter(|(_, _, fit)| *fit == PFit::Cover).collect::<Vec<_>>();
    let second = cover(&stage)[1].1;
    assert!(stage.send(&PEvent::PointerDown(second.center())));
    assert_eq!((stage.window.modal(), app.desk().view.get()), (Some("view"), Some(1)));
    // Over the whole window: the picture whole, who posted it and when.
    let panel = stage.panel().expect("the viewer is open");
    assert_eq!((panel.rect.size.width, panel.rect.origin.x), (DESK.width, 0.0));
    let whole = |stage: &PStage| pictured(stage, &look).into_iter().filter(|(_, _, fit)| *fit == PFit::Whole).map(|(name, rect, _)| (name, rect)).collect::<Vec<_>>();
    let seen = whole(&stage);
    assert_eq!(seen.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>(), ["phiegg:worlds/w_carousel.png"]);
    assert!(panel.body.contains(seen[0].1.origin));
    let text = shown(&stage, &look);
    assert!(text.contains(&"You".to_string()) && text.contains(&"Today · 09:05".to_string()), "{text:?}");
    // The strip has both, and the one before is an arrow away: by the
    // key, and there is none past the first.
    let side = look.size("view.thumb").unwrap_or_default();
    assert_eq!(cover(&stage).iter().filter(|(_, rect, _)| rect.size.width == side).count(), 2);
    assert!(stage.send(&PEvent::KeyDown(PEvent::LEFT)));
    assert_eq!((app.desk().view.get(), whole(&stage)[0].0.as_str()), (Some(0), "phiegg:worlds/w_brocade.png"));
    assert!(!stage.send(&PEvent::KeyDown(PEvent::LEFT)) || app.desk().view.get() == Some(0));
    // A press on the other of the strip shows it.
    let thumb = cover(&stage).into_iter().filter(|(_, rect, _)| rect.size.width == side).nth(1).expect("the strip").1;
    assert!(stage.send(&PEvent::PointerDown(thumb.center())));
    assert_eq!(app.desk().view.get(), Some(1));
    // A press beside the picture puts it away; so does Escape.
    assert!(stage.send(&PEvent::PointerDown(PPoint::new(panel.body.origin.x + 4.0, panel.body.center().y - 150.0))));
    assert_eq!(stage.window.modal(), None);
    assert!(stage.send(&PEvent::PointerDown(cover(&stage)[0].1.center())));
    assert_eq!(stage.window.modal(), Some("view"));
    assert!(stage.send(&PEvent::KeyDown(PEvent::ESCAPE)));
    assert_eq!(stage.window.modal(), None);
}

#[test]
fn in_a_hand_who_and_when_stand_over_the_strip() {
    let (app, mut stage, look) = staged(HAND);
    assert!(app.desk().show("lobby"));
    *app.desk().now.album.borrow_mut() = app.desk().shot();
    app.desk().view.set(Some(0));
    stage.window.apply(&PDeed::Open("view".to_string()));
    stage.lay(HAND, PMode::of_width(HAND.width));
    let mark = drawn(&stage, &look);
    // The viewer is drawn last: its is the last such line.
    let name = mark.iter().rev().find_map(|mark| match mark {
        PMark::Text { at, text, .. } if text == "Tavi Reyes" => Some(*at),
        _ => None,
    });
    let picture = mark.iter().find_map(|mark| match mark {
        PMark::Picture { rect, fit: PFit::Whole, .. } => Some(*rect),
        _ => None,
    });
    let (name, picture) = (name.expect("who posted it"), picture.expect("the picture"));
    assert!(name.y > picture.origin.y + picture.size.height, "{name:?} {picture:?}");
}
