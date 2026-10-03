//! Material Preview is lit by Blender's own studio HDRI, and hides it.
//!
//! Blender's Material Preview is `forest.exr` — the viewport studio light —
//! with the file's lamps and world off and the image itself not shown
//! (`studiolight_background_alpha` is 0 by default). Until this landed,
//! phiber lit Material Preview with a hand-made three-point rig, so no
//! comparison against Blender's Material Preview was ever of the same scene
//! under the same light. On THYMUS-BOLDROID the figure's mean brightness
//! moved from 69.0 (rig) to 55.6 against Blender's 54.1.
//!
//! The second test is the one that bit: the first conversion of the HDRI went
//! through Blender's view transform and came back clamped to 1.0, which made
//! every metal flat. A studio light with no range is a grey card.

use phiber::v2::scene::BRenderSet;
use phiber::v2::view::draw::canvas::view3d::rendered::{BRayLook, ENV_CDF_SLOT, ENV_SLOT, lit, studio};

fn look(studio: bool) -> BRayLook {
    BRayLook {
        sky: gpui::hsla(0.0, 0.0, 0.24, 1.0),
        grid: gpui::hsla(0.0, 0.0, 0.3, 1.0),
        grid_hide: 0,
        world: None,
        lights: Vec::new(),
        wire: None,
        view: [0.0, 0.0, 1.0, 0.0],
        environment: if studio { studio::environment().map(|_| studio::ROTATION) } else { None },
        set: BRenderSet::default(),
        studio,
    }
}

#[test]
fn the_studio_light_is_blenders_forest_at_full_range() {
    let env = studio::environment().expect("forest.hdr decodes");
    // Blender's copy is 1K equirectangular.
    assert_eq!((env.width, env.height), (1024, 512));
    let max = env.rgb.iter().copied().fold(0.0f32, f32::max);
    let mean = env.rgb.iter().sum::<f32>() / env.rgb.len() as f32;
    // forest.exr peaks around 1437 in the sky and averages about 0.8. A copy
    // that came through a view transform peaks at 1.0 — and lights nothing.
    assert!(max > 500.0, "the studio light has lost its range: max {max}");
    assert!((0.5..1.2).contains(&mean), "the studio light's mean is off: {mean}");
}

#[test]
fn material_preview_is_lit_by_the_studio_light_but_does_not_show_it() {
    let mut cam = phiray::Camera::default();
    lit(&mut cam, &look(true));
    assert_eq!(cam.envmap, ENV_SLOT + 1, "the studio light should be the environment");
    assert_eq!(cam.env_cdf, ENV_CDF_SLOT + 1, "and be importance-sampled like any environment");
    assert_eq!(cam.env_hidden, 1, "but the background stays the viewport grey, as in Blender");
    assert_eq!(cam.count, 0, "Material Preview switches the file's lamps off");
}

#[test]
fn a_rendered_view_shows_its_world_rather_than_hiding_it() {
    let mut cam = phiray::Camera::default();
    lit(&mut cam, &look(false));
    assert_eq!(cam.env_hidden, 0);
}
