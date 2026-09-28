//! The status bar's action log — the `StatusLog` part.
//!
//! Folded, it is one line: the newest thing that happened, with a chevron to
//! unfold it. Unfolded, it is every line, newest first, in a scroller that
//! takes the room the bottom strip grows to hold it (`zone.rs`). The lines
//! come from `state::log`, which describes each action the app applied.

use crate::v2::state::BKey as _;
use crate::v2::state::AppKey;
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::state::val::V_TRUE;
use crate::v2::view::prim::BAction;
use crate::v2::view::prim::BControl;
use crate::v2::view::BView;
use crate::v2::view::BViewItem;

#[derive(Clone, Copy, Debug)]
pub struct BStatusLog {
    pub host: BHostType,
}

impl BStatusLog {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }
}

/// True while the log is unfolded.
pub fn is_open(s: &BState) -> bool {
    s.get(AppKey::StatusLogOpen) == Some(V_TRUE)
}

impl BViewItem for BStatusLog {
    fn build(&self, s: &BState) -> BView {
        let open = is_open(s);
        let toggle = BControl::new()
            .with_icon(if open { "chevron_down" } else { "chevron_up" })
            .with_tooltip(if open { "Fold the log" } else { "Show the log" })
            .with_action(BAction::Toggle(AppKey::StatusLogOpen.path()));
        BView::log(open, toggle)
    }
}
