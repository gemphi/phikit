//! The active object's action and slot, as Blender draws them in three
//! places — the Action Editor's header (`DOPESHEET_HT_editor_buttons.
//! _draw_action_selector`), Properties ▸ Object ▸ Animation
//! (`draw_action_and_slot_selector_for_id`) — and the Dope Sheet sidebar's
//! Action and Slot panels (`DOPESHEET_PT_action`, `DOPESHEET_PT_action_slot`).
//!
//! `template_action`: a browse menu of every action in the file with the
//! one in use ticked, New (a copy of it, or an empty action) and Unlink;
//! then the slot's `template_search`: a menu of the slots that can animate
//! an object. Each item is a `ctx.op.action` value (`scene::action::rule`).

use crate::phiphx::scene::{SceneAction, SceneModel};
use crate::v2::host::BHostType;
use crate::v2::part::modebar::active_mode;
use crate::v2::part::props::BPanelItem;
use crate::v2::scene::action::BSceneActions;
use crate::v2::state::val::BValue;
use crate::v2::state::{BKey as _, BState, CtxKey, MenuKey, OpKey};
use crate::v2::view::prim::{BAction, BCard, BControl, BCtrl, BCtrlKind, BPick};
use crate::v2::view::{BView, BViewItem};

/// The Action Editor's mode, where the header draws the selector.
pub const ACTION_EDITOR: &str = "Action Editor";
/// A frame field's range (`rna_action.cc`: `MINAFRAMEF` … `MAXFRAMEF`).
const FRAMES: (f32, f32) = (-1_048_574.0, 1_048_574.0);

/// The selector over `host`'s active object.
#[derive(Clone, Copy, Debug)]
pub struct BActionTemplate {
    pub host: BHostType,
}

/// What the selector shows: every action, and the one in use with its slot.
///
/// Not the object: `now` needs its name to ask which action it has, but
/// nothing reads it afterwards, and a field kept for the look of the struct
/// is a field that goes stale.
struct BActionView {
    names: Vec<String>,
    current: Option<(SceneAction, String)>,
}

impl BActionView {
    fn now(s: &BState) -> Option<Self> {
        let object = s.get(CtxKey::Object).filter(|o| !o.is_empty())?.to_string();
        crate::v2::scene::source::with_loaded(|model: Option<&SceneModel>| {
            let model = model?;
            let edits = BSceneActions::now(s);
            let current = edits.of(&object, model).and_then(|(name, slot)| edits.action(&name, model).map(|a| (a, slot)));
            Some(Self { names: edits.names(model), current })
        })
    }
}

