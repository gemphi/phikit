//! Tests of `widget`: the four traits, through a label and a row made for
//! the purpose.

use std::cell::Cell;
use std::rc::Rc;

use crate::pui::core::*;
use crate::test::core::Board;
use crate::pui::widget::*;

/// A widget for the tests: a line of text that can be clicked.
#[derive(Default)]
struct Label {
    text: String,
    color: PColor,
    background: PColor,
    radius: f32,
    padding: f32,
    width: f32,
    height: f32,
    disabled: bool,
    click: Option<Box<dyn FnMut()>>,
}

impl Label {
    fn new(text: &str) -> Label {
        Label { text: text.to_string(), width: 50.0, height: 10.0, ..Label::default() }
    }
}

impl PStyle for Label {
    fn color(self, color: PColor) -> Self {
        Label { color, ..self }
    }

    fn background(self, background: PColor) -> Self {
        Label { background, ..self }
    }

    fn corner_radius(self, radius: f32) -> Self {
        Label { radius, ..self }
    }

    fn padding(self, padding: f32) -> Self {
        Label { padding, ..self }
    }

    fn width(self, width: f32) -> Self {
        Label { width, ..self }
    }

    fn height(self, height: f32) -> Self {
        Label { height, ..self }
    }
}

impl PInput for Label {
    fn on_click<F: FnMut() + 'static>(self, callback: F) -> Self {
        Label { click: Some(Box::new(callback)), ..self }
    }

    fn enabled(self, enabled: bool) -> Self {
        Label { disabled: !enabled, ..self }
    }
}

/// The view a label or a row builds.
struct Built {
    frame: PRect,
    label: Label,
    child: Vec<Box<dyn PView>>,
}

impl PDraw for Built {
    fn draw(&self, paint: &mut dyn PPaint, origin: PPoint) {
        paint.fill(PRect { origin, size: self.frame.size }, self.label.background);
        paint.text(origin.plus(PPoint::new(self.label.padding, self.label.padding)), &self.label.text, self.label.color);
    }
}

impl PView for Built {
    fn frame(&self) -> PRect {
        self.frame
    }

    fn set_frame(&mut self, frame: PRect) {
        self.frame = frame;
    }

    fn respond(&mut self, event: &PEvent) -> bool {
        match (event, self.label.disabled, self.label.click.as_mut()) {
            (PEvent::PointerDown(_), false, Some(click)) => {
                click();
                true
            }
            _ => false,
        }
    }

    fn child_list(&self) -> &[Box<dyn PView>] {
        &self.child
    }

    fn child_list_mut(&mut self) -> &mut [Box<dyn PView>] {
        &mut self.child
    }
}

impl PWidget for Label {
    fn build(self) -> Box<dyn PView> {
        let room = self.padding * 2.0;
        Box::new(Built { frame: PRect::new(0.0, 0.0, self.width + room, self.height + room), label: self, child: Vec::new() })
    }
}

/// A widget for the tests: labels side by side.
#[derive(Default)]
struct Row {
    cell: Vec<Label>,
}

impl PCard<Label> for Row {
    fn add(mut self, item: Label) -> Self {
        self.cell.push(item);
        self
    }
}

impl PWidget for Row {
    fn build(self) -> Box<dyn PView> {
        let mut left = 0.0;
        let mut child: Vec<Box<dyn PView>> = Vec::new();
        for label in self.cell {
            let mut view = label.build();
            let size = view.frame().size;
            view.set_frame(PRect { origin: PPoint::new(left, 0.0), size });
            left += size.width;
            child.push(view);
        }
        Box::new(Built { frame: PRect::new(0.0, 0.0, left, 100.0), label: Label::default(), child })
    }
}

#[test]
fn a_widget_is_styled_in_a_chain_and_builds_the_view_it_describes() {
    let ink = PColor::new(0.0, 0.0, 0.0, 1.0);
    let label = Label::new("Name").color(ink).background(PColor::new(1.0, 1.0, 1.0, 1.0)).corner_radius(3.0).padding(4.0).width(60.0).height(12.0);
    assert_eq!((label.radius, label.color), (3.0, ink));
    let view = label.build();
    // The room around the content is on both sides.
    assert_eq!(view.frame(), PRect::new(0.0, 0.0, 68.0, 20.0));
    let mut board = Board::default();
    show(view.as_ref(), &mut board, PPoint::new(10.0, 10.0));
    assert_eq!(board.text, [(PPoint::new(14.0, 14.0), "Name".to_string())]);
}

#[test]
fn a_click_runs_what_the_widget_was_told_unless_it_is_disabled() {
    let count = Rc::new(Cell::new(0));
    let told = |count: &Rc<Cell<i32>>| {
        let count = Rc::clone(count);
        move || count.set(count.get() + 1)
    };
    let mut view = Label::new("Go").on_click(told(&count)).build();
    assert!(send(view.as_mut(), &PEvent::PointerDown(PPoint::new(5.0, 5.0))));
    assert_eq!(count.get(), 1);
    let mut view = Label::new("Go").on_click(told(&count)).enabled(false).build();
    assert!(!send(view.as_mut(), &PEvent::PointerDown(PPoint::new(5.0, 5.0))));
    assert_eq!(count.get(), 1);
    // A widget told nothing handles nothing.
    assert!(!send(Label::new("Still").build().as_mut(), &PEvent::PointerDown(PPoint::new(5.0, 5.0))));
}

#[test]
fn a_container_keeps_what_it_is_given_in_order() {
    let row = Row::default().add(Label::new("a")).extend([Label::new("b"), Label::new("c")]).add(Label::new("d"));
    let name: Vec<&str> = row.cell.iter().map(|label| label.text.as_str()).collect();
    assert_eq!(name, ["a", "b", "c", "d"]);
    let view = row.build();
    let left: Vec<f32> = view.child_list().iter().map(|child| child.frame().origin.x).collect();
    assert_eq!(left, [0.0, 50.0, 100.0, 150.0]);
}

#[test]
fn a_click_in_a_row_reaches_the_cell_under_it() {
    let hit = Rc::new(Cell::new(' '));
    let cell = |name: char| {
        let hit = Rc::clone(&hit);
        Label::new("cell").on_click(move || hit.set(name))
    };
    let mut view = Row::default().extend([cell('a'), cell('b'), cell('c')]).build();
    assert!(send(view.as_mut(), &PEvent::PointerDown(PPoint::new(75.0, 5.0))));
    assert_eq!(hit.get(), 'b');
}
