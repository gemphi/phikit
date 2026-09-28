//! Region — a slice inside a host (Header/Center/Footer/…). Owns its panes.

pub mod set;

use crate::v2::host::BHostType;
use crate::v2::pane::BPaneList;
use crate::v2::pane::BPaneSpec;
use crate::v2::pane::BPaneType;
use crate::v2::part::BPartItem;
use crate::v2::part::BPartType;
use crate::v2::part::modebar::set::modes_for;
use crate::v2::spec::BSpec;
use crate::v2::spec::BSpecKind;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;

// `BRegionType` — DNA region kind. Mirrors `eRegion_Type` in Blender 5.2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BRegionType {
    Header,
    Center, // WINDOW: the primary work canvas
    Footer,
    ToolHeader,
    Tools, // T panel
    UI,    // N panel
    ToolProps,
    AssetShelf,
    AssetShelfHeader,
    Channels,
    HUD,
    NavBar,
    Execute,
    Scrubbing,
    Preview,
    Temporary,
    XR,
}

// `BRegionSpec` — one region within a host. Owns its panes inline.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BRegionSpec {
    pub kind: BRegionType,
    pub panes: &'static [BPaneSpec],
}

impl BSpec for BRegionSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Region(self.kind)
    }
    fn label(&self) -> &'static str {
        self.kind.label()
    }
}

// `BRegionItem` — one region.

#[derive(Clone, Copy, Debug)]
pub struct BRegionItem {
    pub host: BHostType,
    pub region: BRegionSpec,
}

impl BRegionItem {
    pub fn new(host: BHostType, region: BRegionSpec) -> Self {
        Self { host, region }
    }

    fn has_part(panes: &[BView], part: BPartType) -> bool {
        panes.iter().any(|pane| match pane {
            BView::Pane { list, .. } => list
                .iter()
                .any(|child| matches!(child, BView::Part { kind, .. } if *kind == part)),
            _ => false,
        })
    }

    fn ensure_part(
        panes: &mut Vec<BView>,
        pane_kind: BPaneType,
        part: BPartType,
        host: BHostType,
        state: &BState,
    ) {
        if Self::has_part(panes, part) {
            return;
        }
        let part = BPartItem::new(host, part).build(state);
        if let Some(BView::Pane { list, .. }) = panes
            .iter_mut()
            .find(|pane| matches!(pane, BView::Pane { kind, .. } if *kind == pane_kind))
        {
            list.push(part);
            return;
        }
        panes.push(BView::Pane {
            kind: pane_kind,
            list: vec![part],
        });
    }

    /// Like [`Self::ensure_part`], but the part leads its pane.
    fn ensure_first_part(
        panes: &mut Vec<BView>,
        pane_kind: BPaneType,
        part: BPartType,
        host: BHostType,
        state: &BState,
    ) {
        let view = BPartItem::new(host, part).build(state);
        if let Some(BView::Pane { list, .. }) = panes
            .iter_mut()
            .find(|pane| matches!(pane, BView::Pane { kind, .. } if *kind == pane_kind))
        {
            list.insert(0, view);
            return;
        }
        panes.insert(
            0,
            BView::Pane {
                kind: pane_kind,
                list: vec![view],
            },
        );
    }

    /// Drop every declared copy of `part` so it can be placed exactly once.
    fn strip(panes: &mut [BView], part: BPartType) {
        for pane in panes.iter_mut() {
            if let BView::Pane { list, .. } = pane {
                list.retain(|child| !matches!(child, BView::Part { kind, .. } if *kind == part));
            }
        }
    }

    /// One header shape for every host, whichever area it sits in:
    ///   left   — what this editor is (icon + name, the switch trigger), then
    ///            its mode bar and menus,
    ///   centre — what you are looking for (the host's search, if it has one),
    ///   right  — how it is filtered, and the rest of its controls.
    fn complete_header(&self, panes: &mut Vec<BView>, state: &BState) {
        for part in [
            BPartType::Switcher,
            BPartType::Modebar,
            BPartType::Search,
            BPartType::Filter,
        ] {
            Self::strip(panes, part);
        }
        if crate::v2::part::search::search_key(self.host).is_some()
            && !self.host.searches_in_tool_header()
        {
            Self::ensure_part(panes, BPaneType::Center, BPartType::Search, self.host, state);
        }
        if crate::v2::part::filter::has_filter(self.host) {
            Self::ensure_part(panes, BPaneType::Right, BPartType::Filter, self.host, state);
        }
        if !modes_for(self.host).is_empty() {
            Self::ensure_first_part(panes, BPaneType::Left, BPartType::Modebar, self.host, state);
        }
        Self::ensure_first_part(panes, BPaneType::Left, BPartType::Switcher, self.host, state);
    }
}

