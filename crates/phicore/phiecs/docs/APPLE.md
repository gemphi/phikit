# Apple's ECS contract, mapped onto phiecs

Read 2026-11-10 against the working tree. Nothing was built or run; every
line below is from reading source. Lines marked (checked) were re-opened by a
second reader after the survey; the rest rest on one reading.

phikit is the reality kit. Apple's reality kit is an ECS, and its whole
contract is in one file. This is that contract, row by row, against what
phiecs has, and the order to close the gaps in.

## Where the contract is

| Short | Path (from `gemphi/`) |
|---|---|
| **RF** | `phi/crates/kitpro/src/examples/headers/MacOSX-20260419T054331Z-3-001/MacOSX/RealityFoundation/SwiftInterfaces/arm64e-apple-macos.swiftinterface` (882 KB) |
| **SCN** | `phi/crates/kitpro/src/examples/headers/SceneKit.framework-20260419T035550Z-3-001/SceneKit.framework/Versions/A/Headers/` |
| **K** | `phikit/crates/phicore/phiecs/src/` (this crate) |
| **N** | `phinet/crates/phiecs/src/` (the full phiecs) |
| **PP, PX, PR, PB, SK** | `phikit/crates/{phiplay,phiphx,phiray,phiber,simkit}/src/` |

Two things to know before the table.

1. **The `RealityKit.framework` folders hold no ECS.** They carry shader
   headers and a `.tbd`. Every `Entity`, `Component` and `System`
   declaration is in RealityFoundation (RF).
2. **phiecs was modelled on the Rust transcription, not on Apple.** The
   table at N `lib.rs:15-34` maps `EntityGenerator`, `Storage<T>`,
   `EntityBuilder`, `SystemStage` and `EntityDescriptor`. None of those
   names is in RF (grep: 0 hits, checked). Apple's `System` has no stages:
   it orders by `dependencies` (RF:14757-14761, `SystemDependency` at
   RF:10308, checked). The five stages are ours, and worth keeping; they
   are just not Apple's.

## The mapping

