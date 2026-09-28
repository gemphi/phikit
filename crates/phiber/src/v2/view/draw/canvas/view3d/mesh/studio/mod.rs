//! Blender's Solid-mode studio lighting, ported.
//!
//! Blender ref: `BKE_studiolight_default` (`blenkernel/intern/studiolight.cc`)
//! for the rig, `workbench_world_light_lib.glsl::get_world_lighting` for how
//! it is applied.
//!
//! Solid mode is lit by **four** directional lights fixed to the camera, each
//! with its own colour and a *wrap* factor that softens its terminator — not
//! by one Lambert term, which is what the viewport used before. The wrap is
//! what gives Blender's Solid mode its look: a surface turning away from a
//! light dims gradually instead of falling to black at ninety degrees.
//!
//! Both halves of `get_world_lighting` are ported: the wrapped diffuse rig
//! and the Blinn specular with its roughness / metallic split. See
//! `docs/render-parity.md` §3.

/// One of the rig's lights.
struct BSolidLight {
    /// Direction toward the light, in view space.
    vec: [f32; 3],
    /// Diffuse colour.
    col: [f32; 3],
    /// Specular colour.
    spec: [f32; 3],
    /// Terminator softness, `SolidLight.smooth`. This is the shader's `wrap`.
    smooth: f32,
}

/// `BKE_studiolight_default` — the "Default" studio light, verbatim.
const RIG: [BSolidLight; 4] = [
    BSolidLight {
        vec: [-0.352546, 0.170931, -0.920051],
        col: [0.033103, 0.033103, 0.033103],
        spec: [0.266761, 0.266761, 0.266761],
        smooth: 0.526620,
    },
    BSolidLight {
        vec: [-0.408163, 0.346939, 0.844415],
        col: [0.521083, 0.538226, 0.538226],
        spec: [0.599030, 0.599030, 0.599030],
        smooth: 0.000000,
    },
    BSolidLight {
        vec: [0.521739, 0.826087, 0.212999],
        col: [0.038403, 0.034357, 0.049530],
        spec: [0.106102, 0.125981, 0.158523],
        smooth: 0.478261,
    },
    BSolidLight {
        vec: [0.624519, -0.562067, -0.542269],
        col: [0.090838, 0.082080, 0.072255],
        spec: [0.106535, 0.084771, 0.066080],
        smooth: 0.200000,
    },
];

/// `light_ambient` for the Default rig — black, so the lights carry it all.
const AMBIENT: [f32; 3] = [0.0, 0.0, 0.0];

/// `wrapped_lighting(NL, w)` from `workbench_world_light_lib.glsl`:
/// `clamp((NL + w) / (1 + w)²)`. `NL` is deliberately unclamped — that is
/// what lets a light wrap past the terminator.
fn wrapped(nl: f32, w: f32) -> f32 {
    let denom = (w + 1.0) * (w + 1.0);
    ((nl + w) / denom).clamp(0.0, 1.0)
}

/// `brdf_approx` — a fresnel that brightens a specular colour at grazing
/// angles, attenuated by roughness. Blender's comment: "very rough
/// approximation, we don't need it to be correct, just fast".
fn brdf_approx(spec: [f32; 3], roughness: f32, nv: f32) -> [f32; 3] {
    let fresnel = (-8.35 * nv).exp2() * (1.0 - roughness);
    std::array::from_fn(|k| spec[k] + (1.0 - spec[k]) * fresnel)
}

/// `blinn_specular` — normalized Blinn. Pi is already divided into the
/// light's power, so the factor is `shininess / 8 + 1`.
fn blinn(shininess: f32, spec_angle: f32, nl: f32) -> f32 {
    spec_angle.powf(shininess) * nl * (shininess * 0.125 + 1.0)
}

fn dot(a: (f32, f32, f32), b: [f32; 3]) -> f32 {
    a.0 * b[0] + a.1 * b[1] + a.2 * b[2]
}

