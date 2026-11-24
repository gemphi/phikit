//! Drawing the activity bar, and the menu it shows.

use super::super::menu::{self, INK, LIT, ROUND, ROW};
use super::super::{GAP, ICON, INK as TEXT, ROW as LINE, person};
use super::{CORNER, EDGE, FACE, FILL, PActivity, PIN, RIM, SHARE, share, square};
use crate::pui::core::letter::draw_letter;
use crate::pui::core::light::{bond, edge, label, pane};
use crate::pui::core::{PFit, PLight, PPaint, PPoint, PRect};
use crate::pui::window::PWindow;

impl PActivity {
    /// Draw the bar, and the menu shown in `side`. Standing, the two are
    /// one pane at the window's edge, its corners round; every icon sits in a chip lit
    /// against it, the strongest for the row whose menu or whose own
    /// deed is shown, and stronger than at rest for the one the pointer
    /// (`over`) is on. The menu has the app's name over its groups.
    /// Under a caption the mark, the name and the button are the
    /// caption's.
    pub fn draw(&self, bar: PRect, side: PRect, window: &PWindow, over: Option<PPoint>, paint: &mut dyn PPaint) {
        let plan = self.plan(bar, paint.theme());
        let shown = self.shown(window, plan.lying);
        let headed = PActivity::headed(paint.theme());
        let (ink, text) = (paint.color(INK), paint.color(TEXT));
        let (icon, round, line, gap) = (paint.size(ICON), paint.size(ROUND), paint.size(LINE), paint.size(GAP));
        let (chip, corner) = (paint.size(ROW) - gap / 4.0, paint.size(CORNER));
        let name = match self.logo.label.is_empty() {
            true => window.title.as_str(),
            false => self.logo.label.as_str(),
        };
        match (plan.lying, shown.is_some()) {
            (true, true) => [bar, side].into_iter().for_each(|rect| pane(paint, rect, corner, FILL)),
            (true, false) => pane(paint, bar, corner, FILL),
            // Standing, the bar and its menu are one pane of two parts,
            // their lights running against each other where they meet.
            (false, true) => bond(paint, bar, side, corner, FILL),
            (false, false) => edge(paint, bar, corner, false, FILL),
        }
        // A plain look's faint line down the side toward the page.
        let last = match shown.is_some() {
            true => side,
            false => bar,
        };
        let rim = paint.theme().size(RIM).unwrap_or(1.0);
        if let Some(colour) = paint.theme().color(EDGE).filter(|colour| colour.a > 0.0 && !plan.lying) {
            paint.fill(PRect::new(last.origin.x + last.size.width - rim, last.origin.y + corner, rim, (last.size.height - corner * 2.0).max(0.0)), colour);
        }
        match (plan.lying || !headed, self.logo.icon.is_empty()) {
            (true, _) => {}
            (false, false) => paint.icon(&self.logo.icon, square(plan.head, chip), ink),
            (false, true) => {
                // An app with no mark of its own has its name's letter,
                // drawn and standing as every icon does: no ground of its own.
                { let mark = square(plan.head, share(paint, SHARE, chip, icon)); draw_letter(paint, name, mark, ink) };
            }
        }
        for (rect, item) in &plan.item {
            let on = item.on || shown.is_some_and(|shown| shown.entry.slug == item.entry.slug) || item.deed.as_ref().is_some_and(|deed| window.shows(deed));
            match (on, over.is_some_and(|at| rect.contains(at))) {
                (true, _) => label(paint, square(*rect, chip), round, PLight::Strong, false, LIT),
                // At rest too a row stands on a ground that is seen:
                // an icon without one looks as if it were missing.
                (false, true) => label(paint, square(*rect, chip), round, PLight::Strong, false, ""),
                (false, false) => label(paint, square(*rect, chip), round, PLight::Rest, false, ""),
            }
            // A row with a picture has it round, on its ground; one
            // with neither picture nor icon the letter its name begins
            // with, drawn as every icon is; any other its icon.
            let face = square(*rect, share(paint, FACE, chip, icon * 7.0 / 5.0));
            match (&item.face, item.entry.icon.is_empty()) {
                (Some((sheet, column, index)), _) => paint.picture(sheet, face, face.size.width / 2.0, PFit::Cell { column: *column, index: *index }),
                (None, true) => { let mark = square(*rect, share(paint, SHARE, chip, icon)); draw_letter(paint, &item.entry.label, mark, ink) },
                (None, false) => paint.icon(&item.entry.icon, square(*rect, share(paint, SHARE, chip, icon)), ink),
            }
        }
        // The app's own rows and the places the person is in are told
        // apart, as a list of servers is: a short line over the first place.
        let first_place = plan.item.iter().position(|(_, item)| self.live.iter().any(|live| std::ptr::eq(live, *item)));
        if let (Some(at), Some(colour)) = (first_place.filter(|at| *at > 0 && !plan.lying), paint.theme().color("line.ink")) {
            let rect = plan.item[at].0;
            let wide = rect.size.width / 2.0;
            paint.fill(PRect::new(rect.center().x - wide / 2.0, rect.origin.y - rim / 2.0, wide, rim), colour);
        }
        if let Some(user) = &self.user {
            // The user is their own picture, as large as a row's; with
            // none, their mark, as large as the ground of an icon.
            let side = user.face.as_ref().map_or(chip, |_| share(paint, FACE, chip, icon * 7.0 / 5.0));
            person(user.face.as_ref(), &user.entry.label, square(plan.user, side), paint);
        }
        if let Some(shown) = shown {
            let (pin, room) = PActivity::part(side, paint.theme());
            if headed {
                paint.text(PPoint::new(side.origin.x + gap, pin.center().y - line / 2.0), name, text);
                paint.icon(PIN, square(pin, icon), ink);
            }
            menu::draw(&shown.group, room, window, over, paint);
        }
    }
}
