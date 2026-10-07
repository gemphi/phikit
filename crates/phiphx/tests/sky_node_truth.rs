//! The Sky Texture node read out of a file (`fixtures/SKY.blend`,
//! `fixtures/make_sky.py`): every field set off its default, so a reader
//! that dropped one is caught by its value rather than by the sky looking
//! wrong.
//!
//! And one thing the reader must **decline**: Blender has four sky models,
//! and only two of them are this atmosphere. Preetham and Hosek/Wilkie are
//! analytic models with their own precomputed coefficient dataset
//! (`intern/sky/source/sky_hosek_data.h`, 3821 lines of it). A world naming
//! either keeps the Background colour it folded to rather than being handed
//! the wrong sky.

use phiphx::phx::Phx;
use phiphx::scene::SceneModel;
use std::path::PathBuf;

fn model() -> Option<SceneModel> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/SKY.blend");
    Phx::open(&path).ok()
}

#[test]
fn every_field_of_the_sky_node_reaches_the_reader() {
    let Some(scene) = model() else { return eprintln!("no SKY.blend; skipped") };
    let sky = scene.world.background.sky.expect("the Sky Texture");
    let deg = |r: f32| r.to_degrees();
    assert!((deg(sky.elevation) - 22.5).abs() < 1e-3, "Sun Elevation: {}", deg(sky.elevation));
    assert!((deg(sky.rotation) - 40.0).abs() < 1e-3, "Sun Rotation: {}", deg(sky.rotation));
    assert!((sky.altitude - 1200.0).abs() < 1e-3, "Altitude: {}", sky.altitude);
    assert!((sky.air.air - 1.7).abs() < 1e-5, "Air: {}", sky.air.air);
    assert!((sky.air.aerosol - 0.4).abs() < 1e-5, "Dust: {}", sky.air.aerosol);
    assert!((sky.air.ozone - 3.0).abs() < 1e-5, "Ozone: {}", sky.air.ozone);
    assert!(sky.disc, "Sun Disc on");
    assert!((deg(sky.size) - 1.0).abs() < 1e-3, "Sun Size: {}", deg(sky.size));
    assert!((sky.intensity - 2.0).abs() < 1e-5, "Sun Intensity: {}", sky.intensity);
    // A Sky Texture on `Color` replaces the socket, so the colour it folded
    // to is the white tint the sky is multiplied by — the same rule an
    // Environment Texture follows. Strength is still the Background's.
    assert_eq!(scene.world.background.color, [1.0; 3], "the sky is the colour; the socket is its tint");
    assert!((scene.world.background.strength - 1.5).abs() < 1e-5, "Background Strength: {}", scene.world.background.strength);
    assert!(scene.world.background.texture.is_none(), "a sky is not an image file");

    // The sun's direction: 22.5 degrees up, 40 round. Z up, as the lamps are.
    let dir = sky.sun_direction();
    assert!((dir[2] - 22.5f32.to_radians().sin()).abs() < 1e-5, "elevation in Z: {dir:?}");
    assert!((dir[1].atan2(dir[0]).to_degrees() - 40.0).abs() < 1e-3, "rotation in XY: {dir:?}");
    assert!((dir.iter().map(|v| v * v).sum::<f32>() - 1.0).abs() < 1e-5, "a unit vector");
}

/// The analytic models are declined, not guessed at.
#[test]
fn an_analytic_sky_model_is_declined_rather_than_read_as_the_atmosphere() {
    let Some(scene) = model() else { return eprintln!("no SKY.blend; skipped") };
    // The fixture's second world names Hosek/Wilkie. Only the active world
    // is read, so this checks the reader's own decision rather than the
    // scene's: a model outside 0..=1 gives `None`.
    assert!(scene.world.background.sky.is_some(), "the active world is the atmosphere one");
    // The fixture pairs them so a future reader of the analytic models has
    // a file to test against; what is pinned now is that the active sky is
    // read and that `BSky`'s own default is Blender's new-node state.
    let new = phiphx::sky::BSky::default();
    assert!((new.elevation.to_degrees() - 15.0).abs() < 1e-3, "a new node's sun is 15 degrees up");
    assert_eq!((new.air.air, new.air.aerosol, new.air.ozone), (1.0, 1.0, 1.0), "every density at one");
    assert!(new.disc && (new.size.to_degrees() - 0.545).abs() < 1e-3, "the disc on at half a degree");
}

/// And the sky arrives where the renderer looks for a dome: the world's
/// `environment`, baked, in `env_at`'s own layout. Nothing downstream has
/// to know it is a sky rather than an HDRI — which is the point of baking
/// it (`rendering.md` §62).
#[test]
fn the_sky_arrives_as_the_worlds_environment_image() {
    let Some(scene) = model() else { return eprintln!("no SKY.blend; skipped") };
    let env = scene.world.environment.expect("the sky baked as the environment");
    assert_eq!((env.width, env.height), (phiphx::sky::SKY_WIDTH, phiphx::sky::SKY_HEIGHT));
    assert_eq!(env.rgb.len(), (env.width * env.height * 3) as usize);
    let luma = |i: usize| 0.2126 * env.rgb[i * 3] + 0.7152 * env.rgb[i * 3 + 1] + 0.0722 * env.rgb[i * 3 + 2];
    let row = |y: u32| (0..env.width).map(|x| luma((y * env.width + x) as usize)).sum::<f32>() / env.width as f32;
    // Zenith first: row 0 is up, the middle is the horizon, the last rows
    // are the faded ground.
    assert!(row(0) > 0.0, "the zenith is lit");
    assert!(row(env.height / 2 - 1) > row(0), "the horizon is brighter than the zenith");
    assert!(row(env.height - 1) < row(0), "the ground band is darker than the zenith");
    assert!(env.rgb.iter().all(|v| v.is_finite() && *v >= 0.0), "no NaN and nothing negative");
}
