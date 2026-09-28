# UI unity audit

Mechanical scan of every styling call site under `src/v2/{view,part,modal,region,zone,area,host,space,pane,app}`.
Tree compiles clean (`cargo check -p phiber --lib`, 55s, 12 pre-existing warnings).

## What is already unified — don't touch it

| dimension | call sites | off-token |
|---|---|---|
| radius | 40 | **0** — every one resolves through `BStyleRole`; the 6 bare `rounded_full()` are pills/dots |
| background | 56 | **1** (`brand.rs:105`) |
| padding / margin / gap | 73 | **0** |
| border width | 13 | **2** (`border_1()`) |
| text size | 24 | **1** (`thumb.rs:92`) |
| text colour | 41 | **0** |

`BFlex` (`view/draw/flex/`) is a real token layer — `BSpace`/`BDim`/`BLen`/`BFill`/`BRadius`/`BEdge`/`BType`.
`theme/semantic/` derives every shade off one ladder. `view/draw/surface/` owns corner assignment correctly
(no reliance on `overflow_hidden`, which GPUI masks to a rectangle). The token system is not the problem.

## The inconsistencies

### 1. Two different hover colours — highest visual impact

`view/draw/chrome.rs` is declared as "one treatment for every control", and resolves hover through
`ctx.style.control_fill(BControlState::Hover, ctx.floating)`. Docked, that is **the accent at 22% alpha**
(`effect.rs:109`) — a blue wash.

Eleven of the thirteen `.hover()` sites never reach it. They go straight to `colors.bg_hover`, which is
`ladder.color(SHADE_HOVER)` (`palette/set.rs:141`) — **a neutral grey**.

```
bypassing chrome::paint:
  tree/mod.rs:101      list.rs:45        table.rs:74       sort.rs:91
  thumb.rs:75          caption.rs:86     toggle.rs:93      canvas/gizmo/mod.rs:40
  canvas/gizmo/compass/axis.rs:112,150
```

So hovering a menu cap gives a blue wash; hovering the Outliner row, a file-list row, a table row, a sort
header, a thumbnail or a window control gives grey. Same gesture, two hues.

### 2. `BRowShade` means different things in the three row builders

Three widgets implement the same four-state enum with three different mappings:

| | `tree/mod.rs` | `list.rs` | `table.rs` |
|---|---|---|---|
| `Flat` | no fill | `bg_panel` (opaque) | no fill |
| `Alt` | **`host_item_hover`** | `row_alt` | not handled |
| `Selected` | `row_selected` | `row_selected` | `row_selected` |
| `Active` | `row_active` | `row_active` | not handled |
| hover when selected | **none** | `selected_hover(bg)` | `selected_hover(...)` |
| horizontal inset | `pr(pad_md)`, **no left** | `px(pad_md)` = 8 | `px(content_inset)` = 2, nested in another 2 |
| gap | — | `gap_xs` = 4 | `gap_md` = 8 |

Consequences you can see:

- **An alternating row in the Outliner is painted in the hover colour** (`tree/mod.rs:92`), so striped rows
  read as permanently hovered, and a real hover is invisible on them.
- **A selected tree row does not respond to hover at all** (`tree/mod.rs:99`, guarded by `if !selected`),
  while a selected list or table row lightens by `selected_hover`.
- The comment at `tree/mod.rs:64-67` already says palette-only `row_alt`/`row_selected` "let the outliner
  drift to shades no other region used" — then lines 93-94 use exactly those tokens. The intent was
  written down and not carried out.

### 3. Four left gutters for the same "row in a panel"

All of these draw a `MenuItem`-radius row inside a floating or panelled surface:

```
list.rs:42        pad_md            8px
menu.rs           pad_sm            6px
popover/mod.rs    pad_xs            4px   (switcher columns)
table.rs:72,:71   content_inset     2px + 2px
tree/mod.rs:81    none on the left  0px   (left inset comes from indent_unit)
```

