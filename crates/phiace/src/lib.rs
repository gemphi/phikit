//! # phiace
//!
//! The robotic system of phikit. A robot is a word in the phiecs world: its
//! body, sensors and goals are relations under conditions, as a player is in
//! phiplay.
//!
//! `ext/` holds Meta's robotics and embodied-AI repositories as git submodules,
//! as references to read and port from, not code this crate builds (the same
//! rule as the reference trees noted in the workspace `Cargo.toml`). Rendering,
//! 3D and vision repositories sit with their counterpart in
//! `crates/phiray/reference` instead (Ocean, DRTK, PyTorch3D, SceneScript,
//! EFM3D, HOT3D, Project Aria tools).
//!
//! | submodule | what it is (from its GitHub description) |
//! |---|---|
//! | `ext/project_superdex` | a physics engine, robotics authoring tools and an RL interface in one simulation platform |
//! | `ext/fairo` | a modular embodied agent architecture (holds Polymetis and others) |
//! | `ext/habitat-lab` | a library to train embodied AI agents across tasks and environments |
//! | `ext/habitat-sim` | a 3D simulator for embodied AI research (archived upstream) |
//! | `ext/home-robot` | mobile manipulation research tools |
//! | `ext/partnr-planner` | the PARTNR benchmark: planning models for human-robot tasks |
//! | `ext/vjepa2` | V-JEPA 2: self-supervised video world model |
//! | `ext/nwm` | Navigation World Models (CVPR 2025) |
//! | `ext/eai-vc` | visual foundation models for embodied AI |
//! | `ext/theseus` | differentiable nonlinear optimisation |
//! | `ext/digit360` | the Digit 360 tactile sensor platform |
//! | `ext/neuralfeels` | visuo-tactile perception for in-hand manipulation |
//! | `ext/GeoRT` | geometric hand retargeting |
//! | `ext/BenchMARL` | multi-agent reinforcement learning benchmarks |
//!
//! `ext/fairo` is checked out without `droidlet/`: that folder holds file names
//! with a colon, which Windows cannot create. Clone it elsewhere to read droidlet.
//!
//! Nothing here is implemented yet. The nearest existing code is `simkit`
//! (AirSim's AirLib in Rust), which is not a workspace member.
