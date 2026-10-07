//! Single-scattering atmosphere, ported from Blender's
//! `intern/sky/source/sky_single_scattering.cpp`: a ray from the camera out
//! through the atmosphere, stepped in 32 segments, each segment's
//! in-scattering from the sun attenuated by the optical depth there and
//! back — Rayleigh off air, Mie off aerosols, absorption by ozone, over the
//! 21 wavelengths of `spectrum.rs`.
//!
//! The one shortcut Blender takes and this keeps: the density of each
//! scattering type has the same spectrum everywhere, so only a scalar per
//! type is carried along the ray instead of a full spectrum.

use super::spectrum::{BANDS, IRRADIANCE, OZONE_COEFF, QUADRATURE, RAYLEIGH_COEFF};

/// Rayleigh scale height, m.
const RAYLEIGH_SCALE: f32 = 8e3;
/// Mie scale height, m.
const MIE_SCALE: f32 = 1.2e3;
/// Mie scattering coefficient, m⁻¹.
const MIE_COEFF: f32 = 2e-5;
/// Aerosol anisotropy and its square.
const MIE_G: f32 = 0.76;
const SQR_G: f32 = MIE_G * MIE_G;
/// Radius of the Earth and of the atmosphere, m.
pub const EARTH_RADIUS: f32 = 6_360e3;
const ATMOSPHERE_RADIUS: f32 = 6_420e3;
/// Segments of the primary ray (`STEPS`).
const STEPS: usize = 32;

type V3 = [f32; 3];

fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn len(a: V3) -> f32 {
    dot(a, a).sqrt()
}

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn scale(a: V3, k: f32) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}

/// `geographical_to_direction`: a latitude and longitude as a unit vector,
/// Z up — which is also the sun's direction from its elevation.
pub fn geographical_to_direction(lat: f32, lon: f32) -> V3 {
    [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()]
}

/// The three densities at a height above sea level: air, aerosols, ozone.
/// Ozone is a layer around 25 km rather than an exponential falloff.
fn density_at(height: f32) -> V3 {
    [
        (-height / RAYLEIGH_SCALE).exp(),
        (-height / MIE_SCALE).exp(),
        (1.0 - ((height - 25_000.0).abs() / 15_000.0)).max(0.0),
    ]
}

/// Rayleigh and Mie phase functions at `mu`, the cosine between the ray and
/// the sun. Ozone absorbs and does not scatter, so its phase is zero.
fn phase_at(mu: f32) -> V3 {
    let rayleigh = (0.1875 * std::f32::consts::FRAC_1_PI) * (1.0 + mu * mu);
    let mie = (3.0 * (1.0 - SQR_G) * (1.0 + mu * mu))
        / (8.0 * std::f32::consts::PI * (2.0 + SQR_G) * (1.0 + SQR_G - 2.0 * MIE_G * mu).powf(1.5));
    [rayleigh, mie, 0.0]
}

/// Whether the Earth is in the way (`surface_intersection`).
fn hits_earth(pos: V3, dir: V3) -> bool {
    if dir[2] >= 0.0 {
        return false;
    }
    let b = 2.0 * dot(dir, pos);
    let c = dot(pos, pos) - EARTH_RADIUS * EARTH_RADIUS;
    b * b - 4.0 * c >= 0.0
}

/// Where the ray leaves the atmosphere (`atmosphere_intersection`).
fn leaves_atmosphere(pos: V3, dir: V3) -> V3 {
    let b = 2.0 * dot(dir, pos);
    let c = dot(pos, pos) - ATMOSPHERE_RADIUS * ATMOSPHERE_RADIUS;
    let t = (-b + (b * b - 4.0 * c).sqrt()) / 2.0;
    add(pos, scale(dir, t))
}

