//! Scene assembly. `extract_scene`, scene properties, result types, saved UI (workspace, 3D view).
//! Status and next steps: `jobs.md` in this folder.

pub mod cursor;
pub mod marker;
pub mod overlay;
#[cfg(test)]
mod overlay_tests;
pub mod movie;
pub mod output;
pub mod pass;
pub mod playback;
pub mod props;
pub(crate) mod read;
pub mod shading;
pub mod border;
pub mod bsdf;
pub mod types;
pub mod ui;
