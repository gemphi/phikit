//! The log drawer's columns, declared as a set.
//!
//! Folded, the status bar shows one line and there is nothing to order.
//! Unfolded it is a table of everything that has happened, and a table of
//! five hundred rows with no headings is a wall — you cannot find the one
//! event you are after, and you cannot tell what part of a row is what.
//! So the drawer gets the same thing the File Browser's list has: headings
//! you can click to order by (`editors/file_browser`, "Sorting").
//!
//! Declared here rather than written into the renderer, for the reason
//! every other part of phiber is declared: the columns, their widths and
//! their order are read in three places (the heading row, the cells, the
//! ordering) and must agree in all three.

use super::BLogOrder;

/// One column of the unfolded log.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BLogColumn {
    /// What the ordering state stores.
    pub order: BLogOrder,
    pub label: &'static str,
    /// Share of the drawer's width.
    pub share: f32,
    /// Whether clicking the heading orders by it. The sequence number and
    /// the time are the same ordering — the log is appended in time order —
    /// so only one of them carries it.
    pub orders: bool,
}

const fn col(order: BLogOrder, label: &'static str, share: f32, orders: bool) -> BLogColumn {
    BLogColumn { order, label, share, orders }
}

/// The three the log has. `#` is first because it is fixed-width and the
/// eye uses it as the left rule; `Event` takes what is left.
pub static COLUMNS: &[BLogColumn] = &[
    col(BLogOrder::Seq, "#", 0.08, true),
    col(BLogOrder::When, "When", 0.14, true),
    col(BLogOrder::Event, "Event", 0.78, true),
];
