use appkit::pui::core::{PEvent, PPoint};
use appkit::pui::window::PStage;

use crate::Phigen;
use crate::test::{DESK, HAND, rect, shown, staged, voiced, written};

/// A press on the pill: it is at the left of the column of the title,
/// halfway down it. What is pressed is what was drawn, so it is drawn
/// first.
fn pill(app: &Phigen, stage: &mut PStage, look: &appkit::theme::PLook) -> bool {
    shown(stage, look);
    let title = rect(stage, "title");
    let (start, _) = app.desk().column(title.size.width);
    stage.send(&PEvent::PointerDown(PPoint::new(title.origin.x + start + 10.0, title.center().y)))
}

#[test]
fn step_27_a_pill_over_the_chat_names_the_model_that_answers_and_lists_the_others() {
    for size in [DESK, HAND] {
        let (app, mut stage, look) = voiced(size);
        // It is there before anything is said.
        assert!(shown(&stage, &look).contains(&"alpha".to_string()));
        assert!(pill(&app, &mut stage, &look));
        assert_eq!(stage.window.modal(), Some("model"));
        // The models are asked for once a frame: the server may have said.
        stage.tick();
        let text = shown(&stage, &look);
        assert!(["Model", "alpha", "beta"].iter().all(|line| text.contains(&line.to_string())), "{text:?}");
        // A press on another has it answer, and takes the list away.
        let beta = written(&stage, &look, "beta").expect("listed");
        assert!(stage.send(&PEvent::PointerDown(beta)));
        assert!(stage.panel().is_none());
        assert_eq!(app.desk().history.borrow().voice(), "beta");
        Phigen::key(&mut stage, "hi");
        let text = shown(&stage, &look);
        // The pill is over the chat too, beside its name.
        assert!(text.contains(&"beta".to_string()) && text.contains(&"beta: hi".to_string()) && !text.contains(&"alpha".to_string()), "{text:?}");
        assert_eq!(rect(&stage, "title").origin.y, stage.frame().body.origin.y);
    }
}

#[test]
fn with_no_model_connected_the_pill_says_so_and_there_is_nothing_to_choose() {
    let (app, mut stage, look) = staged(DESK);
    assert!(shown(&stage, &look).contains(&"No model".to_string()));
    assert!(pill(&app, &mut stage, &look));
    stage.tick();
    let text = shown(&stage, &look).join(" ");
    assert!(text.contains(app.desk().text.get("roster.none")), "{text}");
    // A press where a row would be does nothing, and the list stays.
    let body = stage.panel().expect("open").body;
    assert!(!stage.send(&PEvent::PointerDown(PPoint::new(body.center().x, body.origin.y + 60.0))));
    assert!(stage.panel().is_some() && app.desk().history.borrow().voice().is_empty());
}

#[test]
fn a_press_on_the_title_beside_the_pill_opens_nothing() {
    let (app, mut stage, look) = voiced(DESK);
    Phigen::key(&mut stage, "a question");
    shown(&stage, &look);
    let title = rect(&stage, "title");
    let (start, wide) = app.desk().column(title.size.width);
    assert!(!stage.send(&PEvent::PointerDown(PPoint::new(title.origin.x + start + wide - 10.0, title.center().y))));
    assert!(stage.panel().is_none());
}
