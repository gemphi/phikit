use std::cell::RefCell;
use std::rc::Rc;

use appkit::pui::PApp;
use appkit::pui::core::{PColor, PEvent, PMark, PMode, PRect, PSheet};
use appkit::pui::window::{PDeed, PLevel, PStage};
use appkit::theme::{PLook, PTheme};

use super::{DESK, HAND, drawn, rect, shown, staged};
use crate::chat::Turn;
use crate::{Phigen, Reply, Source, Whole, data};

#[test]
fn phigen_supplies_everything_its_window_names_and_a_view_for_every_part() {
    let app = Phigen::new(Rc::new(Phigen::look("")));
    assert_eq!((app.name(), app.front(), app.focus()), ("Phigen", "chat", "ask"));
    assert!(app.missing().is_empty(), "{:?}", app.missing());
    let bare: Vec<&str> = app.part().list().iter().map(|part| part.slug()).filter(|part| app.content(part).is_none()).collect();
    assert!(bare.is_empty(), "{bare:?}");
}

#[test]
fn at_a_desk_the_chats_are_in_the_bar_s_menu_beside_the_chat() {
    let (_, mut stage, _) = staged(DESK);
    // Before anything is said the field is in the middle of what the
    // header leaves, a heading over it and the ways to begin under it.
    let part = |stage: &PStage| stage.slot().iter().map(|slot| slot.part.clone()).collect::<Vec<_>>();
    assert_eq!(part(&stage), ["title", "hello", "ask", "card", "thread"]);
    assert_eq!(rect(&stage, "ask"), PRect::new(264.0, 368.0, 776.0, 84.0));
    Phigen::key(&mut stage, "hi");
    assert_eq!(part(&stage), ["title", "talk", "ask", "thread"]);
    // The caption is the window's first 44. Under it the activity bar
    // is 56 wide and its menu 200, 8 clear of the window's side and
    // foot, and there are no tabs over the body. The chat has what is left; its header and
    // footer have the heights the theme gives. The chats are in the
    // menu, in the group under New chat and Search.
    assert_eq!(stage.frame().caption, PRect::new(0.0, 0.0, 1040.0, 44.0));
    assert_eq!(rect(&stage, "title"), PRect::new(264.0, 44.0, 776.0, 56.0));
    assert_eq!(rect(&stage, "talk"), PRect::new(264.0, 100.0, 776.0, 536.0));
    assert_eq!(stage.frame().tab.size.height, 0.0);
    assert_eq!(rect(&stage, "ask"), PRect::new(264.0, 636.0, 776.0, 84.0));
    assert_eq!(rect(&stage, "thread"), PRect::new(72.0, 156.0, 184.0, 540.0));
}

#[test]
fn narrowed_it_is_a_mobile_app_and_shows_the_chat_alone() {
    let (_, mut stage, _) = staged(HAND);
    let part = |stage: &PStage| stage.slot().iter().map(|slot| slot.part.clone()).collect::<Vec<_>>();
    assert_eq!(part(&stage), ["title", "hello", "ask", "card"]);
    Phigen::key(&mut stage, "hi");
    assert_eq!(part(&stage), ["title", "talk", "ask"]);
    // The activity bar is under the chat, 8 clear of it, of the
    // window's foot and of its sides.
    assert_eq!(rect(&stage, "ask"), PRect::new(0.0, 644.0, 390.0, 84.0));
    assert_eq!(stage.frame().activity, PRect::new(8.0, 736.0, 374.0, 56.0));
}

#[test]
fn it_opens_greeting_the_person_and_asking_what_they_want() {
    let (app, mut stage, look) = staged(DESK);
    let text = shown(&stage, &look);
    assert!(text.contains(&"Phigen".to_string()) && text.contains(&"New chat".to_string()));
    assert!(text.contains(&app.desk().text.get("hello").to_string()) && text.contains(&"Ask Phigen".to_string()));
    // Once something is said the chat is shown, from its greeting on. The
    // greeting is longer than a line, so it is looked for whole.
    Phigen::key(&mut stage, "hi");
    let text = shown(&stage, &look);
    assert!(text.join(" ").contains(app.desk().text.get("greeting")), "{text:?}");
}

