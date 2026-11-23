//! A pane: a left, centre or right slot inside a region. `PPane`.

/// Where in its region a pane sits. The left and the right hug their
/// edges; the centre takes what is left.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PSide {
    Left,
    #[default]
    Center,
    Right,
}

impl PSide {
    /// The side a configuration file names: `left`, `center`, `right`.
    pub fn named(name: &str) -> Option<PSide> {
        match name {
            "left" => Some(PSide::Left),
            "center" => Some(PSide::Center),
            "right" => Some(PSide::Right),
            _ => None,
        }
    }
}

/// A left, centre or right slot inside a region.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PPane {
    pub side: PSide,
    /// The names of the parts it holds, in order.
    pub part: Vec<String>,
}

impl PPane {
    pub fn new(side: PSide, part: &[&str]) -> PPane {
        PPane { side, part: part.iter().map(|name| name.to_string()).collect() }
    }
}
