//! Preferences ▸ System ▸ Render Devices — which GPU phiray draws and
//! skins on, as Blender's System section lists its devices (`Cycles Render
//! Devices`, and Display Graphics ▸ Device with its Auto). Auto takes the
//! best the machine has (phiray `gpu/adapter.rs`: a discrete GPU before an
//! integrated one, a software one last); a card picked here is opened at
//! once and the viewport moves to it (`Gpu::switch`) — Blender asks for a
//! restart. The line under the list says which device is drawing.

use crate::v2::host::BHostType;
use crate::v2::part::props::BPanelItem;
use crate::v2::state::val::V_FALSE;
use crate::v2::state::{AppKey, BKey as _, BPath, BState, log, rule::BRule};
use crate::v2::view::BView;
use crate::v2::view::prim::{BAction, BCard, BControl};
use phiray::{Gpu, GpuAdapter, GpuAdapters};

/// The Render Devices panel.
pub struct BRenderDevices;

impl BRenderDevices {
    const TITLE: &'static str = "Render Devices";

    /// A row: what it is, and its Use / Active button.
    fn row(label: String, active: bool, pick: &str) -> BView {
        let button = BControl::new()
            .with_icon(if active { "check" } else { "" })
            .with_label(if active { "Active" } else { "Use" })
            .with_active(active)
            .with_action(BAction::Set(AppKey::Gpu.path(), pick.to_string()));
        BView::split(vec![BView::label(label), BView::Button(button)])
    }

    /// What is drawing now.
    fn status() -> String {
        match Gpu::in_use() {
            Some(a) => format!("Drawing on {} — {}, {}, {} MB per binding", a.name, a.kind_label(), a.backend_label(), a.binding >> 20),
            None if GpuAdapters::all().is_empty() => "No GPU found: the viewport draws on the CPU".to_string(),
            None => "Opening the device…".to_string(),
        }
    }

    /// The Render tab's Device row (`cycles.device` in Blender, CPU or GPU
    /// Compute): here the card phiray draws on — Auto, naming the card it
    /// takes, or any card the machine has — the same choice as this panel,
    /// over `ctx.gpu`. The file's Cycles setting is not what draws.
    pub fn control(open: BPath, s: &BState) -> crate::v2::view::prim::BCtrl {
        use crate::v2::view::prim::{BCtrl, BCtrlKind, BOption};
        let found = GpuAdapter::distinct(GpuAdapters::all());
        let auto = Gpu::in_use().filter(|_| s.get(AppKey::Gpu).unwrap_or("").is_empty()).or_else(|| GpuAdapter::pick(&found, None).cloned());
        let option = |key: &str, label: String| BOption { key: key.to_string(), label, icon: String::new() };
        let mut options = vec![option("", auto.map_or("Auto".to_string(), |a| format!("Auto — {}", a.name)))];
        options.extend(found.iter().map(|a| option(&a.name, format!("{} — {}", a.name, a.kind_label()))));
        BCtrl::read(BCtrlKind::Select, AppKey::Gpu.path(), crate::v2::state::val::BValue::Enum(String::new()), s).with_options(options).with_menu(open, s)
    }

    pub fn view(state: &BState) -> BView {
        let key = BPanelItem::expand_key(BHostType::Prefs, "System", Self::TITLE);
        let open = state.get(&key) != Some(V_FALSE);
        if !open {
            return BCard::panel(Self::TITLE).fold(key, false).into();
        }
        let chosen = state.get(AppKey::Gpu).unwrap_or("");
        let found = GpuAdapter::distinct(GpuAdapters::all());
        let best = GpuAdapter::pick(&found, None).map_or(String::new(), |a| format!(" ({})", a.name));
        let mut rows = vec![Self::row(format!("Auto{best}"), chosen.is_empty(), "")];
        rows.extend(found.iter().map(|a| Self::row(format!("{} — {}, {}", a.name, a.kind_label(), a.backend_label()), chosen == a.name, &a.name)));
        rows.push(BView::label(Self::status()));
        BCard::panel(Self::TITLE).fold(key, true).body(rows).into()
    }
}

/// `ctx.gpu` → the shared device opened on that card (empty: Auto), on a
/// thread of its own; the viewport moves to it on its next frame.
pub struct BRenderDeviceRule;

impl BRule for BRenderDeviceRule {
    fn wants(&self, path: &BPath) -> bool {
        *path == AppKey::Gpu.path()
    }
    fn after(&self, _: &BPath, value: &str, _: &mut BState) {
        let preferred = (!value.is_empty()).then(|| value.to_string());
        log::push(format!("Render device: {}", preferred.as_deref().unwrap_or("Auto")));
        // Already drawing on the card it would pick: nothing to reopen.
        let pick = GpuAdapter::pick(GpuAdapters::all(), preferred.as_deref()).cloned();
        if pick.is_some() && pick == Gpu::in_use() {
            return GpuAdapters::prefer(preferred);
        }
        let _ = std::thread::Builder::new().name("phiray.switch".to_string()).spawn(move || Gpu::switch(preferred));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v2::state::action::apply_action;

    fn texts(view: &BView, out: &mut Vec<String>) {
        match view {
            BView::Label { text } => out.push(text.to_string()),
            BView::Button(c) => out.push(format!("[{}]", c.label.as_deref().unwrap_or(""))),
            BView::Card(card) => card.body.iter().for_each(|v| texts(v, out)),
            BView::Split { list } | BView::Row { list } | BView::Column { list } => list.iter().for_each(|v| texts(v, out)),
            _ => {}
        }
    }

    /// On the machine it runs on (the cloud's software Vulkan here): Auto
    /// names the card it takes, each card its kind and backend, the line
    /// under them the device drawing; picking a card says so, and picking
    /// the one already drawing reopens nothing.
    #[test]
    fn the_panel_lists_the_machines_gpus_and_says_which_one_draws() {
        let Some(gpu) = Gpu::shared() else { return };
        let mut state = BState::new();
        let mut shown = Vec::new();
        texts(&BRenderDevices::view(&state), &mut shown);
        let a = &gpu.adapter;
        assert_eq!(shown[..2], [format!("Auto ({})", a.name), "[Active]".to_string()]);
        assert!(shown.contains(&format!("{} — {}, {}", a.name, a.kind_label(), a.backend_label())), "{shown:?}");
        assert!(shown.last().is_some_and(|l| l.starts_with(&format!("Drawing on {}", a.name))), "{shown:?}");

        let generation = Gpu::generation();
        apply_action(&mut state, &BAction::Set(AppKey::Gpu.path(), a.name.clone()));
        assert_eq!(log::latest().map(|e| e.text.clone()), Some(format!("Render device: {}", a.name)));
        assert_eq!(Gpu::generation(), generation, "already drawing on it: nothing reopened");
        let mut shown = Vec::new();
        texts(&BRenderDevices::view(&state), &mut shown);
        assert_eq!(shown[1], "[Use]", "Auto no longer the choice");
        GpuAdapters::prefer(None);
    }
}