/// A theme that writes down every token it is asked for and does not have.
struct Strict<'a> {
    look: &'a PLook,
    missed: RefCell<Vec<String>>,
}

impl Strict<'_> {
    fn note<T>(&self, token: &str, found: Option<T>) -> Option<T> {
        if found.is_none() {
            self.missed.borrow_mut().push(token.to_string());
        }
        found
    }
}

impl PTheme for Strict<'_> {
    fn color(&self, token: &str) -> Option<PColor> {
        self.note(token, self.look.color(token))
    }

    fn size(&self, token: &str) -> Option<f32> {
        self.note(token, self.look.size(token))
    }
}

#[test]
fn every_colour_and_size_phigen_is_drawn_with_is_in_its_theme() {
    for size in [DESK, HAND] {
        let (_, mut stage, look) = staged(size);
        let strict = Strict { look: look.as_ref(), missed: RefCell::new(Vec::new()) };
        // A chat nothing was said in, alone and under the welcome.
        stage.draw(&mut PSheet::new(&strict));
        stage.window.apply(&PDeed::Open("welcome".to_string()));
        stage.lay(size, PMode::of_width(size.width));
        stage.draw(&mut PSheet::new(&strict));
        stage.send(&PEvent::KeyDown('\u{1b}'));
        // Both speakers have said something, and each of the other
        // modals is open in turn: the profile, the settings, the models.
        Phigen::key(&mut stage, "hello");
        for modal in ["profile", "setting", "model"] {
            stage.window.apply(&PDeed::Open(modal.to_string()));
            stage.lay(size, PMode::of_width(size.width));
            stage.tick();
            assert!(stage.panel().is_some());
            stage.draw(&mut PSheet::new(&strict));
        }
        // And the Search window, with the profile closed.
        stage.send(&PEvent::KeyDown('\u{1b}'));
        assert!(stage.window.apply(&PDeed::Show("search".to_string())));
        stage.lay(size, PMode::of_width(size.width));
        stage.draw(&mut PSheet::new(&strict));
        assert!(strict.missed.borrow().is_empty(), "not in data/theme.tsv: {:?}", strict.missed.borrow());
    }
}

/// Somebody else answering: the last thing said, backwards.
struct Mirror;

impl Reply for Mirror {
    fn reply(&mut self, said: &str, _past: &[Turn]) -> Box<dyn Source> {
        Box::new(Whole(said.chars().rev().collect()))
    }
}

#[test]
fn whoever_stands_behind_reply_answers() {
    let look = Rc::new(Phigen::look(""));
    let app = Phigen::answered(Rc::clone(&look), Box::new(Mirror));
    let mut stage = app.stage();
    stage.lay(DESK, PMode::Desktop);
    Phigen::key(&mut stage, "stressed");
    assert!(shown(&stage, &look).contains(&"desserts".to_string()));
}

#[test]
fn the_window_s_size_can_be_changed_by_a_row_after_the_theme() {
    let look = Phigen::look("size\twindow.width\t420\n");
    assert_eq!((look.size("window.width"), look.size("window.height")), (Some(420.0), PLook::read(data::THEME).size("window.height")));
}

#[test]
fn nothing_phigen_draws_has_a_fill_of_its_own() {
    for size in [DESK, HAND] {
        let (_, mut stage, look) = staged(size);
        Phigen::key(&mut stage, "hello");
        stage.window.apply(&PDeed::Open("profile".to_string()));
        stage.lay(size, PMode::of_width(size.width));
        stage.tick();
        let mark = drawn(&stage, &look);
        // What is flat is the window's ground, a rule, a caret and the
        // veil behind a modal. Everything else wears the light.
        let plain = ["window.fill", "line.ink", "control.accent", "modal.veil"].map(|token| look.color(token).expect("in the theme"));
        let flat: Vec<&PMark> = mark.iter().filter(|mark| matches!(mark, PMark::Fill { color, .. } if !plain.contains(color))).collect();
        assert!(flat.is_empty(), "{flat:?}");
        let lit = mark.iter().filter(|mark| matches!(mark, PMark::Blend { .. })).count();
        assert!(lit >= 8, "{lit}");
    }
}
