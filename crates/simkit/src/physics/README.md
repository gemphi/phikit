---
id: simkit-physics
title: Physics Engine
custom_edit_url: https://github.com/gemphi/phi/edit/main/crates/simkit/src/physics/README.md
---

# SimKit Physics Engine

This module implements a fast, deterministic kinematic and rigid-body physics engine tailored for drones and cars.

## Core Components

*   **`physics_engine.rs`**: The main execution loop, state management, and collision detection.
*   **`physics_body.rs`**: Defines rigid bodies, inertia tensors, wrenches (forces and torques), and kinematics state.
*   **`physics_world.rs`**: Manages gravity, magnetic field modeling, and environment queries.

The physics engine is designed to run in a headless environment, making it perfect for training machine learning models or testing control algorithms rapidly without the overhead of a full 3D renderer like Unreal Engine.
