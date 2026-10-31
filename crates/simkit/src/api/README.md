---
id: simkit-api
title: Simulation API
custom_edit_url: https://github.com/gemphi/phi/edit/main/crates/simkit/src/api/README.md
---

# SimKit API Module

This module defines the core data types and structures used for communicating with the Simulation Engine, closely mirroring AirLib's capabilities.

## Contents

*   **`mod.rs`**: Core types (`VehicleState`, `MultirotorState`, `CarState`, `CarControls`, etc.) and Enums (`WeatherParameter`, `DrivetrainType`, `YawMode`, etc.).
*   **`client/`**: The `msgpack-rpc` implementation of the client, broken down into manageable submodules.

These structures represent the public API boundary that clients (like `phiace` rendering crates or autonomous flight controllers) use to read and interact with the simulation.
