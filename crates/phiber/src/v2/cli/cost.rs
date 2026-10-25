//! What a background render measures: a frame's cost, and the switches a
//! measurement turns (split from `render.rs`).

/// What one frame cost, for the `times.txt` the comparisons read.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BFrameCost {
    pub frame: f32,
    pub posed_ms: f64,
    pub drew_ms: f64,
    pub samples: u32,
}

impl BFrameCost {
    /// The line `times.txt` holds — `frame posed_ms render_ms samples`, the
    /// shape `animation_compare` and `blender_animation.py` both expect.
    pub fn line(&self) -> String {
        format!("{} {:.0} {:.0} {}\n", self.frame, self.posed_ms, self.drew_ms, self.samples)
    }
}

/// The measurement switches, each off by default. A render for its own sake
/// leaves every one of these alone.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BLookOverride {
    /// `PHIBER_NO_WORLD`: the world contributes nothing (lamps only).
    pub no_world: bool,
    /// `PHIBER_NO_LIGHTS`: every lamp is dropped (the world only).
    pub no_lights: bool,
    /// `PHIBER_VIEW`: a view transform in place of the file's.
    pub view: Option<String>,
    /// `PHIBER_OVERRIDE`: one grey Lambertian surface in place of every
    /// material, so a render measures the light and not the materials.
    pub material: Option<f32>,
    /// `PHIBER_SCALE`: divide the resolution, for a quick pass.
    pub scale: u32,
    /// `PHIRAY_DENOISE=0`: the filter off, which is what a measurement
    /// wants — the denoiser smooths frame-to-frame noise.
    pub denoise: Option<bool>,
}

impl BLookOverride {
    /// The switches as the environment sets them, which is how the
    /// examples have always taken them.
    pub fn from_env() -> Self {
        let flag = |name: &str| std::env::var(name).is_ok();
        Self {
            no_world: flag("PHIBER_NO_WORLD"),
            no_lights: flag("PHIBER_NO_LIGHTS"),
            view: std::env::var("PHIBER_VIEW").ok(),
            material: std::env::var("PHIBER_OVERRIDE").ok().map(|a| a.parse().unwrap_or(1.0)),
            scale: std::env::var("PHIBER_SCALE").ok().and_then(|v| v.parse().ok()).unwrap_or(1),
            denoise: std::env::var("PHIRAY_DENOISE").ok().map(|v| v != "0"),
        }
    }
}
