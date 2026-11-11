//! Devices: one per scene-file variant. `Phx` finds the device and delegates;
//! a device owns reading and writing its own variant and nothing else.
//!
//! Like Cycles' render devices, they share one contract and know nothing of
//! each other. The list and its fixed order are in [`list`]; the layout and
//! the phases that fill it are in `LAYOUT.md`.
pub mod blend;
pub mod list;
pub mod usd;

pub use list::DeviceList;

use crate::phx::{Peek, Progress};
use crate::scene::{DeferredModifiers, SceneLoadError, SceneModel};
use std::path::Path;

/// Which variant a device reads and writes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceKind {
    Usd,
    Blend,
    Phx,
}

/// Where a scene comes from: a name, the path when there is one (textures and
/// sidecars are found beside it), and the bytes.
#[derive(Clone, Copy, Debug)]
pub struct Source<'a> {
    pub name: &'a str,
    pub path: Option<&'a Path>,
    pub bytes: &'a [u8],
}

/// What a scene-file variant must be able to do.
pub trait Device: Sync {
    fn kind(&self) -> DeviceKind;

    /// File extensions the device owns: lower case, no dot.
    fn extension(&self) -> &'static [&'static str];

    /// Whether the head of a file (its first bytes) is this variant's.
    fn detect(&self, head: &[u8]) -> bool;

    /// Header-only look: version, render info, thumbnail. No scene is built.
    fn peek(&self, bytes: &[u8]) -> crate::Result<Peek>;

    /// Read into a scene. A device that evaluates (Blend) hands back the
    /// evaluated scene.
    fn read(&self, source: &Source, progress: Progress) -> Result<SceneModel, SceneLoadError>;

    /// The progressive open path: the base scene now, the deferred work for
    /// the host to finish after. A device with nothing to defer reads whole.
    fn read_progressive(
        &self,
        source: &Source,
        progress: Progress,
    ) -> Result<(SceneModel, Option<DeferredModifiers>), SceneLoadError> {
        Ok((self.read(source, progress)?, None))
    }

    /// Write a scene as this variant's file.
    fn write(&self, scene: &SceneModel) -> Result<Vec<u8>, SceneLoadError>;
}
