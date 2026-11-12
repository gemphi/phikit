# phiecs — specs

What each part must do. One row is one requirement, with the Apple source
it answers and what else was read. "Today" says whether the crate already
meets it. The longer account of each gap is [docs/APPLE.md](docs/APPLE.md).

Two rules stand over every row. **General only:** phiecs holds nothing
that knows what a bone, a mesh or a robot is. **One owner:** a concept
phiphx or phiray already owns is used here through a handle or a trait,
never defined again.

## Sources

| Short | What | Where (from `gemphi/`) |
|---|---|---|
| **RF** | RealityFoundation, the whole ECS contract, one file | `phi/crates/kitpro/src/examples/headers/MacOSX-20260419T054331Z-3-001/MacOSX/RealityFoundation/SwiftInterfaces/arm64e-apple-macos.swiftinterface` |
| **RF, other platforms** | the same interface for iPhoneOS and XROS | `phi/apple-source/ruskit/Interfaces/{MacOSX,iPhoneOS,XROS}/RealityFoundation` |
| **SCN** | SceneKit headers: the documented frame order | `phi/crates/kitpro/src/examples/headers/SceneKit.framework-20260419T035550Z-3-001/SceneKit.framework/Versions/A/Headers/` |
| **N** | the full phiecs | `phinet/crates/phiecs/src/` |
| **MOM** | momentum, the character model | `phikit/crates/phiphx/reference/momentum/momentum/` |
| index | every path above, kept current | [reference/README.md](reference/README.md) |
| index | the character references | [../../phiphx/reference/README.md](../../phiphx/reference/README.md) |

Apple's interface gives signatures, not behaviour. Where a row states
behaviour Apple does not write down, the row says "ours".

## entity

| Part | Must | Apple | Also read | Today |
|---|---|---|---|---|
| `Word` | be a stable id for the life of the world; ids are never reused | `id` RF:10610 | - | met |
| `Kind` | give every word one kind, from a list declared at start and passed in as data; the same names the database keys on (ours) | - | - | met: `entity/kind.rs` |
| `World::spawn(kind)` | coin a unique word each call, with the display name as an aspect, so two things may share a name | `name` RF:10560 | - | met: `entity/spawn.rs` |
| `Enabled` | switch a word and its subtree off without removing it | RF:10569-10581 | - | met |
| `forget` | emit `WillRemove` before the word's aspects and edges go | RF:7086-7114 | - | met: `WillRemove` first |
| `Tree` | give each word at most one parent and refuse a loop | `parent`, `children`, `addChild` RF:15395-15414 | N `node/tree.rs:34-93` | no |
| `set_parent(.., keep_world)` | when asked, keep the child's world transform across the move; emit `DidReparent` | `setParent(_:preservingWorldTransform:)` RF:15395, `DidReparentEntity` RF:7110 | - | no |
| pin | be a word: a named child with an offset `Pose`. Whatever is attached is its child, so its place is derived and cannot disagree (ours) | `GeometricPin` RF:9088, `AttachedTransformComponent` RF:13669 | MOM `character/locator.h:19-26` | no |
| `Query` | select words by kind and by aspects held, with and, or, not, and walk them mutably | `EntityQuery(where:)` RF:1517, `QueryPredicate` RF:13375-13379 | N `query/` | `each::<A>()` only |
| `related(kind, to)` | join a query on a relation (ours: Apple has no relations) | - | N `query/` | no |

## comp

| Part | Must | Apple | Also read | Today |
|---|---|---|---|---|
| `Aspect` | be plain data bound to a word, with bind and unbind hooks | `Component` RF:6537 | - | met |
| `Aspect::name()` | name the aspect so it can be listed, saved and addressed by text | `registerComponent` RF:6568 | - | met |
| change tick | record the tick of the last `read_mut` per store, so readers ask "changed since" | `DidChange` RF:7242 | - | met: a tick a slot, `DidChange` |
| `Transient` | mark an aspect that is never saved or journalled | `TransientComponent` RF:6571 | - | met: `Aspect::TRANSIENT` |
| `Pose` | be translation, rotation, scale as plain `f32` arrays, for anything that has a place: an object, a bone, a link, a pin | `Transform` RF:15094-15137 | - | no |
| `WorldMatrix` | be derived every step, parents before children, and never written by hand (ours) | `position(relativeTo:)`, `convert` RF:5810-5855 | - | no |
| `Clock` | hold `tick`, `time`, `dt`, `fps`; advance once at the start of `step` | `deltaTime` RF:14752 | - | no |
| `Stored` | turn an aspect into named fields of `Value` and back, with no dependency | `Transform: Codable`, `Entity.write(to:)` RF:12125, 13684, 15151 | - | met: `comp/stored.rs` |
| `Journal` | keep, for each timed aspect, its value at every tick it changed; answer `at(word, tick)` with the value at or before that tick (ours: Apple keeps no history) | - | - | met: `MemoryJournal` |
| stable id, `Owner` | give a word an id that is the same in another process, and say who may write it | `SynchronizationComponent` RF:2815-2848 | - | met |
| `Asset`, `Loaded` | name what a word is drawn from, by kind and key, and whether it is ready. The asset itself stays with its owner | `ReferenceComponent` RF:13729, `Entity(named:)` RF:12124 | - | no |

