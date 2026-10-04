//! `View3DShading` as a file saved it — the viewport's (`View3D.shading`)
//! and the Workbench render engine's (`Scene.display.shading`, what F12
//! draws with) are the one struct, read from a base offset into a block.

use crate::blend::file::types::*;

/// The fields Phiber draws from, raw (`DNA_view3d_types.h`).
#[derive(Debug, Clone, PartialEq)]
pub struct BlendShading {
    /// `type`: 2 Wireframe, 3 Solid, 4 Material Preview, 5 Rendered.
    pub ty: u8,
    /// `flag` (`eV3DShadingFlag`: the outline `1 << 0`, X-ray `1 << 1`,
    /// Shadow `1 << 2`, specular `1 << 4`, Cavity `1 << 5`, matcap flip
    /// `1 << 6`, X-ray in Wireframe `1 << 8`, culling `1 << 10`).
    pub flag: u16,
    /// `xray_alpha`, `xray_alpha_wire`.
    pub xray_alpha: [f32; 2],
    /// `cavity_type` (0 SSAO, 1 Curvature, 2 Both), `shadow_intensity`.
    pub cavity_type: u8,
    pub shadow_intensity: f32,
    pub outline_color: [f32; 3],
    /// `[cavity_valley_factor, cavity_ridge_factor, curvature_ridge_factor,
    /// curvature_valley_factor]`.
    pub cavity: [f32; 4],
    /// `light` (0 Flat, 1 Studio, 2 MatCap), `color_type` (0 Material, 1
    /// Random, 2 Single, 3 Texture, 4 Object, 5 Attribute), `matcap`.
    pub light: u8,
    pub color_type: u8,
    pub matcap: String,
    pub single_color: [f32; 3],
    /// `background_type` (0 Theme, 1 World, 2 Viewport), `background_color`.
    pub background_type: u8,
    pub background_color: [f32; 3],
}

impl Default for BlendShading {
    /// `DNA_view3d_defaults.h`: Solid, Studio, Material, the specular and
    /// X-ray-in-Wireframe flags, grey single colour, dark theme background.
    fn default() -> Self {
        Self { ty: 3, flag: (1 << 4) | (1 << 8), xray_alpha: [0.5, 0.0], cavity_type: 1, shadow_intensity: 0.5, outline_color: [0.0; 3], cavity: [1.0; 4], light: 1, color_type: 0, matcap: "basic_1".to_string(), single_color: [0.8; 3], background_type: 0, background_color: [0.05; 3] }
    }
}

impl BlendFile {
    /// The `View3DShading` at `base` within `block` (`None` base: the
    /// defaults, as a block without one reads).
    pub(crate) fn shading_at(&self, block: Option<&BlendBlock>, base: Option<usize>, dna: &DnaType) -> BlendShading {
        let d = BlendShading::default();
        let Some((block, base)) = block.zip(base) else { return d };
        let at = |field: &str| self.off("View3DShading", field, dna).map(|o| base + o);
        let byte = |field: &str, default: u8| at(field).and_then(|o| block.data.get(o).copied()).unwrap_or(default);
        let f32s = |o: Option<usize>, n: usize| -> Option<Vec<f32>> {
            let o = o?;
            (o + 4 * n <= block.data.len()).then(|| (0..n).map(|i| self.read_f32(&block.data[o + 4 * i..o + 4 * i + 4])).collect())
        };
        let one = |field: &str, default: f32| f32s(at(field), 1).map_or(default, |v| v[0]);
        let three = |field: &str, default: [f32; 3]| f32s(at(field), 3).map_or(default, |v| [v[0], v[1], v[2]]);
        let flag = at("flag").and_then(|o| block.data.get(o..o + 2)).map_or(d.flag, |b| if self.header.little_endian { u16::from_le_bytes([b[0], b[1]]) } else { u16::from_be_bytes([b[0], b[1]]) });
        let matcap = at("matcap[256]").and_then(|o| block.data.get(o..(o + 256).min(block.data.len()))).map_or(d.matcap.clone(), |raw| {
            let n = raw.iter().position(|&c| c == 0).unwrap_or(raw.len());
            String::from_utf8_lossy(&raw[..n]).to_string()
        });
        BlendShading {
            ty: byte("type", d.ty),
            flag,
            xray_alpha: [one("xray_alpha", 0.5), one("xray_alpha_wire", 0.0)],
            cavity_type: byte("cavity_type", d.cavity_type),
            shadow_intensity: one("shadow_intensity", d.shadow_intensity),
            outline_color: three("object_outline_color[3]", d.outline_color),
            cavity: [one("cavity_valley_factor", 1.0), one("cavity_ridge_factor", 1.0), one("curvature_ridge_factor", 1.0), one("curvature_valley_factor", 1.0)],
            light: byte("light", d.light),
            color_type: byte("color_type", d.color_type),
            matcap,
            single_color: three("single_color[3]", d.single_color),
            background_type: byte("background_type", d.background_type),
            background_color: three("background_color[3]", d.background_color),
        }
    }
}