Text in a dropdown, in the file list, in the host switcher and in the Outliner therefore starts in four
different columns. This is the same class of defect as the gutter work already done for headings/values —
it was fixed for the card axis and not for the row axis.

### 4. No elevation scale

`BShadowStyle` has four rungs, but `BEffects` has a single `surface_shadow` slot, so a theme picks one
globally. Everything that lifts gets the identical shadow:

```
popover/mod.rs:199   the popover
flex/apply.rs:169    everything with .lifted() — modal/part.rs:83,:123 and log.rs:61
```

A modal, a dropdown and the action-log drawer are at three different depths and cast the same shadow.
The viewport-floating toolbar and sidebar (`BPlace::Float`, `pane.rs`) cast none at all despite floating
over the canvas. This is the `jobs.md` "one elevation scale (0 base · 1 zone · 2 popover · 3 modal)" item —
currently unimplemented, and `BEffects` cannot express it without a per-role shadow map beside `BRoleMap`.

### 5. Half the draw layer still hand-builds with `div()`

118 direct `div()` sites against 81 `BFlex` sites. The direct sites are not wrong in their *values* — the
scan found almost no literals — but each one re-decides geometry independently, which is how findings 1-3
arose. The largest are:

```
canvas/node.rs 8   popover/mod.rs 7   menu.rs 6   tab/mod.rs 5
thumb.rs 5         tree/mod.rs 5      atom.rs 4   progress.rs 4
resize.rs 4        sort.rs 4          table.rs 4
```

### 6. Small off-token values

```
view/draw/brand.rs:105              gpui::rgb(BACKDROP_BG)          the one hardcoded colour in the app
view/draw/thumb.rs:92               typography.body * 0.85          a fifth type size, not on the scale
view/draw/popover/mod.rs:187        .border_1()                     ignores strokes.border
view/draw/field.rs:37               .border_1()                     ignores strokes.border
view/draw/canvas/gizmo/mod.rs:40-41 pad_xs * 0.75, pad_xs / 2.0      3px and 2px, not spacing rungs
view/draw/canvas/node.rs:125        gpui::relative(0.22)            magic proportion
view/draw/canvas/gizmo/compass/mod.rs:129  cap * 0.34                magic proportion
view/draw/caption.rs:100            h * 3.0                          magic proportion
```

### 7. `BInk` is a parallel vocabulary nobody uses

`BInk` exists as the ink token, but 31 of 41 text-colour sites go straight to `colors.text` /
`colors.text_dim`; only 3 use `BInk`/`icon_ink`. Harmless today (same values) but it means a future
ink change has to be made in two places.

## Ranked fix order

1. Route every `.hover()` through `chrome::paint` (or a sibling that takes a role), so there is one hover.
2. Give `BRowShade` one resolver in `theme/semantic/` and have tree/list/table all call it — kills the
   alt-equals-hover bug and the missing selected-hover in one move.
3. One row gutter: a `BSpace::RowGutter` all five row builders take.
4. Add a per-role shadow map beside `BRoleMap` and give popover/modal/floating distinct rungs.
5. Sweep the eight off-token values in §6.
6. Migrate the eleven biggest `div()` files onto `BFlex` so this cannot regrow.

---

# Correction and root cause (after measuring)

Two findings above were stated from reading the code and are wrong. Measuring the
painted frame corrected them, and turned up the real defect underneath.

**§1 was over-broad.** Two hover families is correct, not a bug. `chrome.rs` is
scoped to header strips by its own doc, and Blender likewise highlights a menu
entry in blue while highlighting an Outliner row in grey. The two compass-axis
hovers are per-axis identity colours, not generic hovers, and were miscounted.
What was real: ten sites each named `colors.bg_hover` for themselves instead of
asking one function. Fixed — `tone::item_hover` is now the single choice, with no
change in appearance.

