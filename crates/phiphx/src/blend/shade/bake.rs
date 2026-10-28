//! The Base Color chain as a bakeable program.
//!
//! Most of this file's materials drive a Mix node's *Factor* with an image
//! and keep the two colours in the Mix itself — the floor is
//! `lerp(dark red, orange, pattern)`, a swimsuit is seven such layers — so
//! "the first image upstream" is not the base colour, and folding the image
//! to white loses the pattern entirely. Instead the chain from the
//! Principled's Base Color back to its images is compiled here into a small
//! stack program ([`BlendBake`]), and run over the decoded images when the
//! scene's textures attach (`scene/texture.rs`), producing the one texture
//! the kernel samples.
//!
//! Two ways to run it:
//! - **One source image** ([`BlendBake::single`]): every texel of that image
//!   is put through the program — a recolour at the image's own resolution —
//!   and the image's Mapping (folded to an affine) travels out separately,
//!   to be applied to the mesh's UVs, so a pattern tiled 116× across a floor
//!   keeps every tile at full resolution.
//! - **Several images**: the program is evaluated over the UV unit square at
//!   [`GRID`]², each image sampled through its own affine.
//!
//! Node formulas are the ones `eval.rs` already uses (`blend::mix`,
//! `blend::ramp_eval`, `scalar::*`); a node the compiler does not know makes
//! the whole bake `None`, and the material falls back to the folded tint.

/// One source image of a bake: its key (`image.rs`) and the affine its
/// Vector input folds to — `u' = m[0]·(u, v, 1)`, `v' = m[1]·(u, v, 1)`.
#[derive(Clone, Debug, PartialEq)]
pub struct BBakeImage {
    pub key: String,
    pub affine: [[f32; 3]; 2],
}

pub const IDENTITY: [[f32; 3]; 2] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];

/// One step of the program, postfix: operands are pushed, an operation pops
/// its arguments (last argument pushed last) and pushes its result.
#[derive(Clone, Debug, PartialEq)]
pub enum BBakeOp {
    /// Push image `i`'s colour at this texel (linear).
    Image(u16),
    /// Push image `i`'s alpha.
    ImageAlpha(u16),
    /// Push a colour.
    Const([f32; 4]),
    /// Push a float.
    ConstF(f32),
    /// Pops b, a, factor (`ShaderNodeMix` / legacy MixRGB).
    Mix { blend: u8, clamp_fac: bool, clamp_res: bool },
    /// Pops the factor (`ShaderNodeValToRGB`); `alpha` takes the band's alpha
    /// as a float.
    Ramp { stops: Vec<(f32, [f32; 4])>, ipo: u8, alpha: bool },
    /// Pops colour, factor.
    Invert,
    /// Pops gamma, colour.
    Gamma,
    /// Pops contrast, bright, colour.
    BrightContrast,
    /// Pops factor, value, saturation, hue, colour.
    HueSat,
    /// Pops c, b, a (`args` says how many were pushed; missing read 0).
    Math { op: u16, clamp: bool, args: u8 },
    /// Pops max, min, value.
    Clamp { range: u16 },
    /// Pops steps, to max, to min, from max, from min, value
    /// (`ShaderNodeMapRange`, float).
    MapRange { ty: u32, clamp: bool },
    /// Pops a colour (or vector); pushes its channel `channel` in `mode`
    /// (`NodeCombSepColorMode`; Separate XYZ is RGB over a vector).
    SepColor { mode: u32, channel: u8 },
    /// Pops the third, second, first channel; pushes the colour they make
    /// in `mode` (Combine Color; Combine XYZ is RGB).
    CombColor { mode: u32 },
}

/// The compiled Base Color program of one material.
#[derive(Clone, Debug, PartialEq)]
pub struct BlendBake {
    pub images: Vec<BBakeImage>,
    pub ops: Vec<BBakeOp>,
}

impl BlendBake {
    /// One source image: run as a recolour of its texels, its affine folded
    /// into the mesh's UVs by the caller.
    pub fn single(&self) -> Option<&BBakeImage> {
        match self.images.as_slice() {
            [one] => Some(one),
            _ => None,
        }
    }
}

/// A value on the stack: a colour, or a float spread grey.
#[derive(Clone, Copy)]
struct V {
    rgba: [f32; 4],
    float: bool,
}