| Apple interface | What it specifies | phiecs today | Gap | What to build |
|---|---|---|---|---|
| **Entity identity** | `id: UInt64` (RF:10610), `name` (RF:10560), `isEnabled` / `isActive` / `isEnabledInHierarchy` (RF:10569-10581), `clone(recursive:)` (RF:10629), `Scene.findEntity` (RF:8717) | `Word(u32)`, identity is the spelling (K `word.rs:7,38-46`); ids never reused (K `world/mod.rs:61-63`) | Two things with one name are one entity, so phiber spells `kind:name` (PB `part/project/db/mod.rs:140`). No enabled flag, no clone | Keep `Word`. `World::spawn(kind)` coins a unique `kind#n` with the display name as an aspect; an `Enabled` aspect |
| **Hierarchy** | `parent`, `setParent(_:preservingWorldTransform:)`, `children`, `addChild` (RF:15395-15414); `DidReparentEntity` (RF:7110) | `Tree` in `entity/tree.rs` (from N), held as `World.tree`; `World::set_parent(child, parent, keep_world)`, `clear_parent`, `world_of`; `DidReparent` emitted. **Met.** | None for `parent`, `children`, `setParent(_:preservingWorldTransform:)` and `DidReparentEntity`. Not met: `addChild`, `clone(recursive:)`. | Bring `Tree` (95 lines, dependency-free) into K `src/tree.rs`; `World.tree` |
| **Component** | `protocol Component` (RF:6537, checked), `registerComponent()` (RF:6568), `TransientComponent` (RF:6571), typed and type-erased `ComponentSet` (RF:10510-10529) | `Aspect` with `on_bind` / `on_unbind` (K `aspect.rs:7-10`), `Dense<A>`, `register/bind/read/read_mut/unbind` (K `world/mod.rs:87-107`) | An aspect has no name, so nothing can be listed, saved or addressed by string. `read_mut` hands out `&mut` unseen, so change detection is impossible | `Aspect::name()`, `World::aspect_list(word)`, a per-store changed tick, a `Transient` marker |
| **Queries** | `EntityQuery(where:)` (RF:1517, checked), `QueryPredicate.has` with `&&`, `\|\|`, `!` (RF:13375-13379, 13666) | K: `each::<A>()` only (K `world/mod.rs:110`). N: `with`, `With2`, `Related` (N `query/`) | No not, no or, no mutable walk. Consumers keep their own `Vec<Word>` and clone it per system (PP `play/world.rs:181,208`) | `Query::has::<A>().and::<B>().not::<C>().related(kind, to)`. The relation join is something Apple lacks |
| **Systems** | `System { static dependencies; init(scene:); update(context:) }` (RF:14757-14761), `.before` / `.after` (RF:10308-10312), `SystemUpdateCondition` (RF:14773) | `System { run, stage, name }`, five fixed stages (K `system.rs:8-40`, checked); order is stage then insertion | No ordering inside a stage, no run condition. simkit registers each system twice to get command-before-integrate (SK `ecs/world.rs:182-185`) | `after()` / `before()` by name, sorted once per stage; `when(&World) -> bool` |
| **Update loop and events** | `SceneEvents.Update`, `DidAddEntity`, `WillRemoveEntity` (RF:7086-7114); `ComponentEvents.DidAdd/WillRemove/DidChange` (RF:7221-7242); `subscribe` (RF:7403). SceneKit documents the phase order: update, animation, physics, constraints, render (SCN `SCNSceneRenderer.h:343-397`) | `EventQueue` resource in `system/event.rs`: typed double buffer, `emit`, `read`, `swap` last in `step`; `World::emit` / `events`; `DidAdd` and `WillRemove` emitted by `coin_bare` and `forget`. Stages documented as SceneKit's order on `Stage`. **Met** for lifecycle; `DidChange` waits for change ticks (order 4). | `DidChange` (RF:7242) needs the per-store changed tick. No `subscribe`, by design: readers pull. | An `EventQueue` resource: typed double buffer, `emit`, readable for one step. Pull, not closures. Document the stages as SceneKit's order: Ingest = input, PreCouple = animation, Couple = physics, PostCouple = constraints, Readout = render handoff |
| **Transform** | `Transform { scale, rotation, translation }`, `matrix` (RF:15094-15137), `position(relativeTo:)`, `convert`, `look(at:)` (RF:5810-5855) | `Pose { t, r, s }` and `WorldMatrix` in `comp/space/pose.rs` (plain f32, column-major `[[f32; 4]; 4]`), with `mul`, `inverse`, `point`, `pose`; `propagate` at PostCouple. **Met** for the pose and its derived matrix. | Not met: `position(relativeTo:)`, `convert`, `look(at:)`. A shear under non-uniform scale is lost in `WorldMatrix::pose`. | `Pose { t, r, s }` and `WorldMatrix` aspects as plain f32 arrays, a PostCouple `propagate` system over `Tree` |
| **Attachment** | `GeometricPin(named:skeletalJointName:...)` (RF:9088, checked), `AttachedTransformComponent { source, target }` (RF:13669, checked); physics joints are `pin0` / `pin1` (RF:2230) | A pin is a word: `World::pin(parent, name, offset)` makes a named child with an offset `Pose`; a part is a child of the pin, so its `WorldMatrix` is derived and cannot drift. **Met** structurally; no `Pin` or `Attached` type, as space.md says. | `AttachedTransformComponent { source, target }` as a type, and joint-named pins on a skeleton (`skeletalJointName`), wait for the skeleton row. | `Pin { joint, offset }` and `Attached { to, pin }`: the child's world matrix is derived from the parent's joint every step, so a gap cannot open |
| **Animation definition** | name, blendLayer, fillMode, bindTarget, trim, offset, delay, speed, repeatMode (RF:10419-10432); `AnimationLibraryComponent` (RF:9245) | Absent. Nearby: `BPlay` (PX `blend/anim/play.rs:18-33`), `BTape` (PP `play/mod.rs:63-72`) | A world can only emit animation through a tape in a resource | Do not port the definition zoo: phiphx's Blender model is the spec. Take only the header as a `Clip` aspect |
| **Playback and handoff** | `pause/resume/stop`, settable `time`, `speed`, `blendFactor` (RF:10444-10497); `AnimationHandoffType.snapshotAndReplace / replace / compose / stop` (RF:13799-13812, checked) | phiplay `BAct { mv, start, phase, was }`; `act()` keeps one previous move (PP `play/world.rs:140-149`); `record` blends over `BLEND` frames (PP `play/world.rs:216`) | **A pose pop, in the code as written (checked).** `act()` stores the moment the old move was interrupted at, then `record()` overwrites it with `t: 1.0`, the old move's END. And there is one `was` slot, so a third move inside the blend throws the blend away | A `Playing` aspect with `Handoff::{Snapshot, Replace, Compose, Stop}`. Snapshot keeps the last emitted pose as the blend source - Apple's `snapshotAndReplace` |
| **Bind targets** | `BindTarget`: `.transform`, `.jointTransforms`, `.opacity`, `.parameter`, paths (RF:463-586); `BindableValue { baseValue, animatedValue }` (RF:625) | Absent. Visibility is faked by moving an object 100 m down and scaling to 1e-3 (PB `scene/play/motion.rs:121-126`). Nearest idea: N `State<T>` now/next, unused by `World` | No `(entity, aspect, field)` address; no base versus animated value | `Bind { word, aspect, field }`, `Animated<T> { base, live }` |
| **Skeleton** | `Skeleton { joints[{ name, parentIndex, inverseBindPoseMatrix, restPoseTransform }] }` (RF:8030-8045), `SkeletalPosesComponent` (RF:5182) | Three unrelated skeletons, none an aspect: N kit (67-joint enum, linear find), phiplay `BSkeleton` (PP `pose/mod.rs:23-36`), phiphx rigs (PX `scene/rig.rs:104`) | Per-frame string matching per bone (PP `pose/solve.rs:38`); no shared type | One `Skeleton` with a name index; a `JointPose` aspect on the character |
| **IK** | `IKRig`, constraints with reach and pole vector (RF:4413-4498), `IKComponent` (RF:8238), animatable constraint targets (RF:573) | Absent in phiecs. phiphx has Blender's IK solver (PX `blend/rig/ik/mod.rs`); phiplay is FK and plants the lowest contact point vertically (PP `pose/solve.rs:58-71`) | The solver exists but nothing in the world can aim it | `Reach { joint, target, weight }` aspects written PostCouple, passed as IK targets by phiber's adapter |
| **Physics** | `PhysicsBodyComponent`, mode, `CollisionComponent` with filter, `PhysicsSimulationComponent`, five joints, `CollisionEvents.Began/Updated/Ended` (RF:2054-2295, 7163-7180, 13921) | Absent in phiecs. phiray `sim::World` has the arrays (PR `sim/state.rs:10-97`), `step`, `poses()` by index | Bodies are keyed by array index and name, not entity. No collision events | A bridge: `Body`, `Collider`, `Velocity` aspects; a Couple system that owns the `sim::World`, maps Word to body, emits `Contact` began/ended |
| **Saving** | `registerComponent`, `TransientComponent`, `Transform: Codable`, `Entity.write(to:)`, `Entity(contentsOf:)` (RF:6568, 12125, 13684, 15151) | K: none. N: edges only, as TSV rows (N `row.rs`, `world/rows.rs`) | No aspect can be written or read | `trait Stored: Aspect { KIND; fields(); from_fields() }` over a small dependency-free `Value`; `World::dump/load`. Mapped onto phiora's tables (`kinds`, `items`, `links`, `log`) in `connect.md` |
| **Sync** | `SynchronizationComponent { identifier, isOwner }` (RF:2815-2848) | Absent | No stable cross-process id, no owner | Two ideas only: a stable id (md5 of `kind:name`, as phiora) and an `Owner` aspect. Transport later |
| **Assets** | `Entity(named:) async`, `ReferenceComponent { loadingPolicy, state }` (RF:12124, 13729), `LowLevelMesh` in-place replace (RF:9585) | Absent. phiber builds a second read-only world of assets per scene (PB `part/project/db/mod.rs:132-216`) | Assets and scene entities live in different worlds; no load state | `Asset { kind, key }`, `Loaded { NotLoaded, Loading, Ready }`. In-place mesh replace is already `Tracer::repose` |
| **Time** | `SceneUpdateContext.deltaTime` (RF:14752, checked), controller `time` and `clock`. **Apple keeps no history**: state is now | `Clock { tick, time, dt, fps }` in `comp/time/clock.rs`, a resource advanced first in `step`; `Clock::tick_of_frame` maps a Blender frame to a tick. **Partly met.** | Still open: `Track<A>`, `World::at`, the `Journal` trait (order 3). | `Clock { tick, time, dt, fps }`; `Track<A>` for aspects marked timed; `World::at::<A>(word, tick)`; a `Journal` trait with a memory impl here and the database outside. A Blender frame maps to a tick through `SceneTimeline` (PX `scene/types_ext.rs:9-17`) |

