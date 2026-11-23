mod draw;

use crate::pui::core::*;
use crate::pui::window::part::PSeat;
use crate::pui::window::*;
use crate::service::PTable;
use crate::test::window::window;
use crate::theme::PLook;

/// What an app would keep in two files: the bar's rows, and their menus.
const ITEM: &str = "# slug\tlabel\ticon\tseat\tkind\ttarget\nbook\tBook\t\thead\nlayout\tLayout\tgrid\ttop\tspace\tlayout\nshading\tShading\tpalette\ttop\tspace\tshading\nidle\tIdle\tbox\ttop\nmore\tMore\tbox\tcenter\nhelp\tHelp\tbook\tbottom\tmodal\thelp\n";
const MENU: &str = "# row\tseat\tgroup\tslug\tlabel\ticon\tkind\ttarget\nlayout\ttop\tfile\tnew\tNew\tplus\ttell\tbegin\nlayout\ttop\tfile\topen\tOpen\tfolder\tspace\tshading\nlayout\tcenter\tlist\tindex\tIndex\tlayers\tside\nlayout\tbottom\tfoot\tpref\tPreferences\tsettings\tmodal\tpref\nmore\tcenter\tlook\tlook\tLook\t\thead\nmore\tcenter\tlook\tlight\tLight\tsun\ttell\tlook light\nmore\tcenter\tlook\tdark\tDark\tmoon\ttell\tlook dark\nmore\ttop\todd\tbroken\tBroken\tbox\tdance\tnow\nnobody\ttop\tfile\tlost\tLost\tbox\tspace\tlayout\n";
pub(crate) const USER: &str = "name\tAda Lovelace\nmodal\tprofile\n";
pub(crate) const LOOK: &str = "size\ticon.size\t20\nsize\ttext.advance\t8\nsize\ttext.line\t20\nsize\tgap\t10\nsize\tactivity.row\t40\nsize\tactivity.width\t200\nsize\tactivity.low\t50\n";

pub(crate) const BAR: PRect = PRect { origin: PPoint { x: 0.0, y: 0.0 }, size: PSize { width: 50.0, height: 600.0 } };
pub(crate) const SIDE: PRect = PRect { origin: PPoint { x: 50.0, y: 0.0 }, size: PSize { width: 200.0, height: 600.0 } };

pub(crate) fn bar() -> PActivity {
    PActivity::read(ITEM).hold(MENU).with(&PTable::read(USER))
}

/// A window that has picked that row of the bar.
pub(crate) fn picked(row: &str) -> PWindow {
    let mut window = window();
    window.choose(row);
    window
}

#[test]
fn a_bar_is_read_from_a_table_a_row_a_line_with_its_seat_and_its_mark() {
    let bar = bar();
    let said: Vec<(&str, PSeat, bool, usize)> = bar.item.iter().map(|item| (item.entry.slug.as_str(), item.seat, item.deed.is_some(), item.group.len())).collect();
    assert_eq!(said, [("layout", PSeat::Top, true, 3), ("shading", PSeat::Top, true, 0), ("idle", PSeat::Top, false, 0), ("more", PSeat::Center, false, 1), ("help", PSeat::Bottom, true, 0)]);
    // The row seated at the head is the app's own mark, not a row.
    assert_eq!(bar.logo, PEntry::new("book", "Book", ""));
    assert_eq!(bar.user.as_ref().map(|user| (user.entry.label.as_str(), user.deed.clone())), Some(("Ada Lovelace", Some(PDeed::Open("profile".to_string())))));
    // No user without a name and a modal; nothing at all, no bar.
    assert!(PActivity::read(ITEM).with(&PTable::read("name\tAda\n")).user.is_none());
    assert!(PActivity::default().is_empty() && !bar.is_empty());
}

#[test]
fn it_is_as_thick_as_the_theme_says_and_a_menu_is_beside_it_only_while_one_is_shown() {
    let look = PLook::read(LOOK);
    assert_eq!([bar().thick(&look), PActivity::default().thick(&look)], [50.0, 0.0]);
    // A window picks the row whose own deed it shows, of those with a
    // menu: it shows the layout space.
    assert_eq!((bar().first(&window()), PActivity::read(ITEM).first(&window())), ("layout", ""));
    let mut window = picked("layout");
    assert_eq!([bar().wide(&window, false, &look), bar().wide(&window, true, &look)], [200.0, 0.0]);
    // Put away at a desk, there is none; drawn out in a hand, there is.
    assert!(window.apply(&PDeed::Fold) && window.apply(&PDeed::Side));
    assert_eq!([bar().wide(&window, false, &look), bar().wide(&window, true, &look)], [0.0, 200.0]);
    // A row with no menu shows none, picked or not.
    assert_eq!(bar().wide(&picked("shading"), false, &look), 0.0);
    assert_eq!((bar().deed("layout"), bar().deed("more")), (Some(PDeed::Show("layout".to_string())), None));
}

