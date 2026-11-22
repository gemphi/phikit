//! Posting a picture from this disk, muting a person, and a theme: its
//! swatch on its card, and wearing it.

use appkit::pui::core::{PAsset, PEvent, PFit, PMark, PMode};
use appkit::pui::window::PDeed;
use appkit::theme::PTheme;
use serde_json::json;

use super::tools::icon;
use super::{DESK, drawn, shown, staged};
use crate::hosted::Hosted;
use crate::part::Desk;

/// A picture of the web's own, copied to a folder of the system's.
fn picture(name: &str) -> String {
    let from = Hosted::own().place("phiegg:worlds/w_brocade.png").expect("the web's picture");
    let to = std::env::temp_dir().join(format!("phiogy-test-{}-{name}", std::process::id()));
    std::fs::copy(from, &to).expect("it is copied");
    to.to_string_lossy().to_string()
}

#[test]
fn step_33_a_picture_of_this_disk_is_posted_in_a_room_and_opens_in_the_viewer() {
    let (app, mut stage, look) = staged(DESK);
    let desk = app.desk();
    assert!(desk.show("lobby"));
    stage.send(&PEvent::KeyUp(' '));
    let path = picture("post.png");
    // What is no picture, and what is not there, is not posted.
    assert!(!desk.post(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")) && !desk.post("nowhere.png") && desk.said().len() == 3);
    assert!(desk.post(&path));
    // It is a message of its own, by me, its picture named by where
    // the file is, as the web names a picture by an address.
    let said = desk.said();
    let last = said.last().expect("it was said");
    let key = last.image.first().cloned().expect("its picture");
    assert!(last.who == "you" && last.text.is_empty() && key.starts_with("file:///") && !key.contains('\\'), "{last:?}");
    assert_eq!(desk.kept.get("sent")["phiogy/lobby"][0]["image"], json!([key.clone()]));
    // The file is found by that name, drawn in the stream, and is the
    // last of the room's pictures in the viewer.
    assert!(Hosted::own().load(&key).is_some());
    assert!(drawn(&stage, &look).iter().any(|mark| matches!(mark, PMark::Picture { name, fit: PFit::Cover, .. } if *name == key)));
    assert_eq!(desk.shot().last().map(|shot| (shot.key.clone(), shot.by.clone())), Some((key, "You".to_string())));
    // The control before Send asks the system for a file; what it
    // answers is told as a word, and posted.
    let post = icon(&stage, &look, "image-plus").expect("the control").center();
    assert!(stage.send(&PEvent::PointerDown(post)));
    assert_eq!(stage.errand(), [PDeed::Choose("post".to_string())]);
    assert!(stage.tell(&format!("post {}", picture("told.png"))) && desk.said().len() == 5);
}

#[test]
fn step_34_what_a_muted_person_said_is_not_shown_until_they_are_unmuted() {
    let (app, mut stage, look) = staged(DESK);
    let desk = app.desk();
    assert!(desk.show("lobby"));
    stage.send(&PEvent::KeyUp(' '));
    let words = "The mark is drawn on the font now.".to_string();
    assert!(shown(&stage, &look).contains(&words));
    desk.mute("tavi");
    let text = shown(&stage, &look);
    assert!(text.contains(&desk.text.fill("stream.muted", "name", "Tavi Reyes")) && !text.contains(&words), "{text:?}");
    assert_eq!(desk.kept.get("quiet")["tavi"], json!(true));
    // On their profile the control says what a press does now.
    *desk.now.who.borrow_mut() = Some("tavi".to_string());
    stage.lay(DESK, PMode::of_width(DESK.width));
    let text = shown(&stage, &look);
    assert!(text.contains(&desk.text.get("person.unmute").to_string()) && !text.contains(&desk.text.get("person.mute").to_string()));
    assert!(super::press(&mut stage, &look, desk.text.get("person.unmute")) && !desk.muted("tavi"));
    assert!(shown(&stage, &look).contains(&words));
}

/// The light of the window, as the look gives it now.
fn light(desk: &Desk) -> Option<appkit::pui::core::PColor> {
    desk.shade.color("shade.light")
}

#[test]
fn step_35_a_theme_shows_its_swatch_and_once_bought_is_worn_and_taken_off() {
    use crate::part::shop::{colour, hex};
    // A colour of the web's, as the window's theme writes one.
    assert_eq!((hex("#3B4657").as_deref(), hex("hsl(212 58% 55%)").as_deref(), hex("teal")), (Some("#3B4657"), Some("#4a88cf"), None));
    let (app, mut stage, look) = staged(DESK);
    let desk = app.desk();
    let shop = desk.shop("s_one").expect("the shop");
    let theme = shop.product.iter().find(|thing| thing.wear).cloned().expect("a theme");
    let swatch = colour(&theme.swatch);
    assert_eq!(swatch.len(), 3);
    // Its card shows its colours, side by side.
    assert!(desk.show("s_one.theme"));
    stage.send(&PEvent::KeyUp(' '));
    stage.lay(DESK, PMode::of_width(DESK.width));
    let filled = drawn(&stage, &look).iter().filter(|mark| matches!(mark, PMark::Fill { color, .. } if swatch.contains(color))).count();
    assert_eq!(filled, 3);
    // Bought, it can be put on: the window's light is its own colour.
    let plain = light(desk);
    desk.pick("s_one", &theme);
    assert!(desk.pay() && desk.stands("s_one", &theme) == "shop.wear" && light(desk) == plain);
    desk.pick("s_one", &theme);
    assert_eq!((desk.stands("s_one", &theme), light(desk), desk.worn().as_str()), ("shop.worn", Some(swatch[1]), "s_one/theme1"));
    assert!(shown(&stage, &look).contains(&desk.text.get("shop.worn").to_string()));
    // The same press takes it off again.
    desk.pick("s_one", &theme);
    assert_eq!((desk.stands("s_one", &theme), light(desk), desk.worn().as_str()), ("shop.wear", plain, ""));
}
