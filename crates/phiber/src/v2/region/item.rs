//! `BRegionItem` — one region.

use super::{BRegionSpec, BRegionType};
use crate::v2::host::BHostType;
use crate::v2::pane::{BPaneList, BPaneType};
use crate::v2::part::modebar::set::modes_for;
use crate::v2::part::{BPartItem, BPartType};
use crate::v2::state::BState;
use crate::v2::view::{BView, BViewItem, BViewList};

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
            BView::Pane { children, .. } => children
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
        if let Some(BView::Pane { children, .. }) = panes
            .iter_mut()
            .find(|pane| matches!(pane, BView::Pane { kind, .. } if *kind == pane_kind))
        {
            children.push(part);
            return;
        }
        panes.push(BView::Pane {
            kind: pane_kind,
            children: vec![part],
        });
    }

    fn complete_header(&self, panes: &mut Vec<BView>, state: &BState) {
        // Every mode-capable editor keeps the existing BModebar data/action
        // path in the center slot. Existing declarations remain untouched.
        if !modes_for(self.host).is_empty() {
            Self::ensure_part(
                panes,
                BPaneType::Center,
                BPartType::Modebar,
                self.host,
                state,
            );
        }

        // Normalize the host switcher to exactly one control at the far end of
        // the right pane. This preserves declared header content and prevents
        // duplicate controls on hosts that already listed a Switcher part.
        for pane in panes.iter_mut() {
            if let BView::Pane { children, .. } = pane {
                children.retain(|child| {
                    !matches!(
                        child,
                        BView::Part {
                            kind: BPartType::Switcher,
                            ..
                        }
                    )
                });
            }
        }
        Self::ensure_part(
            panes,
            BPaneType::Right,
            BPartType::Switcher,
            self.host,
            state,
        );
    }
}

impl BViewItem for BRegionItem {
    fn build(&self, state: &BState) -> BView {
        if self.region.panes.is_empty() {
            return BView::empty();
        }
        let mut children = BPaneList::new(self.host, self.region.panes).items(state);
        if self.region.kind == BRegionType::Header {
            self.complete_header(&mut children, state);
        }
        BView::Region {
            kind: self.region.kind,
            children,
        }
    }
}
