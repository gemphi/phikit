# Jobs — `phiber`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 8 | 36 |

## Todo

- [ ] Everything in `src/v2/*/jobs.md`
- [ ] Animation playback of THYMUS (`rigAction`) end to end — needs `phiphx/blend` animation + constraints
- [ ] Migrate the draw layer off bare `div()` onto `BFlex` — 535 raw style calls left across the renderer; the row, tab, strip and split families are done, and the fingerprint is the safety net: a migration must move nothing (`unity.md`)
- [ ] Next migration targets by raw count: `thumb.rs` 35, `tab/cap.rs` 22, `button.rs` 22, `resize.rs` 21, `sidebar.rs` 20, `body.rs` 19. The canvas files draw graphics and are poor `BFlex` candidates — leave them
- [ ] Keep going on standalone functions: 50 free `fn`s in the draw layer still take a render context (was 58). The remaining clusters are `card.rs` (corner/panel_tone/head/controls/draw), `pane.rs` (is_floating_pane/floating_bg/strip_width/control_floor), `atom.rs` and `resize.rs`. The 40 `view(…)` entry points are the dispatcher seam and stay
- [ ] `brand.rs` welcome backdrop is fixed brand colours (`BACKDROP_BG` and 4 layers), so it stays a dark slab under a light theme — decide whether a brand mark follows the theme (`unity.md`)
- [ ] Should a panel card sit at `BElevation::Raised`? Blender's panels cast nothing, so cards are at `Base` for now (`unity.md`)
- [ ] Check the pre-existing layout-golden drift the re-record absorbed: `Compositing` → `ECS Wand` and the removed Auto Keyframing / Time Jump chevrons (`unity.md`)

## Done

