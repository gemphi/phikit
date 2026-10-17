//! The native `.blend` reader. Parses Blender's SDNA container and extracts
//! the scene Blender would show, without Blender installed. Entered through
//! `crate::phx::Phx`; `BlendImporter` serves the asset pipeline.
//! Status and next steps: `jobs.md` in this folder.

use crate::Result;
use std::path::Path;


mod anim;
mod asset;
pub mod curve;
mod file;
mod idprop;
pub(crate) mod modifier;
mod object;
pub mod rig;
mod scene;
pub mod shade;
pub use shade::material::VIEWPORT_PBR;
pub mod write;
pub(crate) use shade::image::set_source as set_image_source;
pub mod mesh;

pub use file::quick::{BlendQuickInfo, BlendRenderInfo, BlendThumbnail};
pub use file::types::{
    BHeadType, BlendBlock, BlendCollection, BlendFile, BlendHeader, BlendMaterial, BlendScene,
    BlendSceneObject, DnaField, DnaStruct, DnaType,
};
pub use modifier::BlendEvalMesh;
pub use modifier::driven::BlendDrivenSource;
pub use modifier::driver::{BModValue, BModWrite, BModifierDrivers};
pub use modifier::skin::{CpuSkin, SKIN_MIN, SkinJob, SkinPair, Skinner, set_skinner};
pub(crate) use modifier::supported_modifiers;
pub(crate) use modifier::{DeferredRaw, apply_eval_jobs};

