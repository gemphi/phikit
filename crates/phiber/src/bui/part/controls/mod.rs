//! Controls view — header control cluster (Blender space_view3d header).
//!
//! Renders the store's static `CtrlCluster`: orientation dropdown, snap
//! dropdown, snap/proportional toggles, shading quad, overlay/gizmo toggles.

use super::super::bview::BAction;
use super::super::theme::BPalette;
use super::super::{BKey, BState, BView};
use crate::store::hosts::BHostType;
use crate::store::parts::{BPartType, controls};

pub struct ControlsView<'a> {
    pub host: BHostType,
    pub state: &'a BState,
}

impl<'a> ControlsView<'a> {
    pub fn new(host: BHostType, state: &'a BState) -> Self {
        Self { host, state }
    }

    fn dropdown(&self, items: &'static [controls::CtrlEntry], key: &str, value: BKey) -> BView {
        let host = self.host;
        let dk = BKey::Dropdown {
            host,
            part: BPartType::Controls,
            key: key.to_string(),
        };
        let is_open = self.state.get(&dk) == Some("true");
        let current = self.state.get(&value);
        let pick = items
            .iter()
            .find(|e| current == Some(e.idx))
            .or_else(|| items.iter().find(|e| e.active))
            .or_else(|| items.first());
        let children: Vec<BView> = items
            .iter()
            .map(|e| {
                BView::button()
                    .with_icon(e.icon)
                    .with_label(e.label)
                    .with_action(BAction::Set(value.clone(), e.idx.to_string()))
            })
            .collect();
        BView::dropdown_action(
            pick.map(|e| e.label).unwrap_or(""),
            pick.map(|e| e.icon.to_string()),
            children,
            is_open,
            BAction::Toggle(dk),
        )
    }

    pub fn build(self) -> BView {
        let host = self.host;
        let state = self.state;
        let cluster = controls::set::with_idx(host);
        if cluster.orientation.is_empty() {
            return BView::empty();
        }
        let p = BPalette::from_state(state);

        let mut items: Vec<BView> = Vec::new();

        // Transform orientation + snap element dropdowns.
        items.push(self.dropdown(cluster.orientation, "orientation", BKey::CtxOrientation));
        items.push(self.dropdown(
            cluster.snap,
            "snap",
            BKey::MenuItem {
                host,
                part: BPartType::Controls,
                label: "snap".to_string(),
            },
        ));

        // Snap magnet + proportional editing toggles.
        for t in cluster.toggles {
            let k = BKey::MenuItem {
                host,
                part: BPartType::Controls,
                label: t.idx.to_string(),
            };
            items.push(
                BView::button()
                    .with_icon(t.icon)
                    .with_tooltip(t.label)
                    .with_active(state.get(&k) == Some("true"))
                    .with_action(BAction::Toggle(k)),
            );
        }

        items.push(BView::vseparator());

        // Shading quad (wireframe / solid / material / rendered).
        let shading = state.get(&BKey::CtxShading);
        for s in cluster.shading {
            let active = shading == Some(s.idx) || (shading.is_none() && s.active);
            items.push(
                BView::button()
                    .with_icon(s.icon)
                    .with_tooltip(s.label)
                    .with_active(active)
                    .with_action(BAction::Set(BKey::CtxShading, s.idx.to_string())),
            );
        }

        items.push(BView::vseparator());

        // Overlay / gizmo visibility toggles (default on).
        for t in cluster.view_toggles {
            let k = BKey::MenuItem {
                host,
                part: BPartType::Controls,
                label: t.idx.to_string(),
            };
            let active = match state.get(&k) {
                Some("true") => true,
                Some("false") => false,
                _ => t.active,
            };
            items.push(
                BView::button()
                    .with_icon(t.icon)
                    .with_tooltip(t.label)
                    .with_active(active)
                    .with_action(BAction::Toggle(k)),
            );
        }

        BView::row(Some(p.gap_xs), items)
    }
}
