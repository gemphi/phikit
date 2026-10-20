//! A step's figure: the screenshot as the face of the manual's deck
//! (`BDeck`), the two narrower cards behind it showing their edges above
//! and below, and a numbered ring, in Phiber's orange, on what the step
//! presses. Drawn with
//! ImageMagick, in the deck palette's light shades — the shades the app's
//! own light theme draws the screen in.

use super::drive::Res;
use phiber::v2::manual::BDeck;
use phiber::v2::theme::style::color::BResolvedColors;
use phiber::v2::theme::{BColor, BThemeMode, palette_in};
use std::path::Path;
use std::process::Command;

/// `#rrggbb` for a palette colour, which the palette keeps as HSL.
fn hex(c: BColor) -> String {
    let (h, s, l) = (c.0 / 360.0, c.1 / 100.0, c.2 / 100.0);
    let f = |n: f32| {
        let k = (n + h * 12.0) % 12.0;
        let a = s * l.min(1.0 - l);
        let v = l - a * (-1.0f32).max((k - 3.0).min(9.0 - k).min(1.0));
        (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8
    };
    format!("#{:02x}{:02x}{:02x}", f(0.0), f(8.0), f(4.0))
}

/// The face's corner radius and the cards' behind it, in pixels.
const FACE_RADIUS: i32 = 14;
const CARD_RADIUS: i32 = 18;
/// The ring round what to press, and the number badge on it.
const RING: i32 = 30;
const BADGE: i32 = 17;

/// Write the figure to `out` from the screenshot `shot`: the ring at `mark`
/// on the screen when there is one, and step `n`'s number on it when the
/// figure is a step.
pub fn deck(shot: &Path, out: &Path, n: Option<usize>, mark: Option<(i32, i32)>, deck: &BDeck) -> Res<()> {
    let palette = palette_in(deck.palette, BThemeMode::Light);
    let (w, h) = super::drive::SCREEN;
    let layers = deck.layers();
    let face = deck.face();
    let frame_h = (h as f32 / face.3).round() as i32;
    let at = |(x, y, lw, lh): (f32, f32, f32, f32)| {
        let (x0, y0) = ((x * w as f32).round() as i32, (y * frame_h as f32).round() as i32);
        (x0, y0, x0 + (lw * w as f32).round() as i32 - 1, y0 + (lh * frame_h as f32).round() as i32 - 1)
    };
    let mut args: Vec<String> = vec!["-size".into(), format!("{w}x{frame_h}"), "xc:none".into()];
    // The cards behind, back to front: the deepest in the control shade,
    // the next in the header's.
    for (k, layer) in layers.iter().take(layers.len() - 1).enumerate() {
        let (x0, y0, x1, y1) = at(*layer);
        let fill = if k == 0 { palette.bg_control } else { palette.bg_header };
        args.extend(["-fill".into(), hex(fill), "-draw".into(), format!("roundrectangle {x0},{y0} {x1},{y1} {CARD_RADIUS},{CARD_RADIUS}")]);
    }
    // The face: the screenshot with its corners rounded.
    let (fx, fy, ..) = at(face);
    args.extend([
        "(".into(),
        shot.display().to_string(),
        "(".into(),
        "-size".into(),
        format!("{w}x{h}"),
        "xc:black".into(),
        "-fill".into(),
        "white".into(),
        "-draw".into(),
        format!("roundrectangle 0,0 {},{} {FACE_RADIUS},{FACE_RADIUS}", w - 1, h - 1),
        ")".into(),
        "-alpha".into(),
        "off".into(),
        "-compose".into(),
        "CopyOpacity".into(),
        "-composite".into(),
        ")".into(),
        "-geometry".into(),
        format!("+{fx}+{fy}"),
        "-compose".into(),
        "Over".into(),
        "-composite".into(),
    ]);
    if let Some((mx, my)) = mark {
        let (cx, cy) = (mx + fx, my + fy);
        // Phiber's orange: the UI's own accent is blue, and a marker in it
        // would read as part of the screen.
        let accent = hex(BResolvedColors::from_palette(palette).brand_warm);
        // The badge sits up and to the right of the ring, clear of what the
        // ring circles, and inside the frame.
        let (bx, by) = ((cx + RING).min(w - BADGE - 2), (cy - RING).max(BADGE + 2));
        args.extend([
            "-fill".into(),
            "none".into(),
            "-stroke".into(),
            accent.clone(),
            "-strokewidth".into(),
            "4".into(),
            "-draw".into(),
            format!("circle {cx},{cy} {},{cy}", cx + RING),
        ]);
        if let Some(n) = n {
            args.extend([
                "-fill".into(),
                accent,
                "-stroke".into(),
                "white".into(),
                "-strokewidth".into(),
                "3".into(),
                "-draw".into(),
                format!("circle {bx},{by} {},{by}", bx + BADGE),
                "-stroke".into(),
                "none".into(),
                "-fill".into(),
                "white".into(),
                "-font".into(),
                "DejaVu-Sans-Bold".into(),
                "-pointsize".into(),
                "20".into(),
                "-gravity".into(),
                "NorthWest".into(),
                "-annotate".into(),
                format!("+{}+{}", bx - if n < 10 { 6 } else { 12 }, by - 12),
                n.to_string(),
            ]);
        }
    }
    args.push(out.display().to_string());
    let status = Command::new("convert").args(&args).status()?;
    match status.success() {
        true => Ok(()),
        false => Err(format!("convert could not draw {}", out.display()).into()),
    }
}