/// A pane with no parts, or only parts whose body is empty.
fn pane_is_blank(pane: &BView) -> bool {
    match pane {
        BView::Empty => true,
        BView::Pane { list, .. } => list
            .iter()
            .all(|part| match part {
                BView::Empty => true,
                BView::Part { body, .. } => body.is_empty(),
                _ => false,
            }),
        _ => false,
    }
}

impl BViewItem for BRegionItem {
    fn build(&self, state: &BState) -> BView {
        if self.region.panes.is_empty() {
            return BView::empty();
        }
        let mut list = BPaneList::new(self.host, self.region.panes).items(state);
        if self.region.kind == BRegionType::Header {
            self.complete_header(&mut list, state);
        }
        // An execute bar with nothing to execute — the file browser's, with
        // no purpose set — is not there at all, rather than an empty strip
        // of its height (Blender's `FILEBROWSER_PT_execute` polls the same
        // way); so is a tool header whose active tool has no settings — the
        // Image editor in View mode, whose tool header Blender's Rendering
        // workspace keeps hidden — and an asset shelf, or its header, outside
        // a painting mode (Blender hides `RGN_TYPE_ASSET_SHELF` there) or
        // folded away. An empty band the height of a drawer that is not
        // open is what left the Image editor short of its Timeline.
        let droppable = matches!(
            self.region.kind,
            BRegionType::Execute | BRegionType::ToolHeader | BRegionType::AssetShelf | BRegionType::AssetShelfHeader
        );
        if droppable && list.iter().all(pane_is_blank) {
            return BView::empty();
        }
        BView::Region {
            kind: self.region.kind,
            list,
        }
    }
}

// `BRegionList` — iterate the regions of a host.

#[derive(Clone, Copy, Debug)]
pub struct BRegionList {
    pub host: BHostType,
}

impl BRegionList {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

impl BViewList for BRegionList {
    fn items(&self, state: &BState) -> Vec<BView> {
        self.host
            .spec()
            .regions
            .iter()
            .map(|r| BRegionItem::new(self.host, *r).build(state))
            .filter(|v| !v.is_empty())
            .collect()
    }
}

// ── Layout (was `view/render/region.rs`) ──
// Region-level layout: `BRegionType` decodes into orientation and placement.
// Its size, padding and surface are `BFlex` decisions (`gpui/region.rs`).
// 
// Blender ref: `ARegion` alignment flags (`RGN_ALIGN_TOP` / `BOTTOM` /
// `LEFT` / `RIGHT`) plus header/footer heights.
use crate::v2::view::layout::{BAxis, BPlacement, BRegionLayout};

impl BRegionLayout for BRegionType {
    fn axis(&self) -> BAxis {
        match self {
            // NavBar / Tools are vertical strips — icons stack top-to-bottom.
            BRegionType::NavBar | BRegionType::Tools => BAxis::Column,
            // Center / Flex regions must not force items_center so that a
            // Canvas child can fill the full height of the region.
            _ => BAxis::Row,
        }
    }

    fn placement(&self) -> BPlacement {
        match self {
            BRegionType::Header
            | BRegionType::ToolHeader
            | BRegionType::HUD
            | BRegionType::Scrubbing => BPlacement::Top,
            // The shelf header is the strip along the shelf's top edge, so it
            // sits at the foot of the host with the shelf beneath it — not as
            // a second band at the top, which is where it had drifted.
            BRegionType::AssetShelfHeader
            | BRegionType::Footer
            | BRegionType::Execute
            | BRegionType::AssetShelf => BPlacement::Bottom,
            BRegionType::Tools | BRegionType::NavBar => BPlacement::Left,
            BRegionType::UI | BRegionType::ToolProps | BRegionType::Channels => BPlacement::Right,
            _ => BPlacement::Flex,
        }
    }
}
