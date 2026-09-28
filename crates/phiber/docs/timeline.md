# The Timeline — Blender's Dope Sheet in Timeline mode

> Companion to `rendering.md` and `assessment.md`. What the Layout
> workspace's bottom editor is now, which Blender source each part is
> ported from, and the gaps that remain. §2c covers the Graph editor,
> which shares the window.

The default Layout workspace's bottom editor was a toolbar of five buttons
over three placeholder labels. It is now the Dope Sheet window in Timeline
mode, ported from `space_action.cc`, `time_scrub_ui.cc`, `view2d_draw.cc`,
`anim_draw.cc`, `action_draw.cc`, `keyframes_draw.cc` and
`bl_ui/space_time.py`, in Blender's factory theme colours — and, since
2026-09-26, the Graph editor's window too (`space_graph.cc`,
`graph_draw.cc`).

## 0. Tracking — the animation space

Status: **done** (ported and guarded), **partial**, **todo**. The workspace
layouts are read from Blender's own `startup.blend` with `bpy`
(`phiber/tests/truth/blender_workspaces.py` → `workspaces.txt`): the
Animation workspace is a camera-view 3D Viewport on the left (559 of
1920 px), a 3D Viewport in the centre, Outliner over Properties on the
right, and a Dope Sheet across the bottom 433 of 1054 px with its channel
region, sidebar and footer shown.

