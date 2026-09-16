//! BAction dispatch — translates actions into get/set on hierarchical keys.

use super::PhiberApp;
use crate::bui::bview::BAction;
use crate::bui::state::BKey;
use crate::store::hosts::BHostType;
use crate::store::modals::BModalType;
use crate::store::spaces::BSpaceType;
use crate::store::zones::BZoneType;
use gpui::Context;

impl PhiberApp {
    pub fn handle_action(&mut self, action: &BAction, cx: &mut Context<Self>) {
        match action {
            BAction::Set(key, val) => self.apply_set(key, val),
            BAction::Toggle(key) => self.apply_toggle(key),
            BAction::Open(key) => self.apply_open(key),
            BAction::Resize(key, pos) => self.apply_resize(key, *pos as f32),
            BAction::OpenFile(_path) => {
                self.state.hash.remove(&BKey::Modal.key());
                self.close_dropdowns(None);
            }
            BAction::Orbit { x, y } => self.apply_orbit(*x as f32, *y as f32),
            BAction::ZoomDelta(d) => self.apply_zoom(*d as f32),
        }
        cx.notify();
    }

    fn apply_set(&mut self, key: &BKey, val: &str) {
        match key {
            BKey::CtxSpace => {
                if let Some(space) = BSpaceType::from_name(val) {
                    self.state.set(&BKey::CtxSpace, val);
                    let space_spec = space.with_set();
                    if let Some(center) = space_spec
                        .items
                        .iter()
                        .find(|a| a.zone == BZoneType::Center)
                    {
                        self.state
                            .set(&BKey::CtxHost, &format!("{:?}", center.default_host));
                    }
                    // Clear all area host overrides so each area resets to its default host.
                    let area_keys: Vec<String> = self
                        .state
                        .hash
                        .keys()
                        .filter(|k| k.starts_with("ui::area_host::"))
                        .cloned()
                        .collect();
                    for k in area_keys {
                        self.state.hash.remove(&k);
                    }
                    self.close_dropdowns(None);
                }
            }
            BKey::Modal => {
                if let Some(modal) = BModalType::from_name(val) {
                    let current = self.state.get(&BKey::Modal).and_then(BModalType::from_name);
                    if current == Some(modal) {
                        self.state.hash.remove(&BKey::Modal.key());
                    } else {
                        self.state.set(&BKey::Modal, modal.name());
                    }
                }
                self.close_dropdowns(None);
            }
            BKey::CtxTool => {
                self.state.set(&BKey::CtxTool, val);
                self.close_dropdowns(None);
            }
            BKey::CtxMode => {
                self.state.set(&BKey::CtxMode, val);
                self.close_dropdowns(None);
            }
            BKey::SwitcherPreviewHost => {
                // Set the preview host — filters modes column, does NOT close popup.
                self.state.set(&BKey::SwitcherPreviewHost, val);
            }
            BKey::SwitcherApplyHost => {
                // Apply the host switch from the switcher — switches the actual area host
                // and sets CtxHost, but does NOT close the switcher popup.
                let new_host_name = val.to_string();
                // Find the currently displayed host from CtxHost (the source of truth).
                let old_name = self.state.get(&BKey::CtxHost).map(|s| s.to_string());
                if let Some(old_name) = old_name {
                    if let Some(old_host) = BHostType::from_name(&old_name) {
                        let slot = self
                            .state
                            .hash
                            .keys()
                            .filter(|k| k.starts_with("ui::area_host::"))
                            .find(|k| self.state.hash.get(*k).is_some_and(|v| v == &old_name))
                            .cloned();
                        match slot {
                            Some(k) => {
                                self.state.hash.insert(k, new_host_name.clone());
                            }
                            None => {
                                self.state.set(&BKey::AreaHost(old_host), &new_host_name);
                            }
                        }
                    }
                }
                self.state.set(&BKey::CtxHost, &new_host_name);
            }
            BKey::AreaHost(old_host) => {
                // The switcher sends AreaHost(currently_displayed_host), but area
                // slots are keyed by AreaHost(default_host). Find the slot whose
                // current value matches old_host and update it. If no slot has an
                // override (old_host is still the default), create one.
                let old_name = format!("{:?}", old_host);
                let slot = self
                    .state
                    .hash
                    .keys()
                    .filter(|k| k.starts_with("ui::area_host::"))
                    .find(|k| self.state.hash.get(*k).is_some_and(|v| v == &old_name))
                    .cloned();
                match slot {
                    Some(k) => {
                        self.state.hash.insert(k, val.to_string());
                    }
                    None => {
                        self.state.set(&BKey::AreaHost(*old_host), val);
                    }
                }
                self.state.set(&BKey::CtxHost, val);
                self.close_dropdowns(None);
            }
            BKey::FilePath | BKey::FileSelected => {
                self.state.set(key, val);
            }
            _ => {
                self.state.set(key, val);
                self.close_dropdowns(None);
            }
        }
    }

