use std::rc::Rc;

use crate::pui::PApp;
use crate::theme::{PLook, PTheme};
use crate::test::window::{host_set, space};
use crate::pui::window::*;

/// An app for the tests: it supplies its four sets and nothing else.
struct Book {
    space: PSet<PSpace>,
    host: PSet<PHost>,
    part: PSet<PPart>,
    modal: PSet<PModal>,
}

impl Book {
    fn new() -> Book {
        let mut book = Book { space: PSet::new(), host: host_set(), part: PSet::read("menubar\tMenu Bar\ncanvas\tCanvas\n"), modal: PSet::new() };
        book.space.add(space("layout", &["view3d", "outliner"]));
        book.space.add(space("shading", &["view3d"]));
        book
    }
}

impl PApp for Book {
    fn name(&self) -> &str {
        "Book"
    }

    fn space(&self) -> &PSet<PSpace> {
        &self.space
    }

    fn host(&self) -> &PSet<PHost> {
        &self.host
    }

    fn part(&self) -> &PSet<PPart> {
        &self.part
    }

    fn modal(&self) -> &PSet<PModal> {
        &self.modal
    }

    fn theme(&self) -> Rc<dyn PTheme> {
        Rc::new(PLook::default())
    }
}

#[test]
fn appkit_opens_an_app_s_window_on_its_spaces_showing_the_first() {
    let window = Book::new().open();
    let tab: Vec<&str> = window.space.iter().map(PSpace::slug).collect();
    assert_eq!((window.title.as_str(), tab), ("Book", vec!["layout", "shading"]));
    assert_eq!(window.current().map(PSpace::slug), Some("layout"));
}

#[test]
fn an_app_that_supplies_everything_it_names_is_missing_nothing() {
    assert!(Book::new().missing().is_empty());
}

#[test]
fn an_app_is_told_what_it_names_and_has_not_supplied() {
    let mut book = Book::new();
    book.space.add(space("nodes", &["node"]));
    book.part = PSet::read("menubar\tMenu Bar\n");
    assert_eq!(book.missing(), ["host node", "part canvas"]);
}

#[test]
fn any_app_is_opened_the_same_way() {
    // Through the trait alone, as appkit does it.
    let app: &dyn PApp = &Book::new();
    assert_eq!(app.open().title, app.name());
}