| # | Part | Blender source | Status | Notes |
| --- | --- | --- | --- | --- |
| 1 | Timeline window: grid, range, preview range, key columns, strip, playhead | `action_main_region_draw`, `time_scrub_ui.cc`, `view2d_draw.cc` | **done** | §2 |
| 2 | Timeline header: transport, frame fields, View / Marker / Playback | `playback_controls`, `TIME_MT_view`, `TIME_PT_playback` | **done** | §1; while playing, one pause button two slots wide stands where the two play buttons were (`row.scale_x = 2`, `BControl::span`) |
| 3 | Scrubbing | `anim.change_frame` | **done** | press and drag, guarded end to end |
| 4 | Dope Sheet: channel region, Summary + object rows, backdrops | `action_draw.cc`, `anim_channels_defines.cc` | **partial** | §2b; object channels stay folded — no action groups or F-curve rows, no expanders, no channel toggles (the Drivers editor lists F-curve rows with their swatches, §2e; the Graph editor does not yet) |
| 5 | Graph editor: value grid and scale, curves, vertices | `graph_main_region_draw`, `graph_draw.cc` | **partial** | §2c; opens on `graph_create`'s view |
| 6 | Graph editor handles and handle vertices | `draw_fcurve_handles`, `draw_fcurve_handle_vertices`, `TH_HANDLE_*` | **done** | §2c; phiphx reads `BezTriple.h1 / h2` and `f1 / f3`, guarded against Blender's counts (1922 auto-clamped handles, 310 selected in the rig); the active key's `TH_VERTEX_ACTIVE` ring needs `FCurve.active_keyframe_index` |
| 7 | Graph editor value cursor, View All, scrollers, normalisation | `sipo->cursorVal`, `graph_view_all`, `view2d_scrollers_draw` | **partial** | the value cursor is drawn (§2e); View All, scrollers and normalisation are not; `BCurveSet::framed()` is ready for View All |
| 8 | NLA editor: tracks, action line, strips | `nla_draw.cc`, `space_nla.cc` | **done** | §2d; phiphx reads `AnimData.nla_tracks`, guarded against a bpy-made fixture |
| 9 | Drivers editor: view, base-ten grid, driver curves, cursor, channel list | `SIPO_MODE_DRIVERS` in `space_graph.cc`, `ED_drivers_editor_init` | **done** | §2e; phiphx reads `AnimData.drivers` with Blender 5.0's bone-hide versioning, guarded against Blender's 101 for the rig; no sidebar Drivers tab, no driver variables |
| 10 | Animation workspace layout: left camera viewport, bottom Dope Sheet share | `startup.blend` | **done** | left viewport (440 px seed), Dope Sheet 370 px of 900; the band sits under the centre only — the side columns run to the status bar (`view/draw/space/band.rs`) |
| 11 | Camera view in the left viewport | `view3d.md` rows 13–14 | **todo** | |
| 12 | Play-reverse ticker | `screen_animation_step` with `reverse` | **done** | `ctx.playing` is `true` or `reverse`; the one ticker (`app/playback`) steps `+1` or `−1` a frame at the scene fps, wrapping in the range (`BTransportButton::tick_step`) |
| 13 | Objects move with their F-curves during playback | depsgraph evaluation | **todo** | curves are read and drawn; nothing applies them to transforms |
| 14 | Dope Sheet sidebar and footer | `DOPESHEET_HT_playback_controls` (footer), `DOPESHEET_PT_action` / `_action_slot` (sidebar) | **partial** | the footer is declared and carries `playback_controls` — the transport and the frame fields, the same parts as the Timeline's header — and a footer is a strip like the header (`ED_area_footersize`: the header's height, surface and centre line, in every host); the sidebar's tab is *Action*. Not yet: the Action and Slot panels' contents (no host draws sidebar panels yet, #22), the Playback popover at the footer's start |
| 15 | Markers | `anim_markers.cc` | **todo** | phiphx does not read `Scene.markers` |
| 16 | phiphx: actions, slots, curves, key columns, selection, colour modes | `blend/anim/` | **done** | §3; guarded against Blender's counts |
| 17 | Colours from the theme, not Blender's factory greys | `theme/style/anim.rs` | **done** | every slot the editors paint (`TH_BACK`, `TH_GRID`, `TH_CFRAME`, `TH_VERTEX_SELECT`, channel region, backdrops, key diamonds) is derived from the palette in use |

## 1. The header — `playback_controls`

`host/time/set.rs` declares the header Blender draws: switcher and menus on
the left (View, Marker and the Playback popover's rows — `TIME_MT_view`,
`marker_menu_generic`, `TIME_PT_playback`), the transport in the middle
(`part/playback`), the frame fields on the right (`part/framerange`).

| Blender | phiber | writes |
| --- | --- | --- |
| `use_keyframe_insert_auto` | record toggle | `ctx.auto_key` |
| `screen.frame_jump` end=False / True | ⏮ ⏭ | `frame_step = start` / `end` |
| `screen.keyframe_jump` next=False / True | ◆◀ ▶◆ | `frame_step = key-` / `key+` |
| `screen.animation_play` reverse / forward, `animation_pause` | ◀ ▶ (▶ becomes ⏸ while playing) | `ctx.playing` |
| `screen.time_jump` backward / forward | ‹ › | `frame_step = ∓fps` — Blender's defaults are `time_jump_delta = 1.0`, `time_jump_unit = SECOND` (`DNA_scene_types.h`) |
| `frame_current`, `use_preview_range`, `frame_start` / `frame_end` (or the preview pair while it is on) | number fields, preview toggle | `ctx.frame`, `ctx.preview_range`, `ctx.frame_start` / `frame_end`, `ctx.preview_start` / `preview_end` |

A keyframe jump lands on the next key column of the shown objects and, as
Blender's "No more keyframes to jump to in this direction", stays put when
there is none (`part/time/step.rs`). One honest gap: play-reverse steps one
frame back rather than running the ticker backwards.

The number fields are Blender's `UI_BTYPE_NUM` now: one rounded field with
the label at its start and the value at its end (centred when unlabelled),
a caret in it while typed into, and a step arrow floating over each end that
shows only while the pointer is over the field (`BFlex::hover_group` /
`revealed_by`). The right group went from 440 px to 231 px at 1100 px wide,
and every number in the property panels draws the same way.

## 2. The window — `action_main_region_draw`

`view/draw/canvas/time/` paints, in Blender's order: the grid, the darkened
frames outside the scene range with a line on the first and last frame
(`ANIM_draw_framerange`: `TH_BACK` shaded −25 at alpha −100, and −60), the
preview-range curtains (`ANIM_draw_previewrange`), the key columns, the
frame-number strip (`ED_time_scrub_draw`, 23 px, `regions.scrubbing`), and
the playhead over everything (`ED_time_scrub_draw_current_frame`: the box,
its stalk with a one-pixel shadow, and the tip triangle, in `TH_CFRAME`
`#4772b3`).

The grid step is `calculate_grid_step` verbatim: start from the fps, divide
by 2, 3 or 5 — a division that lands on 2 first, because animators work on
twos — while lines stay `U.v2d_min_gridsize` (45 px) or a label's width
apart, and double until they do. Minor lines appear at `step / divisor` when
they would be more than 9 px apart. The view opens ten frames either side of
the scene range (`action_create`: `tot = sfra − 10 .. efra + 10`).

The keys are the Timeline's collapsed summary row: one diamond per frame
that carries a key on any curve of a shown object, 10 px across
(`icon_size = widget_unit / 2`), `keyframe` fill over a black `keyborder`,
centred `0.8 × widget_unit` under the strip. "Shown" is Blender's
Only-Show-Selected: the selected objects, plus the active object.

Scrubbing is `anim.change_frame`: the left button anywhere in the window
sets the frame, and a drag keeps setting it, to the nearest whole frame.

## 2b. The Dope Sheet shares the window

The Dope Sheet (`BHostType::Action`) is the same canvas with the channel
region shown (`RGN_TYPE_CHANNELS`, `ANIM_UI_get_channel_name_width` =
10 × `widget_unit`): `regions.channels.back` behind the names, `Summary`
first and one row per shown object under it, each name on its row's line.
The summary is expanded there, so it takes `TH_ANIM_ACTIVE`'s backdrop, and
every object row takes `TH_DOPESHEET_CHANNELOB` (`draw_backdrops`), with
that object's own key columns on it (`canvas/time/row`). The rows and the
names come from one list (`BTimeRow::list`), which is what keeps them
aligned — the earlier `Channels` part was a separate column of labels with
its own row height. The placeholder channel list is gone from every
animation editor.

The Animation workspace seeds its bottom zone at 320 px
(`V_ANIMATION_BOTTOM_HEIGHT`), a third of a 900-px window as Blender's
Animation workspace gives the Dope Sheet, and the Dope Sheet's footer — a
region Blender keeps hidden by default — is no longer declared, so the
window is not squeezed to the height of a status bar that showed nothing.

## 2c. The Graph editor shares it too — `graph_main_region_draw`

`BHostType::Ipo` is the same canvas with a value axis (`BTimeView::values`),
offered in the Layout bottom area's switcher as Blender offers it. It opens
where `graph_create` opens: `cur = tot = (0 .. efra) × (sfra − 10 .. 10)`,
so a fresh Graph editor shows frames 0–250 and values −9..10 — Blender
restores the view a file saved, which is not read yet, so this is the view a
*new* Graph editor gets. `BCurveSet::framed()` (the range of every shown key
with a tenth of margin, `graph_view_all`) is ready for a View All / Home.

Painted in Blender's order (`canvas/time/set.rs`):

- the frame grid, `view2d_draw_lines_x_frames`, in `space_graph.grid`
  `#1a1a1a` with minors shaded +16 — the same `calculate_grid_step` as the
  Dope Sheet, from `canvas/time/grid`;
- the value grid, `view2d_draw_lines_y(v2d, show_fractions = true, 10)`:
  `calculate_grid_step_fractions` steps in tens and, under 1, in hundredths
  (`subframe_range` = 100), so a −9..10 view in 400 px steps by 10 with
  minors every 2, and a 0..0.21 view steps by 0.05. The numbers are
  `view2d_draw_scale_y`: `TH_SCROLL_TEXT` `#e6e6e6`, 8 px in from the left,
  kept whole between the strip and the bottom 15 px, with
  `frame_to_string`'s decimals (none at a step ≥ 1, one ≥ 0.5, two ≥ 0.01);
- `ANIM_draw_framerange`'s curtains;
- the curves, `graph_draw_curves` twice — unselected then selected — each
  `draw_fcurve`: `fcu->color` at alpha 1 when selected and
  `U.fcu_inactive_alpha` 0.25 when not, 2.5 px for the active curve and 1 px
  for the rest, `TH_HEADER` shaded +50 grey when muted. The colour is
  `graph_refresh_fcurve_colors`: `CUSTOM` keeps its own, `AUTO_RGB` maps the
  array index to the theme's X/Y/Z axis colours, `AUTO_YRGB` puts W first,
  and `AUTO_RAINBOW` is `getcolor_fcurve_rainbow` verbatim (bandwidth 0.3,
  quartets or triplets by parity, the drift of `cur / tot × 0.7`).
  The line is `draw_fcurve_curve_keys` (`canvas/time/curve/line`): the end
  values held to the view's edges (`FCURVE_EXTRAPOLATE_CONSTANT`), a
  constant step's shelf, a linear run, a Bézier segment sampled at a quarter
  point per pixel of its extent (handles included) and capped at 64, keys
  nearer than three pixels folded into their centre. Not one root-find per
  pixel: 530 curves cost a few thousand cubics a frame, not half a million
  bisections;
- the keys, `draw_fcurve_keyframe_vertices`: `vertex_size` 6 dots,
  `TH_VERTEX` black for an unselected key and `TH_VERTEX_SELECT` `#ff8500`
  for a selected one (`BezTriple.f2 & SELECT`, now read by phiphx as
  `BKey::selected` — `rigAction` has 155 of its keys selected, as Blender
  counts), faded half as far as their curve and never under 0.2, unselected
  first, 0.8 size on a protected curve;
- the strip and the playhead, exactly as the Timeline's.

The channel region lists the shown objects, with no Summary row — the
Graph editor has none (`graph_channel_region_draw`). Scrubbing is the same
`anim.change_frame`, on the Graph editor's own view: halfway across is
frame 125, not 126.

The grid's minimum line distance is `get_min_line_distance_x`'s: the
wider of the two edge labels, measured as `frame_to_string` spells a value
at step 0 — with three decimals, so `260.000` — plus 6 px, or
`U.v2d_min_gridsize`, whichever is more. The Graph and Drivers editors
step their x axis with fractions (`show_fractions`), the Dope Sheet in
whole frames, and the strip's numbers carry the decimals their step needs
(`frame_to_string`: whole at a step of one, then one, two or three).

Handles (`canvas/time/curve/handle`) follow `draw_fcurve_handles`: a
key's first handle shows when the segment before it is Bézier (its own,
when it is the first key), its second when its own segment is; unselected
handles are drawn before selected, and a selected key's handles are drawn
in both passes, as Blender draws them; each line is 1 px in the handle
type's `TH_HANDLE_*` colour at the curve's display alpha — free, auto,
vector, aligned, auto-clamped, indexed by `BezTriple.h1 / h2`, the
unselected set the selected set darkened. Each handle's end carries
`draw_fcurve_handle_vertices`' ring: `1.4 × TH_HANDLE_VERTEX_SIZE` = 7 px,
a 1.5 px outline in `TH_HANDLE_VERTEX` or `_SELECT` by the handle's own
selection (`f1 / f3`), the fill at 0.01. A protected curve shows no
handles (`draw_fcurve_handles_check`), and a curve whose keyframes a
replacing Generator makes unusable (`BKE_fcurve_are_keyframes_usable`)
shows neither handles nor vertices.

