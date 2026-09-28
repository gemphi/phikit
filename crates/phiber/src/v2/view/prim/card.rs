//! `BCard` — Blender's `Panel` (`editors/interface/interface_panel.cc`,
//! `PanelType` in `blenkernel/BKE_screen.hh`): a heading strip and the
//! content under it, with the behaviours a panel can have declared as
//! traits rather than as separate view kinds.
//!
//! What maps to what:
//!
//! | Blender                                  | `BCard`                         |
//! |------------------------------------------|---------------------------------|
//! | `PanelType.label`, `.icon`               | `head.title`, `head.icon`       |
//! | `PanelType.draw_header`                  | `head.controls`                 |
//! | `PanelType.draw_header_preset`           | `head.preset`                   |
//! | `PanelType.draw`                         | `body`                          |
//! | `PNL_CLOSED`, `PANEL_TYPE_DEFAULT_CLOSED`| `fold` (the key holds open/closed) |
//! | `PANEL_TYPE_NO_HEADER`                   | `head: None`                    |
//! | `PanelType.parent` (a sub-panel)         | `BCardKind::Sub { depth }`      |
//! | `TH_PANEL_ACTIVE` outline (`active_property`) | `active`                   |
//! | the grip at the header's right (`ICON_GRIP`)   | drawn for `Panel` cards    |
//!
//! Blender's panels have no footer — actions sit in a footer *region*
//! (`RGN_TYPE_FOOTER`) — but dialogs built from cards need one, so `foot`
//! is the strip under the body, drawn only when it holds something.

use super::action::BAction;
use crate::v2::state::BPath;
use crate::v2::state::val::{V_FALSE, V_TRUE};
use crate::v2::view::BView;

/// Which kind of panel a card is — decides its backdrop and border the way
/// `panel_draw_aligned_backdrop` / `panel_draw_border` do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BCardKind {
    /// A top-level panel: header backdrop (`TH_PANEL_HEADER`), body
    /// backdrop (`TH_PANEL_BACK`), outline (`TH_PANEL_OUTLINE`), a grip.
    Panel,
    /// A sub-panel, `depth` levels under its panel (1 for a panel's own
    /// sub-panels): indented by 0.7 of a unit, its own backdrop
    /// (`TH_PANEL_SUB_BACK`), no outline, no grip. Its own sub-panels are a
    /// level deeper — a tree, as the Outliner's rows are.
    Sub { depth: u8 },
    /// A card with no surface of its own — content grouped under a heading
    /// on whatever it sits on (a floating region's panel, `RGN_ALIGN_FLOAT`).
    Bare,
    /// A one-row strip of controls: a header, footer or tool bar region
    /// (`RGN_TYPE_HEADER`, `RGN_TYPE_FOOTER`, `RGN_TYPE_TOOL_HEADER`). Its
    /// controls are the `body`; it has no heading of its own.
    Strip(BStrip),
    /// A region's column of cards: the resizable, scrolling sidebar
    /// (`RGN_TYPE_UI` / `RGN_TYPE_TOOLS` holding panels). Its cards are the
    /// `body`.
    Column(BColumn),
}

/// Which region column a [`BCardKind::Column`] is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BColumn {
    /// Fixed width the user drags (`view::size`), scrolls, can be hidden.
    Sidebar,
    /// A region's main content: fills the region and scrolls (`RGN_TYPE_WINDOW`
    /// / `RGN_TYPE_UI` content, `View2D` scrolled).
    Body,
}

/// Which strip a [`BCardKind::Strip`] is: they differ only in height, top
/// corners and name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BStrip {
    /// Header height, top corners rounded to sit under a surface's edge.
    Header,
    /// Footer height.
    Footer,
    /// Header height, square.
    Toolbar,
}

/// The heading strip — `PNL_HEADER` tall (1.25 units).
#[derive(Clone, Debug)]
pub struct BCardHead {
    pub title: String,
    pub icon: Option<String>,
    /// `draw_header`: controls before the title (a checkbox that enables the
    /// whole panel, as Motion Blur's does).
    pub controls: Vec<BView>,
    /// `draw_header_preset`: controls pushed to the right, before the grip.
    pub preset: Vec<BView>,
}

