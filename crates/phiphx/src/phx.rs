//! **The phiphx front door.** Everything a host does with a file goes
//! through [`Phx`]: open it into a [`SceneModel`], peek at its preview
//! without parsing, or detect its format. `Phx` has no format logic of its own:
//! it asks the [`DeviceList`] which device owns the file (by its bytes first,
//! then by its extension) and the device does the work. A file no device
//! claims is handed to the Blend device, which says why it cannot read it.
use crate::device::blend::BlendDevice;
use crate::device::{Device, DeviceKind, DeviceList, Source};
use crate::import::FileFormat;
use std::path::Path;
pub use crate::device::Peek;
pub use crate::device::blend::{CpuSkin, SKIN_MIN, SkinJob, SkinPair, Skinner};
pub use crate::scene::{SceneLoadError as PhxError, SceneModel};
/// Progress sink: `(fraction 0..=1, stage)`; return `false` to cancel.
pub type Progress<'a> = &'a (dyn Fn(f32, &str) -> bool + Sync);
/// How much of a file's head decides its device.
const HEAD: usize = 16;
/// The entry point. Stateless: every call is independent and thread-safe.
pub struct Phx;
impl Phx {
    /// Open a file into a scene (no progress, no cancel).
    pub fn open(path: &Path) -> Result<SceneModel, PhxError> {
        Self::open_with(path, &|_, _| true)
    }
    /// Open a file into a scene, reporting progress.
    pub fn open_with(path: &Path, progress: Progress) -> Result<SceneModel, PhxError> {
        let device = Self::device_of(path);
        Self::with_source(device, path, |source| device.read(source, progress))
    }
    /// Open a file progressively: the base scene now, the modifier work
    /// deferred. Returns the base [`SceneModel`] plus a [`DeferredModifierList`]
    /// (when any object carries a modifier) for the host to finish on a worker
    /// thread and swap in — base meshes reach the viewport without waiting on
    /// subdivision, arrays or armature deform. Blender's depsgraph does the
    /// same: the file opens, then the evaluated meshes arrive.
    pub fn open_progressive(
        path: &Path,
        progress: Progress,
    ) -> Result<(SceneModel, Option<crate::scene::DeferredModifierList>), PhxError> {
        let device = Self::device_of(path);
        Self::with_source(device, path, |source| device.read_progressive(source, progress))
    }
    /// Open a file that is already in memory, under the name to record
    /// as its source.
    ///
    /// For a caller with no filesystem to offer: a browser's file picker, an
    /// HTTP body, an asset server. `name` is only what the scene reports as
    /// its origin — it is never opened, and it need not exist.
    ///
    /// The one thing this cannot do is resolve an image the file refers to by
    /// path (`//textures/bark.png`): there is no folder to resolve against,
    /// so those read as missing, which is what Blender shows for an image it
    /// cannot find. Packed images are unaffected.
    pub fn open_bytes(name: &str, data: &[u8]) -> Result<SceneModel, PhxError> {
        Self::open_bytes_with(name, data, &|_, _| true)
    }
    /// [`Phx::open_bytes`], reporting progress.
    ///
    /// Progress starts at the "Indexing blocks" stage rather than "Reading
    /// file": the caller has already done the reading.
    pub fn open_bytes_with(name: &str, data: &[u8], progress: Progress) -> Result<SceneModel, PhxError> {
        let device = Self::device_by(data, Path::new(name));
        device.read(&Source { name, path: None, bytes: data }, progress)
    }
    /// Install the batch skinner used for Armature deform while opening files
    /// (Phiber installs a GPU one; `None` restores the built-in CPU path).
    pub fn use_skinner(skinner: Option<std::sync::Arc<dyn Skinner>>) {
        BlendDevice::use_skinner(skinner);
    }
    /// Version, render info and embedded thumbnail — header blocks only.
    pub fn peek(path: &Path) -> crate::Result<Peek> {
        Self::device_of(path).peek_path(path)
    }
    /// [`Phx::peek`] from bytes already in memory — header blocks only, so
    /// it reads the thumbnail and frame range without parsing the file.
    pub fn peek_bytes(data: &[u8]) -> crate::Result<Peek> {
        Self::device_by(data, Path::new("")).peek(data)
    }
    /// Modifier types the viewport evaluates (Blender RNA ids).
    pub fn modifiers() -> Vec<&'static str> {
        BlendDevice::modifier()
    }
    /// Format from magic bytes, else from the extension.
    pub fn detect(path: &Path, head: &[u8]) -> Option<FileFormat> {
        FileFormat::detect(head)
            .or_else(|| {
                DeviceList::new().detect(head).map(|d| match d.kind() {
                    DeviceKind::Usd => FileFormat::Usd,
                    DeviceKind::Blend => FileFormat::Blend,
                    DeviceKind::Phx => FileFormat::Phx,
                })
            })
            .or_else(|| {
                let ext = path.extension()?.to_str()?;
                FileFormat::from_extension(ext)
            })
    }
    /// The device that owns the file at `path`: by its first bytes, else by
    /// its extension, else Blend (whose error says what is wrong).
    fn device_of(path: &Path) -> &'static dyn Device {
        let mut head = [0u8; HEAD];
        let n = std::fs::File::open(path)
            .and_then(|mut f| std::io::Read::read(&mut f, &mut head))
            .unwrap_or(0);
        Self::device_by(&head[..n], path)
    }
    /// The device that owns a head of bytes, else the one the extension of
    /// `name` names, else Blend.
    fn device_by(head: &[u8], name: &Path) -> &'static dyn Device {
        let list = DeviceList::new();
        list.detect(head)
            .or_else(|| list.by_extension(name.extension()?.to_str()?))
            .or_else(|| list.get(DeviceKind::Blend))
            .expect("the Blend device is always in the list")
    }
    /// Run `read` over the file at `path`, with its bytes in memory unless the
    /// device reads the file itself.
    fn with_source<R>(
        device: &dyn Device,
        path: &Path,
        read: impl FnOnce(&Source) -> Result<R, PhxError>,
    ) -> Result<R, PhxError> {
        let name = path.to_string_lossy();
        if device.streams() {
            return read(&Source { name: &name, path: Some(path), bytes: &[] });
        }
        let bytes = std::fs::read(path).map_err(|e| PhxError::Inaccessible(e.to_string()))?;
        read(&Source { name: &name, path: Some(path), bytes: &bytes })
    }
}/// Test-only hook exposing the private [`classify_native_error`] so the
/// sibling `tests_load` file can assert its category mapping without
/// widening the production surface.
#[cfg(test)]
pub(crate) fn classify_native_error_test_hook(
    err: crate::PointCloudError,
) -> PhxError {
    crate::device::blend::open::classify_native_error(err)
}

#[cfg(test)]
#[path = "test/phx.rs"]
mod phx_test;