Not yet ported, in the order they would show: the active key's handle
rings in `TH_VERTEX_ACTIVE` (needs `FCurve.active_keyframe_index`); Only
Selected Keyframes Handles (`SIPO_SELVHANDLESONLY`, off by default — the
list is built for it); the channel list's groups and F-curves with their
colour swatches under each object (the Drivers editor has them, §2e); the
scrollers; dashed lines for protected curves; the outline shadow behind the
value numbers (`BLF_SHADOW`); normalisation; the header's mode selector and
Graph menus; and the saved View2D.

## 2d. The NLA editor shares it — `nla_draw.cc`

`BHostType::Nla` is the same window at `NLATRACK_HEIGHT` (1.2 ×
`widget_unit` = 24 px, skip 2, first row 23 + 2 down) over
`nla_create`'s view (`sfra − 10 .. efra + 10`). The rows are
`animfilter_nla`'s: each shown object, its action line, then its tracks
last first (`canvas/time/row::BTimeRow::nla`). The action line
(`nla_draw_action_line`) is `TH_ANIM_ACTIVE` at 0.4 across the whole view
when `act_extendmode` holds, `TH_ANIM_INACTIVE` for `<No Action>`,
`TH_NLA_TWEAK` while `ADT_NLA_EDIT_ON`, with the action's key columns on
it as 6-px diamonds. Each strip (`nla_draw_strip`, `canvas/time/strip`) is
a box from `start` to `end` in `TH_STRIP` / `TH_STRIP_SELECT`, muted at
0.1, outlined black or white when active, with its hold before and after
(`EXTEND_HOLD`, 0.15 / 0.3), a line at each whole repeat inset 4 px, and
its name 8 px in. The channel region gives each track `TH_NLA_TRACK`'s
backdrop and the action line its own colour.

