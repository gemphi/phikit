//! A theme's swatch: the few colours it is made of, as its shop writes
//! them, each a colour of the web's (`#3B4657`, `hsl(212 58% 55%)`).
//! They are the thing's own colours, read from its data: on its card
//! they stand side by side, and worn they take the place of some of
//! the window's own, as `data/wear.tsv` says.

use appkit::pui::core::{PColor, PPaint, PRect};
use appkit::service::PTable;
use appkit::theme::{PLook, PTheme};

use crate::data;

/// A colour of the web's as `#rrggbb`. Nothing for one that is written
/// another way than the two a swatch uses.
pub fn hex(css: &str) -> Option<String> {
    let css = css.trim();
    if css.starts_with('#') {
        return Some(css.to_string()).filter(|hex| [4, 7, 9].contains(&hex.len()));
    }
    let inside = css.strip_prefix("hsl(")?.strip_suffix(')')?;
    let part: Vec<f32> = inside.split([' ', ',']).filter(|part| !part.is_empty()).filter_map(|part| part.trim_end_matches(['%', 'd', 'e', 'g']).parse().ok()).collect();
    let (hue, full, light) = (*part.first()?, part.get(1)? / 100.0, part.get(2)? / 100.0);
    // The way from a hue, how full and how light to red, green and blue.
    let spread = full * light.min(1.0 - light);
    let channel = |at: f32| {
        let turn = (at + hue / 30.0).rem_euclid(12.0);
        let value = light - spread * (turn - 3.0).min(9.0 - turn).clamp(-1.0, 1.0);
        (value.clamp(0.0, 1.0) * 255.0).round() as u8
    };
    Some(format!("#{:02x}{:02x}{:02x}", channel(0.0), channel(8.0), channel(4.0)))
}

/// A swatch's colours, those that can be read.
pub fn colour(swatch: &[String]) -> Vec<PColor> {
    let row: String = swatch.iter().filter_map(|css| hex(css)).enumerate().map(|(index, hex)| format!("color\t{index}\t{hex}\n")).collect();
    let look = PLook::read(&row);
    (0..swatch.len()).filter_map(|index| look.color(&index.to_string())).collect()
}

/// Draw a swatch in a place: its colours side by side, the first and
/// the last rounded as the place is.
pub fn draw(paint: &mut dyn PPaint, rect: PRect, swatch: &[String], radius: f32) {
    let list = colour(swatch);
    let wide = rect.size.width / list.len().max(1) as f32;
    for (index, color) in list.iter().enumerate() {
        paint.round(PRect::new(rect.origin.x + index as f32 * wide, rect.origin.y, wide, rect.size.height), radius, *color);
    }
}

/// The rows a worn swatch puts over the window's look: each row of
/// `data/wear.tsv` names a colour of the window and which colour of
/// the swatch, counted from one, takes its place.
pub fn worn(swatch: &[String]) -> String {
    let place = PTable::read(data::WEAR);
    let row = place.list().iter().filter_map(|(token, nth)| {
        let css = swatch.get(nth.parse::<usize>().ok()?.checked_sub(1)?)?;
        Some(format!("color\t{token}\t{}\n", hex(css)?))
    });
    row.collect()
}
