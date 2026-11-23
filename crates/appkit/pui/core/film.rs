//! A film: the pictures of an app that move. A picture that moves has a
//! name, as one in a file has, and is drawn by the same mark
//! (`PPaint::picture`); what differs is where it comes from. The app
//! shows a frame under the name whenever it has a new one, from any
//! thread, and whatever draws takes the latest. Ended, the name is no
//! picture any more.
//!
//! A frame is its width, its height and four bytes a point: blue,
//! green, red and how solid, a row after a row, as a screen keeps them.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

/// How many bytes a point of a frame is.
pub const POINT: usize = 4;

/// One frame of a picture that moves: a still of it.
#[derive(Debug, Clone, PartialEq)]
pub struct PStill {
    /// Which frame of its name it is: each is one more than the last.
    pub serial: u64,
    pub width: u32,
    pub height: u32,
    /// Blue, green, red and how solid, for each point, a row after a row.
    pub pixel: Arc<Vec<u8>>,
}

/// The pictures that move, by name. A copy of a film is the same film.
#[derive(Debug, Clone, Default)]
pub struct PFilm {
    held: Arc<Mutex<HashMap<String, PStill>>>,
}

impl PFilm {
    /// Show a frame under a name: it is that name's picture until the
    /// next. One whose bytes are not its width by its height is not
    /// shown, nor one with no points.
    pub fn show(&self, name: &str, width: u32, height: u32, pixel: Vec<u8>) {
        let whole = width as usize * height as usize * POINT;
        if whole == 0 || pixel.len() != whole {
            return;
        }
        let mut held = self.held.lock().unwrap_or_else(PoisonError::into_inner);
        let serial = held.get(name).map_or(1, |frame| frame.serial + 1);
        held.insert(name.to_string(), PStill { serial, width, height, pixel: Arc::new(pixel) });
    }

    /// The name's picture is over.
    pub fn end(&self, name: &str) {
        self.held.lock().unwrap_or_else(PoisonError::into_inner).remove(name);
    }

    /// Every picture whose name begins so is over.
    pub fn clear(&self, begin: &str) {
        self.held.lock().unwrap_or_else(PoisonError::into_inner).retain(|name, _| !name.starts_with(begin));
    }

    /// The latest frame of that name.
    pub fn frame(&self, name: &str) -> Option<PStill> {
        self.held.lock().unwrap_or_else(PoisonError::into_inner).get(name).cloned()
    }

    /// Which frame that name is at, and how wide and high it is.
    pub fn at(&self, name: &str) -> Option<(u64, u32, u32)> {
        self.held.lock().unwrap_or_else(PoisonError::into_inner).get(name).map(|frame| (frame.serial, frame.width, frame.height))
    }

    /// Whether any picture is moving.
    pub fn running(&self) -> bool {
        !self.held.lock().unwrap_or_else(PoisonError::into_inner).is_empty()
    }
}
