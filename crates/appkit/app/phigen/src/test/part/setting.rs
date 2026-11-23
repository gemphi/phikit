use appkit::pui::core::{PEvent, PMark, PPoint, PSheet};
use appkit::pui::window::PStage;
use appkit::theme::{PLook, PTheme};

use crate::test::{DESK, HAND, listed, pictured, shown, voiced, written};
use crate::{Phigen, data};

/// Open the settings: the bar's Menu, and the Settings row of what it
/// shows, beside the bar at a desk and over the chat in a hand.
fn open(stage: &mut PStage, look: &PLook) -> bool {
    let menu = pictured(stage, look, "layout-grid").expect("on the bar");
    let shown = stage.send(&PEvent::PointerDown(menu));
    let row = listed(stage, look, "Settings").expect("a row of the menu");
    shown && stage.send(&PEvent::PointerDown(row))
}

/// What the window's ground is filled with, in the look it is shown in.
fn ground(stage: &PStage) -> Option<PMark> {
    let mut sheet = PSheet::new(stage.theme().as_ref());
    stage.draw(&mut sheet);
    sheet.mark.first().cloned()
}

#[test]
fn step_28_the_menu_opens_the_settings_where_i_choose_the_look_and_the_model() {
    for size in [DESK, HAND] {
        let (app, mut stage, look) = voiced(size);
        assert!(open(&mut stage, &look));
        assert_eq!((stage.window.modal(), stage.window.drawer()), (Some("setting"), false));
        stage.tick();
        let text = shown(&stage, &look);
        assert!(["Settings", "Look", "Light", "Dark", "Model", "alpha", "beta"].iter().all(|line| text.contains(&line.to_string())), "{size:?} {text:?}");
        // Dark: the next frame is drawn in it, and Light brings it back.
        let light = ground(&stage);
        let dark = written(&stage, &look, "Dark").expect("a pill");
        assert!(stage.send(&PEvent::PointerDown(dark)));
        assert_eq!(app.desk().shade.name(), "dark");
        assert_ne!(ground(&stage), light);
        let back = written(&stage, &look, "Light").expect("a pill");
        assert!(stage.send(&PEvent::PointerDown(back)));
        assert_eq!((app.desk().shade.name(), ground(&stage)), ("light", light));
        // Another model answers from now on, and the settings stay open.
        let beta = written(&stage, &look, "beta").expect("listed");
        assert!(stage.send(&PEvent::PointerDown(beta)));
        assert_eq!((app.desk().history.borrow().voice(), stage.window.modal()), ("beta", Some("setting")));
        assert!(stage.send(&PEvent::KeyDown(PEvent::ESCAPE)) && stage.panel().is_none());
    }
}

#[test]
fn the_user_at_the_bar_s_foot_opens_the_profile() {
    let (_, mut stage, _) = voiced(DESK);
    let bar = stage.frame().activity;
    assert!(stage.send(&PEvent::PointerDown(PPoint::new(bar.origin.x + 28.0, bar.origin.y + bar.size.height - 28.0))));
    assert_eq!(stage.window.modal(), Some("profile"));
}

#[test]
fn a_look_changes_every_colour_of_the_theme_or_none() {
    // The colours `data/theme.tsv` has, and the ones a look changes.
    let token = |text: &str, skip: usize| -> Vec<String> {
        let row = text.lines().map(|line| line.split('\t').skip(skip).collect::<Vec<_>>());
        let mut name: Vec<String> = row.filter(|cell| cell.first() == Some(&"color")).filter_map(|cell| cell.get(1).map(|name| name.to_string())).collect();
        name.sort();
        name
    };
    let shade = Phigen::shade(&PLook::read(data::THEME));
    assert_eq!(shade.list(), ["light", "dark", "glass", "frost", "liquid", "liquid-dark"]);
    let all = token(data::THEME, 0);
    for look in shade.list() {
        let own: String = data::LOOK.lines().filter(|line| line.split('\t').next() == Some(look)).map(|line| format!("{line}\n")).collect();
        let changed = token(&own, 1);
        // A look may add colours of its own, as glass does.
        let left: Vec<&String> = all.iter().filter(|name| !changed.contains(name)).collect();
        assert!(changed.is_empty() || left.is_empty(), "{look} leaves some colours as they were: {left:?}");
        // And every look has a word for a person to read.
        assert!(!crate::test::staged(DESK).0.desk().text.get(&format!("look.{look}")).is_empty(), "{look}");
        assert!(shade.turn(look) && shade.color("window.fill").is_some());
    }
}
