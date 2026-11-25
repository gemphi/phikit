//! The album: the pictures a window shows, each decoded once. A
//! picture is named by its path among the app's assets, as an icon is.
//! The first time one is asked for, its file is read and decoded, on a
//! thread of its own where there are threads, and it is drawn from the
//! frame after it is ready; one whose file is missing or is no picture
//! is never drawn, and is not asked for again.
//!
//! A picture that moves is the app's film's (`PFilm`): its name is
//! asked of the film first, and its latest frame is what is drawn. The
//! frame before it is handed back (`stale`) for the window to let go,
//! since the renderer keeps every picture it was given until told.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use gpui::{App, RenderImage};

use crate::pui::core::{PFilm, PStill};

/// Where a picture stands: being decoded, ready, or not to be had.
enum Held {
    Wait,
    Ready(Arc<RenderImage>),
    Missing,
    /// A frame of the film's, and which.
    Live(u64, Arc<RenderImage>),
}

/// The pictures a window has asked for.
#[derive(Clone, Default)]
pub struct Album {
    held: Arc<Mutex<HashMap<String, Held>>>,
    /// The app's pictures that move.
    film: Option<PFilm>,
    /// Frames no longer shown, for the window to let go.
    stale: Arc<Mutex<Vec<Arc<RenderImage>>>>,
}

/// A frame of a film as the renderer takes it. Its bytes are already in
/// the renderer's order.
pub fn frame(frame: &PStill) -> Option<RenderImage> {
    let picture = image::RgbaImage::from_raw(frame.width, frame.height, frame.pixel.as_ref().clone())?;
    Some(RenderImage::new(vec![image::Frame::new(picture)]))
}

/// A picture's file as the renderer takes it: one frame, its red and
/// blue changed over, which is how the renderer keeps them.
pub fn decode(bytes: &[u8]) -> Option<RenderImage> {
    let mut frame = image::load_from_memory(bytes).ok()?.into_rgba8();
    frame.chunks_exact_mut(4).for_each(|pixel| pixel.swap(0, 2));
    Some(RenderImage::new(vec![image::Frame::new(frame)]))
}

impl Album {
    /// An album that draws that film's pictures too.
    pub fn of(film: Option<PFilm>) -> Album {
        Album { film, ..Album::default() }
    }

    /// The latest frame of a name the film holds, made anew only when
    /// the film has moved on. `None` for a name that is not the film's.
    fn live(&self, name: &str) -> Option<Option<Arc<RenderImage>>> {
        let (serial, ..) = self.film.as_ref()?.at(name)?;
        let mut held = self.held.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(Held::Live(at, picture)) = held.get(name)
            && *at == serial
        {
            return Some(Some(Arc::clone(picture)));
        }
        let made = self.film.as_ref()?.frame(name).and_then(|shown| frame(&shown).map(|picture| (shown.serial, Arc::new(picture))));
        let (at, picture) = made?;
        if let Some(Held::Live(_, old)) = held.insert(name.to_string(), Held::Live(at, Arc::clone(&picture))) {
            self.stale.lock().unwrap_or_else(PoisonError::into_inner).push(old);
        }
        Some(Some(picture))
    }

    /// The frames no longer shown: those replaced, and the last of
    /// every name the film has ended. Each is handed back once.
    pub fn stale(&self) -> Vec<Arc<RenderImage>> {
        let mut held = self.held.lock().unwrap_or_else(PoisonError::into_inner);
        let over: Vec<String> = held.iter().filter(|(name, one)| matches!(one, Held::Live(..)) && self.film.as_ref().is_none_or(|film| film.at(name).is_none())).map(|(name, _)| name.clone()).collect();
        let mut stale = std::mem::take(&mut *self.stale.lock().unwrap_or_else(PoisonError::into_inner));
        for name in over {
            if let Some(Held::Live(_, old)) = held.remove(&name) {
                stale.push(old);
            }
        }
        stale
    }

    /// Whether a picture is moving: another frame is due.
    pub fn moving(&self) -> bool {
        self.film.as_ref().is_some_and(PFilm::running)
    }

    fn put(&self, name: &str, held: Held) {
        self.held.lock().unwrap_or_else(PoisonError::into_inner).insert(name.to_string(), held);
    }

    /// Decode a picture's file and keep it under its name.
    fn keep(&self, name: &str, bytes: &[u8]) {
        self.put(name, decode(bytes).map_or(Held::Missing, |picture| Held::Ready(Arc::new(picture))));
    }

    /// How wide and how high a picture is, once it is ready.
    pub fn size(&self, name: &str) -> Option<(f32, f32)> {
        if let Some((_, width, height)) = self.film.as_ref().and_then(|film| film.at(name)) {
            return Some((width as f32, height as f32));
        }
        match self.held.lock().unwrap_or_else(PoisonError::into_inner).get(name) {
            Some(Held::Ready(picture) | Held::Live(_, picture)) => Some((picture.size(0).width.0 as f32, picture.size(0).height.0 as f32)),
            _ => None,
        }
    }

    /// The picture of that name, when it is ready. Asked for the first
    /// time, it is begun.
    pub fn get(&self, name: &str, cx: &App) -> Option<Arc<RenderImage>> {
        if let Some(live) = self.live(name) {
            return live;
        }
        match self.held.lock().unwrap_or_else(PoisonError::into_inner).get(name) {
            Some(Held::Ready(picture)) => return Some(Arc::clone(picture)),
            // The last frame of a picture that is over is not drawn.
            Some(Held::Wait | Held::Missing | Held::Live(..)) => return None,
            None => {}
        }
        self.put(name, Held::Wait);
        // Where there are no threads, in a browser, it is read and decoded
        // here.
        #[cfg(target_family = "wasm")]
        match cx.asset_source().load(name).ok().flatten() {
            Some(bytes) => self.keep(name, &bytes),
            None => self.put(name, Held::Missing),
        }
        // Where it comes from may be slow (an app's server): its file is
        // read on the thread that decodes it, never while the window draws.
        #[cfg(not(target_family = "wasm"))]
        {
            let (album, name, source) = (self.clone(), name.to_string(), cx.asset_source().clone());
            let begun = std::thread::Builder::new().spawn(move || match source.load(&name).ok().flatten() {
                Some(bytes) => album.keep(&name, &bytes),
                None => album.put(&name, Held::Missing),
            });
            // With no thread to be had, it is not to be had either.
            if begun.is_err() {
                self.held.lock().unwrap_or_else(PoisonError::into_inner).values_mut().filter(|held| matches!(held, Held::Wait)).for_each(|held| *held = Held::Missing);
            }
        }
        None
    }

    /// Whether a picture is still being decoded: another frame is due.
    pub fn waiting(&self) -> bool {
        self.held.lock().unwrap_or_else(PoisonError::into_inner).values().any(|held| matches!(held, Held::Wait))
    }
}
