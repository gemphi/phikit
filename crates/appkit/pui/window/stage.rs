//! A stage: an app's window with the content of its parts built, laid out
//! for a size and a mode. It is what a screen shows and sends events to:
//! appkit lays out, draws its own furniture (the caption, the activity
//! bar, the tabs, an open modal's panel) and routes; the app's parts draw
//! and answer. `stage/laid.rs` is what it says of itself.

mod laid;

use std::rc::Rc;

use super::{PActivity, PBar, PCaption, PDeed, PFrame, PHost, PLevel, PModal, PPanel, PProfile, PRail, PSet, PSlot, PTab, PWindow, PWish};
use crate::pui::PApp;
use crate::pui::core::{PEvent, PMode, PPaint, PPoint, PRect, PSize, PView, send, show};
use crate::theme::PTheme;

/// The theme's token the window is filled with behind its parts.
pub const FILL: &str = "window.fill";

/// An app's window with the content of its parts built.
pub struct PStage {
    pub window: PWindow,
    theme: Rc<dyn PTheme>,
    host: PSet<PHost>,
    modal: PSet<PModal>,
    /// The view of each part and each modal that has one, by its name. A
    /// part shown in two areas is one view, drawn where it was laid last.
    content: Vec<(String, Box<dyn PView>)>,
    activity: PActivity,
    /// What says which rows the app adds to the bar while it runs.
    rail: Option<PRail>,
    caption: PCaption,
    tab: PTab,
    /// Whether the app wants its spaces shown as tabs.
    tabbed: bool,
    /// The host the app puts in front on a mobile window.
    front: String,
    /// The host the app shows in a menu of the activity bar.
    side: String,
    /// Where the app's parts ask for deeds.
    wish: Option<PWish>,
    /// What was asked that only the system can do, until it is taken.
    errand: Vec<PDeed>,
    frame: PFrame,
    slot: Vec<PSlot>,
    /// Where a press on the window's own furniture does something, and
    /// where the pointer is.
    spot: Vec<(PRect, PDeed)>,
    over: Option<PPoint>,
    /// Where the button is held down, while it is; and how the modal
    /// that is open, and the one that is going, slide.
    down: Option<PPoint>,
    slide: laid::PSlide,
    /// The panel of the modal that is open.
    panel: Option<PPanel>,
    size: PSize,
    mode: PMode,
    /// The part the app says takes the keys.
    home: Option<String>,
    /// The part that has the keys now: the last one pressed.
    focus: Option<String>,
    /// Where the parts said a press does something when last drawn.
    drawn: std::cell::RefCell<Vec<PRect>>,
}

impl PStage {
    /// The stage of an app: its window, theme, bar and user, the content
    /// of each of its parts and modals, and the part it says takes the
    /// keys. The modal the user's table names is appkit's own profile.
    pub fn of(app: &dyn PApp) -> PStage {
        let user = app.user();
        let part = app.part().list().iter().map(|part| part.slug().to_string());
        let modal = app.modal().list().iter().map(|modal| modal.slug().to_string());
        let built = |name: String| {
            let view: Option<Box<dyn PView>> = match name == user.get("modal") {
                true => Some(Box::new(PProfile::new(user.clone()).live(app.fact()).dressed(app.shade(), app.wish()))),
                false => app.content(&name),
            };
            Some((name, view?))
        };
        let content = part.chain(modal).filter_map(built).collect();
        let home = Some(app.focus().to_string()).filter(|part| !part.is_empty());
        let (frame, size, focus) = (PFrame::default(), PSize::default(), home.clone());
        let mut stage = PStage { window: app.open(), theme: app.theme(), host: app.host().clone(), modal: app.modal().clone(), content, activity: app.activity().with(&user), rail: app.rail(), caption: PCaption::default(), tab: PTab, tabbed: app.tab(), front: app.front().to_string(), side: app.side().to_string(), wish: app.wish(), errand: Vec::new(), frame, slot: Vec::new(), spot: Vec::new(), over: None, down: None, slide: laid::PSlide::default(), panel: None, size, mode: PMode::Desktop, home, focus, drawn: std::cell::RefCell::default() };
        let pick = stage.activity.first(&stage.window).to_string();
        stage.window.choose(&pick);
        stage.face();
        stage
    }

