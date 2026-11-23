//! What a stage says of itself, as it was last laid out: to the screen
//! that shows it, and to a test.

use std::rc::Rc;

use super::PStage;
use crate::pui::core::light::{label, surface};
use crate::pui::core::{PEvent, PLight, PPaint, PPoint, PRect, show};
use crate::pui::window::PModal;
use crate::pui::window::{PDeed, PFrame, PGrip, PLevel, PPanel, PSlot};
use crate::theme::PTheme;

/// The theme's tokens for the ground of a part of the body: how far
/// inside the part it stands, and how round it is.
/// The region of a host that is over its page, as an app's data names
/// it.
pub const HEADER: &str = "header";
pub const INSET: &str = "part.inset";
pub const RADIUS: &str = "part.radius";

/// The theme's token for how much of its way a modal slides in a frame.
/// A theme without it has modals stand where they are at once.
pub const STEP: &str = "modal.step";

/// How the window's modals slide: the one that is open comes in from
/// the window's edge, and the one that was closed goes out the same
/// way, each with its veil. Only a window on a screen slides; a stage
/// that is only laid out and asked has everything in its place.
#[derive(Debug, Clone, Default)]
pub struct PSlide {
    /// Whether anything slides at all.
    lively: bool,
    /// The modal that is open, and how much of its way is still to come.
    name: Option<String>,
    out: f32,
    /// The modal that is going, where it stood, and how far out it is.
    gone: Option<(String, PPanel, f32)>,
    /// What each view that watches something saw when last asked.
    seen: Vec<Option<u64>>,
}

/// Of the places a point is in, the smallest: a button on a row, not
/// the row.
fn least(list: &[PRect], at: PPoint) -> Option<usize> {
    let area = |rect: &PRect| rect.size.width * rect.size.height;
    list.iter().enumerate().filter(|(_, rect)| rect.contains(at)).min_by(|one, other| area(one.1).total_cmp(&area(other.1))).map(|(index, _)| index)
}

impl PStage {
    /// Let every view take in what has arrived since the last frame,
    /// and ask the app again for the rows it adds to the bar: when they
    /// changed, the window is laid out again. `true` while there is
    /// more to come, or something changed.
    pub fn tick(&mut self) -> bool {
        let busy = self.content.iter_mut().fold(false, |busy, (_, view)| view.tick() || busy);
        let turned = self.fresh();
        if turned {
            self.lay(self.size, self.mode);
        }
        // What slides goes a step further.
        let step = self.theme.size(STEP).unwrap_or(1.0);
        let moving = self.slide.out > 0.0 || self.slide.gone.is_some();
        self.slide.out = (self.slide.out - step).max(0.0);
        self.slide.gone = self.slide.gone.take().map(|(name, panel, out)| (name, panel, out + step)).filter(|(.., out)| *out < 1.0);
        busy || turned || moving
    }

    /// Whether any view watches something outside it (`PView::beat`).
    pub fn beat(&self) -> bool {
        self.content.iter().any(|(_, view)| view.beat().is_some())
    }

    /// Ask every view what it watches, and the app for the rows of its
    /// bar. `true` when anything is other than the last time this was
    /// asked: the window is then to be drawn again.
    pub fn turned(&mut self) -> bool {
        let seen: Vec<Option<u64>> = self.content.iter().map(|(_, view)| view.beat()).collect();
        let rail = self.fresh();
        rail.then(|| self.lay(self.size, self.mode));
        std::mem::replace(&mut self.slide.seen, seen.clone()) != seen || rail
    }

