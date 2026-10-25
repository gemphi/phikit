//! What a background render opens: a `.blend` (or any file `Phx` reads),
//! `play:<choice>` — a match the Play host would build
//! (`scene::play::open`; `play:` alone is the default match) — or
//! `wear:<set>`, a footwear set on its stage (`scene::wear::open`).

use crate::phiphx::Phx;
use crate::phiphx::scene::SceneModel;

pub fn open(file: &str) -> Result<SceneModel, String> {
    if let Some(choice) = file.strip_prefix("play:") {
        return crate::v2::scene::play::open(choice);
    }
    if let Some(set) = file.strip_prefix("wear:") {
        return crate::v2::scene::wear::open(set);
    }
    Phx::open(std::path::Path::new(file)).map_err(|e| format!("{e:?}"))
}
