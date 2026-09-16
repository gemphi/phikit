//! Space view — top-level layout coordinator with full-height Activity Bar.
//! Corresponds to store/spaces/.

use super::super::bview::{BAction, BView};
use super::super::modal::ModalView;
use super::super::part::activity::ActivityView;
use super::super::state::{BKey, BState};
use super::super::theme::BPalette;
use super::BViewBuilder;
use super::zone::ZoneView;
use crate::store::BPanelRole;
use crate::store::modals::BModalType;
use crate::store::spaces::BSpaceType;
use crate::store::zones::BZoneType;

/// The active workspace space (Layout, Modeling, …).
#[derive(Clone, Copy, Debug, Default)]
pub struct SpaceView;

impl SpaceView {
    pub fn new() -> Self {
        Self
    }

    fn zone_togglers(state: &BState, space: BSpaceType, p: &BPalette) -> BView {
        let items: Vec<BView> = space
            .zones()
            .iter()
            .filter(|z| z.is_toggleable())
            .map(|&zone| {
                BView::button()
                    .with_icon(zone.icon())
                    .with_tooltip(zone.tooltip())
                    .with_action(BAction::Toggle(BKey::Zone(zone)))
                    .with_active(ZoneView::new(zone).is_visible(state))
            })
            .collect();
        BView::row(Some(p.gap_xs), items)
    }
}

impl BViewBuilder for SpaceView {
    fn build(&self, state: &BState) -> BView {
        let space_type = state
            .get(&BKey::CtxSpace)
            .and_then(BSpaceType::from_name)
            .unwrap_or(BSpaceType::Layout);
        let space = space_type.with_set();
        let p = BPalette::from_state(state);

        // Pure composition: loop the store's area specs into zone slots.
        // Zones render themselves; the space only places them and triggers.
        let zone_view = |z: BZoneType| {
            ZoneView::new(z).with_areas(
                space
                    .items
                    .iter()
                    .filter(|a| a.zone == z)
                    .copied()
                    .collect(),
            )
        };

        let mut children: Vec<BView> = Vec::new();

        // 1. Top strip — zone renders itself, space attaches its toggle chrome.
        let togglers = Self::zone_togglers(state, space_type, &p);
        let top = zone_view(BZoneType::Top)
            .with_chrome(togglers.clone())
            .build(state);
        if matches!(top, BView::Empty) {
            children.push(togglers);
        } else {
            children.push(top);
        }

        // 2. Full-screen zone takes over the work area when present.
        let full = zone_view(BZoneType::Full).build(state);
        if !matches!(full, BView::Empty) {
            children.push(full);
        } else {
            // Middle row: Left | Center | Right.
            let mut main_row: Vec<BView> = Vec::new();
            for z in [BZoneType::Left, BZoneType::Center, BZoneType::Right] {
                let v = zone_view(z).build(state);
                if !matches!(v, BView::Empty) {
                    main_row.push(v);
                }
            }
            if !main_row.is_empty() {
                children.push(BView::panel(BPanelRole::Work, "", main_row));
            }

            // 3. Bottom strips (Timeline…, global Statusbar) span full width.
            let bottom = zone_view(BZoneType::Bottom).build(state);
            if !matches!(bottom, BView::Empty) {
                children.push(bottom);
            }
        }

        // 4. Modal overlay (space-level trigger).
        if let Some(modal) = state.get(&BKey::Modal).and_then(BModalType::from_name) {
            children.push(ModalView::new(modal, state).build());
        }

        // Main content (everything except the activity bar).
        let main_content = BView::panel(BPanelRole::CenterArea, "", children);

        // Extreme-left full-height Activity Bar + main content, side by side.
        let activity_bar = ActivityView::new(state).build();
        BView::panel(BPanelRole::App, "", vec![activity_bar, main_content])
    }
}