    /// Bring the host the app names in front, where the space shown has it.
    fn face(&mut self) {
        if let Some(index) = self.window.index(&self.front) {
            self.window.bring(index);
        }
    }

    /// Lay the window out for a size and a mode: its frame, the parts of
    /// the space shown, the open modal. Every view is given its rectangle.
    pub fn lay(&mut self, size: PSize, mode: PMode) {
        let theme = Rc::clone(&self.theme);
        let tabbed = self.tabbed && self.window.space.len() > 1;
        (self.size, self.mode) = (size, mode);
        // What was drawn is let go: laid out anew, the parts are asked.
        self.drawn.borrow_mut().clear();
        self.fresh();
        let lying = mode == PMode::Mobile;
        let (thick, menu) = (self.activity.thick(theme.as_ref()), self.activity.wide(&self.window, lying, theme.as_ref()));
        self.frame = PFrame::of(size, mode, theme.as_ref(), thick, menu, tabbed);
        let body = self.frame.body;
        let place = self.window.place(body.size, mode);
        let laid = place.iter().filter_map(|place| Some(self.host.find(&place.host)?.lay(place.rect.moved(body.origin), theme.as_ref())));
        self.slot = laid.flatten().collect();
        // The side host in the room the bar's menu leaves for it. In a
        // hand a menu lies over the space, whose parts are not shown.
        let room = self.activity.room(self.frame.menu, &self.window, lying, theme.as_ref());
        let beside = match room.size.height > 0.0 {
            true => self.host.find(&self.side).map(|host| host.lay(room, theme.as_ref())).unwrap_or_default(),
            false => Vec::new(),
        };
        match lying && menu > 0.0 {
            true => self.slot = beside,
            false => self.slot.extend(beside),
        }
        self.spot = self.activity.spot(self.frame.activity, self.frame.menu, &self.window, theme.as_ref());
        self.spot.extend(self.tab.spot(self.frame.tab, &self.window, theme.as_ref()).into_iter().filter(|_| tabbed));
        self.spot.extend(self.caption.spot(&self.frame, &self.activity, theme.as_ref()));
        // A modal's panel is laid in the room under the caption.
        let (top, under) = (self.frame.caption.size.height, PSize { width: size.width, height: size.height - self.frame.caption.size.height });
        let known = |name: &str| self.modal.find(name).map(|modal| (modal.shape, modal.bare)).unwrap_or_default();
        self.panel = self.window.modal().map(|name| PPanel::of(under, known(name).0, self.window.wide(), lying, theme.as_ref()).bared(known(name).1).lower(top));
        let over = self.panel.iter().filter_map(|panel| Some((self.window.modal()?.to_string(), panel.body)));
        let frame: Vec<(String, PRect)> = self.slot.iter().map(|slot| (slot.part.clone(), slot.rect)).chain(over).collect();
        for (name, rect) in frame {
            let view = self.content.iter_mut().find(|(known, _)| *known == name);
            view.into_iter().for_each(|(_, view)| view.set_frame(rect));
        }
    }

    /// Draw the window: its ground, the tabs, the bar and the caption,
    /// every part that is shown, and over them the modal that is open.
    pub fn draw(&self, paint: &mut dyn PPaint) {
        let ground = paint.color(FILL);
        paint.fill(PRect { origin: PPoint::default(), size: self.size }, ground);
        if self.tabbed && self.window.space.len() > 1 {
            self.tab.draw(self.frame.tab, &self.window, paint);
        }
        if !self.activity.is_empty() {
            self.activity.draw(self.frame.activity, self.frame.menu, &self.window, self.over, paint);
        }
        self.caption.draw(&self.frame, &self.activity, &self.window, paint);
        // A part draws inside its own rectangle and nowhere else, on a
        // ground of its own where it stands in the body.
        for (slot, view) in self.slot.iter().filter_map(|slot| Some((slot, self.view(&slot.part)?))) {
            self.ground(slot.rect, &slot.region, paint);
            paint.clip(slot.rect);
            show(view, paint, PPoint::default());
            paint.unclip();
        }
        self.sheet(paint);
        // What can be pressed is what was just drawn: kept, so that a
        // move of the pointer need not ask every part again.
        *self.drawn.borrow_mut() = self.pressed();
        self.glow(paint);
    }

