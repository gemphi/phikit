//! A card's picture, painted from the option's own numbers: the ball's
//! cover on a shaded disc, a team's shirt in its colours with its code, a
//! kit's pattern on a shirt, a sneaker's profile, a theme's swatches, a
//! venue's elevation, a surface's grain, a seat's back. Every picture is a
//! function of the option (and, for the kit and the shoes, the home side
//! wearing them), so the picker draws the sets themselves rather than
//! pictures of them that could drift.

use super::BPick;
use crate::build::image::BImage;
use crate::build::{ball, field, font, seat};
use crate::choice::BChoice;
use crate::colour::{INK, Rgb, WHITE};
use crate::kit::sneaker::{BShoeTrim, BSneaker};
use crate::kit::uniform::BUniform;
use crate::sport::BSport;
use crate::team::BTeam;
use crate::theme::BTheme;
use crate::turf::BTurf;
use crate::venue::seat::BSeat;
use crate::venue::{BRoof, BShell, BVenue};

/// A card picture's edge, pixels.
pub const SIZE: u32 = 128;

/// The card's ground behind the picture.
const GROUND: Rgb = Rgb(0.11, 0.12, 0.14);

/// Painted at 2 × 2 samples a pixel, so edges are smooth.
fn painted(paint: impl Fn(f32, f32) -> Rgb) -> BImage {
    let half = 0.25 / SIZE as f32;
    BImage::painted(SIZE, SIZE, |u, v| {
        let mut c = Rgb(0.0, 0.0, 0.0);
        for (du, dv) in [(-half, -half), (half, -half), (-half, half), (half, half)] {
            let s = paint(u + du, v + dv);
            c = Rgb(c.0 + s.0 / 4.0, c.1 + s.1 / 4.0, c.2 + s.2 / 4.0);
        }
        c
    })
}

/// Card `i` of `pick`'s row, as `choice` would wear it.
pub fn thumb(pick: BPick, i: usize, choice: &BChoice) -> BImage {
    let home = choice.home.spec();
    match pick {
        BPick::Sport => sport(BSport::ALL[i]),
        BPick::Home | BPick::Away => {
            let t = BTeam::ALL[i].spec();
            shirt(|_, _| t.home, t.trim, Some(t.code))
        }
        BPick::Uniform => {
            let pattern = BUniform::ALL[i].spec().pattern;
            shirt(|u, v| home.home.mix(home.trim, pattern.trim_weight(u, v)), home.trim, None)
        }
        BPick::Sneaker => sneaker(BSneaker::ALL[i], home.home, home.trim),
        BPick::Theme => theme(BTheme::ALL[i]),
        BPick::Venue => venue(BVenue::ALL[i], choice.theme),
        BPick::Turf => turf(BTurf::ALL[i]),
        BPick::Seat => seat_back(BSeat::ALL[i], choice.theme),
    }
}

/// The ball: its cover wrapped on a disc lit from the upper left.
fn sport(s: BSport) -> BImage {
    let spec = s.spec();
    let cover = ball::cover(&spec.ball, 256, 128);
    painted(|u, v| {
        let (x, y) = (u * 2.0 - 1.0, 1.0 - v * 2.0);
        let r2 = (x * x + y * y) / 0.72;
        if r2 > 1.0 {
            return GROUND;
        }
        let z = (1.0 - r2).sqrt();
        let n = glam::Vec3::new(x / 0.85, y / 0.85, z).normalize();
        // Equirectangular: longitude round z-up, latitude from the pole.
        let lon = n.x.atan2(n.z) / std::f32::consts::TAU + 0.5;
        let lat = n.y.acos() / std::f32::consts::PI;
        let px = ((lon * 256.0) as u32).min(255);
        let py = ((lat * 128.0) as u32).min(127);
        let c = cover.at(px, py);
        let light = (0.35 + 0.65 * n.dot(glam::Vec3::new(-0.4, 0.5, 0.77).normalize()).max(0.0)).min(1.0);
        Rgb(c.0 * light, c.1 * light, c.2 * light)
    })
}

