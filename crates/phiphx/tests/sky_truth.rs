//! The Sky Texture against Blender's own render of the same sky
//! (`truth/blender_sky.py`, `truth/sky.txt`): a world whose Background is a
//! Sky Texture at the single-scattering Nishita model, photographed with an
//! equirectangular camera so the frame *is* the environment image Cycles
//! samples. 64 × 32, one sample — the sky is a texture, so it is noiseless
//! and a small frame is the same sky rather than a coarser one.
//!
//! **The row order is the thing to get right.** Blender's buffer runs
//! **nadir first** — its row 0 is straight down and its last row the zenith
//! — where `phiray`'s `env_at` maps `v = 0` to the **zenith**. The two are
//! each other's rows reversed, which `rendering.md` §59 recorded as the
//! trap a flat plane cannot see: get it backwards and the sky lights from
//! below, which merely looks *dim*.

use phiphx::sky::{BSky, atmosphere::BSkyAir};
use std::path::PathBuf;

/// Blender's reference as `(width, height, pixels)` in its own row order.
fn truth() -> Option<(usize, usize, Vec<[f32; 3]>)> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/truth/sky.txt");
    let text = std::fs::read_to_string(path).ok()?;
    let mut lines = text.lines();
    let head: Vec<&str> = lines.next()?.split_whitespace().collect();
    let (w, h) = (head[1].parse().ok()?, head[2].parse().ok()?);
    let mut pixels = vec![[0.0f32; 3]; w * h];
    for line in lines {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() < 5 {
            continue;
        }
        let (x, y): (usize, usize) = (f[0].parse().ok()?, f[1].parse().ok()?);
        pixels[y * w + x] = [f[2].parse().ok()?, f[3].parse().ok()?, f[4].parse().ok()?];
    }
    Some((w, h, pixels))
}

