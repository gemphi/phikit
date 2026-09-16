//! statusbar host constants.

use crate::store::regions::{BRegionSpec, BRegionType};

pub static REGIONS: &[BRegionSpec] = &[BRegionSpec::new(BRegionType::Footer)];
