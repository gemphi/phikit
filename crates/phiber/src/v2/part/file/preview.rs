//! `.blend` previews for the File Browser — the embedded `TEST` thumbnail
//! and `REND` scene info (Blender `filelist_readjob` → `BLO_thumbnail_from_file`).
//! Cached per (path, mtime) so directory listings don't re-read files.

use crate::phiphx::phx::{Peek, Phx};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

type Key = (String, Option<SystemTime>);

fn cache() -> &'static Mutex<HashMap<Key, Option<Arc<Peek>>>> {
    static CACHE: std::sync::OnceLock<Mutex<HashMap<Key, Option<Arc<Peek>>>>> =
        std::sync::OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// Quick info for a `.blend`, or `None` for other files / unreadable ones.
pub fn quick_info(path: &str) -> Option<Arc<Peek>> {
    if !crate::v2::scene::is_blend_path(path) {
        return None;
    }
    let mtime = std::fs::metadata(path).and_then(|m| m.modified()).ok();
    let key = (path.to_string(), mtime);
    if let Some(hit) = cache().lock().ok()?.get(&key) {
        return hit.clone();
    }
    let info = Phx::peek(Path::new(path)).ok().map(Arc::new);
    cache().lock().ok()?.insert(key, info.clone());
    info
}

/// "Scene · 1–250" for the tile subtitle (first `REND` record).
pub fn subtitle(path: &str) -> String {
    quick_info(path)
        .and_then(|i| {
            let s = i.scenes.first()?;
            Some(format!("{} · {}–{} · {}", s.scene, s.frame_start, s.frame_end, i.version))
        })
        .unwrap_or_default()
}
