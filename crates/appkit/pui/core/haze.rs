//! A haze: a picture made small and soft, to stand behind itself as
//! Apple's Photos and Google's do behind a picture looked at alone. It is
//! the picture's own file, read and made so many points across and
//! blurred, as a frame a film shows (`PFilm`), so any app draws it with
//! the mark every picture is drawn with, larger than the window.

use super::film::PFilm;

/// How many points across a haze is, and how far it is blurred.
pub const SIDE: u32 = 96;
pub const BLUR: f32 = 6.0;

/// A picture's file made a haze: its width, its height, and its points
/// as a film keeps them (blue, green, red, how solid). Nothing for what
/// is no picture.
pub fn haze(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let picture = image::load_from_memory(bytes).ok()?;
    let small = picture.thumbnail(SIDE, SIDE).into_rgba8();
    let soft = image::imageops::blur(&small, BLUR);
    let (wide, high) = soft.dimensions();
    let mut point = soft.into_raw();
    point.chunks_exact_mut(4).for_each(|one| one.swap(0, 2));
    Some((wide, high, point))
}

/// Make the haze of a file and show it in a film under a name, on a
/// thread of its own where one can be had (in a browser, at once);
/// nothing is shown for what cannot be read.
pub fn show(film: &PFilm, name: &str, file: std::path::PathBuf) {
    let (film, name) = (film.clone(), name.to_string());
    let work = move || {
        if let Some((wide, high, point)) = std::fs::read(file).ok().as_deref().and_then(haze) {
            film.show(&name, wide, high, point);
        }
    };
    let shared = std::sync::Arc::new(std::sync::Mutex::new(Some(work)));
    let other = shared.clone();
    let begun = std::thread::Builder::new().spawn(move || other.lock().ok().and_then(|mut work| work.take()).into_iter().for_each(|work| work()));
    if begun.is_err() {
        shared.lock().ok().and_then(|mut work| work.take()).into_iter().for_each(|work| work());
    }
}
