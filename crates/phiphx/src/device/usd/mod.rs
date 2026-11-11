//! The USD device: `.usd`, `.usda`, `.usdc` and `.usdz`.
//!
//! The object model is usdkit's (stage, prim, attribute, xform). Reading is in
//! [`read`] (the usdkit parser, and the heuristic readers kept as fallbacks),
//! writing in [`write`] (the printer, and the `SceneModel` writer), the zip
//! container in `package`.
pub mod attribute;
pub mod error;
#[cfg(feature = "usdz-import")]
pub mod package;
pub mod prim;
pub mod read;
pub mod stage;
#[cfg(test)]
mod test;
pub mod write;
pub mod xform;

pub use attribute::{UsdAttribute, UsdValue};
pub use error::UsdaError;
pub use prim::UsdPrim;
pub use read::UsdaParser;
pub use stage::{UsdStage, UsdUpAxis};
pub use write::UsdaPrinter;
pub use xform::XformResolver;

use super::{Device, DeviceKind, Source};
use crate::device::Peek;
use crate::phx::Progress;
use crate::scene::{SceneLoadError, SceneModel};

/// The USD variant of a scene file.
pub struct UsdDevice;

impl Device for UsdDevice {
    fn kind(&self) -> DeviceKind {
        DeviceKind::Usd
    }

    fn extension(&self) -> &'static [&'static str] {
        &["usd", "usda", "usdc", "usdz"]
    }

    fn detect(&self, head: &[u8]) -> bool {
        read::mesh::UsdImporter::can_import(head)
    }

    fn peek(&self, bytes: &[u8]) -> crate::Result<Peek> {
        read::peek::peek(bytes)
    }

    fn read(&self, source: &Source, progress: Progress) -> Result<SceneModel, SceneLoadError> {
        if !progress(0.05, "Reading USD") {
            return Err(SceneLoadError::Cancelled);
        }
        let model = read::scene::open(source)?;
        progress(1.0, "Done");
        Ok(model)
    }

    fn write(&self, scene: &SceneModel) -> Result<Vec<u8>, SceneLoadError> {
        Ok(scene.to_usda().into_bytes())
    }
}
