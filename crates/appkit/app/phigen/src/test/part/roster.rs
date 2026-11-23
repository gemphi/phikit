use appkit::pui::core::{PEvent, PMark, PMode, PPoint};
use appkit::pui::window::{PDeed, PStage};

use crate::test::{DESK, drawn, shown, voiced_by, written};

#[test]
fn the_model_that_answers_is_marked_once_and_more_than_fit_are_scrolled() {
    let name: Vec<String> = (0..40).map(|number| format!("model-{number}")).collect();
    let (app, mut stage, look) = voiced_by(&name.iter().map(String::as_str).collect::<Vec<_>>(), DESK);
    stage.window.apply(&PDeed::Open("model".to_string()));
    stage.lay(DESK, PMode::Desktop);
    stage.tick();
    let marked = |mark: Vec<PMark>| mark.iter().filter(|mark| matches!(mark, PMark::Icon { name, .. } if name == "check-circle")).count();
    assert_eq!(marked(drawn(&stage, &look)), 1);
    // The first are shown; the last is not until the wheel goes down. The
    // pill over the chat names the first too, so it is there once more.
    let times = |stage: &PStage, name: &str| shown(stage, &look).iter().filter(|line| *line == name).count();
    assert_eq!((times(&stage, "model-0"), times(&stage, "model-39")), (2, 0));
    let body = stage.panel().expect("open").body;
    assert!(stage.send(&PEvent::Scroll(body.center(), PPoint::new(0.0, -100000.0))));
    assert_eq!((times(&stage, "model-0"), times(&stage, "model-39")), (1, 1));
    // The one that answers is out of sight, so nothing is marked; a press
    // on one that was scrolled to has it answer.
    assert_eq!(marked(drawn(&stage, &look)), 0);
    let last = written(&stage, &look, "model-39").expect("listed");
    assert!(stage.send(&PEvent::PointerDown(last)));
    assert_eq!(app.desk().history.borrow().voice(), "model-39");
    // Opened again it is where it was left, the one that answers marked;
    // and the wheel goes no further than the last row.
    stage.window.apply(&PDeed::Open("model".to_string()));
    stage.lay(DESK, PMode::Desktop);
    let before = shown(&stage, &look);
    stage.send(&PEvent::Scroll(body.center(), PPoint::new(0.0, -100000.0)));
    assert_eq!((shown(&stage, &look), marked(drawn(&stage, &look))), (before, 1));
}
