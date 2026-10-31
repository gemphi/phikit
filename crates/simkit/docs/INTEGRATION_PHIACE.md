---
id: simkit-phiace
title: Rendering with Phiace
custom_edit_url: https://github.com/gemphi/phi/edit/main/crates/simkit/docs/INTEGRATION_PHIACE.md
---

# SimKit & Phiace Rendering Integration

**Can we use Phiace crates to render SimKit after the refactoring?**
**Yes, absolutely.**

## The Integration Strategy
`phiace` is the workspace's WUI (Web/Window User Interface) kit. It already contains VSCode-like editor tabs, sidebars, and references to the `Fyrox` 3D engine (`phiace/REFS/Fyrox-master/`).

### 1. 3D Viewport Tab
- `SimKit` runs headless physics and manages state via `RpcClient` or directly in memory.
- `phiace` can open an Editor Tab (`crates/apple/apps/VSCODE/editor/tabs.rs`) dedicated to a **3D Viewport**.
- Using the `Fyrox` engine backend within `phiace`, you can consume the `Pose` and `KinematicsState` from `SimKit` at 60Hz and update 3D mesh transforms (Drone/Car models) in the viewport.

### 2. Camera Streams (PIP)
- `SimKit` provides a `sim_get_image` API. 
- `phiace` can render this byte buffer into an image widget within a Sidebar or Status Bar (`crates/apple/apps/VSCODE/status_bar.rs`), giving the user a live FPV (First Person View) of the drone/car.

### 3. Telemetry Dashboards
- The `tree.rs` (File Explorer) concept in `phiace` can be mirrored to create an "Entity Explorer".
- Telemetry from `ImuOutput`, `GpsOutput`, and `BarometerOutput` can be rendered in real-time text panels within the `phiace` WUI kit.
