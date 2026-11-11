//! The PHX device: the engine's own scene file. `cloud/` is the point cloud
//! (`PHX\0` chunks), `scene/` the scene (`PHX_SCENE\x01` + JSON), `read.rs`
//! the asset-import view, `sidecar.rs` the `.phx` written beside other files,
//! `project.rs` the project folder.
pub mod cloud;
pub mod open;
pub mod project;
pub mod read;
pub mod scene;
pub mod sidecar;
use super::{Device, DeviceKind, Source};
use crate::device::Peek;
use crate::phx::Progress;
use crate::scene::{SceneLoadError, SceneModel};
#[cfg(test)]
#[path = "test/device.rs"]
mod device_test;
/// The PHX variant of a scene file.
pub struct PhxDevice;
impl Device for PhxDevice {
    fn kind(&self) -> DeviceKind {
        DeviceKind::Phx
    }
    fn extension(&self) -> &'static [&'static str] {
        &["phx"]
    }
    /// A PHX scene (`PHX_SCENE\x01`) or a PHX point cloud (`PHX\0`).
    fn detect(&self, head: &[u8]) -> bool {
        head.starts_with(scene::PHX_SCENE_MAGIC) || head.starts_with(cloud::chunk::PHX_MAGIC)
    }
    fn peek(&self, bytes: &[u8]) -> crate::Result<Peek> {
        open::peek(bytes)
    }
    fn read(&self, source: &Source, progress: Progress) -> Result<SceneModel, SceneLoadError> {
        if !progress(0.05, "Reading PHX") {
            return Err(SceneLoadError::Cancelled);
        }
        let model = open::scene(source)?;
        progress(1.0, "Done");
        Ok(model)
    }
    fn write(&self, scene: &SceneModel) -> Result<Vec<u8>, SceneLoadError> {
        open::write(scene)
    }
}