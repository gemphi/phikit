//! The bottom area's editor tabs — Reality Composer Pro's strip over its
//! bottom panel (Project Browser | Shader Graph | Behaviors | Audio Mixer
//! | Statistics), here over the editors a bottom area can show (the
//! Timeline, the Dope Sheet, the Graph Editor, the Drivers, the NLA, Play,
//! the Project Browser): one tab per editor, centred, the one showing
//! marked, a click switching the area to it — the switcher's own write
//! (`area::current::area_key`), so either way the area remembers one host.
//!
//! An area with one host, or outside the bottom zone, has no strip.

use crate::v2::area::current;
use crate::v2::host::BHostType;
use crate::v2::state::BState;
use crate::v2::view::prim::{BAction, BControl};
use crate::v2::view::{BView, BViewItem};
use crate::v2::zone::BZoneType;

#[derive(Clone, Copy, Debug)]
pub struct BAreaTabs {
    pub host: BHostType,
}

impl BAreaTabs {
    pub fn new(host: BHostType) -> Self {
        Self { host }
    }

    /// Whether the area being built carries the strip.
    pub fn shown() -> bool {
        current::current_area().is_some_and(|area| area.zone == BZoneType::Bottom && area.hosts.len() > 1)
    }
}

impl BViewItem for BAreaTabs {
    fn build(&self, _: &BState) -> BView {
        let (Some(scope), Some(area)) = (current::current_scope(), current::current_area()) else {
            return BView::empty();
        };
        let slot = current::area_key(scope, &area);
        let tabs = area
            .hosts
            .iter()
            .map(|&host| {
                BView::Tab(
                    BControl::new()
                        .with_icon(host.icon())
                        .with_label(host.label())
                        .with_active(host == self.host)
                        .with_action(BAction::Set(slot.clone(), host.slug().to_string())),
                )
            })
            .collect::<Vec<_>>();
        // Centred on the strip, as every tab strip is: a full-width row with
        // the slack split either side.
        BView::list_row([vec![BView::spacer()], tabs, vec![BView::spacer()]].concat(), crate::v2::view::prim::BRowShade::Flat)
    }
}
