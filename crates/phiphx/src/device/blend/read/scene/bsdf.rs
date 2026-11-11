//! Principled BSDF input values carried by a material slot.

/// Unlinked Principled BSDF input values (Blender defaults when absent).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlendBsdf {
    pub base: [f32; 4],
    pub roughness: f32,
    pub metallic: f32,
    pub alpha: f32,
    pub ior: f32,
    /// "Specular IOR Level" (4.x): scales the dielectric layer's F0 — 0.5
    /// leaves it as the IOR gives it, 0 takes the reflection away, 1
    /// doubles it ([`Self::specular_eta`]).
    pub specular_level: f32,
    pub transmission: f32,
    pub emission: [f32; 3],
    pub emission_strength: f32,
    /// "Subsurface Weight" (4.x) / "Subsurface" (3.x).
    pub subsurface: f32,
    /// Per-channel scattering distance, in `subsurface_scale` units.
    pub subsurface_radius: [f32; 3],
    /// "Subsurface Scale" — the radius multiplier, in metres.
    pub subsurface_scale: f32,
    /// "Sheen Weight" (4.x) / "Sheen" (3.x): the fabric retro-reflection lobe.
    pub sheen: f32,
    pub sheen_roughness: f32,
    pub sheen_tint: [f32; 3],
    /// "Coat Weight" (4.x) / "Clearcoat" (3.x).
    pub coat: f32,
    pub coat_roughness: f32,
    /// Anisotropy and its rotation (silk, satin, brushed metal).
    pub anisotropic: f32,
    pub anisotropic_rotation: f32,
}

impl Default for BlendBsdf {
    fn default() -> Self {
        Self {
            base: [0.8, 0.8, 0.8, 1.0],
            roughness: 0.5,
            metallic: 0.0,
            alpha: 1.0,
            ior: 1.5,
            specular_level: 0.5,
            transmission: 0.0,
            emission: [0.0; 3],
            emission_strength: 0.0,
            subsurface: 0.0,
            subsurface_radius: [1.0, 0.2, 0.1],
            subsurface_scale: 0.05,
            sheen: 0.0,
            sheen_roughness: 0.5,
            sheen_tint: [1.0; 3],
            coat: 0.0,
            coat_roughness: 0.03,
            anisotropic: 0.0,
            anisotropic_rotation: 0.0,
        }
    }
}

impl BlendBsdf {
    /// Emitted radiance (colour × strength).
    pub fn emitted(&self) -> [f32; 3] {
        self.emission.map(|c| c * self.emission_strength)
    }

    /// The IOR of the dielectric specular layer: `ior` adjusted by
    /// Specular IOR Level as Cycles adjusts it (`svm/closure.h`, "Apply IOR
    /// adjustment"): F0 from the IOR, times twice the level, back to an
    /// IOR. Transmission keeps the plain `ior`; only the reflection over
    /// the diffuse takes this. Skin and the pose files' floors set the
    /// level near 0.1, which is a fifth of the reflection the plain IOR
    /// gives — reading `ior` alone made every such surface glossier and
    /// brighter than Blender's.
    pub fn specular_eta(&self) -> f32 {
        let level = self.specular_level.max(0.0);
        if level == 0.5 {
            return self.ior;
        }
        let r = (self.ior - 1.0) / (self.ior + 1.0);
        let f0 = (r * r * 2.0 * level).clamp(0.0, 0.99).sqrt();
        let eta = (1.0 + f0) / (1.0 - f0);
        if self.ior < 1.0 { 1.0 / eta } else { eta }
    }

    /// Scattering distance per channel in metres (radius × scale).
    pub fn scatter(&self) -> [f32; 3] {
        self.subsurface_radius.map(|r| r * self.subsurface_scale)
    }
}

#[cfg(test)]
mod tests {
    use super::BlendBsdf;

    fn f0(eta: f32) -> f32 {
        ((eta - 1.0) / (eta + 1.0)).powi(2)
    }

    /// Cycles' IOR adjustment (`svm/closure.h`): F0 scales with twice the
    /// level, 0.5 leaves the IOR as it is, 0 takes the reflection away.
    #[test]
    fn specular_ior_level_scales_the_reflection_as_cycles_does() {
        let at = |level: f32| BlendBsdf { ior: 1.45, specular_level: level, ..Default::default() }.specular_eta();
        assert_eq!(at(0.5), 1.45);
        assert_eq!(at(0.0), 1.0, "no reflection at all");
        // Pose 23_2's floor: 0.115 is 23% of the plain IOR's reflection.
        assert!((f0(at(0.115)) / f0(1.45) - 0.23).abs() < 1e-4, "{}", f0(at(0.115)) / f0(1.45));
        assert!((f0(at(1.0)) / f0(1.45) - 2.0).abs() < 1e-4);
        // An IOR under 1 stays under 1 (the inside of a bubble).
        assert!(BlendBsdf { ior: 0.75, specular_level: 0.25, ..Default::default() }.specular_eta() < 1.0);
    }
}