/// Open or closed, and the key that says which (`PNL_CLOSED`).
#[derive(Clone, Debug)]
pub struct BFold {
    pub key: BPath,
    pub open: bool,
    /// The fold keys of the other top-level panels in the same region, for
    /// Ctrl-click (`panels_collapse_all`). Filled by [`with_peers`].
    pub peers: Vec<BPath>,
}

impl BFold {
    /// What clicking the header does: write the other state explicitly, so
    /// a panel that is open because its key is absent closes on the first
    /// click rather than being "toggled" to open.
    pub fn flip(&self) -> BAction {
        BAction::Set(self.key.clone(), if self.open { V_FALSE } else { V_TRUE }.to_string())
    }

    /// Ctrl-click on a header (`panels_collapse_all`): this panel open,
    /// every other panel in its region closed.
    pub fn solo(&self) -> Vec<BAction> {
        std::iter::once(BAction::Set(self.key.clone(), V_TRUE.to_string()))
            .chain(self.peers.iter().map(|k| BAction::Set(k.clone(), V_FALSE.to_string())))
            .collect()
    }

    /// Dragging across headers (`panel_drag_collapse`): the state every
    /// header the pointer crosses is set to — the first header's new state.
    pub fn sweep_to(&self, open: bool) -> BAction {
        BAction::Set(self.key.clone(), if open { V_TRUE } else { V_FALSE }.to_string())
    }
}

/// Give every folding top-level card in one region the fold keys of the
/// others, so Ctrl-clicking a header can close them (`panels_collapse_all`
/// works on the region's panel list, not on a panel's children).
pub fn with_peers(mut views: Vec<BView>) -> Vec<BView> {
    let keys: Vec<BPath> = views
        .iter()
        .filter_map(|v| match v {
            BView::Card(card) if card.kind == BCardKind::Panel => card.fold.as_ref().map(|f| f.key.clone()),
            _ => None,
        })
        .collect();
    for v in &mut views {
        if let BView::Card(card) = v
            && card.kind == BCardKind::Panel
            && let Some(fold) = card.fold.as_mut()
        {
            fold.peers = keys.iter().filter(|k| **k != fold.key).cloned().collect();
        }
    }
    views
}

#[derive(Clone, Debug)]
pub struct BCard {
    pub kind: BCardKind,
    /// `None` is `PANEL_TYPE_NO_HEADER`: the body with no strip above it.
    pub head: Option<BCardHead>,
    pub body: Vec<BView>,
    pub foot: Vec<BView>,
    /// Collapsible when set. A closed card draws its header only.
    pub fold: Option<BFold>,
    /// The panel's data is the active one — its outline is lit.
    pub active: bool,
}

impl BCard {
    /// A top-level panel headed `title`.
    pub fn panel(title: impl Into<String>) -> Self {
        Self {
            kind: BCardKind::Panel,
            head: Some(BCardHead {
                title: title.into(),
                icon: None,
                controls: Vec::new(),
                preset: Vec::new(),
            }),
            body: Vec::new(),
            foot: Vec::new(),
            fold: None,
            active: false,
        }
    }

    /// A panel with no heading (`PANEL_TYPE_NO_HEADER`).
    pub fn headless() -> Self {
        Self { head: None, ..Self::panel("") }
    }

    /// This card as a sub-panel of the card it is pushed into. The
    /// sub-panels it already holds move a level deeper with it, so a card
    /// tree built leaves up carries each card's depth.
    pub fn sub(mut self) -> Self {
        self.kind = BCardKind::Sub { depth: 1 };
        for row in &mut self.body {
            if let BView::Card(child) = row {
                child.nest();
            }
        }
        self
    }

    /// One level deeper in its card tree, with every sub-panel under it.
    fn nest(&mut self) {
        if let BCardKind::Sub { depth } = &mut self.kind {
            *depth += 1;
        }
        for row in &mut self.body {
            if let BView::Card(child) = row {
                child.nest();
            }
        }
    }

