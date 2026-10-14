# Phiber consistency & parity plan

Living plan for closing the gaps between how Phiber *looks* and how Blender
*works*. Grouped by theme, ordered by user-visible value. Each item names its
"done" and its test. Mirrors the per-module `jobs.md` discipline; this file is
the cross-cutting roadmap. Updated as items land.

## A — Appearance ↔ behaviour (the loudest inconsistency)

1. **Placeholder menu items (~380).** Menus render fully but most entries only
   report their path. Each one that has a real operator or rule should fire it;
   the rest should say "not available yet" consistently (not silently do
   nothing). Work editor by editor, closing a batch per turn, recording counts.
   Done: a menu entry either acts or clearly states it is a stub. Test: a
   per-editor assertion that known-wired entries dispatch their action.

2. **Region toggles — one family.** Sidebar (N) ✅ and Maximize Area (Ctrl
   Space) ✅ exist; **Toolbar (T)** still a placeholder. Wire T to hide/show the
   tool region, mirroring the N show/hide key. Done: T toggles the viewport
   toolbar like N toggles the sidebar. Test: the op flips the region key.

## B — One surface look

3. **One zone / area / host surface treatment** (`view`, `zone` jobs). Every
   frame from the same elevation + fill rules, so no editor's chrome reads as a
   different material. Audit `region_fill` / `host_*_surface` for one-offs.

4. **One menu look** (`view` jobs): every popover body built from `MenuItem`
   rows, so a dropdown, a filter popover and a header menu are the same object.

5. **Elevation scale** (`view`, `theme` jobs): base · zone · popover · modal as
   one shadow/glass ladder from theme tokens.

## C — Functional parity

6. **Shading / node editors: add & edit trees.** The graph renders ✅; the Add
   menu, relinking and material assignment do not. Blender's node editing.

7. **Sequencer canvas** — currently a bare `Custom` part; give it strips.

8. **Animation drives the frame** (`scene` jobs): object / bone matrices move
   with the playhead (needs phiphx F-curve eval), then timeline channels from
   the file's F-curves (`part` jobs).

## D — Rendering

9. **Combined pass colours** — the Workbench render shows flat patchy colour;
   chase in phiray once the file's Render engine is known.

## E — Architecture consistency (invisible but load-bearing)

10. **Declarative audit** (`host`/`region`/`pane`/`space`/`part` jobs): every
    part built from its `set.rs` table, no inline construction.

11. **State verbs** (`state` jobs): fold `*_encoded` / `iter_keys` into
    `get`/`has`/`set`/`remove`.

## Sequence

Now → **A2 (Toolbar T)**, then batches of **A1 (placeholders)** per editor,
then **B3/B4** surface + menu unification, then **C6** node editing. D9 waits on
the render engine; C8 waits on phiphx animation.
