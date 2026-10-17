# The spatial node view — a wish list

Tick what to include. Format is this project's `jobs.md` format: one line per
job, `- [ ]` open, `- [x]` done. The pictures are the design canvas
("Spatial Node Graph — proposal", five boards); this is what building each
board would actually cost.

## The decision on record

**Spatial mode is a VIEW, not an editor** — settled 2026-10-14. You look at
it and leave; every gesture stays in Flat.

That removes the three hardest pieces outright: picking a socket in
perspective, a folded wire that unfolds under the cursor, and a drag that
knows which plane of a node's stack you grabbed. What is left is a renderer
over the layout, which is why this can move fast.

**The cost of that, stated plainly:** to change anything you have to switch to
Flat. People will bounce between the two constantly, so the toggle has to keep
the camera and be instant — otherwise the mode is a toy you visit once. That
is a job below, not an afterthought.

**Flat remains the parity path.** Blender has no 3D node editor, so there is
no source to port here and no truth test to measure against. Every divergence
lives on this side of the toggle; nothing about Flat changes.

## Prerequisites — needed whatever else is ticked

- [ ] **Read `display_shape` through to all nine socket shapes.** Blender
      stores one per socket (`eNodeSocketDisplayShape`,
      `DNA_node_types.h:117–125`); phiphx's `BlendSocket` does not read it and
      phiber's `paint/dot.rs` resolves a socket to one of two shapes. Pure
      parity, no invention — and it **improves the flat editor today**, before
      any 3D exists. Small. Touches `shade/socket.rs` and `paint/dot.rs`.
- [ ] **The shared layout pass.** Card metrics and one row per socket, in
      graph units, computed once and read by both modes — so Flat and Spatial
      can never disagree about where a socket is. Medium. This is the piece
      that makes the second mode cheap rather than a parallel implementation.

## The spine — nothing appears on screen without all four

- [ ] **A camera over the graph.** Orbit, pan, dolly, plus a framing that
      matches Flat's `NODE_OT_view_all`, so switching modes lands you looking
      at the same thing.
- [ ] **Node solids.** The class radius (2 / 6 / 14 / 26) and the hex prism
      for the field class, over the grouping `theme/style/node_class` already
      has. Board 2.
- [ ] **Links as tubes.** Orthogonal routing with rounded corners, or Blender's
      curve lifted into 3D — pick when the first version is on screen and one
      of them is obviously better.
- [ ] **The mode toggle, per space, remembered per space, camera kept.** So
      Shading can be spatial while Compositing stays flat. The state belongs
      in each space's own set.

## Layers — board 5, in the order I would add them

- [ ] **Height = class.** The relief that makes a forty-node chain readable
      end to end. Cheap once the spine exists; this is the item that earns the
      whole mode.
- [ ] **Frames as floor plates.** `bNode.parent` is already read, and a frame
      already bounds its children, so a labelled group becomes a terrace.
      Cheap.
- [ ] **The three-plane stack** — preview / body / sockets as parallel planes
      rather than one slab, so the preview stops competing with the controls
      for one rectangle. Medium.
- [ ] **The preview plane actually showing the node's output.** Flagged as the
      expensive one: it needs something that can evaluate and draw a single
      node's result, which is a real piece of work and not a view concern.
      Worth ticking only if you want it early; the empty plane is still useful
      as a surface.

## The fold

- [ ] **Long-link folding.** A wire across forty nodes leaves and arrives
      without being drawn through the middle. Both ends keep their true
      positions. As a *view* this is purely drawing — the hard half (unfolding
      under the cursor) was the editor's problem and is now out of scope.

## The fun, with no risk to the data

- [ ] Lit solids and depth shading, so the relief reads as relief.
- [ ] A ground grid under the graph.
- [ ] Socket solids on the connection plane (needs the prerequisite above).
- [ ] Selection and the active node carried as glow rather than outline —
      Blender ranks active above selected above nothing, and in 3D an outline
      is the first thing that stops reading.
- [ ] **Easing on the mode transition**, so Flat → Spatial reads as the same
      graph lifting off the page rather than as two screens swapping. Small,
      and it is most of what makes the toggle feel like one editor.

## Deliberately not included

These are decisions on record, not omissions:

- **Impossible stairs on the class axis.** Beautiful, and false: it draws a
  cycle into a graph that cannot have one (Blender refuses a cyclic link), so
  a reader who trusts the picture goes looking for feedback that is not there.
  Fold distance, never order. Board 5.
- **Any re-layout of `x`.** A node's x comes from `bNode.location`, so a graph
  you arranged in Blender opens as you left it. Only the two new axes are ours
  to assign.
- **Editing in Spatial.** The decision above. If it ever changes, picking in
  perspective is the first piece, not the last.

## What I will not guess

- Colours and intensities beyond the class hues already in
  `theme/style/node_class`. Your rule is that a unified design beats copying
  Blender on looks, so the palette is yours to set and I will match it.
- Whether the preview plane earns its cost. That is a judgement about how you
  work, not about the code.

## Where it would live

A sibling presentation of the same graph, not a second reader:
`phiphx`'s `shade/graph.rs` keeps supplying the data unchanged, the shared
layout pass sits between, and the spatial renderer is its own module beside
`view/draw/canvas/node/` with the `mod.rs` + `set.rs` pair at each level and
no file over 240 lines.

One caution worth repeating from the last few batches: the node editor's own
module docs and `workflow.md` were describing a two-card placeholder long
after the real graph was being drawn. If this gets built, its jobs file is the
thing that has to stay true, or the next person reads a wish list as a
feature list.
