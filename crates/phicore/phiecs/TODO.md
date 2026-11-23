# phiecs — to do

The order is in [PLAN.md](PLAN.md); what each part must do is in
[SPECS.md](SPECS.md); the diagrams are in [ARCHITECTURE.md](ARCHITECTURE.md).
Every file stays under 250 lines and the crate keeps no dependencies.

## the three folders

- [ ] move `word.rs`, `relation.rs`, `world/` under `src/entity/`
- [ ] move `aspect.rs`, `resources.rs` under `src/comp/`
- [ ] move `system.rs` to `src/system/stage.rs`
- [ ] `lib.rs` re-exports every path that exists today, so phiber, phiplay, phiace and simkit change nothing

## entity

- [ ] `entity/kind.rs`: `Kind`, declared from a list passed in; `World::kind_of`
- [ ] `entity/spawn.rs`: `World::spawn(kind)`, `forget` with its event, `Enabled`
- [ ] `entity/tree.rs` from phinet's `node/tree.rs`; `set_parent(child, parent, keep_world)`
- [ ] `entity/query.rs`: `has`, `and`, `not`, `related(kind, to)`, a mutable walk

## comp

- [ ] `comp/aspect.rs`: `Aspect::name()`, `World::aspect_list(word)`, a changed tick per store, `Transient`
- [ ] `comp/stored.rs`: `Stored`, `Value`, the stable id, `Owner`
- [x] `comp/pose.rs`: `Pose { translate, orient, scale }` as USD's xformOps, `Matrix` (2026-11-23; at `comp/` for now, `comp/space/` when the folders move)
- [x] `comp/tree.rs`: `Parent`, `Placed`, `adopt`, `children`, `matrix`, `place` (2026-11-23; `set_parent(keep_world)` still to do)
- [ ] `comp/time/clock.rs`: `Clock { tick, time, dt, fps }`
- [ ] `comp/time/track.rs`, `journal.rs`: `Track<A>`, the `Journal` trait, the in-memory journal
- [ ] `comp/anim/bind.rs`: `Bind { word, aspect, field }`, `Animated<T> { base, live }`
- [ ] `comp/anim/sample.rs`: the `Sample` trait
- [ ] `comp/anim/playing.rs`: `Playing`, `Handoff::{Snapshot, Replace, Compose, Stop}`, the blend rule a field kind
- [ ] `comp/anim/channel.rs`: `ChannelSet`
- [ ] `comp/asset.rs`: `Asset { kind, key }`, `Loaded`

## system

- [ ] `system/schedule.rs`: `add_system` and `step` move here; the clock first, the event swap last
- [ ] `system/stage.rs`: `after` / `before` by name, sorted once a stage, a cycle refused; `when`
- [ ] `system/layer.rs`: `PerKind`; the roles `Intake`, `Animate`, `Simulate`, `Constrain`, `Readout`; `Plugin`
- [ ] `system/event.rs`: typed double buffer, readable for one step
- [ ] `system/space/propagate.rs`: PostCouple, roots first
- [ ] `system/time/record.rs`: Readout rows to every sink; `World::at`, `seek`, `dump`, `load`
- [ ] `system/anim/advance.rs`, `handoff.rs`: PreCouple

## connect

- [ ] a `Sink` for rows and events: the in-memory journal here, the session file and the database outside
- [ ] the kinds and aspect names the database knows are the ones declared here: one list, in a data file
- [ ] see [docs/connect.md](docs/connect.md)

## outside the crate

- [ ] phiplay `play/handoff.rs`: snapshot handoff, removes the pop at `play/world.rs:216`
- [ ] phiphx feature `ecs`: scene to words, actions as `Sample`, the rig solve as `Constrain`
- [ ] the scene glue: `Simulate` over `phiray::sim`, `Readout` to `Tracer::reinstance` and `repose`
- [ ] a feature gate in the phiecs repo so phikit takes the core with no phinet path, ending the hand copy

## usd (2026-11-23)

The world reads OpenUSD, as Reality Composer Pro writes it: a scene is
data, and the same file is what the world runs.

- [x] `usd/token.rs`, `read.rs`, `layer.rs`: a text layer (`.usda`) into prims: spec, type, name, metadata, properties, children
- [x] `usd/stage.rs`: a stage that composes references and payloads to text layers, local opinions over referenced ones; binary layers kept as the prim's asset
- [x] `usd/intake.rs`: prims into words (spelled under their parent), `Pose` from xformOps, `Kind`, and Reality Composer Pro components as a `ComponentSet`
- [x] tests: a small greenhouse, and Apple's own BOT-anist `volume.usda` with its three planters
- [ ] binary layers (`.usdc`, inside `.usdz`): the `openusd` 0.3 crate is in the offline cache; `ext/phiber-refs/apple/src/reality-foundation/scene_usd` reads meshes with it
- [ ] variant sets, inherits, specializes, sublayers, time samples into `Track`s
- [ ] reading is slow in a debug build: Apple's greenhouse is 36 MB of text across its layers, about 15 s