## The two phiecs

- **K has no dependencies; N needs phistd, phiton, phical, serde, rayon, bincode.**
- **Relation numbering differs.** K starts from `Register::new()`, N from
  `Register::named()` with 16 kinds pre-declared (N `relation/set.rs:7-24`).
  Word ids still agree. Anything that persists `Relation::channel()` across
  the two will not.
- **`relate`**: K records the edge and ignores the rate (K `world/relate.rs:16-25`).
  N also counts evidence and binds a `Phasor` to both words.
- **N only**: `Tree`, `State`, `Under` conditions, rows, the rule layer,
  `with_mut` / `with2` / `related`, kit, quest, phasor. **K only**: nothing.
- **Drift.** `phinet/crates/phiano/docs/196_the_fold.md:11` rules that phikit
  takes phiecs as a submodule. Today K is a hand copy, which is how the two
  points above diverged. The clean fix is a feature gate in the phiecs repo
  so phikit takes levels 0-3 with no phinet path.

## Order of work
Done: 1 (clock, events, schedule) and 2 (tree, pose, propagate, pins as words), on files already moved under `entity`, `comp`, `system`. The paths in the list below are the old ones; PLAN.md has the real tree.

Each fits files under 250 lines, adds no phinet path, and keeps tables out
of code (the core has no file reader, so the app crate reads pin tables and
blend lengths and passes them in).

