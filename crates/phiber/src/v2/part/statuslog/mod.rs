//! The status bar's action log — the `StatusLog` part.
//!
//! Folded, it is one line: the newest thing that happened. Unfolded, it is
//! a table of every line — `#`, `When`, `Event` — with headings you can
//! click to order by, in a drawer that slides up over the workspace from
//! the status bar and back down again. The lines come from `state::log`,
//! which describes each action the app applied.
//!
//! Two things about the control, both asked for and both worth keeping:
//!
//! - **One glyph, not two.** It was a `chevron_up` that became a
//!   `chevron_down`, so the button changed identity as you used it and the
//!   icon only ever said where the drawer was going, never that it was a
//!   two-way control at all. It is now the double chevron — apart to
//!   open, together to close — the same pair the rest of the app uses for
//!   expand/collapse.
//! - **The whole row opens it.** The chevron is a small target in a thin
//!   bar; clicking the status bar itself did nothing, which reads as the
//!   bar refusing to open. The head row carries the same toggle now, so
//!   the line you are trying to read is the thing you click.

pub mod set;

use crate::v2::host::BHostType;
use crate::v2::state::AppKey;
use crate::v2::state::BKey as _;
use crate::v2::state::BPath;
use crate::v2::state::BState;
use crate::v2::state::log::BLogEntry;
use crate::v2::state::val::{V_FALSE, V_TRUE};
use crate::v2::view::BView;
use crate::v2::view::BViewItem;
use crate::v2::view::prim::BAction;
use crate::v2::view::prim::BControl;

pub use set::{BLogColumn, COLUMNS};

/// What the unfolded log is ordered by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BLogOrder {
    /// The sequence number — the order things happened in.
    Seq,
    /// When it happened. The same order as `Seq`, read the other way round.
    When,
    /// The text, alphabetically: for finding one kind of event among many.
    Event,
}

impl BLogOrder {
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Seq => "seq",
            Self::When => "when",
            Self::Event => "event",
        }
    }

    pub fn of(slug: &str) -> Self {
        match slug {
            "when" => Self::When,
            "event" => Self::Event,
            _ => Self::Seq,
        }
    }
}

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

/// `status.log.order` — which column the table is ordered by.
pub fn order_key() -> BPath {
    AppKey::StatusLogOrder.path()
}

/// `status.log.ascending` — the direction. Newest first is the default,
/// because the newest line is the one the folded bar was showing.
pub fn ascending_key() -> BPath {
    AppKey::StatusLogAscending.path()
}

pub fn order(s: &BState) -> BLogOrder {
    BLogOrder::of(s.get(&order_key()).unwrap_or(""))
}

pub fn ascending(s: &BState) -> bool {
    s.get(&ascending_key()) == Some(V_TRUE)
}

/// Clicking a heading orders by it; clicking the one already ordered by
/// turns it round. That is what every list with sortable headings does,
/// and it is the only way to reach ascending with one control per column.
pub fn heading_action(s: &BState, column: BLogColumn) -> Option<BAction> {
    if !column.orders {
        return None;
    }
    if order(s) == column.order {
        return Some(BAction::Set(ascending_key(), match ascending(s) {
            true => V_FALSE.to_string(),
            false => V_TRUE.to_string(),
        }));
    }
    Some(BAction::Set(order_key(), column.order.slug().to_string()))
}

/// The entries in the order the headings ask for.
///
/// `state::log` keeps them newest-first, which is `Seq` descending — the
/// default — so that case does no work.
pub fn sorted(entries: &[BLogEntry], by: BLogOrder, ascending: bool) -> Vec<&BLogEntry> {
    let mut out: Vec<&BLogEntry> = entries.iter().collect();
    match by {
        // Time and sequence are the same order: the log is appended as
        // things happen, so sorting by one would sort by the other.
        BLogOrder::Seq | BLogOrder::When => out.sort_by_key(|e| e.seq),
        BLogOrder::Event => out.sort_by(|a, b| a.text.to_lowercase().cmp(&b.text.to_lowercase()).then(a.seq.cmp(&b.seq))),
    }
    if !ascending {
        out.reverse();
    }
    out
}

impl BViewItem for BStatusLog {
    fn build(&self, s: &BState) -> BView {
        let open = is_open(s);
        let toggle = BControl::new()
            // Apart to open, together to close — one control that says it
            // is a two-way control whichever way it is pointing.
            .with_icon(if open { "chevrons_down_up" } else { "chevrons_up_down" })
            .with_tooltip(if open { "Fold the log" } else { "Show every event" })
            // Lit while the drawer is down, as the asset shelf's identical
            // chevron fold already was (`on_state_budget`).
            .with_active(open)
            .with_action(BAction::Toggle(AppKey::StatusLogOpen.path()));
        BView::log(open, toggle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(seq: u64, text: &str) -> BLogEntry {
        BLogEntry { seq, text: text.to_string(), at: web_time::Instant::now() }
    }

    #[test]
    fn the_log_opens_newest_first_and_that_costs_no_sorting() {
        // `state::log` already keeps them newest-first.
        let entries = vec![entry(3, "c"), entry(2, "b"), entry(1, "a")];
        let out = sorted(&entries, BLogOrder::Seq, false);
        assert_eq!(out.iter().map(|e| e.seq).collect::<Vec<_>>(), vec![3, 2, 1]);
    }

    #[test]
    fn time_and_sequence_are_one_order_read_two_ways() {
        let entries = vec![entry(3, "c"), entry(2, "b"), entry(1, "a")];
        for by in [BLogOrder::Seq, BLogOrder::When] {
            assert_eq!(sorted(&entries, by, true).iter().map(|e| e.seq).collect::<Vec<_>>(), vec![1, 2, 3]);
        }
    }

    #[test]
    fn ordering_by_event_is_alphabetical_and_ties_keep_their_sequence() {
        let entries = vec![entry(3, "Render failed"), entry(2, "apple"), entry(1, "Apple")];
        let out = sorted(&entries, BLogOrder::Event, true);
        // Case-insensitive, so the two apples sit together, oldest first.
        assert_eq!(out.iter().map(|e| e.seq).collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[test]
    fn a_heading_orders_by_itself_then_turns_itself_round() {
        let mut s = BState::new();
        let event = COLUMNS.iter().find(|c| c.order == BLogOrder::Event).copied().expect("an Event column");
        // First click: order by it.
        let Some(BAction::Set(path, value)) = heading_action(&s, event) else { panic!("expected a Set") };
        assert_eq!(path, order_key());
        s.set(&path, value);
        assert_eq!(order(&s), BLogOrder::Event);
        // Second click on the same heading: turn it round, not re-order.
        let Some(BAction::Set(path, value)) = heading_action(&s, event) else { panic!("expected a Set") };
        assert_eq!(path, ascending_key());
        s.set(&path, value);
        assert!(ascending(&s), "the second click reverses it");
    }

    #[test]
    fn the_columns_fill_the_drawer_exactly_once() {
        let total: f32 = COLUMNS.iter().map(|c| c.share).sum();
        assert!((total - 1.0).abs() < 0.001, "shares must cover the width exactly, got {total}");
        assert_eq!(COLUMNS.len(), 3);
    }
}
