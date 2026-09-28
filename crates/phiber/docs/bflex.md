# One layout element: the BFlex evaluation

## The question

Why were we still hand-writing `div().flex().flex_col().flex_1().min_h_0().bg(…)`
at every level, instead of one element driven by the level enums that decides
padding, sizing and surface once? Which levels weren't using the traits, and
why did the modals do their own thing?

## What we found

Measured, not estimated: **1,176 hand-written GPUI style calls across 165
`div()`s**, at every level of the tree.

**The level traits existed, but only as partial answers.** `BZoneLayout`,
`BAreaLayout`, `BHostLayout`, `BRegionLayout` and `BPaneLayout` each answered
one to three questions: an axis, a placement, a size. Every walker then took
that answer and assembled its container by hand. Everything the traits didn't
answer (padding, gap, fill, radius, overflow, shrink) was decided inline, in an
`if/else` ladder per walker. Region padding alone was a five-branch ladder over
`(region, host)` pairs.

**Some trait answers were dead, and wrong.** `BZoneLayout::fixed_height()` and
`BRegionLayout::fixed_size()` had **zero callers**. They read the static
`METRICS` defaults, while the walkers re-decided the same sizes from the
resolved theme (`c.style.dimensions`). So the "decision" in the trait
disagreed with what was drawn, and nothing noticed.

**`BFocusedRenderer` pointed the other way.** The trait on the primitives,
panels, canvas, modal card and popover is `#[allow(dead_code)]`, and nothing
calls through it. Its contract is "this module owns all GPUI builder
composition for its element". It is an attestation that each module styles
itself, which is the opposite of what this change is for.

**37 files had no trait at all**: atoms, tabs, menus, seams, gizmos, tree
guides, the Welcome form and the modal title bar.

**The modals were a parallel universe.** The render context's own doc says a
modal "composes panels rather than regions". In practice the modal title bar
was a hand-built header strip: header height, `pad_md`, header surface, bottom
rule, left/right cells. That's a Region::Header with Left/Right panes built a
second time. The fullscreen body was a hand-built Center region with `pad_lg`.
So modals didn't just lack the traits; they re-implemented two levels.

**The level enum already existed.** `BStyleRole` enumerates the whole
hierarchy: Workspace, Zone, Area, Host, Region, Panel, Card, Button, Field,
MenuItem, Tab, Popover, Modal. It was used only to pick a corner radius.

## What changed

**`BFlex`** (`view/render/gpui/flex/`) is the one layout element. It's a typed
record of every container decision: axis; how it takes space (`fills`,
`shares(f)`, `equal_cell`, `keeps_size`); fixed and minimum sizes; alignment;
gap; padding per edge; fill; border; radius; lift; placement (flow, anchor,
overlay, float); and overflow. Every number in it is a theme token: `BSpace`,
`BDim`, `BFill`, `BStyleRole`. It's split into `token.rs`, `element.rs`,
`surface.rs` and `apply.rs`. **`apply.rs` is the only place in the renderer
allowed to call GPUI's style builders.**

**`BFlexed`** is the trait every level implements: `fn flex(&self, c) -> BFlex`.
A walker no longer styles anything; it says what it is and gets a container.

| Level | Implements `BFlexed` | Raw style calls before → after |
|---|---|---|
| Space | `BSpacePart` (root, middle band, centre cell), `BMeasure` | 17 → 0 |
| Zone | `BZoneFrame`, `BZoneSlot` | 27 → 0 |
| Area | `BAreaFrame`, `BAreaScroll` | 24 → 0 |
| Host | `BHostPart` (frame, band, centre cell) | 17 → 0 |
| Region | `BRegionFrame`: the whole padding/size/fill/corner ladder in one impl | 41 → 0 |
| Pane | `BPaneCell`, `BPaneFloat`, `BPaneStretch` | 46 → 0 |
| Modals | `BModalPart`: layer, backdrop, card, body, fullscreen, title bar, Welcome | 83 → 3 |

