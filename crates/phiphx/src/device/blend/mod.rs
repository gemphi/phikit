//! The native `.blend` reader. Parses Blender's SDNA container and extracts
//! the scene Blender would show, without Blender installed. Entered through
//! `crate::phx::Phx`; `BlendImporter` serves the asset pipeline.
//! Status and next steps: `jobs.md` in this folder.

use super::{Device, DeviceKind, Source};
use crate::device::Peek;
use crate::phx::Progress;
use crate::scene::{DeferredModifierList, SceneLoadError, SceneModel};
use crate::Result;
use std::path::Path;


mod anim;
mod file;
pub(crate) mod modifier;
pub(crate) mod open;
pub mod mesh;
pub mod read;
pub use mesh::BlendImporter;
pub mod rig;
pub mod shade;
use read::{idprop, object, scene};
pub use shade::material::VIEWPORT_PBR;
pub mod write;
pub(crate) use shade::image::set_source as set_image_source;
pub(crate) use shade::image::clear_source as clear_image_source;
pub mod physics;
pub use physics::BlendPhysics;

pub use file::kind::{
    BHeadType, BlendBlock, BlendCollection, BlendFile, BlendHeader, BlendMaterial, BlendScene,
    BlendSceneObject, DnaField, DnaStruct, DnaType,
};
pub use modifier::BlendEvalMesh;
pub use modifier::driven::BlendDrivenSource;
pub use modifier::driver::{BModValue, BModWrite, BModifierDriverList};
pub use modifier::skin::{CpuSkin, SKIN_MIN, SkinJob, SkinPair, Skinner, set_skinner};
pub(crate) use modifier::supported_modifiers;
pub(crate) use modifier::{DeferredRaw, apply_eval_jobs};
pub use object::vertex_parent::{is_vertex_parent, PARVERT1, PARVERT3};
pub(crate) use object::world::vertex_children_worlds;

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
pub use object::light_driver::{AREA_ENERGY_SCALE, BLightProp, LA_AREA, SPOT_SIZE_MIN};
pub use object::data::{BlendCameraData, BlendEmptyData, BlendLightData, BlendObjectData};
pub use object::dof::BlendDof;
pub use anim::fcurve::{BFCurve, BKey, COLOR_AUTO_RAINBOW, COLOR_AUTO_RGB, COLOR_AUTO_YRGB, COLOR_CUSTOM, FLAG_ACTIVE, FLAG_MUTED, FLAG_PROTECTED, FLAG_SELECTED, HANDLE_ALIGN, HANDLE_AUTO, HANDLE_AUTO_ANIM, HANDLE_FREE, HANDLE_VECT, IPO_BEZ, IPO_CONST, IPO_LIN};
pub use anim::easing::{EASE_AUTO, EASE_IN, EASE_IN_OUT, EASE_OUT, IPO_BACK, IPO_BOUNCE, IPO_CIRC, IPO_CUBIC, IPO_ELASTIC, IPO_EXPO, IPO_QUAD, IPO_QUART, IPO_QUINT, IPO_SINE};
pub use anim::fcurve::{EXTRAPOLATE_CONSTANT, EXTRAPOLATE_LINEAR};
/// Keyframe Type (`eBezTriple_KeyframeType`, kept in `BezTriple.hide`).
pub use anim::fcurve::{KEYTYPE_BREAKDOWN, KEYTYPE_EXTREME, KEYTYPE_GENERATED, KEYTYPE_JITTER, KEYTYPE_KEYFRAME, KEYTYPE_MOVEHOLD};
pub use anim::driver_list::BlendDriverList;
pub use anim::driver::chan::TRANSCHAN_SCALE_AVG;
pub use anim::driver::space::BDriverSpace;
pub use anim::driver::BlendDriverRead;
pub use anim::nla_eval::{BMotionClip, BMotionTrack};
pub use anim::nla_edit::{BNlaAt, BNlaPick, BNlaStackList, NEW_TRACK};
pub use anim::nla_stack::{BNlaPlaced, STASH_TRACK};
pub use anim::nla::{BLEND_ADD, BLEND_COMBINE, BLEND_MULTIPLY, BLEND_REPLACE, BLEND_SUBTRACT, STRIP_SYNC_LENGTH, STRIP_USR_INFLUENCE, STRIP_USR_TIME_CYCLIC, TRACK_PROTECTED, BNlaStrip, BNlaTrack, BlendNla, EXTEND_HOLD, EXTEND_HOLD_FORWARD, EXTEND_NOTHING, STRIP_ACTIVE, STRIP_CLIP, STRIP_META, STRIP_MUTED, STRIP_SELECT, STRIP_SOUND, STRIP_TRANSITION, TRACK_ACTIVE, TRACK_MUTED, TRACK_SELECTED, TRACK_SOLO, ADT_EDIT_ON, ADT_SOLO_TRACK, ADT_UI_ACTIVE, ADT_UI_SELECTED};
pub use anim::summary::{BlendActionSlot, BlendActionSummary};
pub use anim::follow::BMotionAt;
pub use anim::motion::{BlendMotion, BlendMotionList};
pub use anim::play::{BPlay, BPlayRef};
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
pub use scene::prop::BlendSceneProp;
pub use scene::kind::BlendBsdf;
pub use shade::link::BlendNormalMap;
pub use shade::world::BlendWorld;
pub use shade::map::BlendMapList;
pub use read::mesh::list::BFaceList;
pub use scene::ui::{BlendUiState, BlendView3d, RV3D_CAMOB, RV3D_ORTHO};
pub use shade::image::PackedImageList;

