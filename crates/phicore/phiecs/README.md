# phiecs (phikit's)

The world phiber's Project Browser and phiplay's games run on: words,
relations between them, aspects bound to them, resources, and systems run in
stage order. It has no dependencies.

It is the core of phinet's phiecs (`gemphi/phinet/crates/phiecs`) without the
phasors, quests and kits, so phikit builds with no phinet folder beside it.
Word ids, edge weights, plans and system order match phinet's, so a game plays
out the same on either. `World::relate_by` takes phinet's coupling rate and
ignores it, since there are no phasors here.

| File | What it holds |
|---|---|
| `src/word.rs` | `Word`, `Lexicon` |
| `src/relation.rs` | `Relation`, `Register`, `Edge`, `Graph` |
| `src/aspect.rs` | `Aspect` and its dense store |
| `src/resources.rs` | `Resources`: one value per type |
| `src/system.rs` | `System`, `FnSystem`, `Stage` |
| `src/world/` | `World`: coining, binding, relating, planning, stepping |
