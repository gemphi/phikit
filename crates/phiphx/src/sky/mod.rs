//! The Sky Texture node baked to an environment image.
//!
//! Blender's Nishita sky (`SHD_SKY_SINGLE_SCATTERING`) is not evaluated in
//! the shader: `node_shader_tex_sky.cc` precomputes a 512 × 256 table on the
//! CPU and the kernel samples it through a **non-linear** latitude mapping
//! that concentrates rows near the horizon
//! (`sky_radiance_nishita`'s `y = copysign(sqrt(|elev|·2/π), elev)·0.5+0.5`).
//!
//! Here it is baked **straight into an equirectangular image** instead, in
//! exactly the layout `phiray`'s `env_at` samples — so the sky arrives as
//! the world's environment image and the whole path that already carries one
//! carries it: the CDF for next-event estimation, the prefiltered chain, the
//! MIS weight against the cosine ray (`rendering.md` §59, which measured
//! that path against Cycles and found it sound). No kernel change at all.
//!
//! The cost of baking equirect rather than Blender's table is more samples
//! near the poles and fewer near the horizon for the same pixel count; the
//! gain is that nothing downstream has to know the sky is a sky.

pub mod atmosphere;
pub mod spectrum;

use atmosphere::{BSkyAir, EARTH_RADIUS, geographical_to_direction, single_scattering, sun_radiation};
use spectrum::{spec_to_xyz, xyz_to_rgb};

/// Blender's own table size (`GPU_SKY_WIDTH`, `GPU_SKY_HEIGHT`), which the
/// equirect bake keeps so the two have the same pixel budget.
pub const SKY_WIDTH: u32 = 512;
pub const SKY_HEIGHT: u32 = 256;

/// The Sky Texture node as the bake takes it (`NodeTexSky`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BSky {
    /// `sun_elevation` and `sun_rotation`, radians.
    pub elevation: f32,
    pub rotation: f32,
    /// `altitude`, m above sea level.
    pub altitude: f32,
    pub air: BSkyAir,
    /// `sun_disc`: whether the disc itself is drawn.
    pub disc: bool,
    /// `sun_size`, the disc's angular diameter in radians, and
    /// `sun_intensity`, the multiplier on it.
    pub size: f32,
    pub intensity: f32,
}

impl Default for BSky {
    /// A new Sky node: the sun 15° up, the atmosphere at sea level, every
    /// density at 1, the disc on at half a degree
    /// (`node_shader_tex_sky.cc`'s `node_shader_init_tex_sky`).
    fn default() -> Self {
        Self {
            elevation: 15.0f32.to_radians(),
            rotation: 0.0,
            altitude: 0.0,
            air: BSkyAir { air: 1.0, aerosol: 1.0, ozone: 1.0 },
            disc: true,
            size: 0.545_f32.to_radians(),
            intensity: 1.0,
        }
    }
}

impl BSky {
    /// The sun's direction in the world, Z up, with the node's Rotation
    /// about Z — the direction a Sun lamp would point *from*.
    pub fn sun_direction(&self) -> [f32; 3] {
        geographical_to_direction(self.elevation, self.rotation)
    }

    /// The sun disc's solid angle, steradians.
    fn solid_angle(&self) -> f32 {
        std::f32::consts::TAU * (1.0 - (self.size * 0.5).cos())
    }

    /// The sky as an equirectangular RGB image, `SKY_WIDTH × SKY_HEIGHT`,
    /// scene-linear, in `env_at`'s layout: `u = -atan2(y, x)/2π + 0.5` and
    /// **`v = 0` at the zenith** (`v = 0.5 - atan2(z, |xy|)/π`). Blender's
    /// own buffer runs the other way, which is the trap §59 recorded.
    ///
    /// The sky is symmetric about the sun's azimuth, so only half the
    /// columns are integrated and the other half mirrored — Blender does the
    /// same, and it halves a bake that is 32 ray steps × 8 quadrature steps
    /// × 21 wavelengths a pixel.
    pub fn bake(&self) -> Vec<f32> {
        self.bake_at(SKY_WIDTH, SKY_HEIGHT)
    }