    fn apply_toggle(&mut self, key: &BKey) {
        match key {
            BKey::Modal => {
                self.state.hash.remove(&BKey::Modal.key());
            }
            BKey::Dropdown { .. } => {
                let k = key.key();
                let cur = self.state.hash.get(&k).map(|s| s.as_str()) == Some("true");
                let new_val = if cur {
                    "false".to_string()
                } else {
                    "true".to_string()
                };
                self.state.hash.insert(k.clone(), new_val);
                self.close_dropdowns(Some(&k));
            }
            _ => {
                let k = key.key();
                let cur = self.state.hash.get(&k).map(|s| s.as_str()) == Some("true");
                self.state.hash.insert(
                    k,
                    if cur {
                        "false".to_string()
                    } else {
                        "true".to_string()
                    },
                );
                self.close_dropdowns(None);
            }
        }
    }

    /// Open a dropdown and close all others (menubar hover-to-switch).
    fn apply_open(&mut self, key: &BKey) {
        let k = key.key();
        if self.state.hash.get(&k).map(|s| s.as_str()) == Some("true") {
            return;
        }
        self.state.hash.insert(k.clone(), "true".to_string());
        self.close_dropdowns(Some(&k));
    }

    fn apply_resize(&mut self, key: &BKey, pos: f32) {
        let (min, max) = match key {
            BKey::LeftWidth => (120., 600.),
            BKey::RightWidth => (120., 600.),
            BKey::RightTopHeight => (120., 800.),
            BKey::BottomHeight => (40., 400.),
            _ => return,
        };
        self.state.set(key, &pos.max(min).min(max).to_string());
    }

    /// MMB-drag orbit: delta from the stored drag origin drives yaw/pitch.
    fn apply_orbit(&mut self, x: f32, y: f32) {
        let f = |k: &BKey| self.state.get(k).and_then(|v| v.parse::<f32>().ok());
        if let (Some(lx), Some(ly)) = (f(&BKey::CamDragX), f(&BKey::CamDragY)) {
            let yaw = f(&BKey::CamYaw).unwrap_or(45.0) + (x - lx) * 0.4;
            let pitch = (f(&BKey::CamPitch).unwrap_or(30.0) + (y - ly) * 0.4).clamp(-89.0, 89.0);
            self.state.set(&BKey::CamYaw, &yaw.to_string());
            self.state.set(&BKey::CamPitch, &pitch.to_string());
        }
        self.state.set(&BKey::CamDragX, &x.to_string());
        self.state.set(&BKey::CamDragY, &y.to_string());
    }

    /// Wheel zoom: exponential scale, clamped.
    fn apply_zoom(&mut self, d: f32) {
        let zoom = self
            .state
            .get(&BKey::CamZoom)
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(1.0);
        let zoom = (zoom * if d > 0.0 { 1.1 } else { 1.0 / 1.1 }).clamp(0.2, 10.0);
        self.state.set(&BKey::CamZoom, &zoom.to_string());
    }

    /// Close all open dropdowns. If `except` is provided, skip that key
    /// and also skip any of its ancestor dropdowns (parent menus).
    fn close_dropdowns(&mut self, except: Option<&str>) {
        // Extract the key portion after "dropdown::" to determine ancestry.
        // Top-level keys look like: "...dropdown::::View"  (key portion = "::View")
        // Submenu keys look like:   "...dropdown::View::View Regions" (key portion = "View::View Regions")
        // A submenu's ancestor labels are the segments before "::" in the key portion.
        let ancestors: Vec<String> = except
            .and_then(|k| {
                let pos = k.find("dropdown::")?;
                let key_part = &k[pos + "dropdown::".len()..];
                if key_part.starts_with("::") {
                    // Top-level — no ancestors.
                    return Some(vec![]);
                }
                // Submenu — collect ancestor dropdown keys.
                // Segments: "View::View Regions" → ancestor is "View" → key portion "::View"
                let segments: Vec<&str> = key_part.split("::").collect();
                let mut acc = Vec::new();
                for i in 1..segments.len() {
                    let ancestor_key_part = format!("::{}", segments[..i].join("::"));
                    acc.push(format!("{}dropdown::{}", &k[..pos], ancestor_key_part));
                }
                Some(acc)
            })
            .unwrap_or_default();

        let keys: Vec<String> = self
            .state
            .hash
            .keys()
            .filter(|k| k.contains("::dropdown::"))
            .filter(|k| self.state.hash.get(*k).map(|s| s.as_str()) == Some("true"))
            .filter(|k| Some(k.as_str()) != except)
            .filter(|k| !ancestors.contains(k))
            .cloned()
            .collect();
        for k in keys {
            self.state.hash.insert(k, "false".to_string());
        }
    }

    /// Close all dropdowns — called on global click outside.
    pub fn close_all_dropdowns(&mut self) {
        self.close_dropdowns(None);
    }
}
