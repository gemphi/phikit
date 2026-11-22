//! A room of an app, `phiogy/lobby`: where pages find each other. It is
//! there while a page has its line open, and nothing of it is kept.

use phiecs::prelude::Aspect;

use crate::comp::{APP, Limit, ROOM};

/// A room with a line open.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Room {
    /// The app and the room, with a stroke between.
    pub key: String,
}

impl Aspect for Room {}

impl Room {
    /// The key of a room of an app; nothing when either is not a name
    /// the limits let through.
    pub fn key(limit: &Limit, app: &str, room: &str) -> Option<String> {
        (limit.fits(APP, app) && limit.fits(ROOM, room)).then(|| format!("{app}/{room}"))
    }
}