impl V {
    fn f(v: f32) -> Self {
        Self { rgba: [v, v, v, 1.0], float: true }
    }
    /// As a float (colour → Rec.709 luminance, as Blender converts).
    fn value(&self) -> f32 {
        let [r, g, b, _] = self.rgba;
        if self.float { r } else { 0.2126 * r + 0.7152 * g + 0.0722 * b }
    }
}

/// Run `ops` with `sample(i)` answering each image op. Answers linear RGBA.
pub fn run(ops: &[BBakeOp], sample: &dyn Fn(u16) -> [f32; 4]) -> [f32; 4] {
    let mut stack: Vec<V> = Vec::with_capacity(8);
    let pop = |stack: &mut Vec<V>| stack.pop().unwrap_or(V::f(0.0));
    for op in ops {
        let v = match op {
            BBakeOp::Image(i) => V { rgba: sample(*i), float: false },
            BBakeOp::ImageAlpha(i) => V::f(sample(*i)[3]),
            BBakeOp::Const(c) => V { rgba: *c, float: false },
            BBakeOp::ConstF(f) => V::f(*f),
            BBakeOp::Mix { blend, clamp_fac, clamp_res } => {
                let (b, a, fac) = (pop(&mut stack), pop(&mut stack), pop(&mut stack));
                let t = if *clamp_fac { fac.value().clamp(0.0, 1.0) } else { fac.value() };
                let mut rgba = super::blend::mix(*blend, t, a.rgba, b.rgba);
                if *clamp_res {
                    rgba = rgba.map(|c| c.clamp(0.0, 1.0));
                }
                V { rgba, float: a.float && b.float }
            }
            BBakeOp::Ramp { stops, ipo, alpha } => {
                let fac = pop(&mut stack);
                let c = super::blend::ramp_eval(stops, *ipo, fac.value()).unwrap_or([0.0; 4]);
                if *alpha { V::f(c[3]) } else { V { rgba: c, float: false } }
            }
            BBakeOp::Invert => {
                let (c, fac) = (pop(&mut stack), pop(&mut stack));
                let t = fac.value();
                let rgb = [0, 1, 2].map(|k| c.rgba[k] + t * (1.0 - 2.0 * c.rgba[k]));
                V { rgba: [rgb[0], rgb[1], rgb[2], c.rgba[3]], float: false }
            }
            BBakeOp::Gamma => {
                let (g, c) = (pop(&mut stack), pop(&mut stack));
                V { rgba: super::scalar::gamma(c.rgba, g.value()), float: false }
            }
            BBakeOp::BrightContrast => {
                let (contrast, bright, c) = (pop(&mut stack), pop(&mut stack), pop(&mut stack));
                V { rgba: super::scalar::bright_contrast(c.rgba, bright.value(), contrast.value()), float: false }
            }
            BBakeOp::HueSat => {
                let (fac, val, sat, hue, c) = (pop(&mut stack), pop(&mut stack), pop(&mut stack), pop(&mut stack), pop(&mut stack));
                V { rgba: super::blend::hue_sat(c.rgba, hue.value(), sat.value(), val.value(), fac.value()), float: false }
            }
            BBakeOp::Math { op, clamp, args } => {
                let c = if *args > 2 { pop(&mut stack).value() } else { 0.0 };
                let b = if *args > 1 { pop(&mut stack).value() } else { 0.0 };
                let a = pop(&mut stack).value();
                let mut out = super::scalar::math(*op, a, b, c).unwrap_or(a);
                if *clamp {
                    out = out.clamp(0.0, 1.0);
                }
                V::f(out)
            }
            BBakeOp::Clamp { range } => {
                let (max, min, v) = (pop(&mut stack), pop(&mut stack), pop(&mut stack));
                V::f(super::scalar::clamp(*range, v.value(), min.value(), max.value()))
            }
            BBakeOp::MapRange { ty, clamp } => {
                let mut a = [0.0; 6];
                for slot in a.iter_mut().rev() {
                    *slot = pop(&mut stack).value();
                }
                V::f(super::scalar::map_range(*ty, a[0], a[1], a[2], a[3], a[4], a[5], *clamp))
            }
            BBakeOp::SepColor { mode, channel } => {
                let c = pop(&mut stack).rgba;
                V::f(super::sepcomb::separate_color(*mode, [c[0], c[1], c[2]])[(*channel).min(2) as usize])
            }
            BBakeOp::CombColor { mode } => {
                let (b, g, r) = (pop(&mut stack), pop(&mut stack), pop(&mut stack));
                let [r, g, b] = super::sepcomb::combine_color(*mode, [r.value(), g.value(), b.value()]);
                V { rgba: [r, g, b, 1.0], float: false }
            }
        };
        stack.push(v);
    }
    pop(&mut stack).rgba
}