/// `get_world_lighting` — the rig's light on a surface, per channel.
///
/// `normal` is in view space; `base` is the surface's own colour, and
/// `roughness` / `metallic` come from its material. The viewer direction `I`
/// is straight down the view axis: the viewport projects orthographically
/// unless a lens says otherwise, which is the same assumption Workbench makes
/// for an orthographic view.
pub(super) fn lighting(
    base: [f32; 3],
    roughness: f32,
    metallic: f32,
    normal: (f32, f32, f32),
) -> [f32; 3] {
    let length = (normal.0 * normal.0 + normal.1 * normal.1 + normal.2 * normal.2)
        .sqrt()
        .max(1e-8);
    // Two-sided, as Blender is with backface culling off: a surface lit from
    // behind is lit, not black.
    let facing = if normal.2 < 0.0 { -1.0 } else { 1.0 };
    let n = (
        facing * normal.0 / length,
        facing * normal.1 / length,
        facing * normal.2 / length,
    );
    let i = (0.0f32, 0.0f32, 1.0f32);

    // Metals have no diffuse and take their specular tint from the surface;
    // dielectrics keep their colour and reflect a dim neutral highlight.
    let diffuse_color: [f32; 3] = std::array::from_fn(|k| base[k] * (1.0 - metallic));
    let mut specular_color: [f32; 3] =
        std::array::from_fn(|k| 0.05 + (base[k] - 0.05) * metallic);

    // `R = -reflect(I, N)`, the direction a highlight comes from.
    let ni = n.0 * i.0 + n.1 * i.1 + n.2 * i.2;
    let r = (
        2.0 * ni * n.0 - i.0,
        2.0 * ni * n.1 - i.1,
        2.0 * ni * n.2 - i.2,
    );

    let mut diffuse_light = AMBIENT;
    let mut specular_light = AMBIENT;
    for light in &RIG {
        let wrap = light.smooth;

        // Diffuse: `NL` unclamped, so the light can wrap past the terminator.
        let diffuse = wrapped(dot(n, light.vec), wrap);
        for (channel, colour) in diffuse_light.iter_mut().zip(light.col) {
            *channel += diffuse * colour;
        }

        // Specular: Blinn on the half vector, blended toward a wrapped
        // environment term for soft lights.
        let half = {
            let h = (light.vec[0] + i.0, light.vec[1] + i.1, light.vec[2] + i.2);
            let len = (h.0 * h.0 + h.1 * h.1 + h.2 * h.2).sqrt().max(1e-8);
            (h.0 / len, h.1 / len, h.2 / len)
        };
        let spec_angle = (half.0 * n.0 + half.1 * n.1 + half.2 * n.2).clamp(0.0, 1.0);
        let spec_nl = dot(n, light.vec).clamp(0.0, 1.0);
        let wrap_nl = dot(r, light.vec);
        // Gloss is reduced by the wrap, which simulates a bigger light.
        let gloss = (1.0 - roughness) * (1.0 - wrap);
        let shininess = (10.0 * gloss + 1.0).exp2();
        let spec = blinn(shininess, spec_angle, spec_nl);
        let spec_env = wrapped(wrap_nl, wrap + (1.0 - wrap) * roughness);
        let blended = spec + (spec_env - spec) * (wrap * wrap);
        for (channel, colour) in specular_light.iter_mut().zip(light.spec) {
            *channel += blended * colour;
        }
    }

    let nv = (n.0 * i.0 + n.1 * i.1 + n.2 * i.2).clamp(0.0, 1.0);
    specular_color = brdf_approx(specular_color, roughness, nv);
    for (channel, colour) in specular_light.iter_mut().zip(specular_color) {
        *channel *= colour;
    }

    // "Energy conservation with coloured specular looks strange. Limit this
    // strangeness by using mono-chromatic specular intensity."
    let spec_energy = (specular_color[0] + specular_color[1] + specular_color[2]) * 0.33333;
    std::array::from_fn(|k| {
        diffuse_light[k] * diffuse_color[k] * (1.0 - spec_energy) + specular_light[k]
    })
}

#[cfg(test)]
mod tests;
