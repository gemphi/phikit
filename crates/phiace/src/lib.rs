//! # phiace
//!
//! The robotic system of phikit. A robot is a word in the phiecs world: its
//! body, sensors and goals are relations under conditions, as a player is in
//! phiplay.
//!
//! `reference/` holds Meta's robotics and embodied-AI repositories as git
//! submodules, as references to read and port from, not code this crate
//! builds. Every crate keeps its guiding repositories in its own `reference/`
//! folder: renderers sit in `crates/phiray/reference`, characters, rigs and
//! geometry in `crates/phiphx/reference`.
//!
//! | submodule | what it is (from its GitHub description) |
//! |---|---|
//! | `reference/project_superdex` | a physics engine, robotics authoring tools and an RL interface in one simulation platform |
//! | `ext/fairo` | a modular embodied agent architecture (holds Polymetis and others) |
//! | `reference/habitat-lab` | a library to train embodied AI agents across tasks and environments |
//! | `reference/habitat-sim` | a 3D simulator for embodied AI research (archived upstream) |
//! | `reference/home-robot` | mobile manipulation research tools |
//! | `reference/partnr-planner` | the PARTNR benchmark: planning models for human-robot tasks |
//! | `reference/vjepa2` | V-JEPA 2: self-supervised video world model |
//! | `reference/nwm` | Navigation World Models (CVPR 2025) |
//! | `reference/eai-vc` | visual foundation models for embodied AI |
//! | `reference/theseus` | differentiable nonlinear optimisation |
//! | `reference/digit360` | the Digit 360 tactile sensor platform |
//! | `reference/neuralfeels` | visuo-tactile perception for in-hand manipulation |
//! | `reference/GeoRT` | geometric hand retargeting |
//! | `reference/BenchMARL` | multi-agent reinforcement learning benchmarks |
//!
//! `fairo` is still at `ext/fairo`: a process held its folder open when the
//! others moved (2026-11-10), so it follows with
//! `git mv crates/phiace/ext/fairo crates/phiace/reference/fairo`. It is
//! checked out without `droidlet/`: that folder holds file names with a
//! colon, which Windows cannot create. Clone it elsewhere to read droidlet.
//!
//! Nothing here is implemented yet. The nearest existing code is `simkit`
//! (AirSim's AirLib in Rust), which is not a workspace member.
