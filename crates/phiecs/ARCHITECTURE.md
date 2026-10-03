# phiecs — architecture

Apple's RealityFoundation, read as a language engine. Every name on the left
exists in `crates/apple/src/reality-foundation/`; every name on the right is
what it becomes when the entity is a word.

| RealityKit / reality-foundation | phiecs | why it changed |
|---|---|---|
| `Entity { id, generation }` | `Word(u32)` | Words are interned, never recycled, so a generation counter would never be incremented. Apple's never is either (`entity.rs:52`) — here it is simply absent. |
| `EntityGenerator` (AtomicU64) | `Lexicon` | A language needs id → spelling as much as the reverse: every report a person reads is in words. |
| `Component` + blanket impl | `Aspect`, explicit impl | Apple's `impl<T: Send+Sync+'static> Component for T` (`component.rs:21`) makes every type a component and, by coherence, makes `on_attach`/`on_detach` unoverridable. Here the impl is written by hand and the hooks work. |
| `Storage<T>` = `Vec<(Entity, T)>` | `Dense<A>` = `Vec<Option<A>>` | Apple scans linearly for every get and `retain`s on every insert (`component.rs:44`). Word ids are dense from zero, so the id is the slot. |
| `World::despawn` | `World::forget` | Apple's removes the entity from a list and leaves every component behind — its own comment says so (`world.rs:174`). `forget` walks every store and every edge. |
| `Parent` / `Children` | `Edge` / `Edges` keyed by `Relation` | A tree has one edge kind and one parent. A language is a multigraph, and the kind is the payload. |
| `add_child` / `remove_child` | `relate` / `unrelate` | Same call shape; relating also moves both phasors on that channel, which is the whole of learning here. |
| `Transform` (position/rotation/scale) | `Phasor` (64 × angle, amplitude, evidence) | Phical stores no point. |
| `Query::iter() -> Vec<…>` | `with()` / `with2()` / `related()` | Apple allocates a vector per query (`query.rs:23`); these are iterators, and `related()` is the join along an edge kind that a single-parent tree cannot express. |
| `SystemStage::{Input,PreUpdate,Update,PostUpdate,Render}` | `Stage::{Ingest,PreCouple,Couple,PostCouple,Readout}` | Apple's stages assume a 16 ms frame; a language engine advances per observation. |
| `World::update` + `Scheduler::run` | `World::step` | Apple writes the staged loop twice (`world.rs:67`, `system.rs:93`); there is one here. |
| `Resources` (defined, never wired) | `Resources`, owned by `World` | Its own README flags the orphan (`reality-foundation/README.md:59`). |
| `Scene::load_usdz` / `load_usdz_into` | `Quest::run` / `Quest::write` | The pattern transfers exactly: decode to descriptors, then a separate call that spawns them into a world. |
| `GKStateMachine` (name-keyed scan) | `Condition` | Same shape, keyed on an interned word, so a condition is itself something the world can relate to. |
| *(nothing)* | `Relation`, `Relations` | The kind of an edge. RealityKit has no equivalent and no `ComponentSet`; the honest counterpart is "which relation kinds this word has evidence for", which `Phasor::kinds` answers. |

## Layout

```
src/
  lib.rs            the mapping table above, as docs, plus `prelude`
  word/             mod.rs = Word · set.rs = Lexicon
  aspect.rs         the component trait
  store/            mod.rs = AspectStore · dense.rs = Dense<A>
  relation/         mod.rs = Relation/Relations · set.rs = the named kinds · edge.rs = Edge/Edges
  phasor.rs         Phasor: couple, agreement, resonance, kinds
  condition/        mod.rs = Condition · set.rs = the starting conditions
  world/            mod.rs = World · spawn.rs = coin/forget · relate.rs = relate/calls_for · schedule.rs = step · tests.rs
  query/            mod.rs = with/with_mut · pair.rs = With2/Related
  system/           mod.rs = System/FnSystem · set.rs = Stage
  resources.rs      world-owned resources
  quest/            mod.rs = Quest · oracle.rs = Oracle/Spawned/Scripted · template.rs = prompts + parser · step.rs = transcript · run.rs = the loop and the files
examples/quest.rs   run a quest against a real model, or scripted
```

House rules followed: `mod.rs` holds the type and its structure, `set.rs` holds
the names; every file under 200 lines; module docs say what a thing is *for*
and what a previous shape cost; tests inline for small files and in `tests.rs`
when a file is near the cap.
