//! The light on a 3D picture, as a studio lights a thing: a sky over it
//! and a ground under it that light it a little from all round, more
//! from above, a key light from one way, and the shine that light leaves
//! on what is smooth. Every number is the app's tables'.

use super::lens::{dot, unit};

/// The light: the way the key light comes from and how strong it is,
/// the colours the sky and the ground light with, and how bright and how
/// tight the key light's shine is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PSun {
    pub way: [f32; 3],
    pub key: f32,
    pub sky: [f32; 3],
    pub ground: [f32; 3],
    pub shine: f32,
    pub gloss: f32,
}

impl Default for PSun {
    /// Plain light: as bright from everywhere, no key, no shine.
    fn default() -> PSun {
        PSun { way: [0.0, 1.0, 0.0], key: 0.0, sky: [1.0; 3], ground: [1.0; 3], shine: 0.0, gloss: 1.0 }
    }
}

impl PSun {
    /// A colour (red, green, blue, 0 to 1) where the surface faces
    /// `normal`, seen along `view` (from the eye toward it). A surface
    /// is lit on the side the eye sees, whichever way it was made.
    pub fn lit(&self, colour: [f32; 3], normal: [f32; 3], view: [f32; 3]) -> [f32; 3] {
        let facing = -dot(normal, view).signum();
        let n = unit(normal.map(|v| v * facing));
        let way = unit(self.way);
        let up = n[1] * 0.5 + 0.5;
        let diffuse = dot(n, way).max(0.0) * self.key;
        let half = unit([way[0] - view[0], way[1] - view[1], way[2] - view[2]]);
        let shine = dot(n, half).max(0.0).powf(self.gloss.max(1.0)) * self.shine * self.key;
        [0, 1, 2].map(|i| (colour[i] * (self.ground[i] + (self.sky[i] - self.ground[i]) * up + diffuse) + shine).clamp(0.0, 1.0))
    }

    /// A surface of a material lit (`material.rs`): the smoother, the
    /// tighter and brighter the key light's shine; metal takes its colour
    /// from what it mirrors (the sky above, the ground below) and tints
    /// its shine, as gold and steel do.
    pub fn lit_surface(&self, surface: &super::material::PSurface, view: [f32; 3]) -> [f32; 3] {
        let facing = -dot(surface.normal, view).signum();
        let n = unit(surface.normal.map(|v| v * facing));
        let way = unit(self.way);
        let (rough, metal) = (surface.rough.clamp(0.02, 1.0), surface.metal.clamp(0.0, 1.0));
        let up = n[1] * 0.5 + 0.5;
        let diffuse = dot(n, way).max(0.0) * self.key;
        let half = unit([way[0] - view[0], way[1] - view[1], way[2] - view[2]]);
        let gloss = (2.0 / (rough * rough * rough * rough) - 2.0).clamp(2.0, 2048.0);
        let shine = dot(n, half).max(0.0).powf(gloss) * self.key * (1.0 - rough).powi(2) * (gloss + 8.0) / 64.0;
        // What a mirror of it sees along the view, bounced: the sky over,
        // the ground under, blurred as it is rough.
        let bounce = 2.0 * dot(n, view);
        let r = [0, 1, 2].map(|i| view[i] - bounce * n[i]);
        let sky_share = (r[1] * (1.0 - rough * 0.8) * 0.5 + 0.5).clamp(0.0, 1.0);
        let mirror = [0, 1, 2].map(|i| self.ground[i] + (self.sky[i] - self.ground[i]) * sky_share);
        let fresnel = 0.04 + 0.96 * (1.0 + dot(view, n)).clamp(0.0, 1.0).powi(5);
        [0, 1, 2].map(|i| {
            let c = surface.colour[i];
            let plain = c * (self.ground[i] + (self.sky[i] - self.ground[i]) * up + diffuse) + shine + mirror[i] * fresnel * (1.0 - rough) * 0.25;
            let metallic = c * (mirror[i] * 1.15 + diffuse * 0.35) + shine * c * 2.0;
            (plain * (1.0 - metal) + metallic * metal).clamp(0.0, 1.0)
        })
    }
}