thread_local! {
    /// Set while a progressive open parses, so the reader captures modifier
    /// jobs for a deferred apply instead of applying them inline (`scene::read`,
    /// `Phx::open_progressive`). Thread-local because a reader runs on one
    /// thread and concurrent opens must not see each other's mode.
    static PROGRESSIVE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Run `f` with progressive-open mode on, restoring the previous mode after —
/// even on panic — so a reader on this thread defers modifier evaluation.
pub(crate) fn with_progressive_open<R>(f: impl FnOnce() -> R) -> R {
    struct Guard(bool);
    impl Drop for Guard {
        fn drop(&mut self) {
            PROGRESSIVE.with(|p| p.set(self.0));
        }
    }
    let _guard = PROGRESSIVE.with(|p| Guard(p.replace(true)));
    f()
}

/// Whether the current read should defer modifier evaluation.
pub(crate) fn progressive_open() -> bool {
    PROGRESSIVE.with(std::cell::Cell::get)
}
pub use object::kind::ObjectKind;
pub use object::light_driver::AREA_ENERGY_SCALE;
pub use object::data::{BlendCameraData, BlendEmptyData, BlendLightData, BlendObjectData};
pub use object::dof::BlendDof;
pub use anim::fcurve::{BFCurve, BKey, COLOR_AUTO_RAINBOW, COLOR_AUTO_RGB, COLOR_AUTO_YRGB, COLOR_CUSTOM, FLAG_ACTIVE, FLAG_MUTED, FLAG_PROTECTED, FLAG_SELECTED, HANDLE_ALIGN, HANDLE_AUTO, HANDLE_AUTO_ANIM, HANDLE_FREE, HANDLE_VECT, IPO_BEZ, IPO_CONST, IPO_LIN};
pub use anim::easing::{EASE_AUTO, EASE_IN, EASE_IN_OUT, EASE_OUT, IPO_BACK, IPO_BOUNCE, IPO_CIRC, IPO_CUBIC, IPO_ELASTIC, IPO_EXPO, IPO_QUAD, IPO_QUART, IPO_QUINT, IPO_SINE};
pub use anim::fcurve::{EXTRAPOLATE_CONSTANT, EXTRAPOLATE_LINEAR};
pub use anim::drivers::BlendDrivers;
pub use anim::nla_eval::{BMotionClip, BMotionTrack};
pub use anim::nla_edit::{BNlaAt, BNlaPick, BNlaStacks, NEW_TRACK};
pub use anim::nla_stack::{BNlaPlaced, STASH_TRACK};
pub use anim::nla::{BLEND_ADD, BLEND_COMBINE, BLEND_MULTIPLY, BLEND_REPLACE, BLEND_SUBTRACT, STRIP_SYNC_LENGTH, STRIP_USR_INFLUENCE, STRIP_USR_TIME_CYCLIC, TRACK_PROTECTED, BNlaStrip, BNlaTrack, BlendNla, EXTEND_HOLD, EXTEND_HOLD_FORWARD, EXTEND_NOTHING, STRIP_ACTIVE, STRIP_CLIP, STRIP_META, STRIP_MUTED, STRIP_SELECT, STRIP_SOUND, STRIP_TRANSITION, TRACK_ACTIVE, TRACK_MUTED, TRACK_SELECTED, TRACK_SOLO, ADT_EDIT_ON, ADT_SOLO_TRACK, ADT_UI_ACTIVE, ADT_UI_SELECTED};
pub use anim::summary::{BlendActionSlot, BlendActionSummary};
pub use anim::follow::BMotionAt;
pub use anim::motion::{BlendMotion, BlendMotionList};
pub use scene::cursor::BlendCursor;
pub use scene::marker::BlendMarker;
pub use scene::overlay::BlendOverlay;
pub use scene::shading::BlendShading;
pub use scene::playback::{BlendLoopMode, BlendPlayback, BlendSyncMode, PLAY_IN_MASK, TIME_ALL_3D_WIN, TIME_ALL_ANIM_WIN, TIME_ALL_BUTS_WIN, TIME_ALL_IMAGE_WIN, TIME_CLIPS, TIME_FOLLOW, TIME_NODES, TIME_REGION, TIME_SEQ, TIME_SPREADSHEETS, SCE_SNAP_TO_FRAME, SCE_SNAP_TO_KEYS, SCE_SNAP_TO_MARKERS, SCE_SNAP_TO_SECOND, SCE_SNAP_TO_STRIPS};
pub use object::transform::ObjectTransformRaw;
pub use object::layer::{BASE_HIDDEN, BASE_SELECTED, LAYER_COLLECTION_EXCLUDE, LAYER_COLLECTION_HIDE, ViewLayerInfo};
pub use object::visibility::{BlendShadowCast, BlendVisibility};
pub use idprop::{BIdGroup, BIdValue};
pub use scene::movie::BlendMovie;
pub use scene::border::{BlendRenderBorder, BlendViewBorder};
pub use scene::output::{BlendImageFormat, BlendOutput};
pub use scene::pass::{BPassKind, BlendPassSet};
pub use scene::props::BlendSceneProps;
pub use scene::types::BlendBsdf;
pub use shade::links::BlendNormalMap;
pub use shade::world::BlendWorld;
pub use shade::maps::BlendMaps;
pub use mesh::list::BFaces;
pub use scene::ui::{BlendUiState, BlendView3d, RV3D_CAMOB, RV3D_ORTHO};
pub use shade::image::PackedImages;

/// Structure encapsulating all Blender (.blend) asset import capabilities.
pub struct BlendImporter;

impl BlendImporter {
    /// Read a .blend file and extract a BlendScene.
    /// Zstd-compressed files (Blender 4.5+ "compressed" saves, and all of the
    /// upstream test suite) are decompressed transparently, like blenloader.
    pub fn import_blend_native(path: &Path) -> Result<BlendScene> {
        Self::import_blend_native_with_progress(path, &|_, _| true)
    }

    /// Read a .blend file, reporting progress through `progress(fraction,
    /// stage)`. The callback returns `false` to cancel; a cancelled read
    /// returns [`PointCloudError::Cancelled`].
    ///
    /// Opening a file has **no side effects on disk** (Blender's
    /// `WM_file_read` never writes beside the file). The asset-import
    /// pipeline that wants a `.phx` sidecar calls
    /// [`Self::import_blend_native_with_sidecar`] instead.
    ///
    /// Stage fractions (mirroring the phases of `blo_read_file_internal`):
    /// 0.00–0.05 read bytes · 0.05–0.45 block table + DNA · 0.45–0.90 IDs
    /// (objects/meshes/materials) · 0.90–1.00 scene assembly.
    pub fn import_blend_native_with_progress(
        path: &Path,
        progress: &(dyn Fn(f32, &str) -> bool + Sync),
    ) -> Result<BlendScene> {
        use crate::PointCloudError;
        if !progress(0.0, "Reading file") {
            return Err(PointCloudError::Cancelled);
        }
        let data = std::fs::read(path)?;
        let total = data.len().max(1) as f64;
        if !progress(0.05, "Indexing blocks") {
            return Err(PointCloudError::Cancelled);
        }

        let cancelled = std::sync::atomic::AtomicBool::new(false);
        let mut on_block = |blocks: usize, bytes: u64| {
            // Compressed files decompress to more bytes than `total`; clamp.
            let frac = (bytes as f64 / total).min(1.0) as f32;
            let label = format!("Indexing blocks ({blocks})");
            if !progress(0.05 + 0.40 * frac, &label) {
                cancelled.store(true, std::sync::atomic::Ordering::Relaxed);
            }
        };
        let blend = BlendFile::parse_bytes_with_progress(&data, &mut on_block)?;
        if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
            return Err(PointCloudError::Cancelled);
        }

        let scene = blend.extract_scene_with_progress(&|f, s| progress(0.45 + 0.45 * f, s))?;
        if !progress(0.95, "Building scene") {
            return Err(PointCloudError::Cancelled);
        }
        Ok(scene)
    }

    /// Asset-import variant: read the file **and** emit a portable `.phx`
    /// sidecar beside it (best-effort; never fails the import). Only the
    /// asset pipeline (`AssetImporter`) should call this — opening a file in
    /// the editor must not write next to the user's data.
    pub fn import_blend_native_with_sidecar(path: &Path) -> Result<BlendScene> {
        let scene = Self::import_blend_native(path)?;
        scene.write_sidecar(path);
        Ok(scene)
    }
}

#[cfg(test)]
#[path = "tests/file.rs"]
mod file_tests;

#[cfg(test)]
#[path = "tests/blend.rs"]
mod tests;
