//! A region: a slice of a host, such as its header, its centre, its footer.
//! `PRegion`.

use super::PPane;

/// A slice of a host: its header, its centre, its footer.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PRegion {
    /// Which slice: `header`, `center`, `footer`.
    pub name: String,
    /// Its panes, from left to right.
    pub pane: Vec<PPane>,
}

impl PRegion {
    pub fn new(name: &str, pane: Vec<PPane>) -> PRegion {
        PRegion { name: name.to_string(), pane }
    }
}