1. **Clock and events.** `src/clock.rs` (~50), `src/event.rs` (~120); move
   `add_system` / `step` into `world/schedule.rs` as N has it. Everything
   below stands on it.
2. **Tree, derived transforms, pins.** `src/tree.rs` (95, from N),
   `src/space/{pose,propagate,pin}.rs` (~250). Makes attachment structural.
3. **Journal: every entity over time.** `src/journal/{mod,memory,track}.rs`
   (~290), `src/stored.rs` (~70). The database impl sits outside the core,
   over phiora's tables (~200). Then `BTape` and `BBaked` become readers.
4. **Aspect names and change ticks.** `aspect.rs` 74 to ~140. Ends the
   pointer diff in phiber's `rendered/place.rs`.
5. **Snapshot handoff in phiplay.** `play/handoff.rs` (~80). The smallest
   change with the most visible effect: it removes the pop in the row above.
6. **System dependencies and run conditions.** ~110 lines.
7. **Query predicates with the relation join.** `src/query.rs` (~150).
8. **Physics bridge with collision events.** PB `scene/sim/ecs.rs` (~180).
9. **Playing, Bind, Animated.** `src/anim/` (~220). First use: an `Enabled`
   bind in place of hiding objects under the floor.
10. **Asset handles, one world.** ~80 lines; phiber's project database reads
    the scene world.

Two tables already sit in code and move with items 2 and 5: `CONTACT`
(PP `pose/solve.rs:30`) and `BLEND` (PP `play/world.rs:86`).

## Not verified

- "Never fractures" has two readings: a pose pop at a move handoff, or body
  parts separating. The handoff pop is read from the code. Whether callers
  in `play/invasion.rs` and `play/rally.rs` always wait for `done()` before
  `act()` was not traced, so it is not confirmed as the defect on screen.
- Apple's semantics beyond signatures are not in the interface file: what
  syncs, what `TransientComponent` skips, where systems run against
  animation and physics. Only SceneKit's phase order is documented.
- Only the macOS arm64e interface was read. The visionOS copy was not diffed.
- simkit is not a workspace member, so its phiecs use is not compiled today.
