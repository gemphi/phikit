use super::activity::{LOOK, bar};
use crate::pui::core::*;
use crate::pui::window::part::{PGroup, PItem, PSeat, menu};
use crate::pui::window::*;
use crate::test::window::window;
use crate::theme::{PLook, PTheme};

/// The room a menu has in a bar's side 200 wide: under its head, 50.
const ROOM: PRect = PRect { origin: PPoint { x: 50.0, y: 50.0 }, size: PSize { width: 200.0, height: 550.0 } };

/// The menu of the row of that name.
fn group(row: &str) -> Vec<PGroup> {
    bar().item.into_iter().find(|item| item.entry.slug == row).map(|item| item.group).unwrap_or_default()
}

fn rect(row: &str) -> Vec<PRect> {
    let group = group(row);
    menu::lay(&group, ROOM, &PLook::read(LOOK)).iter().map(|laid| laid.rect).collect()
}

#[test]
fn a_seat_is_named_by_a_table_and_is_the_top_otherwise() {
    assert_eq!(["top", "center", "bottom", "", "left"].map(PSeat::named), [PSeat::Top, PSeat::Center, PSeat::Bottom, PSeat::Top, PSeat::Top]);
}

#[test]
fn a_row_holds_its_menu_in_groups_each_with_its_seat_its_heading_and_its_rows() {
    let layout = group("layout");
    let said: Vec<(&str, PSeat, usize, bool)> = layout.iter().map(|group| (group.slug.as_str(), group.seat, group.item.len(), group.side)).collect();
    assert_eq!(said, [("file", PSeat::Top, 2, false), ("list", PSeat::Center, 0, true), ("foot", PSeat::Bottom, 1, false)]);
    assert_eq!(layout[0].item[1], PItem { entry: PEntry::new("open", "Open", "folder"), deed: Some(PDeed::Show("shading".to_string())), ..PItem::default() });
    // A heading is not a row, and a row that names nothing to do makes
    // neither a row nor a group.
    let more = group("more");
    assert_eq!(more.iter().map(|group| (group.slug.as_str(), group.head.as_str(), group.item.len())).collect::<Vec<_>>(), [("look", "Look", 2)]);
}

#[test]
fn the_top_groups_stand_from_the_top_the_bottom_ones_end_at_the_foot_and_the_side_part_has_the_rest() {
    // Half a gap, 5, from the sides, from each other and from the foot.
    assert_eq!(rect("layout"), [PRect::new(55.0, 50.0, 190.0, 90.0), PRect::new(55.0, 145.0, 190.0, 395.0), PRect::new(55.0, 545.0, 190.0, 50.0)]);
    // The app's side part is inside its group, half a gap from its ends.
    assert_eq!(menu::room(&group("layout"), ROOM, &PLook::read(LOOK)), PRect::new(55.0, 150.0, 190.0, 385.0));
    assert_eq!(menu::room(&group("more"), ROOM, &PLook::read(LOOK)), PRect::default());
}

#[test]
fn a_group_in_the_middle_stands_round_the_middle_while_there_is_room() {
    // A heading and two rows, 130 high, round 325.
    assert_eq!(rect("more"), [PRect::new(55.0, 260.0, 190.0, 130.0)]);
    // In a room too low for that it starts at the top and goes no higher.
    let low = PRect::new(50.0, 50.0, 200.0, 100.0);
    assert_eq!(menu::lay(&group("more"), low, &PLook::read(LOOK))[0].rect.origin.y, 50.0);
}

#[test]
fn a_press_on_a_row_does_its_deed() {
    let spot = menu::spot(&group("layout"), ROOM, &PLook::read(LOOK));
    let tell = (PRect::new(60.0, 55.0, 180.0, 40.0), PDeed::Tell("begin".to_string()));
    let show = (PRect::new(60.0, 95.0, 180.0, 40.0), PDeed::Show("shading".to_string()));
    assert_eq!(spot, [tell, show, (PRect::new(60.0, 550.0, 180.0, 40.0), PDeed::Open("pref".to_string()))]);
    // Under a heading the rows start a row lower.
    assert_eq!(menu::spot(&group("more"), ROOM, &PLook::read(LOOK))[0].0, PRect::new(60.0, 305.0, 180.0, 40.0));
}

#[test]
fn every_group_has_a_ground_of_its_own_and_the_row_shown_is_lit_against_it() {
    let look = PLook::read(&format!("{LOOK}color\tshade.light\t#ffffff\nsize\tshade.share\t0.5\nsize\tshade.rest\t0.5\nsize\tactivity.radius\t12\n"));
    let lit = |window: &PWindow, row: &str| {
        let mut sheet = PSheet::new(&look);
        menu::draw(&group(row), ROOM, window, None, &mut sheet);
        let blend = |mark: &PMark| match mark {
            PMark::Blend { rect, radius, from, to, turn } => Some((*rect, *radius, from.a, to.a, *turn)),
            _ => None,
        };
        (sheet.mark.iter().filter_map(blend).collect::<Vec<_>>(), sheet.text_list().into_iter().map(str::to_string).collect::<Vec<_>>())
    };
    // The three groups, each lit from its right, against the bar's
    // icons beside them; the window shows the layout space, so no row
    // is what it shows.
    let ground = [(PRect::new(55.0, 50.0, 190.0, 90.0), 12.0, 0.0, 0.25, 90.0), (PRect::new(55.0, 145.0, 190.0, 395.0), 12.0, 0.0, 0.25, 90.0), (PRect::new(55.0, 545.0, 190.0, 50.0), 12.0, 0.0, 0.25, 90.0)];
    assert_eq!(lit(&window(), "layout"), (ground.to_vec(), vec!["New".to_string(), "Open".to_string(), "Preferences".to_string()]));
    // Showing the other space lights the row that goes there, the other
    // way: from its left.
    let mut shading = window();
    assert!(shading.show("shading"));
    assert_eq!(lit(&shading, "layout").0[1], (PRect::new(60.0, 95.0, 180.0, 40.0), 6.0, 0.25, 0.0, 90.0));
    // A heading stands over its rows, a gap in.
    assert_eq!(lit(&window(), "more").1, ["Look", "Light", "Dark"]);
    let mut sheet = PSheet::new(&look);
    menu::draw(&group("more"), ROOM, &window(), None, &mut sheet);
    assert!(sheet.mark.contains(&PMark::Text { at: PPoint::new(65.0, 275.0), text: "Look".to_string(), color: look.color("text.dim").unwrap_or_default() }));
    assert!(sheet.mark.contains(&PMark::Icon { name: "sun".to_string(), rect: PRect::new(65.0, 315.0, 20.0, 20.0), color: PColor::default() }));
    assert!(sheet.mark.contains(&PMark::Text { at: PPoint::new(90.0, 315.0), text: "Light".to_string(), color: PColor::default() }));
}
