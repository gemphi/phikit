use std::rc::Rc;

use appkit::pui::PApp;
use appkit::pui::core::{PEvent, PMark, PMode, PPoint, PSheet};
use appkit::pui::window::{PDeed, PStage};
use appkit::service::{PStore, PTable};

use crate::fact::fact;
use crate::test::{DESK, HAND, Named, shown, staged, voiced, written};
use crate::{Phigen, data};

fn store(name: &str) -> PStore {
    let root = std::env::temp_dir().join("phigen-test-fact").join(name);
    std::fs::remove_dir_all(&root).ok();
    PStore::at(&root.to_string_lossy())
}

/// The rows a profile is given, as pairs.
fn row(app: &Phigen) -> Vec<(String, String)> {
    fact(app.desk()).list().to_vec()
}

fn pair(name: &str, value: &str) -> (String, String) {
    (name.to_string(), value.to_string())
}

#[test]
fn what_is_true_of_me_is_said_as_it_is_now() {
    let (app, mut stage, _) = voiced(DESK);
    let server = PTable::read(data::MODEL).get("host").to_string();
    let (nowhere, version) = (app.desk().text.get("fact.nowhere").to_string(), env!("CARGO_PKG_VERSION"));
    assert_eq!(row(&app), [pair("Model", "alpha"), pair("Server", &server), pair("Look", "Light"), pair("Chats", "0"), pair("Kept in", &nowhere), pair("Phigen", version)]);
    // Something is said, another model is chosen and another look: the
    // rows say so the next time they are asked for.
    Phigen::key(&mut stage, "hi");
    assert!(app.desk().choose("beta") && app.desk().dress("dark"));
    assert_eq!(row(&app)[..4], [pair("Model", "beta"), pair("Server", &server), pair("Look", "Dark"), pair("Chats", "1")]);
    // With no model connected there is no server to name.
    let (alone, _, _) = staged(DESK);
    assert_eq!(row(&alone)[..2], [pair("Model", "No model"), pair("Look", "Light")]);
}

/// How many panes of glass the stage draws, in the look it is shown in.
fn glass(stage: &PStage) -> usize {
    let mut sheet = PSheet::new(stage.theme().as_ref());
    stage.draw(&mut sheet);
    sheet.mark.iter().filter(|mark| matches!(mark, PMark::Glass { .. })).count()
}

#[test]
fn step_30_my_profile_says_what_is_true_now_and_lets_me_choose_glass() {
    for size in [DESK, HAND] {
        let kept = store(&format!("glass-{}", size.width));
        let look = Rc::new(Phigen::look(""));
        let open = || Phigen::kept(Rc::clone(&look), Box::new(Named::of(&["alpha", "beta"])), "Hello.", Some(kept.clone()));
        let app = open();
        let mut stage = app.stage();
        let lay = |stage: &mut PStage| stage.lay(size, PMode::of_width(size.width));
        stage.window.apply(&PDeed::Open("profile".to_string()));
        lay(&mut stage);
        stage.tick();
        // Where my things are kept is the folder they are kept in.
        let text = shown(&stage, &look);
        assert!(["About", "alpha", "Kept in", "Appearance", "Light", "Dark", "Glass", "Glass Light"].iter().all(|line| text.contains(&line.to_string())), "{text:?}");
        // A folder's path has no spaces: it is cut where it stops fitting.
        assert!(text.concat().contains(&kept.path()), "{text:?}");
        assert_eq!(glass(&stage), 0);
        // A press on Glass: the window's panes, its bar and the panel that
        // floats, are glass from the next frame. Nothing else is.
        // In a hand the profile is longer than its square: the looks are
        // at its foot, a turn of the wheel away.
        let body = stage.panel().expect("open").body;
        stage.send(&PEvent::Scroll(body.center(), PPoint::new(0.0, -100000.0)));
        let pill = written(&stage, &look, "Glass").expect("a pill");
        assert!(body.contains(pill), "{pill:?} in {body:?}");
        assert!(stage.send(&PEvent::PointerDown(pill)));
        assert_eq!((app.desk().shade.name(), glass(&stage)), ("glass", 2));
        stage.send(&PEvent::KeyDown(PEvent::ESCAPE));
        assert_eq!(glass(&stage), 1);
        // And Phigen opens in it the next time.
        assert_eq!(open().desk().shade.name(), "glass");
    }
}