/// The Blend variant of a scene file.
pub struct BlendDevice;
impl BlendDevice {
    /// Install the batch skinner used for Armature deform while opening files
    /// (None restores the built-in CPU path).
    pub fn use_skinner(skinner: Option<std::sync::Arc<dyn Skinner>>) {
        set_skinner(skinner);
    }
    /// Modifier types the viewport evaluates (Blender RNA ids).
    pub fn modifier() -> Vec<&'static str> {
        supported_modifiers().collect()
    }
}
impl Device for BlendDevice {
    fn kind(&self) -> DeviceKind {
        DeviceKind::Blend
    }
    fn extension(&self) -> &'static [&'static str] {
        &["blend"]
    }
    /// `BLENDER` (plain, any header version) or the zstd frame magic of a
    /// compressed save (Blender 3.0+ writes either).
    fn detect(&self, head: &[u8]) -> bool {
        head.starts_with(b"BLENDER") || head.starts_with(&[0x28, 0xB5, 0x2F, 0xFD])
    }
    fn peek(&self, bytes: &[u8]) -> Result<Peek> {
        BlendFile::quick_info_bytes(bytes)
    }
    /// Streams the header blocks of the file; a 200 MB scene costs a few KB.
    fn peek_path(&self, path: &Path) -> Result<Peek> {
        BlendFile::read_quick_info(path)
    }
    /// `read` opens `Source::path` itself.
    fn streams(&self) -> bool {
        true
    }
    /// Evaluated, as `Phx::open` gives it. With a path, images resolve beside
    /// the file; from bytes alone they read as missing (packed ones still load).
    fn read(&self, source: &Source, progress: Progress) -> std::result::Result<SceneModel, SceneLoadError> {
        match source.path {
            Some(path) => {
                set_image_source(path);
                open::blend(path, progress)
            }
            None => {
                clear_image_source();
                open::blend_bytes(source.name, source.bytes, progress)
            }
        }
    }
    fn read_progressive(
        &self,
        source: &Source,
        progress: Progress,
    ) -> std::result::Result<(SceneModel, Option<DeferredModifierList>), SceneLoadError> {
        match source.path {
            Some(path) => {
                set_image_source(path);
                with_progressive_open(|| open::blend_progressive(path, progress))
            }
            None => {
                clear_image_source();
                with_progressive_open(|| open::blend_progressive_bytes(source.name, source.bytes, progress))
            }
        }
    }
    /// A scene is not written back as a `.blend`: the writer edits an existing
    /// file (`write`), and a `SceneModel` carries no Blender data blocks.
    fn write(&self, _scene: &SceneModel) -> std::result::Result<Vec<u8>, SceneLoadError> {
        Err(SceneLoadError::Unsupported("a scene cannot be written as .blend; edit an existing file with write::".to_string()))
    }
}
#[cfg(test)]
#[path = "test/blend/device.rs"]
mod device_test;
#[cfg(test)]
#[path = "test/blend/file.rs"]
mod file_test;

#[cfg(test)]
#[path = "test/blend/scene.rs"]
mod scene_test;