    /// How many levels under its panel a sub-panel sits (0 for a panel).
    pub fn depth(&self) -> u8 {
        match self.kind {
            BCardKind::Sub { depth } => depth,
            BCardKind::Panel | BCardKind::Bare | BCardKind::Strip(_) | BCardKind::Column(_) => 0,
        }
    }

    pub fn bare(mut self) -> Self {
        self.kind = BCardKind::Bare;
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        if let Some(head) = &mut self.head {
            head.icon = Some(icon.into());
        }
        self
    }

    pub fn controls(mut self, views: Vec<BView>) -> Self {
        if let Some(head) = &mut self.head {
            head.controls = views;
        }
        self
    }

    pub fn preset(mut self, views: Vec<BView>) -> Self {
        if let Some(head) = &mut self.head {
            head.preset = views;
        }
        self
    }

    pub fn body(mut self, views: Vec<BView>) -> Self {
        self.body = views;
        self
    }

    /// A region column holding `cards`.
    pub fn column(kind: BColumn, cards: Vec<BView>) -> Self {
        Self { kind: BCardKind::Column(kind), ..Self::headless() }.body(cards)
    }

    /// A strip region holding `controls` in one row.
    pub fn strip(kind: BStrip, controls: Vec<BView>) -> Self {
        Self { kind: BCardKind::Strip(kind), ..Self::headless() }.body(controls)
    }

    pub fn foot(mut self, views: Vec<BView>) -> Self {
        self.foot = views;
        self
    }

    /// Collapsible, open or closed as `key` says.
    pub fn fold(mut self, key: BPath, open: bool) -> Self {
        self.fold = Some(BFold { key, open, peers: Vec::new() });
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Whether the body is drawn: an unfoldable card always is.
    pub fn is_open(&self) -> bool {
        self.fold.as_ref().is_none_or(|f| f.open)
    }

    pub fn title(&self) -> Option<&str> {
        self.head.as_ref().map(|h| h.title.as_str())
    }
}

impl From<BCard> for BView {
    fn from(card: BCard) -> Self {
        BView::Card(Box::new(card))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::BKey as _;

    #[test]
    fn a_closed_card_flips_open_and_an_open_one_closed() {
        let key = crate::v2::state::ButsKey::Value("sampling").path();
        let open = BCard::panel("Sampling").fold(key.clone(), true);
        let BAction::Set(path, value) = open.fold.as_ref().unwrap().flip() else { panic!() };
        assert_eq!((path, value.as_str()), (key.clone(), V_FALSE));
        let closed = BCard::panel("Sampling").fold(key, false);
        assert!(!closed.is_open());
        let BAction::Set(_, value) = closed.fold.unwrap().flip() else { panic!() };
        assert_eq!(value, V_TRUE);
    }

    #[test]
    fn an_unfoldable_card_is_always_open() {
        assert!(BCard::panel("About").is_open());
        assert!(BCard::headless().head.is_none());
    }

    #[test]
    fn ctrl_click_opens_this_panel_and_closes_its_region_peers() {
        use crate::v2::view::BView;
        let k = |n: &'static str| BPath::owned(vec!["ctx", "buts", "render", n, "expanded"]);
        let views = super::with_peers(vec![
            BView::Card(Box::new(BCard::panel("Sampling").fold(k("sampling"), false))),
            BView::Card(Box::new(BCard::panel("Film").fold(k("film"), true))),
            BView::Card(Box::new(BCard::headless())),
            BView::Card(Box::new(BCard::panel("Output").fold(k("output"), true).sub())),
        ]);
        let BView::Card(first) = &views[0] else { panic!() };
        let fold = first.fold.as_ref().unwrap();
        // Peers are the other top-level panels only: not itself, not a
        // headless card, not a sub-panel.
        assert_eq!(fold.peers, vec![k("film")]);
        assert_eq!(
            fold.solo(),
            vec![
                BAction::Set(k("sampling"), V_TRUE.to_string()),
                BAction::Set(k("film"), V_FALSE.to_string()),
            ]
        );
        assert_eq!(fold.sweep_to(false), BAction::Set(k("sampling"), V_FALSE.to_string()));
    }
}