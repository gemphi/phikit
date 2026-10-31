---
id: simkit-mavlink
title: MAVLink
custom_edit_url: https://github.com/gemphi/phi/edit/main/crates/simkit/src/mavlink/README.md
---

# SimKit MAVLink Integration

This module provides a robust implementation of the MAVLink communication protocol for the SimKit toolkit, enabling interaction with external ground control stations (like QGroundControl) or flight controllers.

## Contents

*   **`connection.rs`**: Handles TCP/UDP and Serial connection handling for streaming MAVLink packets.
*   **`messages.rs`**: Definitions for standard MAVLink messages (Heartbeat, SysStatus, GpsRawInt, Attitude, GlobalPositionInt, etc.).
*   **`mod.rs`**: System definitions and integration entry points.

This implementation allows the Rust-Sim-Kit simulated multirotor to appear as a physical drone to MAVLink-compatible software.