    /// Ask the app for the rows it adds to the bar. `true` when they
    /// are not the ones it gave last.
    pub(super) fn fresh(&mut self) -> bool {
        // Another modal than before: the one that was open goes out
        // from where it is, and the one that is open now comes in.
        let now = self.window.modal().map(str::to_string);
        if self.slide.lively && self.theme.size(STEP).is_some() && now != self.slide.name {
            let was = self.slide.name.take().zip(self.panel);
            self.slide.gone = was.map(|(name, panel)| (name, panel, self.slide.out));
            self.slide.out = now.as_ref().map_or(0.0, |_| 1.0);
        }
        self.slide.name = now;
        let live = self.rail.as_ref().map(|rail| rail()).unwrap_or_default();
        let turned = live != self.activity.live;
        self.activity.live = live;
        turned
    }

    /// Keep where the pointer is: over the caption's mark, which then
    /// shows the pin, and over which of the places a press does
    /// something, which is then lit. `true` when either changed.
    pub(super) fn hover(&mut self, event: &PEvent) -> bool {
        let PEvent::PointerMove(at) = *event else { return false };
        let over = self.caption.mark(&self.frame, &self.activity, self.theme.as_ref()).contains(at);
        // Over the window's own furniture, and over what a part says
        // can be pressed.
        let hand = self.drawn();
        let spot = |point: Option<PPoint>| point.map(|point| (self.spot.iter().position(|(rect, _)| rect.contains(point)), least(&hand, point)));
        let turned = spot(self.over).unwrap_or_default() != spot(Some(at)).unwrap_or_default();
        self.over = Some(at);
        std::mem::replace(&mut self.caption.over, over) != over || turned
    }

    /// The wheel over the bar rolls the rows the app added to it, a row
    /// a notch, and the window is laid out again. `true` when it did.
    pub(super) fn roll(&mut self, event: &PEvent) -> bool {
        let PEvent::Scroll(at, by) = *event else { return false };
        let by = match by.y < 0.0 {
            true => 1,
            false => -1,
        };
        let turned = self.frame.activity.contains(at) && self.window.modal().is_none() && self.activity.roll(by, self.frame.activity, self.theme.as_ref());
        if turned {
            self.lay(self.size, self.mode);
        }
        turned
    }

    /// What can be pressed as it was last drawn; asked of the parts
    /// again where the window was laid out since.
    fn drawn(&self) -> Vec<PRect> {
        let kept = self.drawn.borrow().clone();
        match kept.is_empty() {
            true => self.pressed(),
            false => kept,
        }
    }

    /// What the parts shown say can be pressed. With a modal open, only
    /// what the modal's content says.
    pub(super) fn pressed(&self) -> Vec<PRect> {
        let part = |name: &str| self.content.iter().find(|(known, _)| known == name).map(|(_, view)| view.hand()).unwrap_or_default();
        match self.window.modal() {
            Some(modal) => part(modal),
            None => self.slot.iter().flat_map(|slot| part(&slot.part)).collect(),
        }
    }

    /// Where the pointer is a hand: every place a press on the window's
    /// own furniture does something, and what each part shown says of
    /// itself.
    pub fn hand(&self) -> Vec<PRect> {
        let own = self.spot.iter().map(|(rect, _)| *rect).filter(|_| self.window.modal().is_none());
        own.chain(self.drawn()).collect()
    }

    /// Keep where the button is held down, and let go of it when it is
    /// let go. `true` when letting go changed what is lit: it was held
    /// down on something a part says can be pressed. A press is drawn
    /// again whatever it met, and says nothing here.
    pub(super) fn held(&mut self, event: &PEvent) -> bool {
        let next = match *event {
            PEvent::PointerDown(at) => Some(at),
            PEvent::PointerUp(_) => None,
            _ => return false,
        };
        let hand = self.drawn();
        let lit = self.down.is_some_and(|point| least(&hand, point).is_some());
        self.down = next;
        next.is_none() && lit
    }