/// A sleeveless shirt, front on: `body(u, v)` its colour at (`u` across,
/// `v` hem to shoulders), the neck and armholes edged in `trim`, and
/// `code` across the chest.
fn shirt(body: impl Fn(f32, f32) -> Rgb, trim: Rgb, code: Option<&str>) -> BImage {
    painted(|u, v| {
        let (x, y) = (u - 0.5, 1.0 - v);
        // The outline: a body 0.56 wide from the hem (0.1) to the
        // shoulders (0.82), armholes cut in from 0.55 up, a scooped neck.
        let (hem, top) = (0.1, 0.84);
        if y < hem || y > top {
            return GROUND;
        }
        let shoulder = 0.28 - 0.12 * ((y - 0.55).max(0.0) / 0.29).powf(0.7);
        let neck = 0.11 * (1.0 - ((top - y) / 0.12).clamp(0.0, 1.0).powi(2)).sqrt();
        let half = if y > 0.55 { shoulder } else { 0.28 };
        if x.abs() > half || (y > top - 0.12 && x.abs() < neck) {
            return GROUND;
        }
        let edge = (half - x.abs()).min(if y > top - 0.14 { (x.abs() - neck).abs() } else { 1.0 });
        let (su, sv) = ((x + 0.28) / 0.56, (y - hem) / (top - hem));
        let mut c = if edge < 0.025 && y > 0.5 { trim } else { body(su, sv) };
        if let Some(code) = code {
            let p = [(x / 0.42) * 3.6, (y - 0.42) / 0.42 * 3.6];
            if font::covers(code, [p[0], p[1]], 0.22, 0.15) {
                c = crate::team::BTeamSpec::number_ink(c);
            }
        }
        // A soft shade down the sides, so it reads as cloth.
        c.mix(INK, 0.25 * (x.abs() / 0.28).powi(3))
    })
}

/// A sneaker side on, toe right: the sole, the upper up to its collar, the
/// trim where its pattern puts it.
fn sneaker(s: BSneaker, colour: Rgb, trim: Rgb) -> BImage {
    let spec = s.spec();
    let upper = if spec.coloured { colour } else { WHITE.mix(colour, 0.08) };
    // Metres to the card: a 0.3 m shoe across 0.84 of it.
    let scale = 0.84 / 0.3;
    painted(|u, v| {
        let (x, y) = ((u - 0.08) / scale, (0.86 - v) / scale);
        if !(0.0..=0.3).contains(&x) || y < 0.0 {
            return GROUND;
        }
        let t = x / 0.3;
        // The profile: the collar at the heel, sloping down over the
        // laces to a rounded toe.
        let toe = 0.045 + 0.02 * (1.0 - ((t - 0.86) / 0.14).clamp(0.0, 1.0).powi(2)).sqrt();
        let top = if t < 0.32 { spec.collar } else { spec.collar + (toe - spec.collar) * ((t - 0.32) / 0.5).clamp(0.0, 1.0).powf(0.8) };
        let nose = if t > 0.9 { toe * (1.0 - ((t - 0.9) / 0.1).powi(2)).max(0.0).sqrt() } else { top };
        if y > nose.min(top) {
            return GROUND;
        }
        if y < spec.sole {
            return if matches!(spec.trim, BShoeTrim::GlowSole) { trim.mix(WHITE, 0.3) } else { WHITE.mix(INK, 0.1) };
        }
        let on_trim = match spec.trim {
            BShoeTrim::Toe => t > 0.78,
            BShoeTrim::Heel => t < 0.18,
            BShoeTrim::Blade => (y - spec.sole - (t - 0.35) * 0.35).abs() < 0.012 && t > 0.3 && t < 0.75,
            BShoeTrim::Collar => y > top - 0.018,
            BShoeTrim::Fade => false,
            _ => t < 0.5 && y < spec.sole + 0.03,
        };
        if matches!(spec.trim, BShoeTrim::Fade) {
            return trim.mix(upper, ((y - spec.sole) / (spec.collar - spec.sole)).clamp(0.0, 1.0));
        }
        let strap = spec.strap && (t - 0.3).abs() < 0.04 && y > spec.collar * 0.55;
        if on_trim || strap { trim } else { upper }
    })
}

/// A theme: its three roles in bands, the glow as a line, the seat colours
/// as a stand of dots underneath.
fn theme(t: BTheme) -> BImage {
    let s = t.spec();
    painted(|u, v| {
        if v < 0.62 {
            let band = [s.primary, s.secondary, s.accent][((u * 3.0) as usize).min(2)];
            return if (v - 0.56).abs() < 0.02 { s.glow } else { band.mix(s.sky, 0.15 * v) };
        }
        let (col, row) = ((u * 8.0) as usize, ((v - 0.62) / 0.38 * 4.0) as usize);
        let (cu, cv) = ((col as f32 + 0.5) / 8.0, 0.62 + (row as f32 + 0.5) / 4.0 * 0.38);
        if ((u - cu) * 8.0).hypot((v - cv) * 10.5) < 0.38 { s.seats[row.min(3)] } else { s.apron }
    })
}