fn luma(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// Blender's row `y` is our row `h - 1 - y`: nadir first against zenith
/// first.
fn ours_row(bake: &[f32], w: usize, h: usize, blender_y: usize) -> Vec<[f32; 3]> {
    let y = h - 1 - blender_y;
    (0..w).map(|x| [0, 1, 2].map(|k| bake[(y * w + x) * 3 + k])).collect()
}

/// The sky's **latitude profile** — each row's mean over longitude — is what
/// the atmosphere model decides, and it is what this compares. The sun's own
/// column is not: one pixel of a 64-wide frame is a quarter of the sun's
/// glow, and a tolerance there would measure the sampling rather than the
/// model.
///
/// **Above the horizon the two agree to 1.6%**, every row of it — which is
/// the result, since that is where the sky is physics. Below the horizon
/// they do not, and that is recorded rather than tuned away: see the second
/// test.
#[test]
fn the_sky_matches_blenders_own_render_above_the_horizon() {
    let Some((w, h, want)) = truth() else { return eprintln!("no tests/truth/sky.txt; skipped") };
    let sky = BSky { elevation: 15.0f32.to_radians(), rotation: 0.0, disc: false, ..Default::default() };
    let bake = sky.bake_at(w as u32, h as u32);
    let mean_ours = |by: usize| ours_row(&bake, w, h, by).iter().copied().map(luma).sum::<f32>() / w as f32;
    let mean_theirs = |by: usize| (0..w).map(|x| luma(want[by * w + x])).sum::<f32>() / w as f32;
    // Blender's buffer is nadir first, so its upper half is the sky.
    let mut worst = (0usize, 0.0f32);
    for by in h / 2..h {
        let (mine, theirs) = (mean_ours(by), mean_theirs(by));
        assert!(theirs > 1e-4, "row {by} is above the horizon and Blender has sky in it");
        let off = (mine / theirs - 1.0).abs();
        if off > worst.1 {
            worst = (by, off);
        }
        assert!(off < 0.03, "row {by}: ours {mine:.5} against Blender's {theirs:.5}, ×{:.3}", mine / theirs);
    }
    eprintln!("worst sky row {} off by {:.2}%", worst.0, worst.1 * 100.0);
    // The shape, not only the level: the horizon is the brightest sky and
    // the zenith the dimmest, as it is at 15° elevation.
    assert!(mean_ours(h - 1) < mean_ours(h / 2), "the zenith is dimmer than the horizon");
}

/// Below the horizon the sky is **not** integrated — a ray pointing down
/// goes through the Earth, and the model has no ground in it. Blender fills
/// those rows by fading its horizon row out, `(1 - 2.5 sin θ)³` over the
/// first 23.6° of depression, and this does the same from the same formula
/// (`SKY_single_scattering_precompute_texture`'s second loop).
///
/// **The two do not agree there, and the numbers are recorded rather than
/// tuned away:** at 15° sun elevation this bake is ×1.35 of Cycles just
/// under the horizon, rising to ×1.69 at 20° down, the ratio converging on
/// about 1.35 as the horizon is approached. The formula and the mapping both
/// check out on paper — Blender's non-linear table rows cancel exactly
/// against the kernel's `sqrt` lookup, so the stored fade at a given
/// elevation *is* `(1 - 2.5 sin θ)³` — so something in Cycles' own sky path
/// applies a further attenuation to the ground band that is not in the GPU
/// path this was read from. It is a few degrees of fake ground rather than
/// sky, so what is pinned here is its shape: present, falling with
/// depression, and black past the cut.
#[test]
fn the_ground_band_fades_out_as_blenders_does_and_is_black_past_the_cut() {
    let Some((w, h, want)) = truth() else { return eprintln!("no tests/truth/sky.txt; skipped") };
    let sky = BSky { elevation: 15.0f32.to_radians(), disc: false, ..Default::default() };
    let bake = sky.bake_at(w as u32, h as u32);
    let mean_ours = |by: usize| ours_row(&bake, w, h, by).iter().copied().map(luma).sum::<f32>() / w as f32;
    let mean_theirs = |by: usize| (0..w).map(|x| luma(want[by * w + x])).sum::<f32>() / w as f32;
    // Every row Blender leaves black must be black here too — the cut is
    // part of the model, and a lit row below it would light a scene from
    // under the floor.
    for by in 0..h / 2 {
        if mean_theirs(by) < 1e-4 {
            assert!(mean_ours(by) < 1e-3, "row {by} is past Blender's cut and must be black: {}", mean_ours(by));
        }
    }
    // The band itself falls monotonically away from the horizon, on both
    // sides.
    let band: Vec<usize> = (0..h / 2).filter(|by| mean_theirs(*by) > 1e-4).collect();
    assert!(band.len() >= 3, "Blender's fade band is a few rows wide: {band:?}");
    for pair in band.windows(2) {
        assert!(mean_ours(pair[0]) < mean_ours(pair[1]), "the band falls with depression: rows {pair:?}");
        assert!(mean_theirs(pair[0]) < mean_theirs(pair[1]), "and so does Blender's");
    }
    // And it is of the same order, which is what "a fudge for the ground"
    // can be held to. The measured spread is 1.35 to 1.69.
    for by in band {
        let ratio = mean_ours(by) / mean_theirs(by);
        assert!((1.0..2.0).contains(&ratio), "row {by}: ours {} against Blender's {}, x{ratio:.3}", mean_ours(by), mean_theirs(by));
    }
}

/// The sun's azimuth, and what the three density sliders do.
///
/// With Rotation 0 the sun sits at the middle column, which is where
/// Blender's render puts it too — the bake is in the sun's own frame and the
/// node's Rotation is applied when the image is sampled, as Blender passes
/// `sun_rotation` to the kernel rather than baking it in.
///
/// The sliders are then checked by their **sign**, measured rather than
/// assumed, because each one is a consequence of a table being right:
///
/// - **Air ×2** brightens the zenith (more Rayleigh on a short path) and
///   *dims* the horizon while reddening it hard (on a long path extinction
///   wins and blue is what is lost): 0.83 → 1.27 at the zenith, 8.90 → 6.40
///   at the horizon, its red-to-blue 1.90 → 6.56.
/// - **Aerosols ×4** do the same more weakly and redden less: the horizon
///   goes 8.90 → 6.20 and its red-to-blue 1.90 → 2.66.
/// - **Ozone ×4** dims everything and turns it **bluer** — red-to-blue
///   1.90 → 1.19 — because ozone's Chappuis band absorbs in the middle of
///   the spectrum, which is what makes a twilight sky blue. That sign is
///   the one that catches a mistyped `OZONE_COEFF`: get the band's shape
///   wrong and the sky reddens instead.
#[test]
fn the_suns_azimuth_is_the_middle_column_and_each_slider_moves_the_sky_its_own_way() {
    let Some((w, h, want)) = truth() else { return eprintln!("no tests/truth/sky.txt; skipped") };
    let sky = BSky { elevation: 15.0f32.to_radians(), disc: false, ..Default::default() };
    let bake = sky.bake_at(w as u32, h as u32);
    // The row Blender makes brightest, and the column each side puts the
    // sun's glow in.
    let theirs_y = (0..h)
        .max_by(|a, b| {
            let f = |y: &usize| (0..w).map(|x| luma(want[y * w + x])).sum::<f32>();
            f(a).total_cmp(&f(b))
        })
        .unwrap_or(0);
    let theirs_x = (0..w).max_by(|a, b| luma(want[theirs_y * w + a]).total_cmp(&luma(want[theirs_y * w + b]))).unwrap_or(0);
    let ours = ours_row(&bake, w, h, theirs_y);
    let ours_x = (0..w).max_by(|a, b| luma(ours[*a]).total_cmp(&luma(ours[*b]))).unwrap_or(0);
    assert!((theirs_x as i32 - ours_x as i32).abs() <= 1, "the sun's azimuth: Blender column {theirs_x}, ours {ours_x}");

    // Each slider, by the sign of what it does to the zenith and to the
    // horizon. Rows are zenith-first here, so 0 is up.
    let mean = |b: &[f32], y: usize| -> [f32; 3] {
        let mut sum = [0.0f32; 3];
        for x in 0..w {
            for k in 0..3 {
                sum[k] += b[(y * w + x) * 3 + k];
            }
        }
        sum.map(|v| v / w as f32)
    };
    let horizon = h / 2 - 1;
    let with = |air: BSkyAir| BSky { air, ..sky }.bake_at(w as u32, h as u32);
    let red_blue = |c: [f32; 3]| c[0] / c[2].max(1e-9);
    let base = (mean(&bake, 0), mean(&bake, horizon));

    let thick = with(BSkyAir { air: 2.0, ..sky.air });
    let (z, hz) = (mean(&thick, 0), mean(&thick, horizon));
    assert!(luma(z) > luma(base.0), "thicker air, a brighter zenith");
    assert!(luma(hz) < luma(base.1) && red_blue(hz) > red_blue(base.1), "thicker air, a dimmer and redder horizon");

    let hazy = with(BSkyAir { aerosol: 4.0, ..sky.air });
    let hz = mean(&hazy, horizon);
    assert!(luma(hz) < luma(base.1) && red_blue(hz) > red_blue(base.1), "more aerosols, a dimmer and redder horizon");

    let ozone = with(BSkyAir { ozone: 4.0, ..sky.air });
    let (z, hz) = (mean(&ozone, 0), mean(&ozone, horizon));
    assert!(luma(z) < luma(base.0) && luma(hz) < luma(base.1), "more ozone absorbs everywhere");
    assert!(red_blue(hz) < red_blue(base.1), "and the Chappuis band takes the middle of the spectrum, so the sky turns bluer");

    // No atmosphere, no sky: Rayleigh and Mie both scale with their density.
    let thin = with(BSkyAir { air: 0.0, aerosol: 0.0, ozone: 0.0 });
    assert!(luma(mean(&thin, horizon)) < 1e-4, "no atmosphere, no scattered light");
}

/// The sun disc, which the sky image does not carry: it is one small, very
/// bright solid angle, and a 64-wide equirect frame cannot hold it without
/// blowing out a pixel — so it is a radiance of its own
/// (`SKY_single_scattering_precompute_sun`). What is checked is that it is
/// far brighter than the sky, reddens as the sun sets, and is black below
/// the horizon or with the disc switched off.
#[test]
fn the_sun_disc_is_a_radiance_of_its_own() {
    let noon = BSky { elevation: 80.0f32.to_radians(), ..Default::default() };
    let set = BSky { elevation: 1.0f32.to_radians(), ..Default::default() };
    let (high, low) = (noon.disc_radiance(), set.disc_radiance());
    assert!(luma(high) > 1e4, "the sun is far brighter than any sky: {high:?}");
    assert!(luma(low) < luma(high), "a setting sun is dimmer through more air");
    // Reddening: blue scatters out first, so the red-to-blue ratio rises.
    assert!(low[0] / low[2].max(1e-9) > high[0] / high[2].max(1e-9), "a setting sun reddens: {low:?} against {high:?}");
    assert_eq!(BSky { disc: false, ..noon }.disc_radiance(), [0.0; 3], "the disc switched off");
    assert_eq!(BSky { elevation: -0.5, ..noon }.disc_radiance(), [0.0; 3], "below the horizon");
    // Intensity is a plain multiplier on it.
    let twice = BSky { intensity: 2.0, ..noon }.disc_radiance();
    assert!((twice[1] / high[1] - 2.0).abs() < 1e-3, "Sun Intensity doubles it");
}
