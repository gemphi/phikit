//! Modebar — the top-left mode dropdown of a host's header. Blender ref:
//! `VIEW3D_HT_header` mode picker (`layout.template_header()` +
//! `object.mode` enum). Anchor shows the active mode's icon + label;
//! click opens a popover listing every mode from `set::modes_for(host)`.
//!
//! For the Outliner, Blender places the search field (`filter_text`) and a
//! Filter popover button inline in the header immediately after the display-
//! mode dropdown. We replicate this by returning a `BView::Row` wrapping
//! `[mode_popover, search_field, filter_popover]` when `host == Outliner`.

pub mod item;
pub mod list;
pub mod set;

use super::modebar::set::{default_mode, mode_key, modes_for};
use crate::v2::host::{BHostType, host_slug};
use crate::v2::part::custom::outliner::{set::TOGGLES, tree as otree};
use crate::v2::state::{
    BState, k_menu,
    val::{V_EMPTY, V_FALSE, V_TRUE},
};
use crate::v2::view::prim::{BAction, BControl, BPopoverSide};
use crate::v2::view::{BView, BViewItem, BViewList};

pub use item::BModebarItem;
pub use list::BModebarList;
pub use set::BModeSpec;

#[derive(Clone, Copy, Debug)]
pub struct BModebar {
    pub host: BHostType,
}

impl BModebar {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

fn active_icon(host: BHostType, active: &str) -> &'static str {
    modes_for(host)
        .iter()
        .find(|m| m.label == active)
        .map(|m| m.icon)
        .unwrap_or("object")
}

/// Build the Outliner filter popover body: sort-alpha toggle, separator,
/// then one toggle per restriction column (OUTLINER_PT_filter).
fn outliner_filter_body(s: &BState) -> Vec<BView> {
    let sort_on = otree::sort_alpha(s);
    let mut body = vec![
        BView::button_action(
            if sort_on { "check" } else { "" },
            "Sort Alphabetically",
            BAction::Set(
                otree::k_sort_alpha(),
                if sort_on { V_FALSE.to_string() } else { V_TRUE.to_string() },
            ),
        ),
        BView::menu_sep(),
    ];
    for t in TOGGLES {
        let col_key = otree::k_column(t.state_key);
        let active = match s.get(&col_key) {
            Some(V_TRUE) => true,
            Some(V_FALSE) => false,
            _ => t.default_on,
        };
        body.push(BView::button_action(
            if active { t.icon } else { "" },
            t.tooltip,
            BAction::Set(
                col_key,
                if active { V_FALSE.to_string() } else { V_TRUE.to_string() },
            ),
        ));
    }
    body
}

impl BViewItem for BModebar {
    fn build(&self, s: &BState) -> BView {
        // Host-scoped like every other menubar popover, so opening the mode
        // dropdown in one editor does not open it in the rest.
        let key = k_menu(host_slug(self.host), "mode");
        let open = s.get(&key) == Some(V_TRUE);
        let toggle = BAction::Toggle(key.clone());
        let close = BAction::Set(key, V_EMPTY.to_string());
        let active = s
            .get(&mode_key(self.host))
            .unwrap_or_else(|| default_mode(self.host));
        // Same header-height anchor as the menubar labels, so the mode
        // dropdown lines up with the menus beside it.
        let anchor = BView::MenuAnchor(
            BControl::new()
                .with_icon(active_icon(self.host, active))
                .with_label(active)
                .with_active(open)
                .with_action(toggle),
        );
        let body = BModebarList::new(self.host).items(s);
        let mode_popover = BView::popover(anchor, body, open, close, BPopoverSide::Below);

        // Outliner: append search field + filter popover inline in the header
        // row, matching Blender's `OUTLINER_HT_header` layout (filter_text +
        // OUTLINER_PT_filter icon).
        if self.host == BHostType::Outliner {
            let search_val = otree::search_text(s);
            let search_clear = BAction::Set(otree::k_search(), String::new());
            let search = BView::search_field("Search...", search_val, search_clear);

            let fkey = otree::k_filter_open();
            let fopen = s.get(&fkey) == Some(V_TRUE);
            let ftoggle = BAction::Set(
                fkey.clone(),
                if fopen { V_FALSE.to_string() } else { V_TRUE.to_string() },
            );
            let fclose = BAction::Set(fkey, V_FALSE.to_string());
            let filter_anchor = BView::button_action("filter", "", ftoggle);
            let filter_popover = BView::popover(
                filter_anchor,
                outliner_filter_body(s),
                fopen,
                fclose,
                BPopoverSide::Below,
            );

            // Header layout: [mode] [search] [filter] as a hugging row. The
            // enclosing header Center pane centres this group; the mode
            // dropdown, search field and filter popover keep their natural
            // widths instead of stretching into full-width bars. Mirrors
            // Blender's `OUTLINER_HT_header` (mode picker · filter text ·
            // filter popover) without letting any control run off the pane.
            BView::row(vec![mode_popover, search, filter_popover])
        } else {
            mode_popover
        }
    }
}
