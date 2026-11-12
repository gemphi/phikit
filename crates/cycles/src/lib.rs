// SPDX-FileCopyrightText: 2011-2022 Blender Foundation
// SPDX-License-Identifier: Apache-2.0
//! # cycles
//!
//! Blender's Cycles, ported to Rust top to bottom. The modules are Cycles'
//! own directories, in the order a render passes through them:
//!
//! ```text
//!  session      the render loop, its settings, its tiles, its buffer
//!     │
//!  integrator   the scheduler, one unit of its work, the steps a render takes
//!     │
//!  scene        what the scene asks for: its passes, its integrator, the device scene
//!     │
//!  bvh          the tree a ray walks to find what it hits
//!     │
//!  graph        nodes and sockets: what every part of a scene is made of
//!     │
//!  device       the device contract, the CPU device, the denoiser's settings
//!     │
//!  kernel       the kernels a path moves between
//!
//!  blender      phiphx's scene into a Cycles scene (the join)\n//!  truth        gate 0: render a test scene, judge it against Blender's image\n//!\n//!  util         the clock and the reader of `data/`, under all of them
//! ```
//!
//! Every file names the Cycles file it is ported from. A type keeps Cycles'
//! name made singular (`SessionParams` is [`session::SessionParam`]).

pub mod blender;
pub mod bvh;
pub mod device;
pub mod graph;
pub mod integrator;
pub mod kernel;
pub mod scene;
pub mod session;
pub mod truth;
pub mod util;