That's **270 → 13** across the levels and modals. The 13 remaining are
part-level controls (a seam's hover line, the title-bar icon's size), which
move with the parts.

The dead `fixed_height()` / `fixed_size()` are deleted. The level traits now
answer only structure (axis, which band a region sits in); sizing, padding and
paint are `BFlex`.

## How we know nothing moved

**A layout-and-paint fingerprint** (`tests/layout_fingerprint.rs`, golden in
`tests/golden/`) records every selector-tagged element's bounds and every quad
the frame paints (bounds, clip, fill, border and radii). It covers all 11
spaces at two window sizes, plus 11 modals: 16,519 lines. Every migration step
was checked against it, and each came back identical. It stayed stable over 8
repeated recordings. When I deliberately changed one zone gap, it caught 6,146
moved lines.

Its limit: the test platform doesn't rasterise text or icons, so it covers
containers, not glyph shapes. That's the right coverage for this refactor,
which only touches containers.

**A style budget** (`tests/style_budget.rs`) records every renderer file's raw
style-call count. No file may exceed its budget and a new file starts at zero,
so the rest of the renderer can only move toward `BFlex`, never away from it.
The migrated level and modal files are pinned at zero.

## What's left: the parts

**885 raw style calls in 54 files**, all below the pane:

| Group | Raw calls | Files |
|---|---|---|
| `atom/` (buttons, tabs, menus, fields, toggles, anchor, caption…) | 283 | 15 |
| `prim/` (table, tree, sort, strip, accordion, list, column, row) | 209 | 10 |
| `atom/canvas/` (node graph, gizmos, compass, image, overlays) | 191 | 12 |
| `prim/panel/` (sidebar, body, card, section, heading, header, toolbar…) | 128 | 11 |
| `popover.rs` | 42 | 1 |
| seams (`zone/`, `space/`), `atom.rs`, `modal/title` icon | 32 | 5 |

The worst single files are `atom/canvas/node.rs` (45), `prim/table.rs` (44),
`prim/tree.rs` (43), `popover.rs` (42) and `atom/tab.rs` (39).

**What the parts need that `BFlex` doesn't have yet:**

- **State styling.** Hover, active and selected fills (`.hover(|s| s.bg(…))`,
  `group_hover`) are most of what's left in the atoms. `BFlex` needs a
  `BInteract { rest, hover, active }` decision alongside `fill`, so rest/hover/
  active is one mapping for every control. `atom::chrome` already has the
  mapping; it would become the token.
- **Text.** Colour roles (text, dim, accent) and sizes (body, header,
  caption) as tokens, instead of `text_color(to_hsla(...))` at each call site.
- **Icons.** An `svg()` sized by an icon token (`icon_size`, `icon_size_sm`,
  `brand_size`), so the title-bar icon and every button icon read one rule.

**Order I'd take them in:**

1. `popover.rs` and `prim/panel/`. They're the modal and panel surfaces users
   see most, and closest to what `BFlex` already expresses.
2. `prim/`: table, tree and list rows share one row shape.
3. `atom/`, once `BInteract` exists.
4. `atom/canvas/` last. Gizmos and the node graph are drawing more than
   layout, and some of it rightly stays custom.

**The structural step for modals.** Now that a modal's containers are
`BFlex`, its title bar could be an actual `Region::Header` with Left/Right
panes, and its body an actual `Region::Center`, walked by the same region and
pane walkers as every host. That would delete `modal/title.rs`'s bespoke strip
entirely. It changes what the modal views emit, and the modal title bar
differs from a host header today in fill (header tone vs host header surface)
and in its bottom rule. So it's a visible decision, not a refactor, and yours
to make.

**`BFocusedRenderer`** should go once the parts move. It certifies the
self-styling this replaces. `file_browser_accordion_refinement` still checks
for it, so it stays until then.
