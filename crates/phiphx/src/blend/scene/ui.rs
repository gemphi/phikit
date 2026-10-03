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
    pub shading: u8,
    /// `View3D.shading.flag` (`eView3DShading_Flag`: X-ray is `1 << 1`,
    /// X-ray in Wireframe `1 << 8`).
    pub shading_flag: u16,
    /// `View3D.shading.xray_alpha` and `xray_alpha_wire`.
    pub xray_alpha: [f32; 2],
    /// Workbench's Solid effects as the view had them: `cavity_type`
    /// (`eView3DShading_CavityType`: 0 SSAO, 1 Curvature, 2 Both),
    /// `object_outline_color`, and `[cavity_valley_factor,
    /// cavity_ridge_factor, curvature_ridge_factor, curvature_valley_factor]`;
    /// the outline's and the cavity's switches are `shading_flag` bits 0 and 5.
    pub cavity_type: u8,
    /// `View3DShading.shadow_intensity` (Shadow, 0 ..= 1); the shadow's
    /// switch is `shading_flag` bit 2 (`V3D_SHADING_SHADOW`).
    pub shadow_intensity: f32,
    pub outline_color: [f32; 3],
    pub cavity: [f32; 4],
    /// Lighting, Color and Background as the view had them: `light`
    /// (`eV3DShadingLightingMode`: 0 Flat, 1 Studio, 2 MatCap), `color_type`
    /// (`eV3DShadingColorType`: 0 Material, 1 Random, 2 Single, 3 Texture, 4
    /// Object, 5 Attribute), `single_color`, `background_type`
    /// (`eV3DShadingBackgroundType`: 0 Theme, 1 World, 2 Viewport) and
    /// `background_color`.
    pub light: u8,
    pub color_type: u8,
    pub single_color: [f32; 3],
    pub background_type: u8,
    pub background_color: [f32; 3],
    /// The area's sidebar region is shown (`RGN_TYPE_UI` without
    /// `RGN_FLAG_HIDDEN`) — `SpaceView3D.show_region_ui`.
    pub sidebar: bool,
    /// The Overlays and Gizmos popovers' switches (`overlay.rs`).
    pub overlay: super::overlay::BlendOverlay,
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
            let at = |field: &str| self.off("View3D", "shading", dna).zip(self.off("View3DShading", field, dna)).map(|(a, b)| a + b);
            let sh = at("type");
            let flag = at("flag");
            let (lo, hi) = v3d.map_or((0, 0), |b| (self.ui_byte(b, flag), self.ui_byte(b, flag.map(|o| o + 1))));
            let shading_flag = if self.header.little_endian { u16::from_le_bytes([lo, hi]) } else { u16::from_be_bytes([lo, hi]) };
            let alpha = |field: &str, default: f32| v3d.and_then(|b| self.ui_f32s::<1>(b, at(field))).map_or(default, |a| a[0]);
            let m: [f32; 16] = self.ui_f32s(rv, self.off("RegionView3D", "viewinv[4][4]", dna))?;
            return Some(BlendView3d {
                view_inv: std::array::from_fn(|c| std::array::from_fn(|r| m[c * 4 + r])),
                ofs: self.ui_f32s(rv, self.off("RegionView3D", "ofs[3]", dna)).unwrap_or_default(),
                dist: self.ui_f32s::<1>(rv, self.off("RegionView3D", "dist", dna)).map_or(10.0, |d| d[0]),
                persp: self.ui_byte(rv, self.off("RegionView3D", "persp", dna)),
                camzoom: self.ui_f32s::<1>(rv, self.off("RegionView3D", "camzoom", dna)).map_or(0.0, |z| z[0]),
                shading: v3d.map_or(0, |b| self.ui_byte(b, sh)),
                shading_flag,
                xray_alpha: [alpha("xray_alpha", 0.5), alpha("xray_alpha_wire", 0.0)],
                cavity_type: v3d.map_or(1, |b| self.ui_byte(b, at("cavity_type"))),
                shadow_intensity: alpha("shadow_intensity", 0.5),
                outline_color: v3d.and_then(|b| self.ui_f32s::<3>(b, at("object_outline_color[3]"))).unwrap_or([0.0; 3]),
                cavity: [alpha("cavity_valley_factor", 1.0), alpha("cavity_ridge_factor", 1.0), alpha("curvature_ridge_factor", 1.0), alpha("curvature_valley_factor", 1.0)],
                light: v3d.map_or(1, |b| self.ui_byte(b, at("light"))),
                color_type: v3d.map_or(0, |b| self.ui_byte(b, at("color_type"))),
                single_color: v3d.and_then(|b| self.ui_f32s::<3>(b, at("single_color[3]"))).unwrap_or([0.8; 3]),
                background_type: v3d.map_or(0, |b| self.ui_byte(b, at("background_type"))),
                background_color: v3d.and_then(|b| self.ui_f32s::<3>(b, at("background_color[3]"))).unwrap_or([0.05; 3]),
                lens: v3d.and_then(|b| self.ui_f32s::<1>(b, self.off("View3D", "lens", dna))).map_or(50.0, |l| l[0]),
                sidebar,
                overlay: v3d.map(|b| self.overlay_of(b, dna)).unwrap_or_default(),
            });
        }
        None
    }
}