## comp/anim: one path for any field

| Part | Must | Apple | Also read | Today |
|---|---|---|---|---|
| `Bind` | address one field of one aspect of one word, whatever the aspect is | `BindTarget` RF:463-586 | - | no |
| `Animated<T>` | keep a base value beside the animated one | `BindableValue` RF:625 | - | no |
| `Sample` | give values at a time, one a channel. Implemented by whoever owns the curves; phiecs evaluates none | - | phiphx `blend/anim` | no |
| `Playing` | hold source, time, speed, weight, layer; be paused, resumed, stopped | RF:10444-10497 | - | no |
| `Handoff::Snapshot` | start a new source's blend from the bound fields as last written, whatever was playing | `snapshotAndReplace` RF:13799-13812 | - | no: phiplay blends from the old move's end |
| blend | blend by the field's kind: position and scale by lerp, rotation by slerp, a number by lerp, a switch by step | - | MOM `math/transform.cpp:156-161` | no |
| `ChannelSet` | hold a flat run of numbers with a layout, for what is not a transform: shape-key weights, joint angles | - | - | no |

## system

| Part | Must | Apple | Also read | Today |
|---|---|---|---|---|
| `Stage::ORDER` | run Ingest, PreCouple, Couple, PostCouple, Readout, in that order, every step | none: Apple orders by dependency only. The order matches SceneKit's frame | SCN `SCNSceneRenderer.h:343-397` | met |
| `after` / `before` | order two systems inside a stage by name; a cycle is refused when the system is added | `dependencies` RF:14757-14761, `SystemDependency` RF:10308-10312 | - | no: insertion order |
| `when` | skip a system for a step when its condition is false | `SystemUpdateCondition` RF:14773 | - | no |
| `PerKind` | run once for every word of one kind, so a system is chosen by entity type (ours) | - | - | no |
| roles | be one trait a stage: `Intake`, `Animate`, `Simulate`, `Constrain`, `Readout`. Each layers on `PerKind`; a new role is a new trait, and no existing one changes (ours) | - | - | no |
| `Plugin` | let a crate declare its kinds, register its aspects and add its systems in one call (ours) | `registerComponent` RF:6568 | - | no |
| `EventQueue` | hold typed events for exactly one step after they are emitted; readers pull, nothing is called back (ours) | `SceneEvents` RF:7086-7114, `ComponentEvents` RF:7221-7242, `subscribe` RF:7403 | - | no |
| propagate | derive every `WorldMatrix` in PostCouple, after the Constrain roles | - | - | no |
| record | at Readout, hand each changed timed aspect and each event to every `Sink` | - | - | met: `record()` |
| `Sink` | be the one way out: memory in this crate; the session file and the database are sinks supplied from outside | - | - | met: `MemorySink` |

## What other crates plug in

| Role | Kind | Supplied by | Apple |
|---|---|---|---|
| `Intake` | scene file | phiphx: a scene into words | `Entity(named:)` RF:12124 |
| `Animate` | anything with `Playing` | phiecs itself | RF:10444-10497 |
| `Constrain` | rig | phiphx: constraints and IK | `IKComponent` RF:8238, RF:4413-4498 |
| `Simulate` | body | the scene glue over `phiray::sim`: bodies by word, `Contact` events | `PhysicsBodyComponent` RF:2054-2295, `CollisionEvents` RF:7163-7180 |
| `Readout` | drawn things | the scene glue over `phiray::Tracer`: change ticks to `reinstance` and `repose` | `LowLevelMesh` RF:9585 |
| `Sink` | every kind | philog for the session file, phiora for the database: [docs/connect.md](docs/connect.md) | `Entity.write(to:)` RF:12125 |

## Not specified here

- A skeleton, a mesh, a clip: phiphx owns them. Bones are words; the rig is an `Asset`.
- An animation definition zoo: phiphx's Blender model is the specification for clips.
- Network transport: only the stable id and the owner.
- Anything that needs a file reader or a dependency in this crate.
