//! `BPanelItem` — one collapsible property section. Renders a header row
//! (chevron + title) followed by a column of prop rows (one per
//! `BPropSpec` in `spec.props`) — matches Blender's accordion panels.

use super::prop_view::prop_row;
use super::set::BSectionSpec;
use crate::v2::host::{BHostType, host_slug};
use crate::v2::state::{
    BPath, BState,
    val::{V_FALSE, V_TRUE},
};
use crate::v2::view::prim::{BAction, BPanelRole};
use crate::v2::view::{BView, BViewItem};

#[derive(Clone, Copy, Debug)]
pub struct BPanelItem {
    pub host: BHostType,
    pub tab: &'static str,
    pub spec: BSectionSpec,
    pub expanded: bool,
}

impl BPanelItem {
    pub fn new(host: BHostType, tab: &'static str, spec: BSectionSpec, expanded: bool) -> Self {
        Self {
            host,
            tab,
            spec,
            expanded,
        }
    }

    /// State key for this section's expanded flag:
    /// `ctx.<host>.<tab>.<section>.expanded`. Including the active tab avoids
    /// collisions between common section names such as Surface and Preview.
    pub fn expand_key(host: BHostType, tab: &str, title: &str) -> BPath {
        BPath::owned(vec![
            host_slug(host),
            leak_slug(tab),
            leak_slug(title),
            "expanded",
        ])
    }

    /// The whole heading row is the sole toggle target. Its disclosure chip
    /// is rendered by the focused primitive without a nested handler.
    fn header(&self) -> BView {
        // Panels default expanded when the key is absent, so Toggle would set
        // V_TRUE on the first click and appear to do nothing. Write the
        // explicit next state instead; the full-width heading owns it once.
        let next = if self.expanded { V_FALSE } else { V_TRUE };
        let toggle = BAction::Set(
            Self::expand_key(self.host, self.tab, self.spec.title),
            next.to_string(),
        );
        BView::accordion_header(self.spec.title, self.expanded, toggle)
    }
}

/// Leak a slugified title to `&'static str` so it can live in a `BPath`
/// segment. Titles come from a static `SECTIONS` table so the set of leaked
/// strings is bounded by the compiled panel specs.
fn leak_slug(title: &str) -> &'static str {
    Box::leak(
        title
            .to_ascii_lowercase()
            .replace(' ', "_")
            .into_boxed_str(),
    )
}

impl BViewItem for BPanelItem {
    fn build(&self, s: &BState) -> BView {
        let mut rows = vec![self.header()];
        if self.expanded {
            for p in self.spec.props {
                rows.push(prop_row(p, s));
            }
        }
        // Card role so the whole section reads as a grouped panel.
        BView::panel(BPanelRole::Card, rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::part::panel::prop::BPropSpec;

    static EMPTY: &[BPropSpec] = &[];

    #[test]
    fn initially_expanded_header_first_click_sets_false() {
        let spec = BSectionSpec {
            title: "Surface",
            props: EMPTY,
        };
        let BView::Panel { children, .. } =
            BPanelItem::new(BHostType::Buts, "Material", spec, true).build(&BState::new())
        else {
            panic!()
        };
        let BView::AccordionHeader {
            action: BAction::Set(_, value),
            ..
        } = &children[0]
        else {
            panic!()
        };
        assert_eq!(value, V_FALSE);
    }
}