/// Compose `outer ∘ inner` (both applied to a UV row vector with trailing 1).
pub(crate) fn compose(outer: [[f32; 3]; 2], inner: [[f32; 3]; 2]) -> [[f32; 3]; 2] {
    std::array::from_fn(|r| {
        [
            outer[r][0] * inner[0][0] + outer[r][1] * inner[1][0],
            outer[r][0] * inner[0][1] + outer[r][1] * inner[1][1],
            outer[r][0] * inner[0][2] + outer[r][1] * inner[1][2] + outer[r][2],
        ]
    })
}

/// A Mapping node (Point) as a UV affine: `rot_z(v · scale) + loc`.
pub(crate) fn mapping_affine(loc: [f32; 3], rot_z: f32, scale: [f32; 3]) -> [[f32; 3]; 2] {
    let (s, c) = rot_z.sin_cos();
    [
        [c * scale[0], -s * scale[1], loc[0]],
        [s * scale[0], c * scale[1], loc[1]],
    ]
}

/// Apply an affine to one UV.
pub fn apply(m: [[f32; 3]; 2], uv: [f32; 2]) -> [f32; 2] {
    [
        m[0][0] * uv[0] + m[0][1] * uv[1] + m[0][2],
        m[1][0] * uv[0] + m[1][1] * uv[1] + m[1][2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The floor's shape: `lerp(dark red, orange, image)` — a black texel
    /// takes A, a white one B, exactly as the Mix node reads its factor.
    #[test]
    fn a_factor_driven_mix_recolors_texels() {
        let a = [0.3, 0.0, 0.0, 1.0];
        let b = [1.0, 0.32, 0.0, 1.0];
        let ops = vec![
            BBakeOp::Image(0),
            BBakeOp::Const(a),
            BBakeOp::Const(b),
            BBakeOp::Mix { blend: 0, clamp_fac: true, clamp_res: false },
        ];
        let black = run(&ops, &|_| [0.0, 0.0, 0.0, 1.0]);
        let white = run(&ops, &|_| [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(black, a, "a black texel is colour A");
        assert_eq!(white, b, "a white texel is colour B");
        let grey = run(&ops, &|_| [0.5, 0.5, 0.5, 1.0]);
        assert!((grey[0] - 0.65).abs() < 1e-3, "half-way: {grey:?}");
    }

    /// Layered mixes (the swimsuit's shape): each image masks its own flat
    /// colour over the stack below.
    #[test]
    fn layered_factor_mixes_stack() {
        let ops = vec![
            BBakeOp::Image(0),
            BBakeOp::Const([0.1, 0.1, 0.1, 1.0]),
            BBakeOp::Const([0.9, 0.0, 0.0, 1.0]),
            BBakeOp::Mix { blend: 0, clamp_fac: true, clamp_res: false },
            // Second layer: previous result is A, factor from image 1.
        ];
        let out = run(&ops, &|i| if i == 0 { [1.0; 4] } else { [0.0; 4] });
        assert_eq!(out, [0.9, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn a_mapping_scales_and_moves_uvs() {
        let m = mapping_affine([0.25, 0.5, 0.0], 0.0, [2.0, 3.0, 1.0]);
        assert_eq!(apply(m, [1.0, 1.0]), [2.25, 3.5]);
        let spin = mapping_affine([0.0; 3], std::f32::consts::FRAC_PI_2, [1.0, 1.0, 1.0]);
        let [u, v] = apply(spin, [1.0, 0.0]);
        assert!(u.abs() < 1e-6 && (v - 1.0).abs() < 1e-6, "{u} {v}");
        let two = compose(m, spin);
        let [u2, v2] = apply(two, [1.0, 0.0]);
        let direct = apply(m, apply(spin, [1.0, 0.0]));
        assert!((u2 - direct[0]).abs() < 1e-6 && (v2 - direct[1]).abs() < 1e-6);
    }
}
