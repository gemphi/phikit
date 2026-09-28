//! The light Material Preview stands in.
//!
//! Blender's Material Preview is not a lamp rig: it is the viewport's
//! *studio light*, an HDRI that ships with Blender
//! (`datafiles/studiolights/world/forest.exr`, the default) lighting the
//! scene with the file's own lights and world switched off. The background
//! stays the flat viewport grey — `studiolight_background_alpha` defaults to
//! 0 — so the image is *lit* by the forest and does not *show* it.
//!
//! The same image is bundled here, converted once to Radiance HDR because
//! Blender's copy is DWAB-compressed and the `exr` decoder does not read
//! DWAB. It is CC0 (Greg Zaal, Poly Haven — `assets/studiolight/LICENSE.txt`).
//!
//! Until this existed, Material Preview here used a hand-made three-point
//! rig, so no comparison against Blender's Material Preview was ever of the
//! same scene under the same light.

use crate::phiphx::scene::SceneEnvironment;
use std::sync::{Arc, OnceLock};

/// Blender's default studio light, decoded once.
static FOREST: &[u8] = include_bytes!("../../../../../../../assets/studiolight/forest.hdr");

pub fn environment() -> Option<Arc<SceneEnvironment>> {
    static DECODED: OnceLock<Option<Arc<SceneEnvironment>>> = OnceLock::new();
    DECODED.get_or_init(|| SceneEnvironment::decode(FOREST).map(Arc::new)).clone()
}

/// Blender's `studiolight_rotate_z` default.
pub const ROTATION: f32 = 0.0;
