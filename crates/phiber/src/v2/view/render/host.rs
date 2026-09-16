//! Host-level layout: hosts stack their regions vertically (Header on top,
//! ToolHeader below it, Center flexing, Footer at the bottom).
//! Blender ref: `screen_geom.cc` region packing.

use super::{BAxis, BHostLayout};
use crate::v2::host::BHostType;

impl BHostLayout for BHostType {
    /// Hosts always stack their regions vertically. Region kinds decide
    /// their own placement within this column (top-aligned Header /
    /// bottom-aligned Footer / flex Center).
    fn axis(&self) -> BAxis {
        BAxis::Column
    }
}
