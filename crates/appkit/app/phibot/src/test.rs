//! The botanist's screens, as a person meets them: the creator with its
//! customiser and robot, Start Planting into the greenhouse once it is
//! read, the score, Replay and Exit Game.

use std::rc::Rc;

use appkit::pui::PApp;
use appkit::pui::core::{PEvent, PMark, PMode, PPaint, PPoint, PSheet, PSize};
use appkit::pui::window::PStage;
use appkit::theme::{PLook, PTheme};

use crate::Phibot;

const DESK: PSize = PSize { width: 1040.0, height: 720.0 };

fn staged() -> (Phibot, PStage, Rc<PLook>) {
    let look = Rc::new(Phibot::look(""));
    let app = Phibot::new(Rc::clone(&look));
    let mut stage = app.stage();
    stage.lay(DESK, PMode::of_width(DESK.width));
    (app, stage, look)
}

fn shown(stage: &PStage, look: &PLook) -> Vec<String> {
    let mut sheet = PSheet::new(look);
    stage.draw(&mut sheet);
    sheet.text_list().into_iter().map(str::to_string).collect()
}

/// The place a line of text was drawn, to press it.
fn place(stage: &PStage, look: &PLook, said: &str) -> Option<PPoint> {
    let mut sheet = PSheet::new(look);
    stage.draw(&mut sheet);
    sheet.mark.iter().find_map(|mark| match mark {
        PMark::Text { at, text, .. } if text == said => Some(PPoint::new(at.x + 2.0, at.y + 2.0)),
        _ => None,
    })
}

fn press(stage: &mut PStage, look: &PLook, said: &str) -> bool {
    let Some(at) = place(stage, look, said) else { return false };
    let sent = stage.send(&PEvent::PointerDown(at));
    stage.send(&PEvent::PointerUp(at));
    stage.lay(DESK, PMode::of_width(DESK.width));
    sent
}

#[test]
fn the_creator_shows_the_customiser_and_the_robot() {
    let (_app, stage, look) = staged();
    let text = shown(&stage, &look);
    for word in ["botanist", "Head", "Body", "Backpack", "Type", "Eye Shape", "Material", "Material Color", "Light Color", "Start Planting", "idle"] {
        assert!(text.contains(&word.to_string()), "{word}: {text:?}");
    }
}

#[test]
fn a_choice_changes_the_robot() {
    let (app, mut stage, look) = staged();
    assert!(press(&mut stage, &look, "head3"));
    assert!(press(&mut stage, &look, "metal"));
    let look_now = app.desk().bot.borrow().look();
    let head = look_now.part.iter().find(|(n, _)| n == "head").map(|(_, c)| c.clone()).unwrap();
    assert_eq!((head.shape.as_str(), head.material.as_str()), ("head3", "metal"));
    // The body's own choices are shown once it is the part chosen.
    assert!(press(&mut stage, &look, "Body"));
    assert!(shown(&stage, &look).contains(&"body1".to_string()));
    assert!(!shown(&stage, &look).contains(&"Eye Shape".to_string()));
}

#[test]
fn start_planting_reads_the_greenhouse_then_shows_its_score() {
    let (app, mut stage, look) = staged();
    assert!(press(&mut stage, &look, "Start Planting"));
    // While it is read, the greenhouse says it is loading.
    assert!(shown(&stage, &look).contains(&"Loading assets…".to_string()));
    let start = std::time::Instant::now();
    while app.desk().loading() && start.elapsed().as_secs() < 300 {
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    stage.lay(DESK, PMode::of_width(DESK.width));
    let text = shown(&stage, &look);
    if text.iter().any(|t| t.starts_with("The greenhouse's scene is not on this disk")) {
        eprintln!("{text:?}");
        return;
    }
    for word in ["Greenhouse score", "Propagate plants to earn points", "0", "Replay", "Exit Game", "coffeeBerry", "poppy", "yucca"] {
        assert!(text.contains(&word.to_string()), "{word}: {text:?}");
    }
    // A press on each planter, where it is drawn, walks the robot there
    // and propagates it; with all three, the score says so.
    for plant in ["coffeeBerry", "poppy", "yucca"] {
        let named = place(&stage, &look, plant).expect("the planter's name");
        let side = look.size("garden.planter").unwrap_or_default();
        // Over the middle of its name: the planter's mark.
        let wide = PSheet::new(look.as_ref()).measure(plant);
        let at = PPoint::new(named.x + wide / 2.0, named.y - side / 2.0);
        stage.send(&PEvent::PointerDown(at));
        stage.send(&PEvent::PointerUp(at));
        let start = std::time::Instant::now();
        while start.elapsed().as_secs() < 10 && app.desk().bot.borrow().garden().is_some_and(|g| g.planter.iter().any(|p| p.plant == plant && !p.grown)) {
            stage.tick();
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
        let grown = app.desk().bot.borrow().garden().is_some_and(|g| g.planter.iter().any(|p| p.plant == plant && p.grown));
        assert!(grown, "{plant} was not reached");
    }
    stage.tick();
    let text = shown(&stage, &look);
    assert!(text.contains(&"Awesome job!".to_string()) && text.contains(&"You propagated 3 plants".to_string()), "{text:?}");
    // Exit Game: back to making the robot.
    assert!(press(&mut stage, &look, "Exit Game"));
    assert!(shown(&stage, &look).contains(&"Start Planting".to_string()));
}
