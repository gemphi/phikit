//! Activity host — Phiber-native global navigation rail. Region declarations
//! only; the icon-strip content lives in `part::custom::activity` via the
//! `Custom` part. A single Center region fills the host's full height.

use crate::v2::pane::{BPaneSpec, BPaneType};
use crate::v2::part::BPartType;
use crate::v2::region::{BRegionSpec, BRegionType};

static CENTER_PANES: &[BPaneSpec] = &[BPaneSpec {
    kind: BPaneType::Center,
    parts: &[BPartType::Custom],
}];

pub static REGIONS: &[BRegionSpec] = &[BRegionSpec {
    kind: BRegionType::Center,
    panes: CENTER_PANES,
}];