    /// The view of a part or a modal, if it shows something.
    pub(super) fn view(&self, name: &str) -> Option<&dyn PView> {
        self.content.iter().find(|(known, _)| known == name).map(|(_, view)| view.as_ref())
    }

    /// Send an event. A press on the bar, a tab or an open modal's panel
    /// does its deed. While a modal is open everything else is its
    /// content's, and Escape closes it. Otherwise an event goes to the
    /// parts (`pass`). `true` when it was handled.
    pub fn send(&mut self, event: &PEvent) -> bool {
        let deed = match (event, self.panel) {
            (PEvent::PointerDown(at), Some(panel)) => panel.deed(*at),
            (PEvent::PointerDown(at), None) => self.spot.iter().find(|(rect, _)| rect.contains(*at)).map(|(_, deed)| deed.clone()),
            (PEvent::KeyDown(PEvent::ESCAPE), Some(_)) => Some(PDeed::Close),
            _ => None,
        };
        let handled = match (deed, self.window.modal().map(str::to_string)) {
            (Some(deed), _) => self.act(&deed),
            (None, Some(modal)) => self.give(&modal, event),
            (None, None) => self.pass(event),
        };
        self.grant();
        let held = self.held(event);
        self.hover(event) || self.roll(event) || held || handled
    }

    /// Do what the app's parts wished for while they answered.
    fn grant(&mut self) {
        let wished: Vec<PDeed> = self.wish.iter().flat_map(|wish| wish.borrow_mut().split_off(0)).collect();
        wished.iter().for_each(|deed| _ = self.act(deed));
    }

    /// Do a deed and lay the window out again. A word is the app's: the
    /// first of its views that takes it does it. Picking a row of the bar
    /// does that row's own deed and what it wishes for before its menu is
    /// shown. A word or a modal puts away a menu drawn out in a hand.
    fn act(&mut self, deed: &PDeed) -> bool {
        let done = match deed {
            PDeed::Tell(word) => self.content.iter_mut().any(|(_, view)| view.tell(word)),
            // What only the system can do is kept for the screen.
            PDeed::Copy(_) | PDeed::Launch(_) | PDeed::Save(_) | PDeed::Choose(_) => {
                self.errand.push(deed.clone());
                true
            }
            PDeed::Pick(row) => {
                self.activity.deed(row).iter().for_each(|own| _ = self.act(own));
                self.grant();
                self.window.apply(deed)
            }
            other => self.window.apply(other),
        };
        if let (PDeed::Tell(_) | PDeed::Open(_), true) = (deed, self.window.drawer()) {
            self.window.apply(&PDeed::Side);
        }
        if let PDeed::Show(_) = deed {
            self.face();
        }
        self.lay(self.size, self.mode);
        done
    }

    /// Send an event to the parts. One with a place goes to the part
    /// under it, and a press there gives that part the keys. A key goes
    /// to the part that has the keys, then the one the app named, then
    /// the others, each only if it is shown, until one takes it. Before
    /// the window was laid out none is shown, and the first two are asked.
    fn pass(&mut self, event: &PEvent) -> bool {
        let Some(at) = event.at() else {
            let named = [self.focus.clone(), self.home.clone()].into_iter().flatten();
            let shown = named.chain(self.slot.iter().map(|slot| slot.part.clone())).filter(|part| self.slot.is_empty() || self.slot.iter().any(|slot| slot.part == *part));
            let mut order: Vec<String> = Vec::new();
            shown.for_each(|part| if !order.contains(&part) { order.push(part) });
            return order.into_iter().any(|part| self.give(&part, event));
        };
        let under = self.slot.iter().rev().find(|slot| slot.rect.contains(at)).map(|slot| slot.part.clone());
        if let (PEvent::PointerDown(_), Some(part)) = (event, &under) {
            self.focus = Some(part.clone());
        }
        under.is_some_and(|part| self.give(&part, event))
    }

    fn give(&mut self, name: &str, event: &PEvent) -> bool {
        let view = self.content.iter_mut().find(|(known, _)| known == name);
        view.is_some_and(|(_, view)| send(view.as_mut(), event))
    }
}