**§3 was arithmetically wrong.** I computed the table's row inset as 2px, having
missed that its heading cells carry `row_inset` of their own and its rows sit
inside a `pad_sm`-padded scroll container. Measured (`tests/row_gutter_probe.rs`),
the file-list heading and the sidebar card rows both land 9px from their card
edge, symmetric with the grip 9px off the right edge. That axis is already
consistent. `BList` does compose its inset differently (`pad_md` directly rather
than `content_inset + row_inset`), but its only two uses — the Info report log
and the welcome window's recent list — sit next to nothing that would reveal it,
so it is left alone rather than changed blind.

## The root cause: three of four row tokens collided with control tokens

```
bg_hover     = ladder shade 8       row_selected = ladder shade 8   ← identical
bg_active    = ladder shade 9       row_active   = ladder shade 9   ← identical
                                    row_alt      = ladder shade 6
```

A selected row was painted in *exactly* the hover colour. Hovering an unselected
row made it look selected; a selected row was indistinguishable from an active
control. `tests/shade_ladder.rs` pinned the collision as intended behaviour.

The surface ladder has ten rungs and 8 and 9 were already taken, so there was no
free grey for selection. Selection is not a grey: Blender fills `TH_SELECT` and
`TH_ACTIVE` with the theme's selection colour. It now does the same — the accent
at 0.28 and 0.46 — joining the accent and the axes as a colour that stays off the
ladder because it means something.

## What landed

1. **One row treatment** — `tone::row_fill` / `tone::row_hover`. The tree, the
   list and the table each mapped `BRowShade` themselves and disagreed. Two bugs
   went with it: the Outliner painted alternating rows in the hover colour (so
   stripes read as permanently hovered and a real hover was invisible), and a
   selected tree row answered the pointer with nothing.
2. **Selection off the ladder** — the collision above, fixed for all 11 themes at
   once. `anim.rs`'s summary-channel backdrop was borrowing `row_active` as a
   surface; it takes `bg_active` now, the shade it always resolved to.
3. **One neutral hover** — `tone::item_hover`, the §1 correction.
4. **Measurable rows** — debug selectors on tree rows, table rows and the table
   header, plus `tests/row_gutter_probe.rs`, so gutter claims are measured.

## Verification

Isolated my effect by recording the layout fingerprint from pristine sources and
again with the changes, and diffing the two outputs rather than the golden:

```
existing selectors whose geometry moved : 0
selectors that disappeared              : 0
brand-new selectors                     : 286  (the debug selectors added)
quad paint lines                        : 200 replaced by 86
```

Paint-only, as intended. Tests: `shade_ladder`, `semantic_style_derivations`,
`layout_gutters`, `layout_fingerprint`, `outliner_tree_geometry`,
`file_browser_accordion_refinement`, `level_structure` all green. Remaining
failures in my sandbox are 8 missing `.blend`/texture fixtures plus the two known
`file_browser_typing` cases (the text-input entity, still open).

**Not verified:** gpui's test context does not rasterise, so corner and colour
*appearance* cannot be checked from here — only assignment. The selection alphas
(0.28 / 0.46) are a first choice and want an eye on them.

## The golden was already stale

`tests/golden/layout_fingerprint.txt` on disk no longer matched the sources
*before* this work: 654 selectors moved, 152 lost, 84 gained. All of it traces to
changes made between sessions, not to this pass:

```
tab-layers-Compositing            → tab-layers-ECS Wand        (84 selectors)
Auto Keyframing / Time Jump panel chevrons removed             (68 selectors)
consequent shifts: −20px (shorter tab label), −34px (two fewer
buttons), +53px (wider switcher label)
```

The golden is re-recorded, so it now also blesses those. Worth a look if any of
them was not intended.

## Still open

- Elevation: `BEffects` has one `surface_shadow` slot, so popover, modal and the
  log drawer cast the same shadow and the viewport-floating panes cast none.
  Needs a per-role shadow map beside `BRoleMap`.
- 118 `div()` sites against 81 `BFlex` sites — the condition that let the row
  mappings diverge in the first place.
- The eight off-token values in §6 above.
- `row_alt` is a fixed shade 6, so it is a visible stripe on a body and invisible
  on a card. Correct only because its one real user (the Outliner) sits on a body.