    /// Light what the pointer is over, of the things a part says can be
    /// pressed: softly, and at full strength while the button is held
    /// down on it. It is done here once, for every part of every app:
    /// a part only says where its places to press are. Of the places
    /// the pointer is in, the smallest: a button on a row, not the row.
    pub(super) fn glow(&self, paint: &mut dyn PPaint) {
        let hand = self.drawn();
        let Some(rect) = self.over.and_then(|at| least(&hand, at)).and_then(|index| hand.get(index)) else { return };
        let light = match self.down.is_some_and(|down| rect.contains(down)) {
            true => PLight::Strong,
            false => PLight::Soft,
        };
        let radius = self.theme.size(RADIUS).unwrap_or_default().min(rect.size.height / 2.0);
        label(paint, *rect, radius, light, false, "");
    }

    /// The ground of a part: every part of the body stands on one,
    /// inset and rounded as the theme says (`part.inset`,
    /// `part.radius`). A part inside the bar's menu stands on its
    /// group's ground already, and a theme with no `part.inset` has none.
    ///
    /// The light is at its foot and gone at its head. A part in the
    /// header (`region`) stands in the light of rest, so that what is
    /// over a page is seen to be over it; every other in the faintest.
    pub(super) fn ground(&self, rect: PRect, region: &str, paint: &mut dyn PPaint) {
        let inset = self.theme.size(INSET);
        let body = self.frame.body;
        let light = match region == HEADER {
            true => PLight::Rest,
            false => PLight::Soft,
        };
        if let (Some(inset), true) = (inset, body.contains(rect.center())) {
            surface(paint, rect.inset(inset), self.theme.size(RADIUS).unwrap_or_default(), light, "");
        }
    }

    /// The same stage for a window on a screen: its modals slide in and
    /// out, where the theme says how fast.
    pub fn lively(mut self) -> PStage {
        self.slide.lively = true;
        self
    }

    /// Draw the modal that is going and the one that is open, each
    /// where it is on its way, with its veil as far there.
    pub(super) fn sheet(&self, paint: &mut dyn PPaint) {
        let open = self.panel.zip(self.window.modal()).map(|(panel, name)| (name, panel, self.slide.out));
        let gone = self.slide.gone.as_ref().map(|(name, panel, out)| (name.as_str(), *panel, *out));
        for (name, panel, out) in gone.into_iter().chain(open) {
            let (moved, by) = panel.slid(out);
            moved.shown(self.modal.find(name).map_or(name, PModal::label), 1.0 - out, paint);
            self.view(name).into_iter().for_each(|view| show(view, paint, by));
        }
    }

    /// The same stage for a window with no bar of the system's: its
    /// caption is then what the window is moved by, and carries its
    /// buttons.
    pub fn bare(mut self, bare: bool) -> PStage {
        self.caption.bare = bare;
        self
    }

    /// The theme everything on the stage is drawn in.
    pub fn theme(&self) -> &Rc<dyn PTheme> {
        &self.theme
    }

    /// Where each part is shown.
    pub fn slot(&self) -> &[PSlot] {
        &self.slot
    }

    pub fn frame(&self) -> PFrame {
        self.frame
    }

    /// The panel of the modal that is open.
    pub fn panel(&self) -> Option<PPanel> {
        self.panel
    }

    /// What was asked since this was last taken that only the system
    /// can do: to copy a text, to open a file in the system's own
    /// program, to keep a copy of one where the person says. The screen
    /// takes them after each event, and does them.
    pub fn errand(&mut self) -> Vec<PDeed> {
        std::mem::take(&mut self.errand)
    }

    /// Tell the app a word from outside the window: what the system
    /// answered an errand with. `true` when a view took it.
    pub fn tell(&mut self, word: &str) -> bool {
        self.act(&PDeed::Tell(word.to_string()))
    }

    /// The part that has the keys.
    pub fn focus(&self) -> Option<&str> {
        self.focus.as_deref()
    }

    /// What the system's window is told of the caption: where it is
    /// moved by, and its buttons. Nothing where the system has its own
    /// bar.
    pub fn grip(&self) -> Vec<(PRect, PGrip)> {
        self.caption.grip(&self.frame, &self.activity, self.theme.as_ref())
    }
}