phiphx reads `Object.adt → AnimData` (`blend/anim/nla.rs`): the active
action, `act_extendmode`, `flag`, and `nla_tracks → strips` with each
strip's range, action range, repeat, scale, flags, type, extrapolation and
action, into `SceneModel.nla`. `tests/nla_truth.rs` checks a fixture bpy
builds (`truth/blender_nla_fixture.py`: a cube with `Idle` active, a
`Base Layer` track holding `Walk` repeated twice at 1 .. 97, an `Overlay`
track holding a muted, selected clip at 120 .. 168, and a sphere with
empty animation data) against what bpy lists (`truth/nla_cube.txt`).

## 2e. The Drivers editor shares it — `SIPO_MODE_DRIVERS`

`BHostType::Drivers` is the Graph editor in Drivers mode. Its view is
`ED_drivers_editor_init`'s (`−2 .. 2` both ways: `view/mod.rs::DRIVER_VIEW`)
and its x axis is the driver's input value, not time: no frame range,
no preview range, no playhead, no scrub (`"Driver Editor's X axis is not
time"`), and both grids step in tens whatever the frame rate
(`driver_step`, `BTimeView::base`). The curves are the shown objects'
drivers (`ADS_FILTER_ONLYDRIVERS` under Only Show Selected,
`canvas/time/driver::BDriverList`), coloured over the whole list as
`graph_refresh_fcurve_colors` colours them, and each is drawn as
`draw_fcurve` draws a curve with a modifier or under two keys —
`draw_fcurve_curve`: `evaluate_fcurve` every pixel and a half across the
view, the far edge included (`BCurveLine::sampled`) — so a fresh driver's
Generator `1 − x` is the line it is in Blender and a keyless driver with no
modifier is the flat line at 0. The cursor (`graph_main_region_draw` with
`SIPO_NODRAWCURSOR` off) is a 2-px line across the view at `cursorVal` in
`TH_CFRAME` shaded −10 at alpha −50 — drawn in the Graph editor too — and,
in Drivers mode, a second one up the view at `cursorTime` shaded −40
(`canvas/time/driver/set.rs`; `theme/style/anim.rs::cursor_value /
cursor_time`).

