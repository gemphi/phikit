# phiecs — to do

The order is the order of work in [docs/APPLE.md](docs/APPLE.md); the
diagrams are in [ARCHITECTURE.md](ARCHITECTURE.md). Every file stays under
250 lines and the crate keeps no dependencies.

## clock, events, schedule

- [ ] `src/clock.rs`: `Clock { tick, time, dt, fps }` as a resource, advanced by `step`
- [ ] `src/events.rs`: typed double buffer, `emit`, readable for one step
- [ ] `src/world/schedule.rs`: `add_system` and `step` move here
- [ ] `System::after` / `before` by name, sorted once per stage, a cycle refused
- [ ] `System::when(&World) -> bool`

## tree, space

- [ ] `src/tree.rs` from phinet's `node/tree.rs`; `World.tree`
- [ ] `src/space/pose.rs`: `Pose { t, r, s }`, `WorldMatrix`
- [ ] `src/space/propagate.rs`: PostCouple system, roots first
- [ ] `src/space/pin.rs`: `Pin { joint, offset }`, `Attached { to, pin }`
- [ ] `World::set_parent(child, parent, keep_world)`

## stored, journal

- [ ] `src/stored.rs`: `Stored: Aspect { KIND; fields(); from_fields() }`, `Value`
- [ ] `src/journal/{mod,memory,track}.rs`: `Journal`, `Track<A>`, `World::at`
- [ ] `World::dump` / `load`; stable id and `Owner`
- [ ] the database impl, outside the core, in phiora's table shape

## aspect

- [ ] `Aspect::name()`, `World::aspects_of(word)`
- [ ] a changed tick per store, set by `read_mut`
- [ ] a `Transient` marker; an `Enabled` aspect; `World::spawn(kind)`

## query

- [ ] `src/query.rs`: `has`, `and`, `not`, `related(kind, to)`, a mutable walk

## rig, anim

- [ ] `src/rig/`: `Skeleton` with a name index, `JointPose`, `Reach`
- [ ] `src/anim/`: `Clip`, `Playing`, `Handoff::{Snapshot, Replace, Compose, Stop}`
- [ ] `Bind { word, aspect, field }`, `Animated<T> { base, live }`
- [ ] phiplay `play/handoff.rs`: snapshot handoff, removes the pop at `play/world.rs:216`

## asset, bridges

- [ ] `src/asset.rs`: `Asset { kind, key }`, `Loaded`
- [ ] phiber `scene/sim/ecs.rs`: `Body`, `Collider`, `Velocity`, `Contact` over `phiray::sim::World`
- [ ] Readout systems: `WorldMatrix` to `Tracer::reinstance`, `JointPose` to `Tracer::repose`

## the two phiecs

- [ ] a feature gate in the phiecs repo so phikit takes the core with no phinet path, ending the hand copy
