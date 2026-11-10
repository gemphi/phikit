# phiecs — specs

What each part must do. One row is one requirement, with the Apple source
it answers and what else was read. "Today" says whether the crate already
meets it. The longer account of each gap is [docs/APPLE.md](docs/APPLE.md).

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

## Identity and lifetime

| Part | Must | Apple | Also read | Today |
|---|---|---|---|---|
| `Word` | be a stable id for the life of the world; ids are never reused | `id` RF:10610 | - | met |
| `World::spawn(kind)` | coin a unique word each call, with the display name as an aspect, so two things may share a name | `name` RF:10560 | - | no: identity is the spelling |
| `Enabled` | switch a word and its subtree off without removing it | RF:10569-10581 | - | no |
| `forget` | emit `WillRemove` before the word's aspects and edges go | RF:7086-7114 | - | forgets, no event |

## Aspects

| Part | Must | Apple | Also read | Today |
|---|---|---|---|---|
| `Aspect` | be plain data bound to a word, with bind and unbind hooks | `Component` RF:6537 | - | met |
| `Aspect::name()` | name the aspect so it can be listed, saved and addressed by text | `registerComponent` RF:6568 | - | no |
| change tick | record the tick of the last `read_mut` per store, so readers ask "changed since" | `DidChange` RF:7242 | - | no |
| `Transient` | mark an aspect that is never saved or journalled | `TransientComponent` RF:6571 | - | no |

## Step, systems, events

| Part | Must | Apple | Also read | Today |
|---|---|---|---|---|
| `Stage::ORDER` | run Ingest, PreCouple, Couple, PostCouple, Readout, in that order, every step | none: Apple orders by dependency only. The order matches SceneKit's frame | SCN `SCNSceneRenderer.h:343-397` | met |
| `System::after` / `before` | order two systems inside a stage by name; a cycle is refused when the system is added | `dependencies` RF:14757-14761, `SystemDependency` RF:10308-10312 | - | no: insertion order |
| `System::when` | skip a system for a step when its condition is false | `SystemUpdateCondition` RF:14773 | - | no |
| `Clock` | hold `tick`, `time`, `dt`, `fps`; advance once at the start of `step` | `deltaTime` RF:14752 | - | no |
| `Events` | hold typed events for exactly one step after they are emitted; readers pull, nothing is called back (ours) | `SceneEvents` RF:7086-7114, `ComponentEvents` RF:7221-7242, `subscribe` RF:7403 | - | no |

## Tree and space

| Part | Must | Apple | Also read | Today |
|---|---|---|---|---|
| `Tree` | give each word at most one parent and refuse a loop | `parent`, `children`, `addChild` RF:15395-15414 | N `node/tree.rs:34-93` | no |
| `set_parent(.., keep_world)` | when asked, keep the child's world transform across the move; emit `DidReparent` | `setParent(_:preservingWorldTransform:)` RF:15395, `DidReparentEntity` RF:7110 | - | no |
| `Pose` | be translation, rotation, scale as plain `f32` arrays | `Transform` RF:15094-15137 | - | no |
| `WorldMatrix` | be derived every step in PostCouple, parents before children, and never written by hand (ours) | `position(relativeTo:)`, `convert` RF:5810-5855 | - | no |
| `Pin`, `Attached` | derive an attached child's world matrix from its parent's joint and the pin's offset every step, so the two cannot disagree | `GeometricPin` RF:9088, `AttachedTransformComponent` RF:13669 | MOM `character/locator.h:19-26` | no |

## Time and saving

| Part | Must | Apple | Also read | Today |
|---|---|---|---|---|
| `Stored` | turn an aspect into named fields of `Value` and back, with no dependency | `Transform: Codable`, `Entity.write(to:)` RF:12125, 13684, 15151 | phiora's table shape | no |
| `Journal` | keep, for each timed aspect, its value at every tick it changed; answer `at(word, tick)` with the value at or before that tick (ours: Apple keeps no history) | - | - | no |
| `World::seek(tick)` | restore every timed aspect to its value at a tick | - | - | no |
| stable id, `Owner` | give a word an id that is the same in another process, and say who may write it | `SynchronizationComponent` RF:2815-2848 | - | no |

## Query

| Part | Must | Apple | Also read | Today |
|---|---|---|---|---|
| `Query` | select words by aspects held, with and, or, not, and walk them mutably | `EntityQuery(where:)` RF:1517, `QueryPredicate` RF:13375-13379 | N `query/` | `each::<A>()` only |
| `related(kind, to)` | join a query on a relation (ours: Apple has no relations) | - | N `query/` | no |

## Character

| Part | Must | Apple | Also read | Today |
|---|---|---|---|---|
| `Skeleton` | list joints parents before children, each with name, parent, rest transform; be shared and immutable | `Skeleton` RF:8030-8045 | MOM `character/skeleton.cpp:16-22` | no |
| `Skeleton::check` | refuse an asset whose joint order, pose length, influences or weights are inconsistent, at Ingest, with the reason as an event (ours) | - | MOM `character/linear_skinning.cpp:26-30, 82-88` | no |
| inverse bind | be derived from the skeleton's rest pose, never supplied | `inverseBindPoseMatrix` RF:8030-8045 | MOM `character/character.cpp:783-792` | no |
| `JointPose` | be the one per-tick pose vector on the word; every matrix is recomputed from it and never stored | `SkeletalPosesComponent` RF:5182 | MOM `character/skeleton_state.cpp:104-118` | no |
| `Reach` | name a joint, a target and a weight for the IK solve in PostCouple | `IKComponent` RF:8238, constraints RF:4413-4498 | phiphx `blend/rig/ik` | no |
| `Playing` | hold clip, time, speed, weight; be paused, resumed, stopped | RF:10444-10497 | - | no |
| `Handoff::Snapshot` | start a new move's blend from the pose last written, whatever was playing | `snapshotAndReplace` RF:13799-13812 | - | no: phiplay blends from the old move's end |
| joint blend | blend translation and scale linearly and rotation by slerp, a joint at a time | - | MOM `math/transform.cpp:156-161` | no |
| `Bind`, `Animated<T>` | address one field of one aspect of one word; keep a base value beside the animated one | `BindTarget` RF:463-586, `BindableValue` RF:625 | - | no |

## Physics and assets

| Part | Must | Apple | Also read | Today |
|---|---|---|---|---|
| `Body`, `Collider`, `Velocity` | describe a body by word, not by array index; the Couple system owns the simulation and maps one to the other | `PhysicsBodyComponent`, `CollisionComponent` RF:2054-2295 | phiray `sim/state.rs:10-97` | no |
| `Contact` events | say when two words begin and stop touching | `CollisionEvents` RF:7163-7180 | - | no |
| `Asset`, `Loaded` | name what a word is drawn from and whether it is ready | `ReferenceComponent` RF:13729, `Entity(named:)` RF:12124 | - | no |

## Not specified here

- An animation definition zoo: phiphx's Blender model is the specification for clips.
- Network transport: only the stable id and the owner.
- Anything that needs a file reader or a dependency in this crate.