- [x] One view layer: `bui`, `state`, `store`, `services` and the v1 app files retired to `_unused/phiber-v1/` (v2 draws everything); vendored `ui`, `ui_macros`, `component`, `icons`, `menu` dropped from the workspace
- [x] One bridge to the vendored `theme` crate: `v2::theme::vendor::install`
- [x] Blender-parity open (see `docs/blend-read-parity.md` §0–§0.8)
- [x] Jobs split per folder
- [x] One row treatment: `tone::row_fill` / `tone::row_hover` — the tree, list and table each mapped `BRowShade` themselves; the Outliner painted alternating rows in the hover colour and a selected tree row ignored the pointer (`tests/file_browser_accordion_refinement.rs`)
- [x] Selection off the shade ladder: `row_selected`/`row_active` were ladder 8 and 9, the very shades of `bg_hover`/`bg_active`, so a hovered row looked selected — now the accent at two strengths, in all 11 themes (`tests/shade_ladder.rs`)
- [x] One neutral hover: `tone::item_hover`, replacing ten sites that each named `colors.bg_hover` (`unity.md` §1)
- [x] Rows are measurable: debug selectors on tree rows, table rows and the table header, with `tests/row_gutter_probe.rs`
- [x] One elevation scale: `theme::BElevation` — Base / Raised / Floating / Overlay / Modal, each a rung under the theme's shadow ceiling, so a modal, a dropdown and a floating gizmo are three depths instead of one and a flat theme stays flat (`tests/elevation_scale.rs`)
- [x] `BFlex.lifted: bool` → `elevation: BElevation`; modals at Modal, the log drawer at Overlay, viewport gizmos at Floating (they cast nothing before)
- [x] Three off-token values: two `border_1()` → `strokes.border`, `thumb.rs` `body * 0.85` → `typography.caption`
- [x] The style budget counts `border_1` and `rounded_full`, so a hidden literal is no longer the cheap way past it (`tests/style_budget.rs`)
- [x] `BFlex` can express state: `.hovered(BFill)` plus `BFill::ItemHover` and `BFill::ControlActive`, so a hover is a token rather than a `.hover(|s| s.bg(…))` written wherever a hoverable thing is built
- [x] `sort.rs` migrated onto `BFlex`: 25 raw style calls → 3, with byte-identical painted output (`tests/golden/layout_fingerprint.txt`)
- [x] A stripe has to read against every surface a striped list sits on — guard for `row_alt` being a fixed rung (`tests/row_stripe_reads.rs`)
- [x] Checked `row_alt` and `rounded_full`: no striped list sits on a card (every one is on a body or a modal), and two of the three `rounded_full` sites are circles, not pills — both were latent, neither was a bug
- [x] `table.rs` migrated onto `BFlex`: 44 raw style calls → 0, byte-identical painted output across 166 exercised selectors — row, header and cell are each a `BFlexed`
- [x] The tokens that migration needed: `BFill::Row`/`RowHover` (a row's shade as a token), `BSpace::Stroke` (a row insets by its own border so state never changes what it measures), `margin_x`/`margin_y`
- [x] `flex/token.rs` split — `BFill` to `flex/fill.rs`, the shadow rungs to `flex/paint.rs`; both parents back under the 200-line cap and `token.rs` off the `KNOWN_OVER` allowlist
- [x] The row family is one thing: `tree/mod.rs` (34 → 0) and `list.rs` (14 → 0) join `table.rs`, so all three row builders take their fill, hover and band from the same tokens; byte-identical painted output over 120 exercised tree-row selectors
- [x] Dead code removed with it: `prim::indent` / `BTreeIndent::render_default` had no callers, and took a `renderer_decomposition` allowlist exception with them
- [x] `tab/mod.rs` refactored around `BCapShape` (`tab/shape.rs`): the six caps the file was recomputing from four interacting booleans are one named question asked once, and each shape states its own geometry — 40 raw style calls → 10, byte-identical output over 4,220 exercised tab selectors
- [x] `BSelf` — cross-axis self alignment as a token, the last thing `BFlex` could not say (a cap pinned to the strip's edge, a button centred in its rail, an entry stretched to what is left)
- [x] `flex/element.rs` and `flex/apply.rs` split again — pad/margin builders to `flex/space.rs`, the placement tail to `flex/place.rs`; both allowlisted as `BFlex` mechanics
- [x] The strip family migrated: `strip.rs` (24 → 0) and `progress.rs` (26 → 0) to zero, `menu.rs` 23 → 2; byte-identical output, and the progress bar is painted 1,144 times in the fingerprint so that one is genuinely covered
- [x] Three more fills the token layer was missing: `BFill::Separator` (a rule's shade), `BFill::Flat(tone)` (a tone with no sweep — a well, not a raised surface) and `.dim_ink()`
- [x] One type per level, matched — not a struct per element. The migration had grown 24 unit structs (`BMenuRow`, `BProgressTrack`, …), one per element, each with its own `impl BFlexed`; they are now variants of one `B…Part` enum per level, as `BCardPart` and `BModalPart` already were. 45 impls across 21 files → 21 across 21, byte-identical output
- [x] `BAreaPart`, `BZonePart`, `BPanePart` — the three pre-existing files that also split a level across several structs
- [x] The convention is structural now, not a habit: `tests/one_part_per_level.rs` fails a file that answers `BFlexed` twice, or that answers it for a struct outside a named allowlist
- [x] `tone.rs` deleted — a module of 11 standalone functions. The row mapping moved onto `BRowShade` itself (`fill`/`hover`, matched on self), the surface helpers onto `BFill::Tone`, the gutters onto `BSpace`, and `item_hover` was a one-line alias for `style.host_item_hover` all along
- [x] `split.rs` migrated with it (`BSplitPart`), and `body.rs`/`sidebar.rs` name their surface by its tone instead of by a helper per caller
- [x] `ctrl/mod.rs`: seven free functions, one per control kind, became methods of `BCtrlView` matched in one place — and `check`/`toggle` collapsed into one `flag`, since they differed only in the glyph and the atom
- [x] `toggle.rs` and `rail.rs` the same way (`BZoneToggles`, `BRail`), with `BZoneTogglePart` for the button geometry; a hover is `.hovered(BFill)` now, so it cannot resize what it paints
- [x] Warnings cleared: 17 → 3 in `phiber`. The three left are one issue — `PartialEq` derived on menu items holding `fn` pointers (`part/menubar/item/mod.rs`), which is a question about what menu-item equality means, not a tidy
- [x] The workspace builds warning-free (`--workspace --all-targets`), 21 warnings to 0
- [x] `PartialEq` dropped from `BMenuKind`/`BMenuItem`/`BMenuSpec`: they hold `fn` pointers, nothing compared them, and the derive was a wrong answer waiting for a caller
- [x] Dead test scaffolding removed rather than silenced: two property strategies carried an unused `defaults` beside a `match` whose only arm returned its input — the bounds always came from the descriptor's parser; and `prepare_state` took a `defaults` it never read
- [x] `phiphx::blend::anim` marked `#![allow(dead_code)]` with its reason — stages (b)–(d) of task 17 are written and checked against Blender but not yet wired to a player, so they have no caller yet. They go when the stages land, not before

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
