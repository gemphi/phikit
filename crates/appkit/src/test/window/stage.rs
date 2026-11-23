use crate::pui::PApp;
use crate::pui::core::*;
use crate::pui::window::*;
use crate::test::core::{Seen, Tile, asked};
use std::rc::Rc;

use crate::theme::{PLook, PTheme};

/// A look for the tests: a header 40 high, and a white window.
const LOOK: &str = "size\tregion.header\t40\ncolor\twindow.fill\t#ffffff\n";

/// An app for the tests: a list beside a page, the page three times as
/// wide. The page's `text` part shows nothing yet.
struct Note {
    space: PSet<PSpace>,
    host: PSet<PHost>,
    part: PSet<PPart>,
    modal: PSet<PModal>,
    seen: Seen,
}

impl Note {
    fn new() -> Note {
        let (mut space, mut host) = (PSet::read("main\tMain\n"), PSet::read("list\tList\npage\tPage\n"));
        PSpace::arrange(&mut space, "main\tside\tlist\t1\nmain\tbody\tpage\t3\n");
        PHost::declare(&mut host, "list\tcenter\tcenter\tindex\npage\theader\tcenter\ttitle\npage\tcenter\tcenter\ttext\n");
        Note { space, host, part: PSet::read("index\tIndex\ntitle\tTitle\ntext\tText\n"), modal: PSet::new(), seen: Seen::default() }
    }
}

impl PApp for Note {
    fn name(&self) -> &str {
        "Note"
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
        Rc::new(PLook::read(LOOK))
    }

    fn content(&self, part: &str) -> Option<Box<dyn PView>> {
        let name = ["index", "title"].into_iter().find(|known| *known == part)?;
        Some(Tile::new(name, PRect::default(), true, &self.seen, Vec::new()))
    }

    fn front(&self) -> &str {
        "page"
    }

    fn focus(&self) -> &str {
        "title"
    }
}

fn laid(stage: &PStage) -> Vec<(&str, PRect)> {
    stage.slot().iter().map(|slot| (slot.part.as_str(), slot.rect)).collect()
}

#[test]
fn at_a_desk_every_part_of_every_host_has_its_rectangle() {
    let mut stage = Note::new().stage();
    stage.lay(PSize { width: 800.0, height: 600.0 }, PMode::Desktop);
    assert_eq!(laid(&stage), [("index", PRect::new(0.0, 0.0, 200.0, 600.0)), ("title", PRect::new(200.0, 0.0, 600.0, 40.0)), ("text", PRect::new(200.0, 40.0, 600.0, 560.0))]);
}

#[test]
fn in_a_hand_the_host_the_app_puts_in_front_has_the_window() {
    let mut stage = Note::new().stage();
    stage.lay(PSize { width: 390.0, height: 800.0 }, PMode::Mobile);
    assert_eq!(laid(&stage), [("title", PRect::new(0.0, 0.0, 390.0, 40.0)), ("text", PRect::new(0.0, 40.0, 390.0, 760.0))]);
}

#[test]
fn a_press_gives_the_keys_to_the_part_under_it_and_a_key_it_does_not_take_goes_to_the_part_the_app_named() {
    let app = Note::new();
    let mut stage = app.stage();
    stage.lay(PSize { width: 800.0, height: 600.0 }, PMode::Desktop);
    // The app said which part takes the keys first.
    assert_eq!(stage.focus(), Some("title"));
    assert!(stage.send(&PEvent::KeyDown('a')));
    assert!(stage.send(&PEvent::PointerDown(PPoint::new(10.0, 10.0))));
    assert_eq!(stage.focus(), Some("index"));
    assert!(stage.send(&PEvent::KeyDown('b')));
    assert_eq!(asked(&app.seen), ["title", "index", "index"]);
    // A press on a part that shows nothing is not handled. It has the
    // keys now and takes none, so a key goes on to the part the app named.
    assert!(!stage.send(&PEvent::PointerDown(PPoint::new(400.0, 300.0))));
    assert_eq!(stage.focus(), Some("text"));
    assert!(stage.send(&PEvent::KeyDown('c')));
    assert_eq!(asked(&app.seen), ["title", "index", "index", "title"]);
}

