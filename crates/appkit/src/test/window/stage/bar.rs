//! The stage of an app with an activity bar: which row it picks, where
//! the menu goes at a desk and in a hand, and what picking a row does.

use std::rc::Rc;

use super::Note;
use crate::pui::PApp;
use crate::pui::core::*;
use crate::pui::window::*;
use crate::theme::{PLook, PTheme};

const LOOK: &str = "size\tgap\t10\nsize\tactivity.row\t40\nsize\tactivity.width\t200\nsize\tactivity.low\t50\n";
/// The same look with a caption 40 high.
const CAPPED: &str = "size\tgap\t10\nsize\tactivity.row\t40\nsize\tactivity.width\t200\nsize\tactivity.low\t50\nsize\tcaption.height\t40\n";
const ITEM: &str = "main\tMain\tgrid\ttop\tspace\tmain\nother\tOther\tbox\ttop\tspace\tother\nhome\tHome\thome\ttop\ttell\thome\n";
const MENU: &str = "main\ttop\tfile\tnew\tNew\tplus\ttell\tbegin\nother\ttop\tfile\tback\tBack\tgrid\tspace\tmain\nhome\ttop\tfile\tfind\tFind\tsearch\tspace\tother\n";

/// The note app with a second space, and a bar of three rows, each with
/// a menu: two go to a space, and the third tells the app `home`. The
/// last is its look.
struct Barred(Note, PWish, &'static str);

/// A view that takes the word `home` and wishes the first space shown,
/// as a part does that takes a person back to where they began.
struct Teller(PWish);

impl PDraw for Teller {
    fn draw(&self, _paint: &mut dyn PPaint, _origin: PPoint) {}
}

impl PView for Teller {
    fn frame(&self) -> PRect {
        PRect::default()
    }

    fn set_frame(&mut self, _frame: PRect) {}

    fn tell(&mut self, word: &str) -> bool {
        let known = word == "home";
        self.0.borrow_mut().extend(known.then(|| PDeed::Show("main".to_string())));
        known
    }
}

impl Barred {
    fn stage(size: PSize, mode: PMode) -> PStage {
        Barred::looking(LOOK, size, mode)
    }

    fn looking(look: &'static str, size: PSize, mode: PMode) -> PStage {
        let mut note = Note::new();
        note.space.add(PSpace::of(PEntry::new("other", "Other", "")));
        let mut stage = Barred(note, PWish::default(), look).stage();
        stage.lay(size, mode);
        stage
    }
}

impl PApp for Barred {
    fn name(&self) -> &str {
        self.0.name()
    }

    fn space(&self) -> &PSet<PSpace> {
        self.0.space()
    }

    fn host(&self) -> &PSet<PHost> {
        self.0.host()
    }

    fn part(&self) -> &PSet<PPart> {
        self.0.part()
    }

    fn modal(&self) -> &PSet<PModal> {
        self.0.modal()
    }

    fn theme(&self) -> Rc<dyn PTheme> {
        Rc::new(PLook::read(self.2))
    }

    fn activity(&self) -> PActivity {
        PActivity::read(ITEM).hold(MENU)
    }

    fn content(&self, part: &str) -> Option<Box<dyn PView>> {
        (part == "text").then(|| Box::new(Teller(self.1.clone())) as Box<dyn PView>)
    }

    fn wish(&self) -> Option<PWish> {
        Some(self.1.clone())
    }

    fn tab(&self) -> bool {
        false
    }
}

const DESK: PSize = PSize { width: 800.0, height: 600.0 };
const HAND: PSize = PSize { width: 400.0, height: 600.0 };

fn press(stage: &mut PStage, x: f32, y: f32) -> bool {
    stage.send(&PEvent::PointerDown(PPoint::new(x, y)))
}

#[test]
fn at_a_desk_the_menu_of_the_row_picked_stands_beside_the_bar_until_it_is_put_away() {
    let mut stage = Barred::stage(DESK, PMode::Desktop);
    // The window shows the main space, so that row's menu is the one.
    assert_eq!(stage.window.pick(), "main");
    let frame = stage.frame();
    assert_eq!((frame.activity, frame.menu, frame.body), (PRect::new(0.0, 0.0, 50.0, 600.0), PRect::new(50.0, 0.0, 200.0, 600.0), PRect::new(250.0, 0.0, 550.0, 600.0)));
    // The mark at the bar's head puts it away, and the body has the room.
    assert!(press(&mut stage, 25.0, 25.0));
    assert_eq!((stage.frame().menu.size.width, stage.frame().body), (0.0, PRect::new(50.0, 0.0, 750.0, 600.0)));
    // The row picked brings it back, and puts it away again.
    assert!(press(&mut stage, 25.0, 70.0));
    assert_eq!(stage.frame().menu.size.width, 200.0);
    assert!(press(&mut stage, 25.0, 70.0));
    assert_eq!(stage.frame().menu.size.width, 0.0);
}

#[test]
fn picking_a_row_does_its_own_deed_and_shows_its_menu() {
    let mut stage = Barred::stage(DESK, PMode::Desktop);
    assert!(press(&mut stage, 25.0, 110.0));
    assert_eq!((stage.window.current().map(PSpace::slug), stage.window.pick()), (Some("other"), "other"));
    // The row of its menu goes back to the first space; the menu stays.
    assert!(press(&mut stage, 100.0, 75.0));
    assert_eq!((stage.window.current().map(PSpace::slug), stage.window.pick(), stage.frame().menu.size.width), (Some("main"), "other", 200.0));
}

#[test]
fn in_a_hand_a_menu_is_drawn_out_over_the_space_and_a_word_told_puts_it_away() {
    let mut stage = Barred::stage(HAND, PMode::Mobile);
    let body = PRect::new(0.0, 0.0, 400.0, 550.0);
    // The menu starts put away, and the space's parts are shown.
    assert_eq!((stage.frame().menu, stage.frame().body, stage.slot().is_empty()), (PRect::default(), body, false));
    // A press on the first square draws its menu out over the body,
    // whose parts are not shown under it.
    assert!(press(&mut stage, 25.0, 575.0));
    assert_eq!((stage.frame().menu, stage.slot().is_empty()), (body, true));
    // A word is the app's to do, and nobody here takes it; the menu is
    // put away all the same.
    assert!(!press(&mut stage, 100.0, 75.0));
    assert_eq!((stage.frame().menu, stage.slot().is_empty()), (PRect::default(), false));
    // The other row's menu goes to a space, which puts it away too.
    assert!(press(&mut stage, 75.0, 575.0));
    assert_eq!(stage.frame().menu, body);
    assert!(press(&mut stage, 100.0, 75.0));
    assert_eq!((stage.frame().menu, stage.window.current().map(PSpace::slug)), (PRect::default(), Some("main")));
}

#[test]
fn a_row_whose_own_deed_has_a_part_wish_for_a_space_shows_its_menu_after_that() {
    for (size, mode, at) in [(DESK, PMode::Desktop, PPoint::new(25.0, 150.0)), (HAND, PMode::Mobile, PPoint::new(125.0, 575.0))] {
        let mut stage = Barred::stage(size, mode);
        assert!(stage.window.apply(&PDeed::Show("other".to_string())));
        stage.lay(size, mode);
        // The third row tells the app `home`; a part takes the word and
        // wishes the first space shown, which is done before the row's
        // menu is shown: in a hand going to a space would put it away.
        assert!(press(&mut stage, at.x, at.y));
        assert_eq!((stage.window.current().map(PSpace::slug), stage.window.pick()), (Some("main"), "home"));
        assert!(stage.frame().menu.size.width > 0.0, "{mode:?}");
        // The same row again puts its menu away and tells nothing.
        assert!(press(&mut stage, at.x, at.y));
        assert_eq!((stage.frame().menu.size.width, stage.window.current().map(PSpace::slug)), (0.0, Some("main")));
    }
}

#[test]
fn under_a_caption_the_pin_and_the_mark_are_at_the_top_and_a_modal_is_laid_under_it() {
    let mut stage = Barred::looking(CAPPED, DESK, PMode::Desktop);
    let frame = stage.frame();
    assert_eq!((frame.caption, frame.activity), (PRect::new(0.0, 0.0, 800.0, 40.0), PRect::new(0.0, 40.0, 50.0, 560.0)));
    assert_eq!((frame.menu, frame.body), (PRect::new(50.0, 40.0, 200.0, 560.0), PRect::new(250.0, 40.0, 550.0, 560.0)));
    // The mark is the pin: a press on it puts the menu away, and
    // another brings it back; nothing else on the caption is pressed.
    assert!(press(&mut stage, 25.0, 20.0));
    assert_eq!(stage.frame().menu.size.width, 0.0);
    assert!(!press(&mut stage, 70.0, 20.0));
    assert!(press(&mut stage, 25.0, 20.0));
    assert_eq!(stage.frame().menu.size.width, 200.0);
    assert!(press(&mut stage, 25.0, 20.0));
    assert_eq!(stage.frame().menu.size.width, 0.0);
    // The pointer over it is taken notice of, and where a press does
    // something the pointer is a hand.
    assert!(stage.send(&PEvent::PointerMove(PPoint::new(25.0, 20.0))));
    assert!(stage.hand().contains(&PRect::new(0.0, 0.0, 50.0, 40.0)));
    // A modal's panel has the room under the caption, which stays clear.
    assert!(stage.window.apply(&PDeed::Open("any".to_string())));
    stage.lay(DESK, PMode::Desktop);
    assert_eq!(stage.panel().map(|panel| panel.whole), Some(PRect::new(0.0, 40.0, 800.0, 560.0)));
    // Under the system's bar the window is told nothing of the caption;
    // without one, it is told of its three buttons and where it is moved.
    assert!(stage.grip().is_empty());
    let bare = stage.bare(true);
    assert_eq!(bare.grip().iter().filter(|(_, grip)| *grip != PGrip::Drag).count(), 3);
    assert!(bare.grip().iter().any(|(_, grip)| *grip == PGrip::Drag));
}

#[test]
fn what_only_the_system_can_do_is_kept_for_the_screen_and_taken_once() {
    let wish = PWish::default();
    let mut stage = Barred(Note::new(), wish.clone(), LOOK).stage();
    stage.lay(DESK, PMode::Desktop);
    // A part wishes for a text to be copied, a file opened and one kept.
    let asked = [PDeed::Copy("a path".to_string()), PDeed::Launch("one.png".to_string()), PDeed::Save("one.png".to_string()), PDeed::Choose("post".to_string())];
    wish.borrow_mut().extend(asked.clone());
    // The window itself can do none of them.
    assert!(asked.iter().all(|deed| !stage.window.apply(deed)));
    // After the next event the stage has them, in order, until taken.
    stage.send(&PEvent::KeyDown('a'));
    assert_eq!(stage.errand(), asked);
    assert_eq!(stage.errand(), []);
    // What the system answers is told to the app as a word: here the
    // one a view takes, and one that none does.
    assert!(stage.tell("home") && !stage.tell("post C:/one.png"));
}
