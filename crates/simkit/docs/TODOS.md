---
id: simkit-todos
title: SimKit Refactoring Tasks
custom_edit_url: https://github.com/gemphi/phi/edit/main/crates/simkit/docs/TODOS.md
---

# SimKit Jobs & Tasks

## Epic: 200-Line Rule Compliance
Currently, many files heavily exceed the workspace 200-line limit.

### Job 1: Modularize `api/client.rs` (984 lines)
- [ ] Create `src/api/client/` directory.
- [ ] Move base connection logic to `connection.rs`.
- [ ] Move sensor getter RPCs to `sensors.rs`.
- [ ] Move vehicle control RPCs to `vehicles.rs`.
- [ ] Move camera/image RPCs to `vision.rs`.

### Job 2: Modularize `mavlink/messages.rs` (531 lines)
- [ ] Extract heartbeat/status messages.
- [ ] Extract RC override and command messages.
- [ ] Document missing struct fields to resolve `cargo clippy` warnings.

### Job 3: Modularize `physics/physics_engine.rs` (419 lines)
- [ ] Split collision detection logic.
- [ ] Split kinematics state updates.

## Epic: Phiace Integration
- [ ] Expose public observable states for rendering (see `INTEGRATION_PHIACE.md`).
- [ ] Create zero-copy buffers for camera image streams to UI viewports.