#[test]
fn a_stage_draws_its_ground_and_then_every_part_that_shows_something() {
    let look = PLook::read(LOOK);
    let mut stage = Note::new().stage();
    stage.lay(PSize { width: 800.0, height: 600.0 }, PMode::Desktop);
    let mut sheet = PSheet::new(&look);
    stage.draw(&mut sheet);
    let rect = |mark: &PMark| match mark {
        PMark::Fill { rect, .. } => Some(*rect),
        _ => None,
    };
    let fill: Vec<PRect> = sheet.mark.iter().filter_map(rect).collect();
    assert_eq!(fill, [PRect::new(0.0, 0.0, 800.0, 600.0), PRect::new(0.0, 0.0, 200.0, 600.0), PRect::new(200.0, 0.0, 600.0, 40.0)]);
    assert_eq!(sheet.mark.first(), Some(&PMark::Fill { rect: PRect::new(0.0, 0.0, 800.0, 600.0), radius: 0.0, color: PColor::new(1.0, 1.0, 1.0, 1.0) }));
}

#[test]
fn a_key_sent_before_the_window_was_laid_out_goes_to_the_part_the_app_named() {
    let app = Note::new();
    let mut stage = app.stage();
    assert!(stage.send(&PEvent::KeyDown('a')));
    assert_eq!(asked(&app.seen), ["title"]);
}

#[test]
fn a_part_is_drawn_inside_its_own_rectangle() {
    let look = PLook::read(LOOK);
    let mut stage = Note::new().stage();
    stage.lay(PSize { width: 800.0, height: 600.0 }, PMode::Desktop);
    let mut sheet = PSheet::new(&look);
    stage.draw(&mut sheet);
    // After the ground: a clip, the part, the clip's end, for each part
    // that shows something.
    let index = PRect::new(0.0, 0.0, 200.0, 600.0);
    assert_eq!(sheet.mark[1], PMark::Clip(index));
    assert!(matches!(&sheet.mark[2], PMark::Fill { rect, .. } if *rect == index));
    assert_eq!(sheet.mark[3], PMark::Unclip);
    assert_eq!(sheet.mark.iter().filter(|mark| matches!(mark, PMark::Clip(_))).count(), 2);
}

/// A view that has something more to come a few times.
struct Busy(u32);

impl PDraw for Busy {
    fn draw(&self, _paint: &mut dyn PPaint, _origin: PPoint) {}
}

impl PView for Busy {
    fn frame(&self) -> PRect {
        PRect::default()
    }

    fn set_frame(&mut self, _frame: PRect) {}

    fn tick(&mut self) -> bool {
        self.0 = self.0.saturating_sub(1);
        self.0 > 0
    }
}

/// The same app with something changed: its `text` part busy for that
/// many frames, a theme that gives tabs a height, and its own word on
/// whether its spaces are shown as tabs.
struct Other {
    note: Note,
    busy: u32,
    tab: bool,
}

impl PApp for Other {
    fn name(&self) -> &str {
        self.note.name()
    }

    fn space(&self) -> &PSet<PSpace> {
        self.note.space()
    }

    fn host(&self) -> &PSet<PHost> {
        self.note.host()
    }

    fn part(&self) -> &PSet<PPart> {
        self.note.part()
    }

    fn modal(&self) -> &PSet<PModal> {
        self.note.modal()
    }

    fn theme(&self) -> Rc<dyn PTheme> {
        Rc::new(PLook::read("size\ttab.height\t40\n"))
    }

    fn content(&self, part: &str) -> Option<Box<dyn PView>> {
        (part == "text" && self.busy > 0).then(|| Box::new(Busy(self.busy)) as Box<dyn PView>)
    }

    fn tab(&self) -> bool {
        self.tab
    }
}

#[test]
fn a_stage_is_busy_for_as_long_as_one_of_its_views_has_more_to_come() {
    let mut stage = Other { note: Note::new(), busy: 3, tab: true }.stage();
    assert_eq!([stage.tick(), stage.tick(), stage.tick(), stage.tick()], [true, true, false, false]);
    // A stage whose views change only by a hand is never busy.
    assert!(!Note::new().stage().tick());
}

#[test]
fn an_app_with_several_spaces_has_tabs_over_its_body_unless_it_says_no() {
    let two = || {
        let mut note = Note::new();
        note.space.add(PSpace::of(PEntry::new("other", "Other", "")));
        note
    };
    let high = |tab: bool| {
        let mut stage = Other { note: two(), busy: 0, tab }.stage();
        stage.lay(PSize { width: 800.0, height: 600.0 }, PMode::Desktop);
        (stage.frame().tab.size.height, stage.frame().body.origin.y)
    };
    assert_eq!([high(true), high(false)], [(40.0, 40.0), (0.0, 0.0)]);
}

mod bar;