/// `ray_optical_depth`: the optical depth along a ray to the top of the
/// atmosphere, by Gauss–Laguerre quadrature rather than ray marching —
/// the depth falls off exponentially, which is what that quadrature is
/// for. One scalar per scattering type (`spectrum.rs`'s shortcut).
pub fn ray_optical_depth(origin: V3, dir: V3) -> V3 {
    let end = leaves_atmosphere(origin, dir);
    let length = len([end[0] - origin[0], end[1] - origin[1], end[2] - origin[2]]);
    let segment = scale(dir, length);
    let mut depth = [0.0f32; 3];
    for (node, weight) in QUADRATURE {
        let p = add(origin, scale(segment, node));
        let density = density_at(len(p) - EARTH_RADIUS);
        for k in 0..3 {
            depth[k] += density[k] * weight;
        }
    }
    scale(depth, length)
}

/// How dense the atmosphere is, as the Sky node's three sliders set it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BSkyAir {
    pub air: f32,
    pub aerosol: f32,
    pub ozone: f32,
}

/// `single_scattering`: the spectrum reaching the camera along `ray_dir`.
pub fn single_scattering(ray_dir: V3, sun_dir: V3, origin: V3, air: BSkyAir) -> [f32; BANDS] {
    let end = leaves_atmosphere(origin, ray_dir);
    let length = len([end[0] - origin[0], end[1] - origin[1], end[2] - origin[2]]);
    let segment_length = length / STEPS as f32;
    let segment = scale(ray_dir, segment_length);
    let scale3 = [air.air, air.aerosol, air.ozone];
    let phase = phase_at(dot(ray_dir, sun_dir));

    let mut spectrum = [0.0f32; BANDS];
    let mut optical_depth = [0.0f32; 3];
    // Each segment's density and in-scattering are evaluated at its middle.
    let mut p = add(origin, scale(segment, 0.5));
    for _ in 0..STEPS {
        let d = density_at(len(p) - EARTH_RADIUS);
        let density = [d[0] * scale3[0], d[1] * scale3[1], d[2] * scale3[2]];
        for k in 0..3 {
            optical_depth[k] += segment_length * density[k];
        }
        if !hits_earth(p, sun_dir) {
            let to_sun = ray_optical_depth(p, sun_dir);
            let total = [
                optical_depth[0] + to_sun[0] * scale3[0],
                optical_depth[1] + to_sun[1] * scale3[1],
                optical_depth[2] + to_sun[2] * scale3[2],
            ];
            for wl in 0..BANDS {
                // Transmission from the camera to here and from here to the
                // sun, over the three extinctions at this wavelength.
                let extinction = total[0] * RAYLEIGH_COEFF[wl] + total[1] * 1.11 * MIE_COEFF + total[2] * OZONE_COEFF[wl];
                let attenuation = (-extinction).exp();
                // Scattering out of the beam towards the camera: Rayleigh
                // and Mie by their phase functions. Ozone only absorbs.
                let scattered = phase[0] * density[0] * RAYLEIGH_COEFF[wl] + phase[1] * density[1] * MIE_COEFF;
                spectrum[wl] += attenuation * scattered * IRRADIANCE[wl] * segment_length;
            }
        }
        p = add(p, segment);
    }
    spectrum
}

/// `sun_radiation`: the sun's own spectrum as seen through the atmosphere,
/// per unit solid angle — the disc, not the sky around it.
pub fn sun_radiation(cam_dir: V3, altitude: f32, air: BSkyAir, solid_angle: f32) -> [f32; BANDS] {
    let depth = ray_optical_depth([0.0, 0.0, EARTH_RADIUS + altitude], cam_dir);
    let mut spectrum = [0.0f32; BANDS];
    for wl in 0..BANDS {
        let transmittance = RAYLEIGH_COEFF[wl] * depth[0] * air.air + 1.11 * MIE_COEFF * depth[1] * air.aerosol;
        spectrum[wl] = IRRADIANCE[wl] * (-transmittance).exp() / solid_angle;
    }
    spectrum
}
