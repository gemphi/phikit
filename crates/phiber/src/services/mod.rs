//! `phiber::services` — pure-Rust data models and state services that
//! sit under the render layer.
//!
//! These modules originated in `apple::services::*` in the sibling phiace
//! workspace. They are pulled into phiber here so the app builds
//! standalone inside the zed workspace, with no external `apple`
//! dependency required.
//!
//! Requirements (phiber-blender-space-parity):
//! - `node_graph`: 22.1, 22.5–22.10, 23.1, 24.1, 25.1, 52.5–52.7, 52.15
//! - `outliner`:   14.1, 15.1, 16.x, 17.x, 18.x, 19.x, 21.x, 52.8–52.10

#![allow(dead_code)]

pub mod node_graph;
pub mod outliner;