/// A venue in elevation: its shell's profile across the card, the roof
/// over it, the light band lit in the theme's glow.
fn venue(kind: BVenue, theme: BTheme) -> BImage {
    let spec = kind.spec();
    let t = theme.spec();
    painted(|u, v| {
        let (x, y) = (u * 2.0 - 1.0, 1.0 - v);
        let sky = t.sky.mix(WHITE, 0.2 * y);
        let ground = 0.18;
        if y < ground {
            return t.apron.mix(INK, 0.3);
        }
        let s = (x + 1.0) / 2.0;
        let edge = x.abs();
        // Each shell's silhouette: its height across the card.
        let h = match spec.shell {
            BShell::Tent => 0.5 + 0.18 * edge.powi(2),
            BShell::Lantern => 0.42 + 0.14 * (1.0 - edge * edge),
            BShell::Sails => 0.42 + 0.12 * (s * 5.0).fract(),
            BShell::Diamond => 0.46 + 0.06 * if ((s * 8.0) as i32) % 2 == 0 { 1.0 } else { -1.0 },
            BShell::Containers => 0.36 + 0.06 * ((s * 6.0) as i32 % 3) as f32,
            BShell::Cap => 0.5 - 0.1 * edge.powi(4),
            BShell::Dunes => 0.44 + 0.1 * (s * std::f32::consts::TAU * 1.5).sin(),
            BShell::Arches => 0.42 + 0.12 * (s * std::f32::consts::PI * 4.0).sin().abs(),
        };
        let wide = edge < 0.88;
        if wide && y < ground + h * 0.6 {
            let band = (y - ground) / (h * 0.6);
            return if (0.55..0.72).contains(&band) { t.glow } else { t.secondary.mix(t.primary, 0.25 * band) };
        }
        let roof = match spec.roof {
            BRoof::Closed => wide && y < ground + h * 0.6 + 0.05,
            _ => false,
        };
        if roof {
            return t.secondary.mix(WHITE, 0.3);
        }
        sky
    })
}

/// A surface: a patch of its grain a few metres across.
fn turf(t: BTurf) -> BImage {
    let spec = t.spec();
    painted(|u, v| {
        let c = field::grain(spec, [u * 6.0, v * 6.0]);
        if spec.emissive { c.mix(WHITE, 0.15) } else { c }
    })
}

/// A seat's back, face on, in the theme's first seat colour.
fn seat_back(s: BSeat, theme: BTheme) -> BImage {
    let spec = s.spec();
    let colour = theme.spec().seats[0];
    // The shell's back is a cup wrapped round the sitter: face on, a
    // rounded square darkening toward its turned-in sides. A bench has no
    // back: its plank is drawn deeper, on legs.
    let cup = matches!(spec.back, crate::venue::seat::BBack::Shell { .. });
    let outline = if cup { seat::superellipse(spec.width / 2.0, spec.height / 2.0, 3.0, 32).into_iter().map(|[x, z]| [x, z + spec.height / 2.0]).collect() } else { seat::outline(spec) };
    let bench = outline.is_empty();
    // The pan under it, as wide as the back's widest.
    let span = outline.iter().fold(0.0f32, |m, p| m.max(p[0].abs())).max(0.2);
    let tall = outline.iter().fold(0.0f32, |m, p| m.max(p[1])).max(0.1);
    let scale = 0.7 / (2.0 * span).max(tall + 0.12);
    painted(|u, v| {
        let (x, y) = ((u - 0.5) / scale, (0.82 - v) / scale);
        let plank = if bench { 0.16 } else { 0.08 };
        if y < 0.0 && y > -plank && x.abs() < span {
            return colour.mix(INK, if bench { 0.1 } else { 0.35 });
        }
        if bench && y <= -plank && y > -0.4 && (x.abs() - span * 0.8).abs() < 0.03 {
            return INK.mix(WHITE, 0.4);
        }
        if !inside(&outline, [x, y]) {
            return GROUND;
        }
        let side = if cup { 0.45 * (x.abs() / (spec.width / 2.0)).powi(3) } else { 0.0 };
        colour.mix(WHITE, 0.12 * (y / tall)).mix(INK, side)
    })
}

/// Whether `p` is inside the polygon `outline` (even-odd).
fn inside(outline: &[[f32; 2]], p: [f32; 2]) -> bool {
    let mut odd = false;
    let n = outline.len();
    for k in 0..n {
        let (a, b) = (outline[k], outline[(k + 1) % n]);
        if (a[1] > p[1]) != (b[1] > p[1]) && p[0] < a[0] + (p[1] - a[1]) / (b[1] - a[1]) * (b[0] - a[0]) {
            odd = !odd;
        }
    }
    odd
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_card_is_a_picture_and_the_cards_of_a_row_differ() {
        let choice = BChoice::default();
        for pick in BPick::ALL {
            let pictures: Vec<BImage> = (0..pick.count()).map(|i| thumb(pick, i, &choice)).collect();
            for (i, p) in pictures.iter().enumerate() {
                assert_eq!((p.width, p.height), (SIZE, SIZE));
                // Not all ground: something is drawn.
                let drawn = p.rgba.chunks(4).filter(|px| (px[0] as i32 - 28).abs() + (px[1] as i32 - 31).abs() + (px[2] as i32 - 36).abs() > 30).count();
                assert!(drawn > (SIZE * SIZE / 20) as usize, "{pick:?} {i}: {drawn} pixels drawn");
            }
            for i in 0..pictures.len() {
                for j in i + 1..pictures.len() {
                    assert_ne!(pictures[i], pictures[j], "{pick:?}: cards {i} and {j} look alike");
                }
            }
        }
    }
}
