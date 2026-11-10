# phiecs — plan

The folder tree, today and suggested, and the order to build it in. What
each part must do is in [SPECS.md](SPECS.md); the diagrams are in
[ARCHITECTURE.md](ARCHITECTURE.md). Line counts for new files are estimates.

Two rules shape the tree:

- **Three main folders: `entity`, `comp`, `system`.** Everything else is a
  subfolder of one of them, so a logical unit (space, anim, time) has its
  data under `comp/` and its logic under `system/`, by the same name.
- **phiecs holds only what is general.** A bone, a robot link, a scene
  object, a sensor and a pin are all words with a `Pose` in one `Tree`.
  There is no skeleton type here: the rig is phiphx's asset, and phiecs
  animates any field of any aspect by one path.

## The folder tree

| Path | Holds | State | Lines | Order |
|---|---|---|---|---|
| `src/lib.rs` | the three modules; a prelude that keeps today's paths | exists | 31 | - |
| **`src/entity/`** | **what a thing is and how things are linked** | | | |
| `entity/word.rs` | `Word`, `Lexicon` | moves from `src/word.rs` | 83 | - |
| `entity/kind.rs` | `Kind`: the type of an entity, declared from a list passed in | new | ~60 | 4 |
| `entity/spawn.rs` | `coin`, `spawn(kind)`, `forget`, `Enabled` | new, part from `world/mod.rs` | ~90 | 4 |
| `entity/tree.rs` | `Tree`: parent, children, loop refused; `set_parent` | new, from phinet `node/tree.rs` | ~130 | 2 |
| `entity/relation.rs` | `Relation`, `Register`, `Edge`, `Graph` | moves from `src/relation.rs` | 173 | - |
| `entity/plan.rs` | `plan`, `reach` over relations | moves from `world/plan.rs` | 69 | - |
| `entity/query.rs` | `Query`: `has`, `and`, `not`, `related` | new | ~150 | 7 |
| `entity/world/mod.rs` | `World`: the container; bind, read | moves from `world/mod.rs` | 156 | - |
| `entity/world/relate.rs` | `declare`, `relate`, `calls_for` | moves from `world/relate.rs` | 44 | - |
| **`src/comp/`** | **data bound to a thing** | | | |
| `comp/aspect.rs` | `Aspect`, `Dense<A>`; then names, change ticks, `Transient` | moves from `src/aspect.rs`, grows | 73 to ~140 | 4 |
| `comp/resources.rs` | one value per type | moves from `src/resources.rs` | 32 | - |
| `comp/stored.rs` | `Stored`, `Value`, stable id, `Owner` | new | ~70 | 3 |
| `comp/space/pose.rs` | `Pose`, `WorldMatrix`, the 4x4 arithmetic | new | ~90 | 2 |
| `comp/time/clock.rs` | `Clock { tick, time, dt, fps }` | new | ~50 | 1 |
| `comp/time/track.rs` | `Track<A>`: sparse ticks and values | new | ~90 | 3 |
| `comp/time/journal.rs` | the `Journal` trait, the in-memory journal | new | ~150 | 3 |
| `comp/anim/bind.rs` | `Bind { word, aspect, field }`, `Animated<T> { base, live }` | new | ~70 | 9 |
| `comp/anim/sample.rs` | the `Sample` trait: values at a time, written into bound fields | new | ~50 | 9 |
| `comp/anim/playing.rs` | `Playing`, `Handoff`, the blend rule a field kind | new | ~120 | 9 |
| `comp/anim/channel.rs` | `ChannelSet`: a flat run of numbers with a layout, for what is not a transform | new | ~60 | 9 |
| `comp/asset.rs` | `Asset { kind, key }`, `Loaded` | new | ~80 | 10 |
| **`src/system/`** | **logic, run in stage order** | | | |
| `system/stage.rs` | `Stage`, `System`, `FnSystem`; then `after`, `before`, `when` | moves from `src/system.rs`, grows | 99 to ~150 | 6 |
| `system/schedule.rs` | `add_system`, `step`, the sort inside a stage | new, part from `world/mod.rs` | ~110 | 1, 6 |
| `system/layer.rs` | the layered traits: `PerKind`, the roles `Intake`, `Animate`, `Simulate`, `Constrain`, `Readout`, and `Plugin` | new | ~140 | 6 |
| `system/event.rs` | `EventQueue`: typed double buffer | new | ~120 | 1 |
| `system/sink.rs` | `Sink`: where rows and events go: memory here, the session file and the database outside | new | ~50 | 3 |
| `system/space/propagate.rs` | PostCouple: world matrix from the tree, parents first | new | ~90 | 2 |
| `system/time/record.rs` | Readout: one journal row for each timed aspect that changed; `at`, `seek`, `dump`, `load` | new | ~120 | 3 |
| `system/anim/advance.rs` | PreCouple: advance time, sample, blend, write the bound fields | new | ~110 | 9 |
| `system/anim/handoff.rs` | the snapshot of bound fields a new source blends from | new | ~80 | 9 |
| `docs/`, `reference/` | pages and the index of Apple's interfaces | exists | - | - |