#[test]
fn standing_its_mark_is_at_the_head_its_rows_at_the_top_the_middle_and_the_foot_and_the_user_last() {
    let spot = bar().spot(BAR, PRect::default(), &window(), &PLook::read(LOOK));
    let head = (PRect::new(0.0, 0.0, 50.0, 50.0), PDeed::Fold);
    // A row with a menu shows it, one with none does its own deed, and
    // the one with neither is not there.
    let top = [(PRect::new(0.0, 50.0, 50.0, 40.0), PDeed::Pick("layout".to_string())), (PRect::new(0.0, 90.0, 50.0, 40.0), PDeed::Show("shading".to_string()))];
    // The middle row is round the bar's middle, 300.
    let middle = (PRect::new(0.0, 280.0, 50.0, 40.0), PDeed::Pick("more".to_string()));
    let foot = [(PRect::new(0.0, 510.0, 50.0, 40.0), PDeed::Open("help".to_string())), (PRect::new(0.0, 550.0, 50.0, 50.0), PDeed::Open("profile".to_string()))];
    assert_eq!(spot, [head, top[0].clone(), top[1].clone(), middle, foot[0].clone(), foot[1].clone()]);
    // In a bar too low for that the middle row follows the top ones.
    let low = bar().spot(PRect::new(0.0, 0.0, 50.0, 240.0), PRect::default(), &window(), &PLook::read(LOOK));
    assert_eq!(low[3].0, PRect::new(0.0, 130.0, 50.0, 40.0));
}

#[test]
fn a_press_on_the_row_whose_menu_is_shown_puts_it_away_and_the_menu_has_a_button_that_does() {
    let spot = bar().spot(BAR, SIDE, &picked("layout"), &PLook::read(LOOK));
    assert_eq!(spot[1], (PRect::new(0.0, 50.0, 50.0, 40.0), PDeed::Fold));
    // After the bar's own: the button at the right of the menu's head,
    // and the menu's rows.
    assert_eq!(spot[6], (PRect::new(200.0, 0.0, 50.0, 50.0), PDeed::Fold));
    assert_eq!(spot[7..].iter().map(|(_, deed)| deed.clone()).collect::<Vec<_>>(), [PDeed::Tell("begin".to_string()), PDeed::Show("shading".to_string()), PDeed::Open("pref".to_string())]);
    // The app's side part has the room of the group that holds it.
    assert_eq!(bar().room(SIDE, &picked("layout"), false, &PLook::read(LOOK)), PRect::new(55.0, 150.0, 190.0, 385.0));
    assert_eq!(bar().room(SIDE, &picked("more"), false, &PLook::read(LOOK)), PRect::default());
}

#[test]
fn lying_it_is_squares_in_the_order_of_their_seats_with_the_user_at_the_end() {
    let look = PLook::read(LOOK);
    let (lying, over) = (PRect::new(0.0, 550.0, 400.0, 50.0), PRect::new(0.0, 0.0, 400.0, 550.0));
    let mut window = picked("layout");
    let rect = |spot: &[(PRect, PDeed)]| spot.iter().map(|(rect, _)| *rect).collect::<Vec<_>>();
    // No head: the mark is not shown. The menu starts put away.
    let spot = bar().spot(lying, PRect::default(), &window, &look);
    assert_eq!(rect(&spot), [PRect::new(0.0, 550.0, 50.0, 50.0), PRect::new(50.0, 550.0, 50.0, 50.0), PRect::new(100.0, 550.0, 50.0, 50.0), PRect::new(150.0, 550.0, 50.0, 50.0), PRect::new(350.0, 550.0, 50.0, 50.0)]);
    assert_eq!(spot[0].1, PDeed::Pick("layout".to_string()));
    // Drawn out, it lies over the space, and the same row puts it away.
    assert!(window.apply(&PDeed::Pick("layout".to_string())) && window.drawer());
    let spot = bar().spot(lying, over, &window, &look);
    assert_eq!((&spot[0].1, &spot[5]), (&PDeed::Side, &(PRect::new(350.0, 0.0, 50.0, 50.0), PDeed::Side)));
    assert_eq!(bar().room(over, &window, true, &look), PRect::new(5.0, 150.0, 390.0, 335.0));
}
