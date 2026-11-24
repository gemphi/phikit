//! Search, as Phiegg's is: its tabs, what is sought most often before
//! anything is, and every kind it finds, Phiegg's own records among
//! them.

use appkit::pui::core::{PEvent, PMark, PPoint};
use appkit::pui::window::PStage;
use appkit::theme::PLook;

use super::{DESK, drawn, press, shown, staged};

/// A press on the last place a line of text is drawn inside the
/// window: in the modal, drawn last, a tab or a chip, not the heading of
/// the same words further down, past the window's foot.
fn tab(stage: &mut PStage, look: &PLook, wanted: &str) -> bool {
    let mut at = drawn(stage, look).into_iter().filter_map(|mark| match mark {
        PMark::Text { at, text, .. } if text == wanted && at.y < DESK.height => Some(at.plus(PPoint::new(2.0, 2.0))),
        _ => None,
    });
    at.next_back().is_some_and(|at| stage.send(&PEvent::PointerDown(at)))
}

#[test]
fn step_63_search_is_phiegg_s_its_tabs_what_is_sought_most_and_every_kind_it_finds() {
    let (app, mut stage, look) = staged(DESK);
    let desk = app.desk();
    let word = |name: &str| desk.text.get(name).to_string();
    assert!(press(&mut stage, &look, &word("search.open")));
    assert_eq!(stage.window.modal(), Some("search"));
    let text = shown(&stage, &look);
    // A tab for each kind it finds, as data/search.tsv has them.
    for tab in ["search.all", "search.room", "search.people", "search.host", "search.world", "search.club", "search.show"] {
        assert!(text.contains(&word(tab)), "no {tab} in {text:?}");
    }
    // Before anything is sought: what is sought most often, Phiegg's own.
    assert!(text.contains(&word("search.popular")) && text.contains(&"Once City".to_string()), "{text:?}");
    // Phiegg's worlds, the very records Phiegg reads.
    // The tab, not the heading of the worlds found under All further down.
    assert!(tab(&mut stage, &look, &word("search.world")));
    let text = shown(&stage, &look);
    assert!(text.contains(&"Opalia".to_string()) && !text.contains(&word("search.none")), "{text:?}");
    // A press on what is sought most often seeks it.
    assert!(tab(&mut stage, &look, &word("search.all")) && tab(&mut stage, &look, "Once City"));
    let text = shown(&stage, &look);
    assert!(text.iter().filter(|line| line.contains("Once City")).count() >= 2, "{text:?}");
}