The channel region lists each object over a row per driver
(`ANIMTYPE_FCURVE`), named as `getname_anim_fcurve` names them:
`<axis> <property> (<struct>)`, the struct `bone : constraint` when the
path holds both, the bone or the constraint alone otherwise, nothing for
the ID's own properties — `Influence (MCH-upper_arm_parent.L : Copy
Rotation)`, `Disable (MCH-forearm_ik.L : IK)`, `Blend Weight (MCH-torso.parent
: SWITCH_PARENT)`, `X Euler Rotation (MCH-thumb.02_drv.L)`, `Hide
(thigh_ik.R)` — with the property names `RNA_property_ui_name` gives
(`BTimeRow::driver_name`). Each driver row wears `TH_SHADE2`'s backdrop
and, as `ANIM_channel_draw` paints in the Graph editor, its curve's colour
swatch `ICON_WIDTH` (0.85 × `widget_unit`) wide at the group offset (0.7 ×),
then the width of the visibility toggle, then the name 3 px on.

phiphx reads `AnimData.drivers` (`blend/anim/drivers.rs`, `read.rs::
id_drivers`) with each F-curve's Generator polynomial
(`FMod_Generator.coefficients`, `BFCurve::generator / generated / evaluate`)
— and applies the step Blender 5.0 applies to older files at load
(`version_bone_hide_property_driver`, `versioning_500.cc`, below file
version 5.0.110): a driver on `bones["…"].hide` stored on the armature data
is copied to every object using that armature as `pose.bones["…"].hide`,
after the object's own drivers. That is why bpy lists 101 drivers for the
rig where the file's object holds 93: the other 8 are the armature's, moved.
`tests/drivers_truth.rs` checks every object's drivers — path, index, key
count, coefficients, in order — against bpy's (`truth/blender_drivers.py`
→ `truth/thymus_drivers.txt`). The Drivers editor is offered wherever
Blender's editor menu offers it: the bottom band of the Layout, Animation,
Rendering, Compositing and Geometry Nodes workspaces.

Not ported: the sidebar's Drivers tab (`ED_drivers_editor_init` shows the
UI region with it), driver variables and expressions (only the F-curve and
its Generator are read, so the curve is drawn but nothing evaluates the
driver), `cursorTime / cursorVal` editing, and `FCURVE_VISIBLE`.

## 3. What phiphx had to read

`SceneModel.actions` was a never-filled stub. `blend/anim/summary.rs` now
gives every action its distinct keyed frames across every curve of every
slot (the Dope Sheet's `ActKeyColumn`s), its curve count, its range and its
users — the objects whose `AnimData.action` points at it — and
`SceneAction` carries all of it. `tests/keys_truth.rs` checks the fixture
against `truth/thymus_keys.txt` from Blender: `rigAction` has 530 curves,
22 columns at 0, 10, 20, 25, … 130, 132, and one user, `rig`.

## 4. Room for it

The bottom zone was 80 px shared evenly with the status bar, so the
Timeline's window was 6 px tall and nothing in it could have shown. The
zone is now 140 px, and a host that is one bar tall (`is_fixed_bar`: the
status bar and the top bar) keeps its own height while the editors beside it
share the rest — the shares of the sharing areas are scaled to sum to one,
because flex only hands out all of the free space when the grow factors
reach one. The Layout window at 1100 × 700 gives the Timeline 60 px of
window, against Blender's ≈ 70 at that height; the zone seam still drags.

## 5. Guards

`timeline_scrubs_like_blender.rs` drives the built window in GPUI's test
harness: a press in the Timeline window sets `ctx.frame` to the frame under
the pointer, a drag keeps setting it and lands where the pointer stops, and
the header's play button is inside the Timeline header and starts playback
when clicked. This is the one test that would fail if the canvas painted
but never took the pointer.

`host_visual_fixes.rs` pins Blender's shape: the window is the Time
canvas, the transport is the nine `playback_controls` buttons in order,
play toggles `ctx.playing` and shows a lit pause, the right group is
`ctx.frame`, the preview toggle, `ctx.frame_start`, `ctx.frame_end`, and a
property test that the frame field shows the state's frame. The layout
golden was re-recorded for the moved transport and the taller zone.

`graph_editor_draws_like_blender.rs` switches the Layout bottom area to the
Graph editor in the built window: the window and its header are painted, a
press in the channel region does not scrub, a press halfway across the
window sets frame 125 (the `0 .. efra` view), and — with a loaded action of
two curves — the recorded scene holds exactly three round `vertex_size`
quads, at the pixel each key's frame and value map to on the `−9 .. 10`
axis, the selected keys in `TH_VERTEX_SELECT` orange and the unselected one
black, and six 7-px handle rings with a 1.5-px outline, the first key's
left handle at frame 45. The unit tests under
`canvas/time/{grid,curve,curve/line,curve/handle}` pin the grid steps, the
colour modes, the rainbow, the fades, the line's points and the handle
passes against the Blender source they are ported from;
`phiphx/tests/keys_truth.rs` also checks the selected-key count, the handle
types and the selected-handle count per action against Blender's.

`nla_editor_draws_like_blender.rs` switches the Animation workspace's Dope
Sheet to the NLA editor with a cube's NLA loaded: the action's band runs
across the second row, the upper track's strip is the outlined box at
120 .. 168 on row 2 and the lower track's at 1 .. 97 on row 3, with a
repeat line at frame 49.

`drivers_editor_draws_like_blender.rs` switches it to the Drivers editor
with a rig's two drivers loaded: the cursor's value line crosses the view
at 0 (the window's centre on `−2 .. 2`) and its input line runs from under
the strip to the foot, nothing bordered stands the height of the view (no
playhead), no curtain is painted at `ANIM_draw_framerange`'s alpha, and the
channel region carries a colour swatch on each driver's row and none on the
object's. `canvas/time/driver` pins the channel names against Blender's
spellings and `canvas/time/curve/line` the sampled line: 400 samples plus
the far edge across 600 px, `y = 1 − x` through `(0, 1)`.
