//! The sidebar region's panels — what the active tab draws beside the strip
//! (`BSidebarBody`). Property panels are the Properties editor's own
//! (`props::BPanelItem`) over the same keys; the View tab's focal length is
//! the area's own view (`scene::view_slot`), so each viewport's sidebar
//! edits its own lens.

use super::set::{BSidebarBody, BSidebarTabSpec, tabs_for};
use crate::v2::host::BHostType;
use crate::v2::part::props::{BPanelItem, BSectionSpec};
use crate::v2::scene::camera::view_lens;
use crate::v2::scene::view_slot::BViewSlot;
use crate::v2::state::val::BValue;
use crate::v2::state::{BKey as _, BState, CtxKey, View3dKey};
use crate::v2::view::prim::{BCard, BCtrl, BCtrlKind, with_peers};
use crate::v2::view::{BView, BViewItem};

/// A panel named once and laid out by rows that already exist elsewhere.
type BPanelRows = (&'static str, fn(&BState) -> Vec<BView>);

/// The Timeline sidebar's panels: the transport's two, whose rows are the
/// header popovers' own (`playback::panel`).
static PLAYBACK_PANELS: [BPanelRows; 2] = [
    ("Playback", crate::v2::part::playback::panel::body),
    ("Playhead Snapping", crate::v2::part::playback::panel::snap_body),
];

/// `View3D.lens`'s range in Blender (`rna_space.cc`), mm.
const LENS_RANGE: (f32, f32) = (1.0, 5000.0);

/// One host's sidebar panels.
#[derive(Clone, Copy, Debug)]
pub struct BSidebarPanels {
    pub host: BHostType,
}

impl BSidebarPanels {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }

    /// The tab showing: the one picked, else the first.
    pub fn tab(&self, s: &BState) -> Option<&'static BSidebarTabSpec> {
        let tabs = tabs_for(self.host);
        let picked = s.host(self.host).get("sidebar.tab");
        tabs.iter().find(|t| Some(t.label) == picked).or(tabs.first())
    }

    /// The panels of the tab showing, top to bottom — drawn plain.
    ///
    /// A sidebar is a LIST of panels, not a stack of objects, so its panels
    /// are a menu: a fold triangle, a title, the rows under it, and nothing
    /// behind them (`BCard::plain`). As cards they each carried a border, a
    /// card fill and a heading fill a shade off it, and a sub-panel carried
    /// another shade again — so a column of them was boxes inside a box, each
    /// with its own heading colour, and by the second level the panel you
    /// wanted sat three tones from the region holding it. The triangle says
    /// what folds and the indent says what belongs to what, which is all a
    /// menu uses and all this needs.
    ///
    /// Every tab's panels come through here, so this is the one place it is
    /// said; the Properties editor and the Preferences modal build their own
    /// and keep their cards, where a card is the right answer — there a panel
    /// IS a thing with edges, holding one object's settings.
    pub fn views(&self, s: &BState) -> Vec<BView> {
        let Some(tab) = self.tab(s) else { return Vec::new() };
        let built = match tab.body {
            BSidebarBody::None => Vec::new(),
            BSidebarBody::Object(_) if s.get(CtxKey::Object).is_none_or(str::is_empty) => Vec::new(),
            BSidebarBody::Object(sections) | BSidebarBody::Props(sections) => self.cards(tab.label, sections, s),
            BSidebarBody::View => vec![self.view_card(tab.label, s)],
            BSidebarBody::Action => crate::v2::part::action::template::BActionTemplate::new(self.host).cards(tab.label, s),
            BSidebarBody::Playback => self.folded(tab.label, &PLAYBACK_PANELS, s),
            BSidebarBody::Overlay => self.overlay_cards(tab.label, s),
        };
        built.into_iter().map(Self::plain).collect()
    }

    /// A panel drawn plain, and anything that is not a card left alone.
    fn plain(view: BView) -> BView {
        match view {
            BView::Card(card) => BView::Card(Box::new(card.plain())),
            other => other,
        }
    }

    /// The host's box panel as folding cards, one per sub-panel
    /// (`filter::overlay::groups`). The rows are the header popover's own and
    /// so are the fold keys (`overlay::scope`), so the two places cannot
    /// disagree about what is ticked or about what is folded — the Movie Clip
    /// editor's Marker Display is shut here exactly when it is shut in the
    /// header. The `tab` plays no part: a key per drawing site is the fork.
    fn overlay_cards(&self, _tab: &'static str, s: &BState) -> Vec<BView> {
        use crate::v2::part::filter::{fold, overlay};
        fold::cards(s, &overlay::scope(self.host), overlay::groups(s, self.host), fold::all_open)
    }

    /// Cards built from rows someone else already owns: the title, and the
    /// function that lays the rows out. Each folds on its own key, as every
    /// other panel in every other sidebar does.
    fn folded(&self, tab: &'static str, panels: &'static [BPanelRows], s: &BState) -> Vec<BView> {
        with_peers(
            panels
                .iter()
                .map(|(title, rows)| {
                    let key = BPanelItem::expand_key(self.host, tab, title);
                    let open = BPanelItem::is_open(s, &key, false);
                    let body = if open { rows(s) } else { Vec::new() };
                    BCard::panel(*title).fold(key, open).body(body).into()
                })
                .collect(),
        )
    }

    fn cards(&self, tab: &'static str, sections: &'static [BSectionSpec], s: &BState) -> Vec<BView> {
        with_peers(
            sections
                .iter()
                .map(|section| {
                    let key = BPanelItem::expand_key(self.host, tab, section.title);
                    let open = BPanelItem::is_open(s, &key, section.closed);
                    BPanelItem::new(self.host, tab, *section, open).build(s)
                })
                .collect(),
        )
    }

    /// `VIEW3D_PT_view`: Focal Length, the area's own `View3D.lens`, and
    /// Render Region (`use_render_border`, the view's own region outside
    /// the camera view — `scene::view_border`).
    fn view_card(&self, tab: &'static str, s: &BState) -> BView {
        let key = BPanelItem::expand_key(self.host, tab, "View");
        let open = BPanelItem::is_open(s, &key, false);
        let lens = BCtrl::read(
            BCtrlKind::Number { min: LENS_RANGE.0, max: LENS_RANGE.1, step: 1.0 },
            BViewSlot::key(View3dKey::Lens),
            BValue::Float(view_lens(s)),
            s,
        );
        let border = BCtrl::read(
            BCtrlKind::Check,
            View3dKey::BorderUse.path(),
            BValue::Bool(crate::v2::scene::view_border::BViewBorder::now(s).on),
            s,
        );
        let body = if open {
            vec![BView::split(vec![BView::label("Focal Length"), BView::ctrl(lens)]), BView::split(vec![BView::label("Render Region"), BView::ctrl(border)])]
        } else {
            Vec::new()
        };
        BCard::panel("View").fold(key, open).body(body).into()
    }
}
