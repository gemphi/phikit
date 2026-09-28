//! Pane — a Left/Center/Right slot inside a region.

pub mod set;

use crate::v2::host::BHostType;
use crate::v2::part::BPartList;
use crate::v2::part::BPartType;
use crate::v2::spec::BSpec;
use crate::v2::spec::BSpecKind;
use crate::v2::state::BState;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::BViewList;

// `BPaneType` — pane slot inside a region.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BPaneType {
    Left,
    Center,
    Right,
}

// `BPaneSpec` — one pane within a region. Owns its parts inline.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BPaneSpec {
    pub kind: BPaneType,
    pub parts: &'static [BPartType],
}

impl BSpec for BPaneSpec {
    fn kind(&self) -> BSpecKind {
        BSpecKind::Pane(self.kind)
    }
    fn label(&self) -> &'static str {
        self.kind.label()
    }
}

// `BPaneItem` — one pane.

#[derive(Clone, Copy, Debug)]
pub struct BPaneItem {
    pub host: BHostType,
    pub pane: BPaneSpec,
}

impl BPaneItem {
    pub fn new(host: BHostType, pane: BPaneSpec) -> Self {
        Self { host, pane }
    }
}

impl BViewItem for BPaneItem {
    fn build(&self, state: &BState) -> BView {
        if self.pane.parts.is_empty() {
            return BView::empty();
        }
        BView::Pane {
            kind: self.pane.kind,
            list: BPartList::new(self.host, self.pane.parts).items(state),
        }
    }
}

// `BPaneList` — iterate the panes of a region.

#[derive(Clone, Copy, Debug)]
pub struct BPaneList<'a> {
    pub host: BHostType,
    pub panes: &'a [BPaneSpec],
}

impl<'a> BPaneList<'a> {
    pub fn new(host: BHostType, panes: &'a [BPaneSpec]) -> Self {
        Self { host, panes }
    }
}

impl<'a> BViewList for BPaneList<'a> {
    fn items(&self, state: &BState) -> Vec<BView> {
        self.panes
            .iter()
            .map(|p| BPaneItem::new(self.host, *p).build(state))
            .filter(|v| !v.is_empty())
            .collect()
    }
}

// ── Layout (was `view/render/pane.rs`) ──
// Pane-level layout: tri-cell horizontal grid. Left hugs left, Center
// flexes to fill, Right hugs right. Same shape at every region level.
pub trait BPaneLayout {
    /// True when this pane should grow to consume remaining space in its
    /// row (`flex_1`). Only `Center` grows.
    fn is_center(&self) -> bool;
    /// True when this pane's content should hug the right edge
    /// (`justify_end`). Only `Right` hugs right.
    fn is_right(&self) -> bool;
}

impl BPaneLayout for BPaneType {
    fn is_center(&self) -> bool {
        matches!(self, BPaneType::Center)
    }
    fn is_right(&self) -> bool {
        matches!(self, BPaneType::Right)
    }
}
