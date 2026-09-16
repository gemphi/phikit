//! view3d host constants.

use crate::store::regions::{BRegionSpec, BRegionType};

pub static REGIONS: &[BRegionSpec] = &[
    BRegionSpec::new(BRegionType::Header),
    BRegionSpec::new(BRegionType::Center),
    BRegionSpec::new(BRegionType::Footer),
];