The crate keeps no dependencies, so it reads no files: tables are read by
the app crate and passed in. Moving the six existing files changes no
public path: `lib.rs` re-exports them where they are today.

## What is not here, and where it is

| Concept | Owner | How phiecs meets it |
|---|---|---|
| a rig: rest pose, weights, bone names | phiphx | bones are words in the `Tree`; the word holds an `Asset` handle |
| a clip and how its curves evaluate | phiphx `blend/anim` | it implements `Sample` |
| the pose solve: constraints, IK | phiphx `blend/rig` | one PostCouple system, registered by the scene glue |
| drawing, skinning on the GPU | phiray | a Readout system reads change ticks |
| bodies and contact | phiray `sim` | a Couple system owns the simulation |
| a pin | nobody: it is a word | a named child of a bone, with an offset `Pose` |
| the record on disk | philog, the session file | a `Sink`, attached at start |
| the stored world | phiora: `kinds`, `items`, `links`, `log` | a `Sink`; kinds declared once for both ([docs/connect.md](docs/connect.md)) |
| data files | the lexfile format of `phistd` | the app reads them and passes the lists in |

The glue that turns a phiphx scene into words and drives phiray from the
world is one shared crate (suggested in [the root plan](../../../PLAN.md)),
not a bridge in every app.

## Order of work

```mermaid
graph LR
    w0["0 move the six files under entity, comp, system"]
    w1["1 clock · events · schedule"]
    w2["2 tree · pose · propagate"]
    w3["3 stored · track · journal"]
    w4["4 aspect names · change ticks · spawn"]
    w5["5 phiplay: snapshot handoff"]
    w6["6 system order · run conditions"]
    w7["7 query"]
    w8["8 the scene glue: physics and render"]
    w9["9 anim: bind · sample · playing · handoff"]
    w10["10 asset, one world"]
    w0 --> w1 --> w2 --> w3
    w1 --> w4 --> w3
    w1 --> w5
    w1 --> w6 --> w8
    w2 --> w8
    w4 --> w7
    w2 --> w9
    w4 --> w9
    w5 --> w9
    w4 --> w10
    u2["a part cannot drift from its bone: it is the bone's child"]
    u3["scrub, replay, resume, render frame n alone"]
    u4["phiber stops diffing by pointer"]
    u5["the pose pop at a move change goes"]
    u9["one animation path for bones, objects, materials, robot joints"]
    w2 -.-> u2
    w3 -.-> u3
    w4 -.-> u4
    w5 -.-> u5
    w9 -.-> u9
```

## How a step grows

```mermaid
graph TB
    n0["today<br/>step(dt): each stage, systems in insertion order"]
    n1["after 1<br/>the clock advances first, events swap last"]
    n2["after 2<br/>PostCouple: propagate"]
    n3["after 3 and 4<br/>Readout: one journal row for each timed aspect that changed"]
    n6["after 6<br/>a stage sorts its systems once, each asks when"]
    n9["after 9<br/>PreCouple: every Playing samples, blends and writes its bound fields"]
    n0 --> n1 --> n2 --> n3 --> n6 --> n9
```