    /// The same at a size of its own, which is what the truth test uses:
    /// Blender's reference is rendered small, and the sky is a texture, so
    /// a small bake is the same sky rather than a coarser one.
    pub fn bake_at(&self, width: u32, height: u32) -> Vec<f32> {
        let (w, h) = (width as usize, height as usize);
        let altitude = self.altitude.clamp(1.0, 59_999.0);
        let origin = [0.0, 0.0, EARTH_RADIUS + altitude];
        // The bake is in the sun's own frame — azimuth 0 is the sun — and
        // the Rotation is applied when the image is sampled, as Blender
        // passes `sun_rotation` to the kernel rather than baking it in.
        let sun = geographical_to_direction(self.elevation, 0.0);
        let mut out = vec![0.0f32; w * h * 3];
        let half = w / 2;
        let lat_of = |y: usize| std::f32::consts::FRAC_PI_2 - std::f32::consts::PI * (y as f32 + 0.5) / h as f32;
        // `v = 0` is the zenith, so the rows above the horizon come first
        // and the latitude falls as y rises. The last row whose latitude is
        // still above the horizon is the one the lower hemisphere fades.
        let horizon = (0..h).rfind(|y| lat_of(*y) > 0.0).unwrap_or(h - 1);
        for y in 0..=horizon {
            let lat = lat_of(y);
            for x in 0..=half {
                let lon = std::f32::consts::TAU * (x as f32 + 0.5) / w as f32 - std::f32::consts::PI;
                let dir = geographical_to_direction(lat, lon);
                let rgb = xyz_to_rgb(spec_to_xyz(&single_scattering(dir, sun, origin, self.air)));
                for (k, v) in rgb.iter().enumerate() {
                    out[(y * w + x) * 3 + k] = *v;
                    // The mirrored column, the same angle the other side of
                    // the sun's meridian.
                    out[(y * w + (w - x - 1).min(w - 1)) * 3 + k] = *v;
                }
            }
        }
        // The lower hemisphere is **not** integrated: a ray pointing down
        // passes through the Earth, and `single_scattering` would return
        // the light along it as if the planet were not there. Blender fills
        // those rows by fading the horizon row out instead
        // (`SKY_single_scattering_precompute_texture`'s second loop), and so
        // does this — `(1 - 2.5 sin θ)³` over the first 23.6° of depression,
        // black below that.
        for y in horizon + 1..h {
            let sink = -lat_of(y).sin();
            let fade = if sink < 0.4 { (1.0 - sink * 2.5).powi(3) } else { 0.0 };
            for x in 0..w * 3 {
                out[y * w * 3 + x] = out[horizon * w * 3 + x] * fade;
            }
        }
        out
    }

    /// The sun disc's radiance, scene-linear — `SKY_single_scattering_
    /// precompute_sun`'s two pixels averaged, which is what a disc this
    /// small comes to. Black with the sun below the horizon or the disc off.
    pub fn disc_radiance(&self) -> [f32; 3] {
        let altitude = self.altitude.clamp(1.0, 59_999.0);
        let half = self.size * 0.5;
        let (bottom, top) = ((self.elevation - half).max(0.0), (self.elevation + half).max(0.0));
        if !self.disc || top <= 0.0 {
            return [0.0; 3];
        }
        let solid_angle = self.solid_angle();
        let at = |elev: f32| {
            let dir = geographical_to_direction(elev, 0.0);
            xyz_to_rgb(spec_to_xyz(&sun_radiation(dir, altitude, self.air, solid_angle)))
        };
        let (lo, hi) = (at(bottom), at(top));
        [0, 1, 2].map(|k| (lo[k] + hi[k]) * 0.5 * self.intensity)
    }
}
