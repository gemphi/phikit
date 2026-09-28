//! The activity bar's keys — Phiber's own rail, under `ctx.activity.*`.

use super::BKey;
use crate::v2::state::BPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActivityKey {
    /// The entry last chosen (its label).
    Tab,
    /// The grouping whose secondary strip is open, if any.
    Group,
    /// An option picked from the secondary strip, `"<entry>:<option>"`,
    /// consumed by `follow`.
    Pick,
}

impl BKey for ActivityKey {
    fn path(&self) -> BPath {
        let tail: &'static str = match self {
            Self::Tab => "tab",
            Self::Group => "group",
            Self::Pick => "pick",
        };
        BPath::owned(vec!["activity", tail])
    }
}
