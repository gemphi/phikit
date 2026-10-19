//! The world a render stands in: the sky, the environment image behind it,
//! and the file's lamps — everything a `phiray::Camera` carries that is not
//! its framing.
//!
//! The viewport orbits a pivot and a final render looks through the file's
//! camera (`scene/render_job.rs`), but both stand in the same world, so both
//! finish their camera through `lit`.

use crate::phiphx::scene::SceneLight;
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
/// Material Preview's environment lighting (`phiray::trace::ibl`), after
/// them: the irradiance coefficients, then the prefiltered copies. Only a
/// Material scene adds them (`scene::build`); `Camera::preview` names them
/// only then.
pub const ENV_IRRADIANCE_SLOT: u32 = 2;
pub const ENV_PREFILTERED_SLOT: u32 = 3;

/// The loaded world's environment image and the Mapping node's Z rotation,
/// when the file has one and Blender would be lighting with it — or, when
/// the file names an HDRI that is not where it says
/// (`BlendWorld::missing`), Blender's own studio light in its place: the
/// `forest` world Material Preview lights with, at its default rotation.
/// Blender would light the scene its missing-image magenta; the read
/// report carries the warning instead (`phiphx scene/import.rs`).
pub fn environment() -> Option<(crate::phiphx::scene::SceneEnvironment, f32)> {
    crate::v2::scene::source::with_loaded(|m| {
        let m = m?;
        if let Some(env) = m.world.environment.clone() {
            return Some((env, m.world.background.rotation));
        }
        m.world.background.missing.as_ref()?;
        super::studio::environment().map(|e| ((*e).clone(), super::studio::ROTATION))
    })
}

/// Background + lighting inputs that are not geometry.
pub struct BRayLook {
    /// Viewport background (theme), used when no file is loaded. Its alpha
    /// is the theme's sweep: how much lighter the sky is at the left edge
    /// (0 when the theme has none), fading to the sky by mid-width.
    pub sky: Hsla,
    pub grid: Hsla,
    /// The floor and axes left out (`Camera::grid_hide`).
    pub grid_hide: u32,
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

impl BRayLook {
    /// `Camera::grid_hide` for the switches: bits 1, 2 and 4 leave out the
    /// floor, the X axis and the Y axis; bit 8 adds the Z axis.
    pub fn grid_hide(shown: crate::v2::scene::BOverlays) -> u32 {
        [(!shown.floor, 1), (!shown.axis_x, 2), (!shown.axis_y, 4), (shown.axis_z, 8)].iter().filter(|(on, _)| *on).map(|(_, bit)| bit).sum()
    }
}

/// Color Management as `Camera::view` words — the view transform, exposure
/// and gamma Blender applies to every viewport mode: the panel's, over the
/// loaded file's (`render_set::BColorManagement`).
pub fn view_words(s: &crate::v2::state::BState) -> [f32; 4] {
    crate::v2::scene::source::with_loaded(|m| crate::v2::scene::render_set::BColorManagement::now(s, m).words())
}

pub fn lit(cam: &mut Camera, look: &BRayLook) {
    let mut lights = [Light::default(); LIGHTS];
    let usable: Vec<Light> = look.lights.iter().map(super::lamp::BLamp::record).filter(|l| l.color.iter().any(|c| *c > 0.0)).take(LIGHTS).collect();
    lights[..usable.len()].copy_from_slice(&usable);
    // The file's world lights the scene as it stands — no floor. There used
    // to be one (0.25 a channel, when a scene had no usable lamp) so that a
    // studio render "still read", and it did two wrong things: it lifted any
    // scene whose lamps were all dark, and because it clamped per channel it
    // *re-tinted* a coloured world — a magenta dome came out pink
    // (`rendering.md` §53). Blender has no such floor; a scene with no lamps
    // and a near-black world renders near black. Material Preview is lit by
    // the studio HDRI below, which is where "still reads" belongs.
    //
    // The sky's alpha is not opacity: it is the theme's sweep — how much
    // lighter the left edge is (`BRayLook::sky.a`, `sky_at` in the kernels)
    // — and a file's world has none.
    let (sky, env) = match look.world {
        Some(w) => {
            let shown = w.map(|c| c.max(0.0).powf(1.0 / 2.2));
            ([shown[0], shown[1], shown[2], 0.0], w.map(|c| c.max(0.0)))
        }
        // No world at all: the viewport's own canvas colour is all there is
        // to light with, so it keeps its floor.
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
    cam.env_hidden = 0;
    cam.preview = [0; 4];
    if look.studio && look.environment.is_some() {
        studio_lighting(cam);
    }
    cam.bounces = look.set.diffuse_bounces;
    cam.count = usable.len() as u32;
    cam.lights = lights;
    cam.view = look.view;
}

/// Material Preview's lighting words: Blender's studio HDRI at the slots
/// `scene::build` puts it in — the image to reflect, its sampling table,
/// its irradiance and prefiltered copies (`ENV_*_SLOT`) — and hidden, so
/// the background stays the viewport grey. A camera without them lights
/// the diffuse from nothing and reflects the raw image: every surface a
/// dark mirror. `lit` sets them for the viewport and the render job;
/// `still` for a camera built by hand (the examples).
pub fn studio_lighting(cam: &mut Camera) {
    cam.envmap = ENV_SLOT + 1;
    cam.env_cdf = ENV_CDF_SLOT + 1;
    cam.env_rot = 0.0;
    cam.env_hidden = 1;
    cam.preview = [ENV_IRRADIANCE_SLOT + 1, ENV_PREFILTERED_SLOT + 1, phiray::trace::ibl::LEVELS, 0];
}