impl BActionTemplate {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }

    fn set(value: String) -> BAction {
        BAction::Set(OpKey::Action.path(), value)
    }

    /// A header menu over `host`'s own open key, its anchor showing `label`.
    fn pick(&self, name: &'static str, icon: &'static str, label: &str, tooltip: &'static str) -> BPick {
        BPick::new(MenuKey::Top(self.host.slug(), name).path(), BControl::new().with_icon(icon).with_label(label).with_tooltip(tooltip))
    }

    /// `template_action`: browse, then New and Unlink.
    fn action_row(&self, s: &BState, view: &BActionView) -> Vec<BView> {
        let current = view.current.as_ref().map(|(a, _)| a.name.as_str());
        let pick = self.pick("action_browse", "action", current.unwrap_or(""), "Browse Action to be linked");
        let browse = view.names.iter().fold(pick, |pick, n| pick.item(n, Some(n.as_str()) == current, Self::set(format!("assign:{n}")))).view(s);
        match current {
            Some(_) => vec![
                browse,
                BView::Button(BControl::new().with_icon("copy").with_tooltip("New Action").with_action(Self::set("new".into()))),
                BView::Button(BControl::new().with_icon("x").with_tooltip("Unlink Action").with_action(Self::set("unlink".into()))),
            ],
            None => vec![browse, BView::Button(BControl::new().with_icon("plus").with_label("New").with_tooltip("New Action").with_action(Self::set("new".into())))],
        }
    }

    /// The slot's `template_search`: the slots an object can use — its own
    /// type's and a legacy action's — and none.
    fn slot_menu(&self, s: &BState, action: &SceneAction, slot: &str) -> BView {
        let pick = self.pick("action_slot", "object", slot.get(2..).unwrap_or(""), "The slot that animates the object");
        let pick = action
            .slots
            .iter()
            .filter(|x| matches!(x.id_code(), "OB" | "XX"))
            .fold(pick, |pick, x| pick.item(x.name(), x.identifier == slot, Self::set(format!("slot:{}", x.identifier))));
        pick.item("None", slot.is_empty(), Self::set("slot:".into())).view(s)
    }

    /// The header's selector: in the Action Editor, with an object.
    pub fn header(&self, s: &BState) -> BView {
        let Some(view) = BActionView::now(s).filter(|_| active_mode(s, self.host) == ACTION_EDITOR) else { return BView::row(Vec::new()) };
        let mut list = self.action_row(s, &view);
        if let Some((action, slot)) = &view.current {
            list.push(self.slot_menu(s, action, slot));
        }
        BView::row(list)
    }

    /// Properties ▸ Object ▸ Animation: Action and Slot, labelled.
    pub fn rows(&self, s: &BState) -> Vec<BView> {
        let Some(view) = BActionView::now(s) else { return Vec::new() };
        let mut out = vec![BView::split(vec![BView::label("Action"), BView::row(self.action_row(s, &view))])];
        if let Some((action, slot)) = &view.current {
            out.push(BView::split(vec![BView::label("Slot"), self.slot_menu(s, action, slot)]));
        }
        out
    }

    /// The Dope Sheet sidebar's Action tab: the Action panel (its name,
    /// Manual Frame Range, Start / End, Cyclic) and the Slot panel (its
    /// name and type), over the object's action.
    pub fn cards(&self, tab: &'static str, s: &BState) -> Vec<BView> {
        // No action on the active object — one played by NLA strips alone,
        // or none at all: the panel still stands, holding the selector to
        // browse to an action or make one (the rows Properties ▸ Object ▸
        // Animation shows), rather than leaving the sidebar a blank column.
        let Some((action, slot)) = BActionView::now(s).and_then(|v| v.current) else {
            let rows = self.rows(s);
            return if rows.is_empty() { Vec::new() } else { vec![self.card(tab, "Action", rows, s)] };
        };
        let (range, cyclic) = crate::v2::scene::source::with_loaded(|m| m.map_or((action.manual_range, action.cyclic), |m| BSceneActions::now(s).range_of(&action.name, m)));
        let (start, end) = range.unwrap_or((action.frame_start, action.frame_end));
        let check = |key: OpKey, label: &str, on: bool| BView::ctrl(BCtrl::read(BCtrlKind::Check, key.path(), BValue::Bool(on), s).with_label(label));
        let frame = |key: OpKey, value: f32| BView::ctrl(BCtrl::read(BCtrlKind::Number { min: FRAMES.0, max: FRAMES.1, step: 1.0 }, key.path(), BValue::Float(value), s));
        let mut body = vec![BView::row(vec![BView::Tab(BControl::new().with_icon("action").with_label(action.name.as_str()))]), check(OpKey::ActionRange, "Manual Frame Range", range.is_some())];
        if range.is_some() {
            body.push(BView::split(vec![BView::label("Start"), frame(OpKey::ActionStart, start)]));
            body.push(BView::split(vec![BView::label("End"), frame(OpKey::ActionEnd, end)]));
        }
        body.push(check(OpKey::ActionCyclic, "Cyclic Animation", cyclic));
        let mut out = vec![self.card(tab, "Action", body, s)];
        if !slot.is_empty() {
            let name = BView::split(vec![BView::label("Name"), BView::label(slot.get(2..).unwrap_or(""))]);
            let kind = BView::split(vec![BView::label("Type"), BView::label(if slot.starts_with("OB") { "Object" } else { "Unspecified" })]);
            out.push(self.card(tab, "Slot", vec![name, kind], s));
        }
        out
    }

    /// Properties ▸ Object ▸ Animation, closed until opened
    /// (`PropertiesAnimationMixin`: `DEFAULT_CLOSED`), with an object.
    pub fn panel(&self, tab: &'static str, s: &BState) -> Option<BView> {
        s.get(CtxKey::Object).filter(|o| !o.is_empty())?;
        let open = BPanelItem::is_open(s, &BPanelItem::expand_key(self.host, tab, "Animation"), true);
        Some(self.card_as(tab, "Animation", if open { self.rows(s) } else { Vec::new() }, true, s))
    }

    fn card(&self, tab: &'static str, title: &'static str, body: Vec<BView>, s: &BState) -> BView {
        self.card_as(tab, title, body, false, s)
    }

    fn card_as(&self, tab: &'static str, title: &'static str, body: Vec<BView>, closed: bool, s: &BState) -> BView {
        let key = BPanelItem::expand_key(self.host, tab, title);
        let open = BPanelItem::is_open(s, &key, closed);
        BCard::panel(title).fold(key, open).body(if open { body } else { Vec::new() }).into()
    }
}

impl BViewItem for BActionTemplate {
    fn build(&self, s: &BState) -> BView {
        self.header(s)
    }
}
