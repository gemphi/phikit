//! The world a render stands in: the sky, the environment image behind it,
//! and the file's lamps — everything a `phiray::Camera` carries that is not
//! its framing.
//!
//! The viewport orbits a pivot and a final render looks through the file's
//! camera (`scene/render_job.rs`), but both stand in the same world, so both
//! finish their camera through `lit`.

use crate::phiphx::scene::SceneLight;
use glam::Mat4;
use gpui::Hsla;
use phiray::{Camera, LIGHTS, Light};

fn rgba(c: Hsla) -> [f32; 4] {
    let c = c.to_rgb();
    [c.r, c.g, c.b, c.a]
}

/// Theme colour (display) → linear albedo.
fn linear(c: Hsla) -> [f32; 4] {
    let [r, g, b, a] = rgba(c);
    [r.powf(2.2), g.powf(2.2), b.powf(2.2), a]
}

/// The atlas slots the world takes when there is an environment image: the
/// image itself, then its sampling table. Both are added before any material
/// texture, so the camera can name them without waiting for the scene to be
/// built.
pub const ENV_SLOT: u32 = 0;
pub const ENV_CDF_SLOT: u32 = 1;

/// The loaded world's environment image and the Mapping node's Z rotation,
/// when the file has one and Blender would be lighting with it.
pub fn environment() -> Option<(crate::phiphx::scene::SceneEnvironment, f32)> {
    crate::v2::scene::source::with_loaded(|m| {
        m.and_then(|m| m.world.environment.clone().map(|e| (e, m.world.background.rotation)))
    })
}

/// Background + lighting inputs that are not geometry.
pub struct BRayLook {
    /// Viewport background (theme), used when no file is loaded. Its alpha
    /// is the theme's sweep: how much lighter the sky is at the left edge
    /// (0 when the theme has none), fading to the sky by mid-width.
    pub sky: Hsla,
    pub grid: Hsla,
    /// Loaded world colour (linear), if a file is open.
    pub world: Option<[f32; 3]>,
    /// The world's environment image is in the atlas at [`ENV_SLOT`]; the
    /// value is the Mapping node's Z rotation in radians.
    pub environment: Option<f32>,
    pub lights: Vec<SceneLight>,
    /// Wireframe only: the colour of the wire. There is no environment to
    /// reflect in a wireframe, so `Camera::env` carries it (`wire.wgsl`).
    pub wire: Option<Hsla>,
    /// Colour management of the loaded file (`Camera::view`).
    pub view: [f32; 4],
    /// Properties → Render, as far as the camera carries it.
    pub set: crate::v2::scene::BRenderSet,
    /// Material Preview: lit by Blender's studio HDRI (`studio.rs`) with
    /// the file's lights and world off, and the HDRI itself *not shown* —
    /// the background stays the viewport grey, as Blender's does.
    pub studio: bool,
}

/// The loaded file's colour management as `Camera::view` words — the view
/// transform, exposure and gamma Blender applies to every viewport mode.
pub fn view_words() -> [f32; 4] {
    crate::v2::scene::source::with_loaded(|m| {
        m.map(|m| phiray::View::from_blender(&m.render.view_transform).words(m.render.exposure, m.render.gamma))
    })
    .unwrap_or_else(|| phiray::View::Standard.words(0.0, 1.0))
}

/// One Blender light in phiray terms.
pub fn light(l: &SceneLight) -> Light {
    let m = Mat4::from_cols_array_2d(&l.matrix_world);
    let dir = (-m.z_axis.truncate()).normalize_or_zero().to_array();
    let e = l.energy.max(0.0);
    let (kind, size, cone) = match l.kind.as_str() {
        "SUN" => (0, 0.0093, 0.0),
        "SPOT" => (2, l.size, (l.spot_size * 0.5).cos()),
        "AREA" => (3, l.size, 0.0),
        _ => (1, l.size, 0.0),
    };
    Light { pos: m.w_axis.truncate().to_array(), kind, color: l.color.map(|c| c * e), size, dir, cone }
}

/// The world and lights a camera carries, whatever its framing.
///
/// A final render looks through the file's camera rather than the viewport's
/// orbit (`scene/render_job.rs`), but it stands in the same world — so the
/// two share this rather than each assembling the sky, the environment image
/// and the lamps for themselves. A camera that skips it renders the scene in
/// the dark, lit only by the studio rig.
pub fn lit(cam: &mut Camera, look: &BRayLook) {
    let mut lights = [Light::default(); LIGHTS];
    let usable: Vec<Light> = look.lights.iter().map(light).filter(|l| l.color.iter().any(|c| *c > 0.0)).take(LIGHTS).collect();
    lights[..usable.len()].copy_from_slice(&usable);
    // Blender's default world (0.05) is near black; keep a floor so studio
    // renders still read, and show the world colour as the background. The
    // sky's alpha is not opacity: it is the theme's sweep — how much lighter
    // the left edge is (`BRayLook::sky.a`, `sky_at` in the kernels) — and a
    // file's world has none.
    let (sky, env) = match look.world {
        Some(w) => {
            let shown = w.map(|c| c.max(0.0).powf(1.0 / 2.2));
            ([shown[0], shown[1], shown[2], 0.0], w.map(|c| c.max(if usable.is_empty() { 0.25 } else { 0.0 })))
        }
        None => (rgba(look.sky), linear(look.sky)[..3].try_into().unwrap_or([0.25; 3]).map(|c: f32| c.max(0.25))),
    };
    // Wireframe hands the wire colour through `env` (see `BRayLook::wire`).
    let env = match look.wire {
        Some(w) => linear(w)[..3].try_into().unwrap_or([1.0; 3]),
        None => env,
    };
    cam.sky = sky;
    cam.env = [env[0], env[1], env[2], 1.0];
    cam.envmap = look.environment.map_or(0, |_| ENV_SLOT + 1);
    cam.env_rot = look.environment.unwrap_or(0.0);
    cam.env_cdf = look.environment.map_or(0, |_| ENV_CDF_SLOT + 1);
    cam.env_hidden = u32::from(look.studio && look.environment.is_some());
    cam.bounces = look.set.diffuse_bounces;
    cam.count = usable.len() as u32;
    cam.lights = lights;
    cam.view = look.view;
}
