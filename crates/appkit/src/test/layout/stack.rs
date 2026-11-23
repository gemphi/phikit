//! Groups stacked to fill a pane, tabs over them on one ground, the tab's
//! light moving, and swatches across a width.

use crate::pui::block::{PSwatch, PTabMove};
use crate::pui::core::*;
use crate::pui::layout::{PGrouped, stack, tabbed};
use crate::theme::PLook;

const LOOK: &str = "color\tshade.light\t#ffffff\nsize\tshade.share\t0.5\nsize\tshade.rest\t0.5\nsize\tshade.soft\t0.25\ncolor\ttext.ink\t#202020\nsize\ttext.line\t20\nsize\ttext.advance\t8\nsize\tgap\t16\nsize\ticon.size\t20\nsize\trow.radius\t10\nsize\tsection.side\t10\nsize\tsection.top\t10\nsize\tsection.radius\t20\nsize\tsection.space\t8\nsize\ttab.step\t0.5\nsize\tswatch.size\t30\nsize\tswatch.ring\t3\n";

/// Two groups, each a block so high, with a heading.
fn two<'a>() -> Vec<PGrouped<'a>> {
    let block = |high: f32| -> crate::pui::layout::PDrawn<'a> { Box::new(move |_: &mut dyn PPaint, at: PPoint, wide: f32| (at.y + high, vec![PRect::new(at.x, at.y, wide, high)])) };
    vec![PGrouped { head: "One".to_string(), draw: block(40.0) }, PGrouped { head: "Two".to_string(), draw: block(40.0) }]
}

#[test]
fn groups_share_out_the_room_a_pane_has_left_between_them() {
    let look = PLook::read(LOOK);
    let mut sheet = PSheet::new(&look);
    // Each is a heading (20 and half a gap) and 40, with 10 over and under.
    let (end, pressed) = stack(&mut sheet, PPoint::new(0.0, 0.0), 200.0, 0.0, &mut two());
    assert_eq!(end, (10.0 + 28.0 + 40.0 + 10.0) * 2.0 + 8.0);
    // Inside its ground: 10 in from its edge, under its heading.
    assert_eq!(pressed[0], [PRect::new(10.0, 38.0, 180.0, 40.0)]);
    // Given more room, the second group moves down to fill it.
    let (end, pressed) = stack(&mut PSheet::new(&look), PPoint::new(0.0, 0.0), 200.0, 400.0, &mut two());
    assert_eq!(end, 400.0);
    assert_eq!(pressed[1][0].origin.y, 400.0 - 10.0 - 40.0);
}

#[test]
fn tabs_stand_over_the_ground_of_what_they_choose_and_their_light_moves() {
    let look = PLook::read(LOOK);
    let mut sheet = PSheet::new(&look);
    let tab = [("", "Head", true), ("", "Body", false)];
    let motion = PTabMove::default();
    let (tabs, pressed) = tabbed(&mut sheet, PPoint::new(0.0, 0.0), 300.0, 0.0, tab.as_slice(), &motion, &mut two());
    assert_eq!(tabs.len(), 2);
    // The tabs stand above the ground that holds the groups, not on it:
    // it is as wide as given and begins under them, and the groups stand
    // inside it.
    let ground = sheet.mark.iter().filter_map(|mark| match mark {
        PMark::Blend { rect, .. } => Some(*rect),
        _ => None,
    }).find(|rect| rect.size.width == 300.0 && rect.origin.y > 0.0);
    let ground = ground.expect("a ground under the tabs");
    assert!(tabs.iter().all(|t| t.origin.y + t.size.height < ground.origin.y));
    assert!(pressed.iter().flatten().all(|r| r.origin.y > ground.origin.y && r.origin.y + r.size.height <= ground.origin.y + ground.size.height));
    // Another tab chosen: the light is on its way for two frames, then there.
    motion.to(1);
    assert_eq!(motion.now(), (0, 1, 0.0));
    assert!(motion.tick(&look) && motion.tick(&look));
    assert_eq!(motion.now(), (0, 1, 1.0));
    assert!(!motion.tick(&look));
}

#[test]
fn swatches_fill_a_row_shrinking_a_little_to_keep_to_one() {
    let look = PLook::read(LOOK);
    let mut sheet = PSheet::new(&look);
    let colour = vec![(PColor::new(1.0, 0.0, 0.0, 1.0), true); 8];
    // Eight at 30 with their rings need 8 * 36 and 7 * 3: 309; given 290,
    // they shrink to keep to one row.
    let (rect, end) = colour.as_slice().draw_in(&mut sheet, PPoint::new(0.0, 0.0), 290.0);
    assert!(rect.iter().all(|r| r.origin.y == rect[0].origin.y && r.size.width < 30.0 && r.size.width >= 22.5));
    assert!(rect.last().is_some_and(|r| r.origin.x + r.size.width <= 290.0) && end < 40.0);
}
