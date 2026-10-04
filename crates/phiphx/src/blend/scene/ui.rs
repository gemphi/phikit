//! "Load UI" (#28) — the workspace and 3D view Blender restores on open.
//!
//! Blender ref (`wm_files.cc` / `DNA_windowmanager_types.h`,
//! `DNA_screen_types.h`, `DNA_view3d_types.h`): `wmWindowManager.windows`
//! → `wmWindow.workspace_hook` → `WorkSpaceInstanceHook { *active,
//! *act_layout }` → `WorkSpaceLayout.screen` → `bScreen.areabase` →
//! `ScrArea { spacetype, totrct, regionbase }` → `ARegion { regiontype,
//! regiondata }` → `RegionView3D { viewinv, ofs, dist, persp, camzoom }`.
//! Older files without a hook fall back to `FileGlobal.curscreen`.
//! When a screen has several 3D views, the largest one wins (the one a
//! user reads as "the viewport").

use crate::blend::file::types::*;

pub const SPACE_VIEW3D: u8 = 1;
pub const RGN_TYPE_WINDOW: u32 = 0;
/// `RGN_TYPE_UI` — the sidebar (N panel).
pub const RGN_TYPE_UI: u32 = 4;
/// `ARegion.flag`: the region is hidden (`RGN_FLAG_HIDDEN`).
pub const RGN_FLAG_HIDDEN: u32 = 1;
/// `RegionView3D.persp`: 0 ortho, 1 perspective, 2 looking through the camera.
pub const RV3D_ORTHO: u8 = 0;
pub const RV3D_CAMOB: u8 = 2;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlendView3d {
    /// `RegionView3D.viewinv` (view → world, `m[col][row]`).
    pub view_inv: [[f32; 4]; 4],
    pub ofs: [f32; 3],
    pub dist: f32,
    pub persp: u8,
    /// `RegionView3D.camzoom` — how far the camera frame is zoomed in the
    /// region while looking through the camera (`BKE_screen_view3d_zoom_to_fac`).
    pub camzoom: f32,
    /// `View3D.lens` (mm; Blender default 50).
    pub lens: f32,
    /// `View3D.shading.type` (`eDrawType`: 2 wire, 3 solid, 4 material, 5 rendered).
    /// `View3D.shading`, whole (`shading.rs`).
    pub shading: super::shading::BlendShading,
    /// The area's sidebar region is shown (`RGN_TYPE_UI` without
    /// `RGN_FLAG_HIDDEN`) — `SpaceView3D.show_region_ui`.
    pub sidebar: bool,
    /// The Overlays and Gizmos popovers' switches (`overlay.rs`).
    pub overlay: super::overlay::BlendOverlay,
    /// The view's own render region (`View3D.render_border`, `border.rs`).
    pub border: super::border::BlendViewBorder,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlendUiState {
    /// Active workspace name without the `WS` prefix ("Layout", "Shading").
    pub workspace: String,
    pub screen: String,
    pub view3d: Option<BlendView3d>,
    /// `Scene.camera` object address (0 when unset).
    pub camera: u64,
}

impl BlendFile {
    fn ui_id_name(&self, b: &BlendBlock, dna: &DnaType) -> String {
        let Some(o) = self.id_name_offset(dna) else { return String::new() };
        let end = (o + 66).min(b.data.len());
        let raw = &b.data[o.min(end)..end];
        let n = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
        String::from_utf8_lossy(&raw[n.min(2)..n]).to_string()
    }

    fn ui_f32s<const N: usize>(&self, b: &BlendBlock, off: Option<usize>) -> Option<[f32; N]> {
        let o = off?;
        if o + 4 * N > b.data.len() { return None; }
        Some(std::array::from_fn(|i| self.read_f32(&b.data[o + 4 * i..o + 4 * i + 4])))
    }

    fn ui_byte(&self, b: &BlendBlock, off: Option<usize>) -> u8 {
        off.and_then(|o| b.data.get(o).copied()).unwrap_or(0)
    }

    pub(crate) fn extract_ui_state(&self, scene: Option<&BlendBlock>, dna: &DnaType) -> BlendUiState {
        let mut ui = BlendUiState::default();
        if let Some(sc) = scene {
            ui.camera = self.ptr(sc, self.off("Scene", "*camera", dna));
        }
        let wm = self.blocks.iter().find(|b| b.code_str == "WM");
        let win = wm.and_then(|w| self.block_at(self.ptr(w, self.off("wmWindowManager", "windows", dna))));
        let hook = win.and_then(|w| self.block_at(self.ptr(w, self.off("wmWindow", "*workspace_hook", dna))));
        let mut screen = None;
        if let Some(h) = hook {
            if let Some(ws) = self.block_at(self.ptr(h, self.off("WorkSpaceInstanceHook", "*active", dna))) {
                ui.workspace = self.ui_id_name(ws, dna);
            }
            let layout = self.block_at(self.ptr(h, self.off("WorkSpaceInstanceHook", "*act_layout", dna)));
            screen = layout.and_then(|l| self.block_at(self.ptr(l, self.off("WorkSpaceLayout", "*screen", dna))));
        }
        if screen.is_none() {
            let glob = self.blocks.iter().find(|b| b.code_str == "GLOB");
            screen = glob.and_then(|g| self.block_at(self.ptr(g, self.off("FileGlobal", "*curscreen", dna))));
        }
        if let Some(sr) = screen {
            ui.screen = self.ui_id_name(sr, dna);
            ui.view3d = self.largest_view3d(sr, dna);
        }
        ui
    }

    fn largest_view3d(&self, sr: &BlendBlock, dna: &DnaType) -> Option<BlendView3d> {
        let (next, ty, rct) = (self.off("ScrArea", "*next", dna), self.off("ScrArea", "spacetype", dna), self.off("ScrArea", "totrct", dna));
        let mut best: Option<(i64, BlendView3d)> = None;
        let mut addr = self.ptr(sr, self.off("bScreen", "areabase", dna));
        let mut guard = 0;
        while let Some(area) = self.block_at(addr) {
            guard += 1;
            if guard > 4096 { break; }
            addr = self.ptr(area, next);
            if self.ui_byte(area, ty) != SPACE_VIEW3D { continue; }
            let r = rct.filter(|o| o + 16 <= area.data.len()).map(|o| {
                let i = |k: usize| self.read_i32(&area.data[o + 4 * k..o + 4 * k + 4]) as i64;
                (i(1) - i(0)).max(0) * (i(3) - i(2)).max(0)
            });
            let size = r.unwrap_or(0);
            if best.as_ref().is_some_and(|(s, _)| *s >= size) { continue; }
            if let Some(v) = self.area_view3d(area, dna) {
                best = Some((size, v));
            }
        }
        best.map(|(_, v)| v)
    }

    /// The area's regions, in order.
    fn area_regions<'b>(&'b self, area: &'b BlendBlock, dna: &DnaType) -> Vec<&'b BlendBlock> {
        let next = self.off("ARegion", "*next", dna);
        let mut addr = self.ptr(area, self.off("ScrArea", "regionbase", dna));
        let mut out = Vec::new();
        while let Some(rg) = self.block_at(addr) {
            if out.len() > 256 { break; }
            addr = self.ptr(rg, next);
            out.push(rg);
        }
        out
    }

    /// `View3D.render_border` (an `rctf`: xmin, xmax, ymin, ymax) with
    /// `flag2`'s `V3D_RENDER_BORDER`.
    fn view_border_of(&self, v3d: &BlendBlock, dna: &DnaType) -> super::border::BlendViewBorder {
        let rect = self.off("View3D", "render_border", dna).and_then(|o| self.ui_f32s::<4>(v3d, Some(o)));
        super::border::BlendViewBorder::of(self.int_field(v3d, "View3D", "flag2", dna), rect.unwrap_or([0.0, 1.0, 0.0, 1.0]))
    }

    fn area_view3d(&self, area: &BlendBlock, dna: &DnaType) -> Option<BlendView3d> {
        let data = self.off("ARegion", "*regiondata", dna);
        let regions = self.area_regions(area, dna);
        let sidebar = regions.iter().any(|rg| {
            self.int_field(rg, "ARegion", "regiontype", dna) == RGN_TYPE_UI
                && self.int_field(rg, "ARegion", "flag", dna) & RGN_FLAG_HIDDEN == 0
        });
        for rg in regions {
            if self.int_field(rg, "ARegion", "regiontype", dna) != RGN_TYPE_WINDOW { continue; }
            let Some(rv) = self.block_at(self.ptr(rg, data)) else { continue };
            let v3d = self.block_at(self.ptr(area, self.off("ScrArea", "spacedata", dna)));
            let m: [f32; 16] = self.ui_f32s(rv, self.off("RegionView3D", "viewinv[4][4]", dna))?;
            return Some(BlendView3d {
                view_inv: std::array::from_fn(|c| std::array::from_fn(|r| m[c * 4 + r])),
                ofs: self.ui_f32s(rv, self.off("RegionView3D", "ofs[3]", dna)).unwrap_or_default(),
                dist: self.ui_f32s::<1>(rv, self.off("RegionView3D", "dist", dna)).map_or(10.0, |d| d[0]),
                persp: self.ui_byte(rv, self.off("RegionView3D", "persp", dna)),
                camzoom: self.ui_f32s::<1>(rv, self.off("RegionView3D", "camzoom", dna)).map_or(0.0, |z| z[0]),
                shading: self.shading_at(v3d, self.off("View3D", "shading", dna), dna),
                lens: v3d.and_then(|b| self.ui_f32s::<1>(b, self.off("View3D", "lens", dna))).map_or(50.0, |l| l[0]),
                sidebar,
                overlay: v3d.map(|b| self.overlay_of(b, dna)).unwrap_or_default(),
                border: v3d.map(|b| self.view_border_of(b, dna)).unwrap_or_default(),
            });
        }
        None
    }
}
