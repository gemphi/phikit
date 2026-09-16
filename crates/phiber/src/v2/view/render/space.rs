//! Space-level layout: the app root. One space is always active; it stacks
//! its zones vertically (Top strip → Middle band → Bottom strip → global
//! Statusbar).

use super::{BAxis, BSpaceLayout};
use crate::v2::space::BSpaceType;

impl BSpaceLayout for BSpaceType {
    /// Space has no per-space layout choices — every space is a column of
    /// zones. Zone kinds control their own children's orientation.
    fn axis(&self) -> BAxis {
        BAxis::Column
    }
}
