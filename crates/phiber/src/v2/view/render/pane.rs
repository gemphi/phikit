//! Pane-level layout: tri-cell horizontal grid. Left hugs left, Center
//! flexes to fill, Right hugs right. Same shape at every region level.

use crate::v2::pane::BPaneType;

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
