//! What leads the tracer's atlas before any material image: the environment
//! Material Preview and Rendered light from (`ENV_SLOT`, `ENV_CDF_SLOT`, and
//! Material Preview's irradiance and prefiltered copies) or, under MatCap
//! lighting in Solid, the matcap's diffuse and specular images
//! (`Matcap::DIFFUSE_SLOT`, `::SPECULAR_SLOT`). Slots are fixed by this
//! order, so the kernels address them by constant.

use super::super::mesh::BMeshPalette;
use phiray::Atlas;

impl BMeshPalette {
    /// The leading entries for this palette's shading, in slot order.
    pub fn lead(&self, atlas: &mut Atlas) {
        // Workbench modes never read the world, so no image goes up. Material
        // Preview takes Blender's studio light; Rendered takes the file's world.
        if !self.shading.uses_viewport_color() {
            let env = if self.shading == crate::v2::scene::BShading::Material {
                super::studio::environment()
            } else {
                super::environment().map(|(env, _)| std::sync::Arc::new(env))
            };
            if let Some(env) = env {
                atlas.add_hdr(env.width, env.height, &env.rgb);
                atlas.add_env_cdf(env.width, env.height, &env.rgb);
                // Material Preview lights from the environment in one pass:
                // its irradiance and prefiltered copies go up with it
                // (`ENV_IRRADIANCE_SLOT`, `ENV_PREFILTERED_SLOT`).
                if self.shading == crate::v2::scene::BShading::Material {
                    let lighting = atlas.add_env_lighting(env.width, env.height, &env.rgb);
                    debug_assert_eq!((lighting.irradiance, lighting.prefiltered), (super::ENV_IRRADIANCE_SLOT, super::ENV_PREFILTERED_SLOT));
                }
            }
        }
        // MatCap lighting: the matcap's diffuse and specular images lead the
        // atlas (`Matcap::DIFFUSE_SLOT`, `::SPECULAR_SLOT`).
        if let Some(name) = self.matcap {
            let m = crate::v2::scene::matcap::Matcap::named(name);
            let diffuse = atlas.add_hdr(m.size, m.size, &m.diffuse);
            debug_assert_eq!(diffuse, crate::v2::scene::matcap::Matcap::DIFFUSE_SLOT);
            if let Some(specular) = &m.specular {
                let id = atlas.add_hdr(m.size, m.size, specular);
                debug_assert_eq!(id, crate::v2::scene::matcap::Matcap::SPECULAR_SLOT);
            }
        }
    }
}
