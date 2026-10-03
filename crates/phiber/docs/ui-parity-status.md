# Phiber UI — status

**Suite: 1037 passing, 7 failing** (phiber, on the tree as it stands on the
machine — all seven are the other agent's work in progress: a region's host
corners, its overlay-family key test, the Preferences palette override and
its rows' actions, the layout golden it recorded, the activity bar's
treatment and the Outliner's guide lines; see batch 100). phiphx 191
passing, 6 known asset-dependent; phiray 77 passing. Batches 17–124 are
extracted on the machine; every item below is covered by a test. The
Animation and Rendering workspaces have a full audit of their own in
`animation-rendering-status.md`.

---

## The root causes

Almost everything reported traced to one of these. Each was producing symptoms
in places that looked unrelated, which is why fixing them once fixed many.

1. **No shared notion of "a control in a strip."** Anchors, buttons and tabs
   each had their own rest/hover/active logic; the tab atom special-cased
   `BHostType::Topbar`. → `atom/chrome.rs` owns the mapping. The only branch is
   floating vs docked: floating chrome rests on a fill because it sits over the
   canvas; docked chrome doesn't, because that made headers into pills.
2. **Geometry duplicated instead of derived.** → one `MODULE = 36`; one
   `BCapSide`; one seam overlay for zone, area and panel edges; one gutter
   identity; one `pane::floating_strip_room`.
3. **Structure invented per site.** Two hosts each built a search while the
   shared header rule added a third. → `part::search` owns the middle,
   `part::filter` the right, switcher and menus the left.
4. **A predicate that was too broad.** `is_floating_pane` matched *any*
   Left/Right pane in a viewport host — including the header's — so the
   viewport's menu bar rendered as an absolute overlay. → only a Center
   region's side panes float. This was also the gizmo/menu overlap.
5. **Assets stubbed, not authored.** 180 of 450 icons shared one of two
   drawings; 41 ignored the theme colour; 49 were off the shared canvas. → the
   48 on screen are drawn, all 450 take `currentColor` on a 24-unit canvas.
6. **Tests pinning spelling, not invariants.** → rewritten to assert the
   guarantee, not the builder chain.
7. **Declarations that lied about the render.** Twenty-four host files each
   declared `Left=[Menubar] Center=[Modebar] Right=[Switcher]`, and
   `region::item::complete_header` silently rewrote it on every build. → one
   `host::header::EDITOR` (`Left=[Switcher, Modebar, Menubar]`, empty Centre
   and Right as the flex slot), referenced by all of them. Image, UV and the
   Sequencer also declared a second mode dropdown in their tool header; in
   Blender `RGN_TYPE_TOOL_HEADER` carries tool settings, so they now use
   `host::header::TOOL` (`Left=[ToolProps]`). *(sync81)*
8. **Colours manufactured per host.** `theme/semantic.rs::host_lightness`
   handed every host its own lightness (viewport 24 %, Properties 19 %,
   Outliner 16 %, node editors 10 %, top bar 9 %) and every strip, tab and
   hover was a lift or recess off that; the palette itself was thirty-three
   hand-set values, and the animation editors nudged lightness off them
   again. That is why hosts differed and why the Timeline sat on its own
   greys. → the shade ladder (below). *(batches 29–30)*
9. **The area card painted the window shade.** Batch 30 moved
   `workspace_background` to shade 2 so the seams read as the window, but
   the area card — the surface every editor's regions sit on — was filled
   with that same token, so the Outliner body and the activity rail went
   to shade 2, and the icon rails (`NavBar`) painted nothing and showed the
   window through. Separately, Solid mode handed the tracer the *file's*
   world colour as its sky, so a file with a black world gave a black
   viewport where Blender's Workbench shows the theme's canvas. → the card
   is `BFill::HostBody` (`host_background`: panel for an editor, chrome for
   a bar), a rail is `BFill::HostRail` (chrome, flat), and only Rendered
   looks through to the world (`rendered/mod.rs`). The layout golden now
   records the bodies at shade 5 and the chrome at shade 3. *(batch 31)*

---

## One element, one rule

- **Header** — `host::header::EDITOR`, one declaration for every editor:
  switcher, then mode selector where `modes_for(host)` is non-empty, then the
  host's menus, all in the left group; search in the middle, filters right.
- **Strip** — a header row and a vertical rail are `prim::strip` differing only
  in axis (`Row`/`Column`) and fit (`Hug`/`Divide`).
- **Cap** — `BCapSide` decides rounding, gap and paint inset; right rails
  mirror; `Free` is floating chrome, fully rounded.
- **Seam** — zone, area and panel edges share one zero-footprint overlay. The
  gutter is per boundary; the *drag* only where there is width to trade. A
  modal keeps the same seam from the window edge; a dialog body is padded
  like a region's content (`PadMd`), not more.
- **Input** — one `field::shell`: the clickable is the field.
- **Row** — groups split by `Spacer`, placed start / centre / end. One
  geometry, selected or not — a row that changed size with its state was the
  "dancing" on hover.
- **Icon** — one 24-unit canvas, `currentColor`, no shared drawings in use.
- **Shade** — every surface, line and letter is a step of the theme's ladder
  (below); a renderer never shades a colour itself (`tests/shade_ladder.rs`).
- **Sortable heading** — a tab: the header's tab radius, the sorted column on
  the active shade at rest (`view/draw/sort.rs`).
- **Module** — `BDim::CardHead` = the button height plus `HEAD_ROOM` (4 px):
  30 px (`BDimensions::card_head_height`). A card heading, a card row, a
  split row, a rail entry — the activity bar's, the Properties navbar's,
  the Preferences rail's, a floating strip's — the file list's column
  heading and the footer's line are all this one module; a button or field
  inside it stays 26 px, so it has 2 px over and under it and never
  overlaps the heading it sits in (the user's batch-40 complaint). Blender's
  `PNL_HEADER` is 1.25 units — the user chose one module. A rail's lead is
  derived from it (`rail_lead`). *(batches 32, 40)*
- **Card heading** — `BFill::Heading(tone)` → `heading_surface(tone)`: a
  heading is **one step over what it heads**, swept like every surface
  where a theme has gradients on (a panel's heading 8 → 7 over its card at
  7 → 6) and flat where not (7 over 6), so the same lift shows in Graphite,
  Apple and Cyberpunk. The section sweep the batch-36 mocks were approved
  with ran top to bottom; with every sweep now running left to right
  (below) the heading and the card would have been the same colour, so the
  heading moved up a step and kept its lift. The file list's column heading
  is the same fill over the same card (`table`, `tone::heading_bg`); a
  headed body takes the card's surface as it takes its radius (`body.rs`,
  `tone::card_bg`), so the File Browser is built of the same headings and
  contents as the sidebar beside it. Batch 36 first tried a darker heading,
  then a swapped body; both were withdrawn the same day. *(batches 36, 40)*
- **Sub-panel tree** — a sub-panel (`BCardKind::Sub { depth }`) is a branch
  of its panel's tree, as an Outliner child row is: indented one `pad_sm`
  under its parent's rows with a guide down its left edge (`BEdge::Guide`,
  the separator hairline the Outliner's hierarchy lines use), recessed a
  step toward the region per level (`BSurfaceTone::Nested(depth)`: the body
  shade at depth 1, the header shade at depth 2) and its heading a step over
  its own well. `BCard::sub()` carries the depth: a card pushed into a
  sub-panel moves a level deeper with everything under it. Properties'
  Sampling ▸ Render ▸ Denoise reads as three levels in indent and shade
  (`tests/sub_panels_are_a_tree.rs`). *(batch 40)*
- **Sweep** — every gradient runs left to right (`BGradientAxis::Trailing`)
  from one step over the slot at the left edge back to the slot by half the
  width (`SWEEP_STOP` = 0.5; `BGradientColors.stop`), flat from there: the
  lit edge reads against a border and the surface keeps its colour. Headers,
  bodies, cards, wells, headings and the canvases all sweep the same way
  when a theme has gradients on (`viewport_gradient`); the tracer paints the
  same lift into its sky (`viewport_sweep_lift` → `Camera::sky.a`,
  `sky_at` in the WGSL), so the 3D viewport sweeps like the node and image
  canvases beside it. Off, everything is flat. *(batch 40)*
- **Floating strip** — the viewport's toolbar and sidebar-tab strip paint no
  rail of their own: each entry is a 30 px button (`control_fill(floating)`:
  the control shade at rest, hover, the accent when active), its icon the
  icon shade or, active, the highlight shade (`icon_ink`). The strip is one
  module wide and inset `pad_sm` from the canvas edge. *(batch 40)*
- **Actions bar** — the cluster at a rail's foot (collapse, the paging
  arrows) sits on a bar painted as a floating control is (`rail::bar` →
  `chrome::paint`): the control shade at rest, the hover shade under the
  pointer, always visible, so nothing sits directly on the rail unmarked.
  *(batch 40)*
- **Typeface** — *Preferences ▸ Interface ▸ Typography ▸ Family* chooses one
  of eight faces (`BFontFamily`: System · Helvetica Neue · Segoe UI · Inter ·
  Menlo · Consolas · Cascadia Code · JetBrains Mono — the system's, the two
  Mac faces, Inter, and VS Code's four code faces), a style descriptor like
  the shadow (`BStyleParser::Font`, key `typography.family`), set once at
  the renderer root with the family's own fallback stack behind it
  (`backend.rs`); every letter inherits it. *(batch 40)*
- **Strip line** — every control in a header or tool header sits on one
  centre line: the centre of the cap band (the strip less `menu_top_gap`,
  which the header pane pads off the top). A field, an icon button, the
  brand mark centre on it. Nothing rides a pixel above its neighbour
  (`tests/chrome_geometry.rs`, `central_reuse.rs`, `file_browser_layout.rs`).
  The execute bar has no caps, so its controls centre on the bar itself.
- **Strip control** — the editor switcher, a pulldown menu (`MenuAnchor`)
  and an icon toggle (display mode, shading, filter, the Image editor's
  toggles) are all *buttons*: the button height, all corners rounded,
  centred on the strip line — one element, one shape, as the approved
  mock-ups draw them and as Blender's pulldowns are `UI_UNIT_Y` buttons.
  A header toggle is `BView::Button` with `active`, no longer a cap cut into
  the strip; the workspace tabs in the top bar stay caps, as Blender's are.
  *(batch 37)*
- **Strip of columns** — the file browser's path strip is the columns under
  it, read across (`pane::columns`, `region::tools_column_width`): its left
  pane is the tools region — as wide, padded the same — with the folder
  moves at its start and the refresh at its end, flush with the source
  list; its centre pane pads itself like the content region, so the
  directory field starts where the file list does; the strip itself pads
  nothing. Parts `FileNav` · `FileRefresh` · `PathBar` · `TypeFilter`
  (`part/file/nav.rs`). *(batch 37)*
- **Field margin** — a text field pads `pad_md` either side (Blender's
  `UI_TEXT_MARGIN_X`, 0.4 of a unit), and its text clips inside that margin,
  never at the edge (`field::shell`). *(batch 37)*
- **Blank frame** — a traced frame that comes back all zero (the kernel
  never wrote it) is published empty, not painted: the viewport keeps the
  theme canvas and the failure is printed once (`rendered/worker.rs`,
  `rendered/mod.rs`). The 3D viewport is never black by accident.
  *(batch 37)*
- **Rail grid** — beside a column of cards (Properties, Preferences) the
  rail's caps share the rows' grid: the first cap sits level with the first
  card heading — a headless card's rows start where a heading would — and
  both step by the row pitch, so *Appearance* is level with *Graphite*
  (`rail_lead`, `content_pad`; `tests/preferences_rail_shares_the_row_grid.rs`).
- **Zone toggle** — the top bar's three zone toggles are drawn, not
  loaded: a frame with the zone's third filled (or outlined when hidden),
  in `BFlex` with an ink stroke (`BEdge::Ink`, `BFill::Ink`), rounded by the
  button radius at the glyph's scale — square in Ember, a pill in Plum.
- **Rail** — every vertical strip of tabs — Properties' navigation bar,
  Preferences' category bar, the activity bar — is `BView::Rail` on the
  chrome shade: one cap per entry, the active one on the active shade at the
  theme's radius, the collapse toggle at the foot. Preferences' opens
  expanded (`strip::default_expanded`). Never a zebra list.
- **Theme** — one seed ladder, one accent, one radius scale, one set of
  effects (sweeps, surface shadow) and one seam gap (`theme/style/mod.rs::
  defaults`). Typography, spacing, dimensions, strokes and roles never
  change between themes, so a header is the same height and a tab the
  same shape in all six.

## Fixed

Search overflow · unusable filename field · execute buttons passing labels as
icon names · card/heading gutter · rail cap centred on the first heading · caps
compressing 26→16.5px · header icons riding low · 18px resizer gap → 4px ·
sidebar's dark parent · three spurious bottom bars · empty left band · outliner
off-palette shades · pills · one-sided floating buttons · square menu rows ·
modal vs host mismatch · double search fields (two hosts) · viewport detached
from the activity bar · viewport header as a floating overlay · empty header
right slots · uniform menu widths · expand arrows on hover · one hover padding ·
compass caps writing a yaw where Z needs a pitch · overlays clearing the
floating strips · file columns filling their row · canvas squaring off the zone
corners · vertical-pill icon caps · gizmos in modal-dim instead of the chrome
fill · modal bands stacking with gaps · header right controls stranded mid-row ·
toolbar icons not square when collapsed · outliner rows shrinking instead of
scrolling · third-level disclosure missing · hierarchy lines stopping at the
content · prefs drawing two headers · header filters not folding on a narrow
window · 48 placeholder icons · rows moving on hover/selection · headers
declaring an order they did not render · duplicate mode dropdown in three tool
headers · per-host lightness · the *Radius Strength Falloff* placeholder tool
settings over the Image editor (a tool header with nothing to show is dropped,
as Blender's Rendering workspace hides it) · the asset-shelf drawer reserving an
empty band under the Image editor outside Paint mode (the window reaches the
Timeline, guarded end to end) · modal gaps wider than the spaces' · file-list
headings without the header's tab rounding · the Outliner body, activity
rail and Properties nav rail on the window shade · a black viewport in Solid
mode from the file's world colour · card headings taller than the rows under
them · Preferences' rail off the rows' grid · the header touching the rail and
the content (the region's pad) · header fields a pixel below the menus ·
squarish zone toggles in a rounded theme · Preferences without its header in
the dialog (`USERPREF_HT_header`: switcher, View and Preferences menus, Save
Preferences at the right; the execute strip polls away while the header shows) ·
the Animation workspace's left viewport orbiting where Blender looks through
the camera (`scene/camera_view`, `view3d/frame.rs`; frame, passepartout and
"Camera Perspective" as `view3d_draw_border` — see `view3d.md` rows 13–14) ·
grid lines a step over the canvas on the viewport, image and graph hosts
where Blender's `#545454` grid reads light over `#3D3D3D` (shade 11) ·
no way out of (or into) the camera view: the gizmo's camera button toggles
it for its area (`BCameraSlot`, carried by the canvas), an orbit drag leaves
it carrying the camera's view over (`BCameraView::leave`), and the file's
`camzoom` is read for it (phiphx `RegionView3D.camzoom`) *(batch 38)* ·
play-reverse stepping one frame instead of running: `ctx.playing` is `true`
or `reverse` and the one ticker steps either way; while playing, one pause
button two slots wide stands where the two play buttons were
(`playback_controls`, `BControl::span`) · the Dope Sheet's footer, hidden
though Blender 5.2 draws `playback_controls` in it: declared, with the same
transport and frame-field parts as the Timeline's header; every footer is
now a strip like the header — the header's height, surface and centre line
(`ED_area_footersize`); the Dope Sheet sidebar's tab is *Action*
(`DOPESHEET_PT_action`), not View / Options *(batch 39)* · the module grew
to 30 px so a button no longer overlaps the heading it sits in; eleven
themes, each with a Dark / Light switch; every sweep left to right, back on
its shade by half way, the canvases and the tracer's sky included; a heading
a step over what it heads; sub-panels a tree — indented, guided, recessed a
step per level; the viewport's strips bare buttons with theme-shade icons; a
rail's actions bar always painted; the typeface a Preferences choice of
eight *(batch 40)* · picking a theme or its mode clears every `ui_style`
override, so a header and a viewport nudged under one theme never stay
under the next (`BThemePickRule`, Blender's preset load) — the green
canvas under Barbie; the brand mark 16 px, an icon's size *(batch 41)*.

## Batch 41 — the app does what Blender does

- **Playback plays the file** — phiphx reads every object's rest transform,
  the curves its action slot binds and its parent (`anim/motion.rs`:
  `BlendMotion`, `BlendMotionList::world_at(frame)`, the parent solve
  shared with the load path, pinned to Blender's matrices for THYMUS in
  `anim/tests.rs`); `SceneModel::motion` maps every mesh, camera, light and
  empty to its motion. phiber poses the scene at `ctx.frame`
  (`scene/motion.rs`): the object list, the lights the tracer sees, the
  overlay's cameras and empties, the camera view and the F12 plan all take
  the shown frame; the memo that keys the object list takes the frame only
  while the scene moves, so a still scene never rebuilds as the playhead
  runs (`tests/playback_moves_the_scene.rs`). The ticker keeps time, not
  ticks — a late tick steps the frames that fell due, capped, the fps read
  live (`app/playback.rs`); reverse play animates the time canvas too.
  Not yet: armature deform per frame (`poses_at` exists, unwired), NLA,
  drivers, constraints, easing interpolations.
- **A click selects** — the canvas point becomes the same ray the tracer's
  kernels build (`scene/pick.rs::BPickView::ray`, mirrors `solid.wgsl`),
  cast against every visible object's triangles with a box cull; nearest
  hit becomes `ctx.object`, the Outliner row lights, the Properties tabs
  refill, the overlay names it. The click rides the orbit surface (gpui's
  2 px drag threshold tells them apart); the floor-anchored `BHit` squares
  are gone. Shift-click extends. A selection set now exists
  (`ctx.selected`, `scene/select.rs`: `BSelection`, the active object is
  the selection until one is made; `object.select_all` all / none /
  invert over the visible scene) and every selected object is outlined in
  the accent on the GPU path too (`cache::meshes` builds outline-only
  meshes when the tracer draws) *(`tests/viewport_click_selects.rs`)*.
- **Rendering works** — the loaded-scene scope was cleared at the end of
  every render pass, so everything that ran after it (F12's
  `ensure_render`, the Properties refresh on select, the tracer's
  paint-time world and lamps) saw no model; the scope now persists between
  passes. F12 re-renders every time (`app/op.rs::render_image`); Ctrl F12
  renders the range frame by frame and writes each where Output points,
  numbered like `BLI_path_frame` (`render_save::frame_file`); F11 only
  shows.
- **Menus** — every item answers (`tests/menus_answer_every_click.rs`: 368
  leaves, 141 run an operator, 227 report themselves to the status bar and
  the Info editor as "not available yet" — never a silent close). New kinds
  in `menubar/item`: `Fire(OpKey, value)` (one intent, `Set(ctx.op.<key>)`,
  consumed by a rule or by `app/op.rs`), `Toggle(key, on_by_default)` (a
  check in the icon slot, flipped through `ctx.op.flip`), `Recent` (the
  recent files, live). Rules: `BReportRule`, `BFlipRule`, `BSelectOpRule`,
  `BPickSelectsRule`, `BSceneEditRule` (add a primitive — Plane, Cube,
  Circle, UV Sphere, Ico Sphere, Cylinder, Cone, Torus, Monkey, named
  `Cube.001` as `BLI_uniquename` names them — delete, duplicate in place,
  hide, unhide; three keys `ctx.scene.added/deleted/copied` over the file's
  list), `BView3dOpRule` (frame all / selected, viewpoints, orbit steps,
  zoom, persp/ortho), `BFrameRangeRule`, `BOutlinerOpRule` (hierarchy, one
  level in / out), `BPropsFoldRule` (expand / collapse all), `BSpaceStepRule`,
  `BFactoryRule`, `BPaletteDismissRule`, `BImageZoomStepRule`. App
  operators (`app/op.rs`): Quit, Fullscreen, Undo / Redo over the document
  keys (32 steps, `app/undo.rs`), New (template → workspace, clean scene),
  Revert, Render image / animation, Export on the browser's commit — OBJ,
  PLY, STL, USDA, glTF of every visible object in world space
  (`scene/export.rs`) — and Import (OBJ, USD; glTF with the `gltf-import`
  feature) merged into the scene under the Scene Collection
  (`scene/import.rs`). Help URLs now close the menu and log. Still
  reporting: Save / Save As (.blend writing), Link / Append, Undo History,
  Rename, screenshots, markers, node menus, file-browser Select, the UV
  editor's operators.
- **Workspaces** — the tabs read *UV Lights* and *Texturing* (the slugs
  stay `UvEditing` / `TexturePaint`, so a file saved with Blender's names
  still lands on them).
- **Activity bar like Slack** — every primary (Home, Chat, Spaces, Stream,
  Settings, Profile) is a stacked entry, its icon with its name under it
  (`tab/stack.rs`, rail 64 px), and opens its panel beside the rail: a
  bordered card on the card shade headed by the primary's name, its rows
  menu rows with the icon slot reserved so the labels line up. Home lists
  New File, Open… and the recent files; Chat the session's channels
  (`collab.channel.*`), Stream its peers, Spaces the workspaces, Settings
  the categories — nothing invented: an empty table says so.
- **Profile like Discord** — the Profile panel (`activity/profile.rs`)
  leads with the picture, 80 px round (`BView::Avatar` → `draw/avatar.rs`:
  the image at `profile.avatar`, else the user glyph in a ring), the name
  and handle beside it, then four accordions — **Profile** (the fields,
  open first), **Settings**, **Channels** they own, **Followers** — each
  folding on `ctx.activity.profile.<section>.expanded`.

## Batch 42 — the keys, the curves' easing, the markers

- **Hotkeys** — a menu item's shortcut label *is* its binding
  (`app/keymap`): `BKeystroke::parse` reads "Shift Ctrl Space", "Numpad 7",
  "F12" as the labels spell them, the key listener turns a gpui keystroke
  into the same shape, and `action_for` walks the topbar and the current
  editor's menus for the first item that carries it, then runs the item's
  own action — so a key does exactly what its menu row does, and a
  shortcut added to a label works without another table. Emulate Numpad:
  the number row is the numpad. The screen keys Blender binds outside any
  menu ride on top: Space play / pause, Shift Ctrl Space reverse, Escape
  stops playback or closes the open modal, Left / Right step a frame, Up /
  Down jump keyframes, Shift Left / Right to the range ends, Delete. A
  focused field keeps its typing (`tests/keys_run_the_menus.rs`).
- **Easing and extrapolation** — `fcurve::eval` now runs every
  `BEZT_IPO_*` Blender has: Back, Bounce, Circ, Cubic, Elastic, Expo,
  Quad, Quart, Quint, Sine, each with `BEZT_IPO_EASE_AUTO / IN / OUT /
  IN_OUT` (`anim/easing.rs`, the same formulae as `BLI_easing`), the
  per-key `back`, `amplitude` and `period`, and `FCURVE_EXTRAPOLATE_LINEAR`
  past the ends along the end handles' slope. Muted curves are skipped.
- **Selection** — a click on empty canvas deselects (Blender's default
  `deselect_all`); Shift on empty keeps the set. The Outliner paints the
  active object's row a step lighter than the other selected rows
  (`BRowShade::Active`).
- **Markers** — phiphx reads `Scene.markers` (`blend/scene/marker.rs`,
  `SceneModel::markers`). phiber keeps the list in `ctx.markers` once
  edited (`scene/marker.rs`: `BMarkerList`, `BMarkerOpRule`): **M** adds
  `F_<frame>` at the playhead once per frame, Marker ▸ Delete removes the
  one under it, Jump to Next / Previous moves the playhead or logs "No more
  markers", and Undo walks the edits. The animation window draws them as
  `anim_markers.cc` does — a 42 px band along the foot in the scrub shade
  once the scene has any, a dashed stalk (six-pixel period, half drawn) up
  from each, the marker glyph astride the frame and its name to the right
  (`canvas/time/marker/`); the Drivers editor, whose x axis is not time,
  shows none.

## Batch 43 — the Outliner's toggles mean it, box select, rename, the top bar wraps

- **Restriction toggles from the file** — phiphx reads
  `Object.visibility_flag` (`OB_HIDE_VIEWPORT / _SELECT / _RENDER`;
  `restrictflag` before 2.80) and `Collection.flag`
  (`COLLECTION_HIDE_*`) into `BlendVisibility`
  (`blend/object/visibility.rs`), carried on every object and collection
  and listed in `SceneSelection::{viewport_disabled, unselectable,
  render_disabled}`; `seed_view_layer` writes them to the rows' keys, so
  the monitor, the arrow and the camera open as the file has them. And
  they do what they say: a render (F12, Ctrl F12) leaves out what the
  **camera** hides — its own or a collection's, inherited down the tree —
  and not what the eye hides, as Blender renders what the viewport hides;
  the **arrow** keeps an object out of a click (the ray goes through to
  what is behind), a box and Select All (`BHidden::{now, for_render,
  for_select}`, `pick_where`).
- **Icon row** — a closed row shows what it holds after its name, one
  glyph per kind with its count when there is more than one
  (`outliner_draw_iconrow`, `outliner_draw_iconrow_number`): a closed
  collection its child collections and its objects by type, all the way
  down; a closed object its mesh (`BOutlineRow::summary`, `BView::Badge`
  → `draw/badge.rs`).
- **Box select** — Select ▸ Box Select (**B**) arms the gesture
  (`ctx.view3d.box.select`); the next drag on the canvas draws the box as
  `WM_gesture_draw` does (a dashed outline over a solid one) and the
  release selects every selectable object whose bounds, projected through
  the view (`BPickView::project`, the ray's inverse), overlap it — Shift
  adds, Ctrl subtracts (`scene/boxsel.rs`, `view3d/boxsel.rs`,
  `BSelectOpRule` `box:` / `box+:` / `box-:`). The active object stays;
  Escape cancels an armed box. Blender reads an ID pass under the box, so
  an object hidden behind another still counts here; that is the gap.
- **Rename** — Edit ▸ Rename Active Item (**F2**) opens Blender's popup
  (`TOPBAR_PT_name`): one field holding the active object's name with the
  caret in it; Return (or OK) renames, Escape leaves it. The rename is a
  scene edit (`ctx.scene.renamed`, `BSceneEdit::rename` — an added or
  duplicated object is renamed in its own entry, a file object gets an
  entry that is updated in place and dropped when it gets its file name
  back), so Undo takes it back; the active object, the selection and the
  row's toggles follow the new name; a taken name is refused with a
  report (`modal/rename`).
- **Top bar wrap** — below 960 px the top bar's three cells no longer fit
  one line: the left cell (the brand and the File… menus) keeps the first
  line and the workspace tabs and the right-hand controls go under it, the
  bar growing a line (`BFlex::wraps`, `filter::topbar_wraps`). An editor's
  header does not wrap — Blender clips and scrolls it — so an area's
  chrome keeps its height whatever its width (`tests/header_wraps.rs`).

## Batch 44 — the file saves

- **A `.blend` writer that is the reader run backwards** — phiphx keeps
  every block of the opened file in memory, so `BlendFile::to_bytes`
  writes the header, every block in the layout the file header calls for
  (`BHead4`, `SmallBHead8`, `LargeBHead8`) and `ENDB`, and a file nothing
  touched comes back **byte for byte** — checked on the 3.x THYMUS file
  and the 5.0 fixture; a zstd- or gzip-wrapped file is re-wrapped the
  same way (`blend/write/mod.rs`, `tests/save_truth.rs`). Blender's reader
  resolves pointers through the saved addresses and never reads a block
  nothing points at, which is what lets a save be the opened file with
  the changes written through it rather than a second writer of every
  struct Blender has.
- **Fields patched by DNA** (`write/field.rs`) — `set_int` / `set_text` /
  `set_ptr` at the offset and width the file's own DNA gives, dotted paths
  walking embedded structs (`Scene.r.cfra`), `set_id_name` for the
  two-letter-coded `ID.name`, `push_block` for a new `DATA` block with a
  fresh address, `remove_blocks`. On the way a reader bug: `int_field`
  returned 0 for `char`-wide fields, which is what `Collection.flag` is —
  so the collection's camera and arrow toggles now read as well as write.
- **What a save writes through** (`write/edit.rs`, `BlendEdits`): the
  frame and range (`Scene.r.cfra / sfra / efra`), each object's toggles
  (`visibility_flag` — the DNA member is spelt `restrictflag` in every
  file so far, so the writer takes whichever the file has), its name, its
  eye and selection (`Base.flag`), the active object (`ViewLayer.basact`),
  the collections' eye and toggles (`LayerCollection.flag`,
  `Collection.flag`), and the marker list rebuilt over the file's own
  `TimeMarker` blocks — kept, added, dropped, relinked in frame order
  (`write/marker.rs`). Everything else in the file is untouched:
  vertex counts and world matrices compared before and after.
- **Save in phiber** — **Ctrl S** writes the opened file (the file before
  the save kept as `file.blend1`, Blender's `use_save_versions`);
  **Shift Ctrl S** opens the browser on the current name and folder and
  Save As moves the document to the new path; **Save Copy** writes
  without moving; **Ctrl Alt S** Save Incremental writes the next
  numbered name (`cube.blend` → `cube1.blend`, `shot_010` → `shot_011`)
  and follows it. The Info editor gets Blender's `Saved "cube.blend"`
  line; what the writer cannot make yet — an object added, duplicated or
  deleted, which needs new `OB` / `ME` blocks — is a `Warning:` line per
  kind, never a silent drop (`scene/save.rs`, `app/op/file.rs`,
  `tests/save_writes_the_file.rs`). A scene that was not opened from a
  `.blend` (the default scene, New File) has nothing to write through and
  says so.
- Also: the rename popup's OK button left the caret in its hidden field,
  so the next Ctrl S was typed into it; closing the popup now takes the
  caret with it.

## Batch 45 — new objects save too

- **An object cloned, pointing nowhere** (`write/object.rs`) — rather
  than spell every default of a struct with a hundred fields per Blender
  version, a new object is a copy of one the file already has with every
  pointer and every `ListBase` zeroed, walked from the DNA
  (`pointer_spans`: `*next`, `(*func)()`, `ListBase`, embedded structs
  such as `ID` included), so a field added in a later Blender is cleared
  too and no data block is ever shared with the template. What stays are
  the plain values a fresh object shares with its neighbours — draw type,
  colour, rotation mode. Then its own: name, mesh, transform, identity
  `parentinv` / `constinv`.
- **A mesh in the file's own layout** (`write/mesh.rs`) — Blender 5.0
  keeps a mesh's attributes in `Mesh.attribute_storage` (`Attribute` table
  → `AttributeArray` → raw block per attribute); 3.6 – 4.x keep them as
  `CustomData` layers in `vdata / edata / pdata / ldata`, with 4.0 turning
  the edge list from `MEdge` structs into an `.edge_verts` int pair. The
  writer produces whichever the file's version calls for — `position`,
  `.edge_verts` / `MEdge`, `.corner_vert`, `.corner_edge`, `sharp_face`
  (a fresh primitive is flat), `poly_offset_indices`, the counts — and
  refuses a file older than 3.6 with a warning naming the version. The
  layouts were read off the fixtures' DNA rather than assumed
  (`AttrType`: Bool 0, Int32 3, Int32_2D 4, Float3 7; `eCustomDataType`:
  `CD_MEDGE` 3, `CD_PROP_INT32` 11, `CD_PROP_INT32_2D` 46,
  `CD_PROP_FLOAT3` 48, `CD_PROP_BOOL` 50).
- **Into the scene, and out** (`write/link.rs`) — a `CollectionObject`
  appended to the collection's `gobject` list (the master collection's
  nodes sit in the scene's data, a `GR` collection's in its own) and a
  `Base` appended to the view layer's `object_bases`; a deleted object has
  its links taken out of every list, its `OB` block and the data after it
  dropped, and its mesh too when no other object uses it.
- **`BlendEdits` gained `added`, `duplicated`, `deleted`** — the
  structural edits go first (they move blocks), then the field patches
  re-find their blocks. A duplicate is a linked copy of its source (the
  same mesh, in place); the active object may be one the save just added.
- **In phiber** (`scene/save.rs`) — an added primitive is written with the
  mesh the viewport draws (triangulated, as the spatial primitives are); a
  duplicate of a file object is a linked copy, a duplicate of an added
  primitive is that primitive again; deletions go by file name. The
  `Warning:` lines for new objects are gone; the one left is for a file
  older than 3.6.
- Two reader fixes on the way: `int_field` read `char`-wide fields as 0
  (batch 44's `Collection.flag`), and a 3.6-native mesh (`position` +
  `poly_offset_indices`, no `MPoly`) was chunked into triangles because
  the 3.5 fallback never looked at `poly_offset_indices`.
- Truth: `tests/save_truth.rs` adds a cube, duplicates the first object
  and deletes the last in the 3.6 and 5.0 fixtures; the saved file reads
  back with 8 vertices / 6 quads at its location, selected and active,
  the duplicate sharing its source's mesh, every other object unchanged.
  `tests/save_writes_the_file.rs` does the same through Ctrl S.

## Batch 46 — a new file saves, quads, the exclude box, markers move

- **New File is a real scene** (`scene/template.rs`,
  `assets/blend/startup.blend`) — Blender's New ▸ General opens
  `startup.blend` (`wm_homefile_read`); phiber now ships one: a 90 KB
  Blender 5.0 file with the Cube, the Light and the Camera, frames 1–250,
  made by `save_truth.rs::make_startup_template` (the MATERIALS-BSDF
  fixture with everything else deleted through the writer), embedded with
  `include_bytes!` and written to the temp folder when it differs. New
  File imports it like any `.blend`, then marks the document *untitled*
  (`BPhiberApp.untitled`: the loaded path is dropped once the model is
  cached), so **Ctrl S on it opens Save As** — `wm_save_mainfile` with no
  `G.main->filepath` — and Save As writes the template plus the edits to
  the chosen path, which becomes the document's.
- **Quads on save** (`write/mesh.rs::BlendNewMesh::from_triangles`) — the
  viewport's primitives are triangle lists; the writer now welds
  vertices by position and pairs coplanar (normal dot ≥ 0.9999), convex
  triangles across their shared edge into quads, so a saved cube is
  8 vertices / 6 faces, as `primitive_cube_add` makes it. A pair that
  bends stays two triangles.
- **The exclude checkbox** (`part/outliner/mod.rs::exclude_box`,
  `BView::TreeRow.lead`) — collection rows carry Blender's checkbox
  before the icon (`outliner_draw.cc`: `LAYER_COLLECTION_EXCLUDE` drawn
  as `ICON_CHECKBOX_HLT / DEHLT`); a tree row can now hold *lead* views
  between the disclosure and the icon. Unchecking excludes the collection
  from the view layer — its objects vanish from the viewport and the
  Outliner's counts (`layer::hidden_by_collections` honours exclude), and
  the bit is written back on save (`BlendCollectionEdit.exclude`).
  Seeding reads the file's exclude separately from *hide in viewport*.
- **Markers select and move** (`scene/marker.rs`,
  `canvas/time/marker/`) — a press in the marker band selects the marker
  under it (`ed_marker_select`: Shift extends, the band's empty part
  deselects), a drag begun on one moves it frame by frame
  (`ed_marker_move`, refused onto a taken frame), and Delete takes the
  selected markers, falling back to the playhead's. The selection lives
  in `ctx.markers.selected`; selected glyphs paint in
  `TH_TIME_MARKER_LINE_SELECTED` / `ICON_MARKER_HLT` / `TH_TEXT_HI`. A
  press above the band still scrubs.
- **Drag start under gpui** — gpui hands `on_drag` the element-relative
  offset at the first move past its threshold, and the press's re-render
  has already replaced the element by then. Box select now uses that
  offset directly (its start was offset by the window's origin); the
  marker drag remembers what the press hit in a `thread_local` that
  outlives the element.
- Truth: `tests/save_writes_the_file.rs` (New File → Save As → the cube,
  light and camera plus the added object read back),
  `write/mesh.rs` (the 12-triangle cube welds to 6 quads),
  `tests/outliner_view_layer.rs` (unchecking THYMUS-GRP hides
  `head_base_geo`), `tests/markers_select_and_move.rs` (select, Shift,
  deselect, drag 60 → 90, scrub above the band).

## Batch 47 — a startup file per Blender version, unsaved changes, the Playback panel

- **Three factory startups, matched by version** (`scene/template.rs`,
  `assets/blend/startup-{3,4,5}.blend`) — a `.blend`'s layout goes with
  its version, so the file New File saves through is now the layout of the
  Blender it will be opened in. Each template is Blender's own factory
  scene written by that Blender: 4.2 LTS and 5.0 from `bpy`
  (`assets/blend/make_startup.py`, `wm.read_factory_settings` →
  `wm.save_as_mainfile`), 3.6 LTS from Blender's repository
  (`release/datafiles/startup.blend` at `v3.6.23`, zstd-wrapped; PyPI no
  longer carries a 3.x `bpy`). `BTemplate::for_major` matches 3 and below
  → 3.6, 4 → 4.2, 5 and up → 5.0; New File takes the open document's
  version (a 3.6 pipeline stays 3.6), or the major the operator names
  (`ctx.op.new_file = Layout:4`), else the newest. The status bar says
  which: "New file (Layout, Blender 3.6 layout)". The old
  `startup.blend` (a cut-down fixture) is retired.
- **Unsaved changes** (`app/op/close.rs`, `modal/file_close/`) —
  Blender's `wm->file_saved`: an action that records an undo step marks
  the document (`WM_file_tag_modified`), reading or writing it clears the
  mark (a Save Copy does not); the window title carries it
  (`wm_window_title`: `* cube [C:\…\cube.blend] - Phiber`, `(Unsaved) -
  Phiber` for a document never saved, and the platform's edited dot). Quit,
  New File and Open — the browser, and a Recent entry — first ask "Save
  changes before closing?" with the file's name and Save / Don't Save /
  Cancel in the platform's order (`block_create__close_file_dialog`);
  Save writes then goes on (an untitled document asks where and stops
  there, as Blender does), Don't Save goes on, Cancel stays. The action
  waits in `ctx.modal.file_close.then` and runs once answered.
- **The Playback panel** (`part/playback/{panel,settings}.rs`) — the
  Timeline's Playback entry is `TIME_PT_playback` now, a panel popover
  rather than a menu: Sync (Play Every Frame / Frame Dropping / Sync to
  Audio), Audio (Scrubbing, Play Audio), Playback (Limit to Frame Range,
  Allow Preroll, Follow Current Frame, Loop: Infinite / Stop at End Frame
  / Stop at Start Frame / Restore Frame / Bounce), Play In (the nine
  editors), Show (Subframes), then Set Start / End Frame — laid out with
  `use_property_split`. It stays open while its boxes are ticked
  (`MenuKey::Panel`, its own key outside the menu sweep) and closes when a
  menu opens. Every setting is read from the file (`Scene.audio.flag`,
  `Scene.flag`, `Scene.r.flag`, `Scene.playback_loop_mode`, the active
  screen's `bScreen.redraws_flag`; phiphx `scene/playback.rs`) and written
  back on save (`write/playback.rs`, `BlendEdits.playback`). Two of them
  drive the app: **Limit to Frame Range** clamps a scrub into the (preview)
  range (`anim.change_frame` with `SCER_LOCK_FRAME_SELECTION`), and the
  **loop mode** decides what the ticker does at the end of the range
  (`screen_animation_step`: round again, stop at the end or the start, back
  to where playback began, or turn round).
- Truth: `scene/template.rs` (each template parses as its version with
  Camera / Cube / Light, 1–250), `phiphx/tests/save_truth.rs` (a cube
  added to each template saves through in that layout),
  `tests/save_writes_the_file.rs` (a 3.6 document's New File saves a 3.6
  file; `Layout:5` wins over what is open), `tests/unsaved_changes_ask.rs`
  (the title's `* `, Cancel / Save / Don't Save on Quit, Open and New
  File, an untitled Save asks where), `tests/playback_popover.rs` (the
  rows, a box ticked keeps the panel, a menu closes it; the settings read
  from a file and written back), `part/time/step.rs` (the five loop modes,
  the frame lock), `write/playback.rs` (round trip in the 3.6 and 5.x
  layouts).

## Batch 48 — one seam, one cap, one grid line, one solid shade

The user's list, each item taken to one owner rather than a copy per part:

- **The resize seam** (`view/draw/resize.rs::BSeamLine`) — one struct
  draws the line for every boundary: a zone's, an area's, a panel's. It is
  `gap − 3` thick at rest in the theme's border shade and swells to
  `gap + 2` in the accent under the pointer (`BResizeStyle::rest_thickness
  / hover_thickness`; the two offsets are the Resize preferences "Seam
  Under Gap at Rest" and "Seam Over Gap on Hover"), clamped to the hit
  band where it is drawn, so no theme field bends another. The three
  seam painters that each had a line of their own are gone.
- **Menu buttons** (`view/draw/anchor.rs`) — every menubar anchor, in an
  editor's header as in the top bar, is a cap cut into its strip
  (`BCapSide::Below`: top corners rounded, bottom flush with the header's
  rule the popover drops from) — the one corner rule the top bar's tabs
  already used.
- **Modal headings** (`view/draw/modal/part.rs`) — a dialog's title bar
  is a header strip: the header height, `pad_md` sides and `gap_xs`
  between its icon, title and controls, as an area's header lays out.
- **The file browser's refresh** (`assets/icons/refresh.svg`) — a
  clockwise circular arrow (Blender's `FILE_REFRESH`) in place of the
  sideways arc.
- **Outliner rows** (`view/draw/tree/{mod,guide}.rs`, `column.rs`) — the
  tree is a table with indentation: a row's depth is a run of indent units
  before its arrow slot, which every row keeps (a leaf's blank, as
  Blender reserves it); the hierarchy lines are drawn once behind the
  column, one per indent unit of the deepest row, the whole column down,
  in the theme's opposite shade (`text_dim`: light on dark, dark on
  light). No row draws a line of its own any more.
- **Solid shading** (`theme/semantic/set.rs::solid_shade`,
  `scene/color.rs`) — every mesh, a primitive or a loaded model, takes the
  theme's one solid shade (two steps off its dim text, Blender's Solid
  with `color_type = SINGLE`); no object brings a grey of its own.
  Wireframe keeps the material's viewport colour.
- **The grid** (`theme/style/anim.rs::grid_line`, `semantic/stroke.rs`)
  — the 3D floor and the UV editor's square draw the same line: the anim
  canvases' grid shade lifted a tenth, a grid stroke wide
  (`draw_image_grid`). The floor's own alpha-blended grid colour is gone.
- Truth: `visual_surface_refinement` (the seam rule per palette, the
  shared line in every seam module, the floor drawing the UV grid line),
  `outliner_depth` (opposite-shade guides behind the column, indentation
  before the arrow), `material_textures` (Solid has no colour of the
  object's own), `layout_gutters` (the seam's resting thickness on
  screen), the resize property tests (no field bends another).

## Batch 49 — the cap rule everywhere, no recessed strips

- **Menu rows** (`view/draw/menu.rs`) round as their anchor does — top
  corners only, through the same `BCapSide::Below` — so dropdowns, menu
  items and tabs are one shape. A dropdown control already draws its
  anchor through `anchor.rs`, so it took the rule in batch 48.
- **Sub-panels** (`theme/semantic/mod.rs`) share their panel's shade at
  every depth (Blender's panel and sub-panel are both `#3D3D3D`); the
  step-back-per-level that drew darker strips through the Properties
  editor is gone. Their headings stay a step over, as a panel's.
- Checked against the layout golden: the file browser dialog's sidebar
  card heads and its column heading are the same heading surface as the
  spaces' cards — the dialog is built by the same zone builder as a
  space, with no fills of its own. If a strip still reads dark on your
  machine, it is the theme's `bg_header` (the header/footer tone), which
  the modal title bar shares with every area header.

## Batch 50 — Undo History, badge tooltips

- **Edit ▸ Undo History** (`app/undo.rs`, `menubar/tree`) — Blender's
  `TOPBAR_MT_undo_history`: every step carries the name the status bar
  gave its action ("Frame 40", "Added Cube.001"); the submenu lists
  "Original" and each step oldest first, the undone ones too, with the
  current one checked, and picking an entry undoes or redoes to it
  (`ed.undo_history`, `ctx.op.undo_history = <index>`). The app publishes
  the listing for the render pass, since the menu is built from the state
  and the history is the app's.
- **Count badges** (`view/draw/badge.rs`) name what they hold on hover —
  "Mesh × 3" — as Blender's icon-row tooltip does.
- Truth: `tests/undo_history.rs`, `app/undo.rs`.

## Batch 51 — keys go where the pointer is; G and F2 on markers

- **Pointer-routed keys** (`view/draw/host.rs::BPointer`,
  `app/keymap`) — Blender sends a key to the keymap of the region under
  the pointer first (`wm_event_do_handlers`). Every host now records
  itself as the pointer moves over it, and the keymap tries that editor's
  menu shortcuts first, then the screen-wide keys (Space, arrows, Escape),
  then every other menu in `PRIORITY` order. So a key two editors share
  does what the editor under the pointer does — `Home` over the Timeline
  frames the Timeline, `F2` over it renames a marker — and with the pointer
  nowhere in particular the old order holds.
- **G grabs the selected markers** (`scene/grab.rs::BMarkerGrab`,
  `marker.move`) — they follow the pointer's frame from where it stood
  when the grab began (every move measured from the start, so nothing
  drifts), a move that would stack one on an unselected marker is refused,
  a click keeps them, Escape or the right button puts them back, and the
  whole grab is **one undo step** (`BHistory::record_held` keeps the
  document from before the gesture while it runs). The confirming click
  does not scrub.
- **F2 renames a marker** (`modal/rename`, `marker.rename`) — over the
  Timeline the rename dialog opens on the selected marker (or the one at
  the playhead), titled "Rename Marker" with the marker glyph; Return
  renames it. Over any other editor F2 is still Rename Active Item.
- Truth: `tests/marker_keys_over_the_timeline.rs` (G, a ten-frame move,
  Escape, G again, a click, one undo step, F2 and a rename),
  `scene/grab.rs` (origin, refusal, cancel, confirm), `app/keymap/set.rs`
  (the editor under the pointer wins a shared key; one it does not bind
  falls through).

## Batch 52 — one layout for dialogs and areas, one corner rule, light lines

Decided with the user: these follow our unified design rather than
Blender's own choices.

- **The file browser's gutters** — its source column drew a framed panel
  round its cards (a box in a box) and the two columns padded their shared
  edge twice (16 px). Now the cards sit directly in the Tools region, one
  `pad_md` in from the area, and the source cards and the file list are one
  `pad_md` apart; the path strip's left pane follows. A test lays the File
  Browser out in the dialog **and** in a workspace area and asserts the
  same geometry (`file_browser_layout.rs`): a dialog raises the same host
  through the same zone builder, with no spacing of its own.
- **One corner rule** (`tab/cap.rs::BCapSide::of`) — only the caps cut into
  a header strip, its menus and tabs, round the top alone; every other
  control — a row in a menu, a dropdown in a panel, a dropdown inside a
  popover — rounds all four corners. A popover's body belongs to no region
  (`BRenderCtx::in_popover`), so nothing inside a header's menu mistakes
  itself for a header cap.
- **Menus close when the pointer leaves** (`popover/leave.rs`) — once the
  pointer is more than 64 px from an open popover and off its anchor's row
  (where the menubar sweeps from menu to menu), it closes.
- **Submenus** (`popover/place.rs`) open one gap clear of the parent's
  frame, their first row level with the row that opened them.
- **No dark lines** — `sep` (menu separators, sub-panel guides, the host
  switcher's divider) was the app shade, darker than every surface it lay
  on; it is now one step over a card. Borders were already the outline
  step. No line is darker than what it lies on.
- **Outliner rows look like Properties cards** — a row that holds others
  paints the card heading surface, a leaf the card body, each with the
  theme's sweep; every row carries an arrow (a leaf's dimmed and inert);
  the guide lines are the opposite shade (batch 48).
- **The activity rail** carries the same sweep as the headers and cards
  (`host_body_surface`).

## Batch 53 — box select on markers, the transport's two popovers

- **Box select over the marker band** (`scene/grab.rs::BMarkerBox`,
  `marker.select_box`) — a drag begun on the band's empty part sweeps a
  box; the markers between its ends are selected live, on top of what was
  selected when it began (a press without Shift clears that first, so
  Shift extends). The box is painted in the selection shade under the
  glyphs while the button is down, and is not a scrub.
- **One owner for the window's gestures** (`canvas/time/gesture.rs::
  BTimeGesture`, `BTimeDrag`) — scrub, marker select, marker drag, box and
  grab were closures in the renderer; they are one struct's methods now,
  and what a drag began on is an enum (`Scrub / Marker / Box`).
- **The transport's popovers** (`part/playback`) — as `playback_controls`
  draws them: **Auto Keyframing** after the record button (Mode: Add &
  Replace / Replace; Only Active Keying Set) and **Time Jump** after the
  jump arrows (Jump Unit: Frame / Second; Delta). The jump arrows now move
  by the popover's delta in its unit (`settings::time_jump_frames`), a
  second by default. Both are panel popovers: a box ticked keeps them open,
  a menu opening or the pointer leaving closes them.
- The two settings are read from and saved to the file since batch 54. The
  Keying popover (`TIME_PT_keyframing_settings`) belongs to the Dope Sheet
  and Graph headers, which Blender leaves out of the Timeline.
- Truth: `tests/marker_box_and_transport_popovers.rs`, `scene/grab.rs`,
  `part/playback/settings.rs`.

## Batch 124 — Backface Culling and Specular Lighting

- **Backface Culling** (`V3D_SHADING_BACKFACE_CULLING`, Workbench's
  `DRW_STATE_CULL_BACK`): a face wound away from the viewer is not drawn.
  In `solid.wgsl` the primary hit — rasterized or traced — is tested by
  its geometric normal against the ray and, when turned away, the ray goes
  on past it (as X-ray steps through layers) to what is behind, up to 32
  faces; the sky where nothing is. Solid's opaque pass only.
- **Specular Lighting** (`V3D_SHADING_SPECULAR_HIGHLIGHT`,
  `WorldData.use_specular`): off, `get_world_lighting` lights the base
  colour diffusely with no highlight and no metallic split — now
  `workbench.wgsl`'s branch. Studio and MatCap; Flat has no lighting to
  take it from.
- **Camera words**: the two spare words of `Camera.shade` — `[…, cull,
  no_specular]`, each the flag inverted where Blender's default sets it,
  so a zeroed camera draws as before (`BSolidFx::shade_words` over
  `BSolidShade`'s light and colour words).
- **From the file**: `View3DShading.flag` bits 10 and 4 (phiphx
  `SceneSolidFx.backface_culling`, `.specular`; off / on in the startup
  file), seeded on open.
- **The popover**: Options leads with Backface Culling as Blender's does
  and ends with Specular Lighting when the lighting is Studio or MatCap
  (`VIEW3D_PT_shading_options`); Wireframe has neither.
- Tests: phiray `tests/shade.rs` (the walls seen from behind draw their
  backs, culled show the sky, nothing culled from the front; specular off
  changes the lit wall and adds no light); phiphx `view_solid_fx.rs`;
  phiber `solid_fx_tests.rs` (a file's flags and the words, the factory
  as a zeroed camera), `filter/shading.rs` (Options 13 rows, 12 under
  Flat).
- **Suites:** see the head of this file.
- **Not yet:** `OB_DRAW_NO_SHADOW_CAST`; `Scene.display.shading` for a
  Workbench F12; Depth of Field; matcaps and the studio-light picker
  (and its `has_specular_highlight_pass` gating); Attribute and Texture
  colour.

## Batch 123 — Shadow: Workbench's one directional light, cast by the kernel

- **The effect** (`workbench_shadow.cc`, `get_shadow` in
  `workbench_world_light_lib.glsl`): Workbench draws stencil shadow volumes
  from every opaque object along `Scene.display.light_direction` and the
  composite multiplies each pixel by `smoothstep(shadow_shift, shadow_focus,
  −N · direction) · shadow_intensity + (1 − shadow_intensity)`, the step
  forced to 0 where the stencil covers the point. In a ray tracer the
  volumes are one shadow ray: `solid.wgsl`'s `shadow_factor` traces from the
  hit against the light's travel (`trace(…, true)`, any hit) and takes the
  same terminator — the direction in view space (`shadow_direction_vs`) by
  the camera's axes, since the normal is stored there.
- **The direction** is `ShadowPass::init`'s: the panel's `(x, y, z)` with y
  and z swapped and x and z negated, unit (`BSolidFx::shadow_words`; the
  default (1, 1, 1)/√3 becomes (−1, 1, −1)/√3 — down, toward +y, toward
  −x). `shadow_focus` is `1 − clamp(focus, 0.0001, 0.99999)(1 − shift)`,
  held a hair above the shift so the step keeps a width (`shadow_fade_words`).
- **Lighting modes**: applied after Studio / MatCap / Flat alike, as the
  composite has it; X-ray's transparent pass takes none. The shadow sits
  in the colour the effects pass reads, so cavity and curvature multiply
  over it as Blender's composite line does.
- **From the file**: `View3DShading.flag`'s `V3D_SHADING_SHADOW` (bit 2)
  and `shadow_intensity` (phiphx `SceneSolidFx.shadow`, `.shadow_intensity`,
  0.5 by default); `Scene.display.light_direction[3]`, `shadow_shift`,
  `shadow_focus` (`SceneRender.shadow`: `SceneShadow`, (1/√3)³ / 0.1 / 0),
  seeded on open beside the view's shading keys.
- **The popover**: Options gains Shadow (switch and intensity slider) and
  an inline **Shadow Settings** group — Direction X / Y / Z, Offset, Focus
  — for Blender's gear popover `VIEW3D_PT_shading_options_shadow`; Solid
  only (`shading.type == 'SOLID'`).
- **Camera words**: `Camera.shadow = [dir_ws, intensity (0 off)]`,
  `Camera.shadow_fade = [shift, focus', _, _]`.
- Tests: phiray `tests/shadow.rs` (a block out of a wall under Flat
  lighting with the light slanting to +x: the wall at x in ½ ..= 1½ falls
  to `1 − intensity` — black at 1 — and the open wall keeps the
  terminator's 0.751 · intensity + (1 − intensity) to the display level,
  the far side and the block's face lit like the open wall, the extent
  pixel-exact within the anti-aliasing jitter; off, the frame of before);
  phiphx `view_solid_fx.rs` (the startup file's switch off, 0.5,
  (1/√3)³, 0.1, 0); phiber `solid_fx_tests.rs` (the words, the panel's +y
  as the world's −z, `1 − 0.5 · 0.7`, a shift of 1 leaves a width),
  `filter/shading.rs` (Options is 11 rows in Solid, 4 in Wireframe).
- Also: `solid_fx.rs` had reached the 240-line read limit, so its tests
  moved beside it (`solid_fx_tests.rs`, as `cursor_tests.rs` is).
- **Suites:** see the head of this file.
- **Not yet:** `OB_DRAW_NO_SHADOW_CAST` (every object casts here);
  `Scene.display.shading` for a Workbench F12; Depth of Field, Specular
  Lighting, Backface Culling, matcaps and the studio-light picker,
  Attribute and Texture colour.

## Batch 122 — Cavity ▸ World: Workbench's screen-space ambient occlusion

- **The effect** (`workbench_cavity.bsl.hh`'s `cavity_compute`, ported to
  `fx.wgsl` as `cavity`): the Alchemy screen-space ambient obscurance.
  For every opaque pixel, `matcap_ssao_samples` points of a Hammersley disk
  (`CavityEffect::load_samples_buf`: `BLI_hammersley_1d` — the radical
  inverse, `reverseBits` on the GPU — for the angle, `fmod((i + 0.5 +
  it_add) / samples, 1)` for the radius, more of them near the centre),
  turned by the pixel's blue noise and slid along their direction
  (`SceneResources::load_jitter_tx`), `matcap_ssao_distance` wide at the
  pixel's depth (Blender's `winmat` over `homcoord`, here the frame's view
  span `1 / scale`, out to the depth under perspective). Each neighbour's
  view position against the pixel's normal: a *cavity* where it stands
  ahead of the surface, an *edge* where it falls behind, over
  `1 / (len (1 + len² attenuation))` with the `0.05 len + 0.0001` bias, the
  background a `cavity_distance` further back. Then `(1 − cavities ·
  valley) (1 + edges · ridge) (1 + curvature)` clamped to 4 — the
  composite's line, the curvature of batch 120 beside it.
- **The jitter**: EEVEE's 64 × 64 blue-noise table (`eevee_lut.cc`,
  `blue_noise[64][64][4]`, its first two channels — all `load_jitter_tx`
  reads) is `phiray/trace/blue_noise.rs`, generated from the source, and
  bound to the effects pass as `[cos φ, sin φ, bn₁]` a texel; the slide
  `(bn₁ − 0.5) / sample count` the kernel derives, since the count is a
  per-frame word. Texel = the frame pixel mod 64, GL's rows.
- **The runs**: as `CavityEffect::init` — `sample_count = min(samples_len ·
  ssao_samples, 512)`, each anti-aliasing sample draws its own run
  (`p.frame % (sample_count / ssao_samples)`), so eight TAA samples cover
  128 disk points. Deterministic frame to frame.
- **Depth**: the opaque pass now also stores the hit's view z (word 20n:
  Blender units, negative ahead, 0 for the background) — Workbench's depth
  buffer, which `drw_point_screen_to_view` is replayed from
  (`view_pos`: the canvas point `solid.wgsl`'s `main` shoots through,
  scaled to the depth under perspective). `fx.wgsl` decodes the full view
  normal from the stored two (`normal_decode`).
- **From the file**: `Scene.display.matcap_ssao_samples / _distance /
  _attenuation` (`SceneDisplay`, phiphx `SceneRender.ssao`:
  `SceneSsao { samples, distance, attenuation }`, 16 / 0.2 / 1 by
  default) seeded on open — scene settings, kept beside the view's shading
  keys (`view3d.shade.matcap_ssao_*`) since the popover edits them there.
- **The popover** (`part/filter/shading.rs`): for World and Both, after
  World Space Ridge / Valley, an inline **SSAO Settings** group — Samples
  (1 ..= 500), Distance, Attenuation — standing in for Blender's gear
  popover `VIEW3D_PT_shading_options_ssao`.
- **Camera words**: `Camera.cavity = [valley, ridge, attenuation,
  distance]`, `Camera.ssao = [samples (0 off), samples_len, _, _]`
  (`BSolidFx::cavity_words`, `::ssao_words`); the effects pass also runs
  when SSAO alone is on (`Tracer::effects`).
- Tests: phiray `tests/cavity.rs` (a block standing out of a wall under
  Flat lighting: the wall beside it darkens, the block's rim brightens, the
  open wall is untouched, deterministic; off or at zero distance nothing
  changes), `blue_noise.rs` (the table's first texels, uniform, the words a
  unit rotation); phiphx `view_solid_fx.rs` (the startup file's 16 / 0.2 /
  1); phiber `solid_fx.rs` (`ssao()`, the words, a file's settings, off
  and Screen give no samples), `filter/shading.rs` (World 14 rows, Both
  16).
- **Suites:** see the head of this file.
- **Not yet:** the Workbench *render* engine's `Scene.display.shading`
  (F12 draws no effects), Shadow, Depth of Field, Specular Lighting,
  Backface Culling, matcaps and the studio-light picker, Attribute and
  Texture colour.

## Batch 121 — Lighting, Color and Background in the Shading popover, drawn by the kernel

- **Color ▸ Object / Random / Single** (`workbench_engine.cc`'s
  `get_material`, `workbench_materials.cc`): Solid shades every face of
  an object with the object's own colour (`Object.col`, now read by
  phiphx into `SceneObject::color` and `BObject3D::Loaded::color`, white
  by default), with a hue hashed from its id name, or with the one
  colour — each with `pack_data(0.0, 0.4, …)`: no metal, roughness 0.4.
  Random is Blender's hash to the bit: `BLI_ghashutil_strhash_p_murmur`
  (MurmurHash2 over `"OB" + name + NUL`, seed 0) then `BLI_hash_int_01`
  (Jenkins' `final` mix of `BLI_hash_int_2d(k, 0)`) as the hue of HSV
  (h, 0.5, 0.8) — the CPU hashes the name (`phiray/trace/hash.rs`), the
  instance record carries the colour and the hash (words 28–31), and the
  kernel turns the hash into the colour (`random_color`, `solid.wgsl`),
  so a moved object keeps its colour and nothing is rebuilt. Material (the
  default) is unchanged; Attribute and Texture draw as Material for now
  and the popover says so.
- **Lighting ▸ Flat** (`WORKBENCH_LIGHTING_FLAT`): the colour itself,
  unlit. Studio is what it was; MatCap draws as Studio until matcap
  images are in, and the popover says so. `Camera.shade` carries the
  light and colour words, `Camera.single` the single colour.
- **Background ▸ Theme / World / Viewport**
  (`ED_view3d_background_color_get`): behind Solid and Wireframe the
  world's horizon colour (`World.horr`, display-encoded) or the chosen
  colour replaces the theme's sky and its sweep; Theme is what it was.
- **From the file:** `View3DShading.light`, `color_type`, `single_color`,
  `background_type`, `background_color` (`phiphx::SceneSolidShade`, with
  Blender's numbering — Flat is 0, Studio 1, MatCap 2), seeded as the
  view opens, read by `BSolidShade` (`scene/solid_shade.rs`).
- **The popover** (`part/filter/shading.rs`) gains Lighting (Solid) and
  Color — the type, the single colour's swatch when Single, Background
  with its swatch when Viewport — before Options and Cavity, following
  `VIEW3D_PT_shading_lighting` / `_color`'s polls.
- Tests: phiray `tests/shade.rs` (Object red/blue by instance, Random two
  hues whose sextants match `hash::hue_of`, Single one green, the sky
  untouched; Flat is the colour exactly and unshaded), `hash.rs` (the
  id-name hash, the mix, the hue in range and apart for neighbours);
  phiphx `view_solid_fx.rs` (the startup file: Studio, Material, 0.8
  grey, Theme, 0.05 grey); phiber `solid_shade.rs` (factory, a file's
  values, the camera words, the background per type),
  `filter/shading.rs` (rows: Solid 11, Both 13, Single + Viewport 15,
  MatCap 16, Wireframe 6).
- Also: `examples/camera_render.rs` had stopped compiling (an older
  `view_words` signature, a missing `grid_hide`) — fixed while the
  examples were open.
- **Suites:** see the head of this file.
- **Not yet:** MatCap images and the studio-light picker, world-space
  lighting and its rotation, Color ▸ Attribute (vertex colours) and ▸
  Texture (the active image in Solid), the wireframe colour type, Object
  colour's alpha (`ob.color[3]`, Blender's transparency in Object mode),
  Backface Culling, Specular Lighting, Shadow, Depth of Field; the
  Workbench *render* engine's own `Scene.display.shading`.

## Batch 120 — the Shading popover

- **`VIEW3D_PT_shading`**, the chevron after the shading radio
  (`part/filter/shading.rs`), with the panels Blender hangs under it whose
  switches the kernel honours, following the shading in use as Blender's
  `poll`s do: **Options** (`VIEW3D_PT_shading_options`) for Solid and
  Wireframe — Outline and its colour swatch, X-Ray and its alpha slider
  (Wireframe's own pair, as Blender's) — and **Cavity**
  (`VIEW3D_PT_shading_cavity`) for Solid — the switch, Type (World /
  Screen / Both, `rna_space.cc`'s items), then World Space Ridge / Valley
  and Screen Space Ridge / Valley as the type asks, `0 ..= 2.5`. Every
  row keys on what the canvas reads (`view3d.shade.<rna>`, the X-ray
  keys), so a box, the file's setting and `fx.wgsl` are one value. The
  outline colour is kept as a `BValue::Color` now, so the swatch edits it.
  Material Preview and Rendered say in one line that Lighting and Color
  are not here yet rather than offer boxes that do nothing.
- Tests: `filter/shading.rs` (the rows per shading: Solid 8, Both 10,
  Wireframe 4, Material 1; the boxes' keys are `BSolidFx`'s).
- **`phiber_color.png` was a dark mirror on a white sweep — fixed.** The
  side-by-side's right half had every surface a glossy bronze and the
  background fading from white: the `color_render` example builds its
  camera by hand, and since batch 108 Material Preview lights from the
  studio image's irradiance and prefiltered copies through
  `Camera::preview` — words only `lit` set (the viewport and the render
  job), so the example's diffuse came from nothing and its reflections
  from the raw image. `still` now gives a hand-built Material Preview
  camera the studio lighting itself (`world::studio_lighting`, shared with
  `lit`; `still::with_studio`, tested). The sweep: the examples still said
  `sky.a = 1`, from before `a` became the theme's left-edge lift — they
  say 0 now, the viewport grey. Female-01 through the fixed example in
  the container matches `blender_color.png`: matte skin, the pink top and
  pants, the magenta lips of the one missing texture, the grey behind.
  Still to look at against the reference: the pants read a touch warmer
  (peach against Blender's pale pink).
- **Suites:** see the head of this file.
- **Not yet, in Blender's order:** Lighting (Studio / MatCap / Flat, the
  studio-light picker, world-space lighting and its rotation), Color
  (Material / Object / Attribute / Single / Random / Texture, the
  wireframe colour type, Background: Theme / World / Viewport), Backface
  Culling, Specular Lighting, Shadow and its settings, Depth of Field,
  Cavity ▸ World's SSAO settings, Material Preview's and Rendered's
  Lighting rows — each with the kernel work behind it.

## Batch 119 — the object outline and the curvature effect, as Workbench draws them

- **The object outline** (`workbench_effect_outline.cc`,
  `workbench_effect_outline_frag.glsl`): every object is drawn into an
  object-id buffer (the instance plus one, the background 0 — Workbench's
  `object_id`), and a pixel whose four neighbours (`U.pixelsize` away)
  belong to another object takes `object_outline_color` at a quarter per
  such neighbour, blended premultiplied over the composite — so the line
  straddles the edge, half on the object and half on what is behind it,
  and the eight anti-aliasing samples soften it as Blender's do. On by
  default, black, as the startup file has it (`V3D_SHADING_OBJECT_OUTLINE`).
- **The curvature effect** (`workbench_curvature.bsl.hh`,
  `curvature_compute`): Cavity ▸ Screen (or Both) brightens ridges and
  darkens valleys from the slope of the view-space normals across the
  pixel's neighbours — `(n_up.y − n_down.y) + (n_right.x − n_left.x)`,
  soft-clamped by `0.5 / ridge²` and `0.7 / valley²`
  (`CavityEffect::init`, `curvature_soft_clamp`), `×2`, the composite's
  factor clamped to 0..4 — never across an object's edge or over the
  background. Off by default, as Blender's is.
- **Where it runs:** the Solid kernel leaves each pixel's scene-linear
  colour, grid, object id and view normal in the image's scratch
  (`fx_store`); an `Fx` kernel (`phiray/trace/wgsl/fx.wgsl`) reads the
  neighbours, applies both, puts the colour through the view transform and
  lays the grid over it, before the anti-aliasing gather. Nothing for X-ray
  (the transparent pass has neither) or Wireframe. `Camera.fx` and
  `Camera.outline` carry the settings.
- **From the file:** `View3DShading`'s `show_object_outline`,
  `object_outline_color`, `show_cavity`, `cavity_type` (`WORLD` SSAO,
  `SCREEN` curvature, `BOTH`) and the four factors are read
  (`phiphx::SceneSolidFx`, `blend/scene/ui.rs`), seeded as the view opens
  (`app::load_ui`, keys `view3d.shade.<rna>`), and read by the viewport
  (`scene/solid_fx.rs`, `BSolidFx`; `rendered/view.rs`).
- **Bands** of a large still overlap by two rows now (effects and gather
  each read a neighbour), still seamless.
- Tests: phiray `tests/fx.rs` (a dark line at each object's edge and on
  the sky beside it, interiors untouched, off = identical; a ridge
  brightens, a valley darkens, a flat wall and the wall in front are
  untouched), `tests/solid.rs` tiles with effects on; phiphx
  `tests/view_solid_fx.rs` (the startup file: outline on, black, no
  cavity, Screen, factors 1); phiber `solid_fx.rs` (factory, a file's
  values, the camera words), `load_ui.rs` (seeded from `overlays.blend`),
  `still.rs` (bands with effects).
- **Suites:** see the head of this file.
- **Not yet:** the Shading popover (`VIEW3D_PT_shading`) to switch these
  in the app — the file's settings and the factory defaults rule until
  then; Cavity ▸ World (the screen-space ambient occlusion,
  `workbench_cavity.bsl.hh`, needs Blender's blue-noise table and the
  Hammersley samples); Workbench's shadows; a final render's
  `Scene.display.shading` (F12 renders without the outline).

## Batch 118 — Solid and Wireframe anti-aliased as Workbench does

- **Workbench's TAA, ported step for step**
  (`workbench_effect_antialiasing.cc`). Solid and Wireframe are drawn
  eight times (`U.viewport_aa`'s default, `taa::SAMPLES`), each sample
  offset by a jitter from Blender's own table — `BLI_jitter_init`'s
  relaxed points, from `RandomNumberGenerator`'s 48-bit congruence
  (`BLI_rand.hh`), `BLI_jitterate1/2` 24 times, the sample nearest the
  centre moved onto it and drawn first, the rest each the farthest from
  the one before (`TaaSamples::init_samples`; `phiray/trace/taa.rs`,
  `offset`). Each sample is gathered over its 3 × 3 neighbourhood with
  Blackman-Harris weights of width 2 for that jitter
  (`filter_blackman_harris`, `setup_taa_weights`; `taa::weights`) in
  `log2(colour + 1)` space, into running sums, and the shown pixel is the
  sums over the weight accumulated so far (`wgsl/taa.wgsl`, a `Taa` kernel
  after the shading kernel each sample). The jitter moves the ray
  (`taa_jitter`, `util.wgsl`) and the prepass alike (`raster.wgsl`, as
  `window_translate_m4` offsets the projection) so the two paths stay
  pixel for pixel. The first sample is the centre, so the first frame is
  the unjittered image of before; seven more settle the edges, and a
  moved view starts over, as Blender's `taa_sample = 0` does.
- **Tiles join without a seam.** A frame larger than the device binds is
  still rendered in bands; each is now drawn a row over into its
  neighbours and laid in without it, since the gather clamps at a tile's
  edge (`rendered/still.rs`, `BBands::of` keeps two rows for the overlap).
  A final Solid render is anti-aliased too (Workbench's `render_aa`).
- **Camera uniform:** `Camera.taa` (jitter x, y, accumulated weight) after
  `vis`; `camera.wgsl` now holds the struct for every kernel.
- Tests: `taa.rs` (the table: eight distinct, relaxed points, the first on
  the centre, wrapping; the filter: full at the centre, nothing past the
  width, a shifted sample weighs its side), `tests/solid.rs`
  (`solid_settles_in_eight_jittered_samples_and_lights_the_surface`,
  `tiles_laid_together_are_the_whole_frame` over a row's overlap, the
  wire and mode-switch tests settled), `tests/slices.rs`, `tests/raster.rs`
  (rasterized = traced under jitter), phiber `still.rs`
  (`a_still_rendered_in_bands_is_the_still_rendered_whole`).
- **Suites:** see the head of this file.
- **Not yet:** SMAA on the first frames (`workbench_effect_smaa`), so the
  edges are aliased until the eighth sample lands; `Scene.display.viewport_aa`
  and `render_aa` from the file (eight always); the accumulation is of
  display-space colour, grid included, where Blender accumulates linear
  before the view transform.

## Batch 117 — opening a file in a third of the time

- **The DNA layout, worked out once** (`phiphx/blend/file/sdna.rs`,
  `DnaType::layout`): every member's offset and size by name, as
  Blender's SDNA holds them (`DNA_sdna_from_data`,
  `DNA_struct_member_offset_by_name`), where each field read used to walk
  the type list and the struct's members with string compares — two
  thirds of the open's samples. `struct_field_offset_exact`,
  `struct_field_offset`, `field_size`, `struct_size`, `int_field`,
  `uint_field`, `nested_int` all answer from it. Test:
  `sdna::tests::members_are_laid_out_once_by_full_and_bare_name`.
- **Skin sources in parallel** (`modifier/mod.rs::skin_sources`): the
  subdivision surfaces under the armatures, one clothed mesh after
  another on the main thread before, now on all cores as
  `evaluate_modifiers` runs its.
- **Female-01: 28.1 s → 10.0 s** opening in the two-core container
  (modifiers 6.8 → 3.0, actions 5.7 → 1.2, motion 9.5 → 2.7); THYMUS
  0.31 s, FLORIS 0.25 s. The user's 23.8 s should fall further, having
  more cores. Table in `frame-cost.md`.
- **Suites:** see the head of this file.

## Batch 116 — a video from the animation; Cancel where the eye is

- **FFmpeg Video writes a movie.** Output ▸ File Format: FFmpeg Video
  (the panel's existing entry, now wired: `output_set::wants_movie`) and
  a new **Encoding** section (`RENDER_PT_encoding`: Container, Video
  Codec, Output Quality, Encoding Speed, Keyframe Interval, Bitrate —
  Blender's RNA items), seeded from the file's `Scene.r.ffcodecdata`
  (`phiphx/blend/scene/movie.rs`: `type`, `codec`,
  `constant_rate_factor`, `ffmpeg_preset`, `gop_size`, `video_bitrate`)
  and read back (`output_set::movie_set`). Ctrl F12 gathers each frame
  as a PNG and, when the range ends or the render is cancelled, encodes
  one movie (`scene/video/mod.rs`): through the machine's `ffmpeg` with
  the arguments `movie_write.cc` amounts to (`-c:v libx264 -crf 23
  -preset medium -g 18 -pix_fmt yuv420p` for a new file's defaults), or —
  with no ffmpeg — a Motion-JPEG AVI written by hand (`video/avi.rs`, the
  RIFF Blender's `avi.cc` wrote), so a movie always comes out. Named as
  `BKE_movie_filepath_get` names it: `shot_0001-0140.mkv`. The Info editor
  reports which. Tests: `movie.rs` (naming, codec names, CRF, presets),
  `video/mod.rs` (the command line; the AVI read back by ffprobe as three
  MJPEG frames, the real encode as H.264 Matroska),
  `render_animation_writes_a_movie.rs` (Ctrl F12 end to end: one movie,
  no stills).
- **Cancel in the Image editor's header** while a render runs, with its
  progress bar, beside "Render Result" (`part/image/header`); the status
  bar's X stays. With 115's Esc-anywhere, three ways to stop a render.
- **Which file animates:** THYMUS-BOLDROID-WAVE-003 (`rigAction`, frames
  0–132, 530 curves; range 1–140 at 24 fps) is the animated one;
  Female-01's 69 actions are one-frame poses on 18 NLA tracks;
  FLORIS is a rig in one pose. `workflow.md` says which to use for what.
- **Suites:** see the head of this file.
- **Not yet:** audio; the Encoding panel written back to the file;
  Overwrite / Placeholders / frame step; a bundled encoder.

## Batch 115 — Esc stops a render anywhere

- A plain Esc cancels a running render whatever has the keyboard —
  Blender's `G.is_break`, set in `wm_event_do_handlers` for exactly this —
  where before only Esc in the render window did. Test in
  `render_job_runs_in_the_background.rs`.

## Batch 114 — Solid and Wireframe find their surfaces as Workbench does

- **A visibility prepass** (`phiray/src/gpu/raster.rs`,
  `trace/wgsl/raster.wgsl`, `visible.rs`): a wgpu render pipeline draws
  every face of the scene — three vertices pulled from the tracer's own
  face, vertex and instance buffers, so no second copy of the mesh — with
  a depth test into a target of hits (face, barycentrics, distance), which
  the Solid and Wireframe kernels read (`vis2.wgsl` `primary`) where they
  walked the BVH. The projection is `solid.wgsl`'s ray model inverted, so
  the hit is the ray's: `phiray/tests/raster.rs` finds **0 pixels
  differing** between the rasterized and the traced frame, orthographic
  and perspective, Solid and Wireframe, over turned and scaled instances.
  X-ray and tiles still trace. Workbench's structure
  (`workbench_mesh_passes.cc` prepass + composite); its shading was ported
  earlier (`workbench.wgsl`).
- **On for hardware, off for software adapters**: on lavapipe the prepass
  over THYMUS's 436 k faces took 378 ms a frame against 83 ms traced, so
  the choice follows `GpuAdapter::is_software`, and the journal's `scene
  built` line names it (`Solid and Wireframe rasterize their primary
  hits`). The 1660 Ti's number is the next journal's.
- **Female-01's journal read** (`frame-cost.md`): opening 23.8 s (modifiers
  6.8, actions 5.2, motion 8.9), a Solid sample 44–64 ms of GPU over
  2.9 M triangles, UI thread 2–3 ms, no slow frames, Trace kernel 25.6 s
  to compile. A how-to for rendering, saving, exporting and playing, and
  what that file's render shows (its 345 unpacked textures), is
  `workflow.md`.
- **Suites:** see the head of this file.
- **Next (115):** the three reader stages (modifiers, actions, motion) that
  are 21 of Female-01's 24 s; Workbench's TAA (eight jittered frames,
  `BLI_jitter_init`, the Blackman-Harris splat) — Solid's edges are
  aliased today; then outline, cavity, shadows, `Scene.display` from the
  file; then Material Preview on EEVEE's model and the shader nodes.

## Batch 113 — the Animation workspace's frame, and a time axis that zooms

- **Profiled** (THYMUS-BOLDROID-WAVE-003: 259 objects, 328 bones; the
  playhead stepped in a debug build): the Animation workspace cost 64 ms a
  frame against the Layout workspace's 39, and the UI thread's stack
  (sampled 250 times) said why. The Outliner built all its rows again on
  every frame — the frame counter bumped `scene_revision`, which its row
  cache and the hidden set key on; the clock's families (`frame`,
  `playing`, the ranges, markers, keys, the new `time_view`) are now left
  out of it, as Blender's Outliner keeps its tree store across frames
  (`view_pan_apply_ex`'s "don't rebuild full tree in outliner" is the same
  thought). The Dope Sheet tessellated two vector paths per key per frame
  (`time/row/set.rs`); a row's keys now go in three paths — every border,
  the plain fills, the picked fills — with the keys outside the window
  skipped (Blender draws its diamonds in one instanced batch,
  `draw_keyframe_shape`). 64 → 35 ms; the Layout workspace stepped the
  same frames at 33, which is the pose evaluation itself. Test:
  `the_rows_are_built_again_only_when_what_they_read_changed` now steps the
  frame, plays and picks a key between two builds. Details in
  `frame-cost.md`.
- **The time axis zooms, scrolls and frames** (Animation item 4): the wheel
  over the Timeline, Dope Sheet, Graph or NLA editor is `view2d.zoom_in` /
  `zoom_out` (3.75 % a notch, `view_zoomstep_apply_ex`, symmetric so a
  notch back restores the view; 0.01–50 px a frame), Ctrl wheel is
  `view2d.scroll_left` / `_right` (forty pixels a notch,
  `VIEW2D_OT_scroll_*`), Home and View ▸ Frame All are `action.view_all`
  (`actkeys_viewall`: the shown keys or strips stretched across the
  window with `ANIM_frame_range_view2d_add_xmargin`'s four diamonds; a lone
  key centred; the scene range when nothing is keyed), View ▸ Frame Scene
  Range is `anim.scene_range_frame` (the preview range when on). One
  `View2D.cur` per editor (`TimeViewKey`, `canvas/time/view/cur.rs` the
  maths, `op.rs` the rule), unset until moved so each editor still opens on
  `action_create`'s default. Tests: `cur.rs` (the formulas against
  Blender's constants), `op.rs` (the rule, Frame All over
  `actions.blend`'s keys), `the_time_axis_zooms_under_the_wheel.rs` (end
  to end: a notch, Ctrl wheel, Home over the painted Timeline).
- **The Blender reference is 5.2.0 LTS** (`refs/blender/blender`,
  `BKE_blender_version.h` 502). Batch 112's two-level trace was checked
  against `kernel/bvh/traversal.h` and `geom/object.h` after the fact:
  the same `bvh_instance_push/pop` with `OBJECT_INVERSE_TRANSFORM` and an
  unnormalised direction, normals by the inverse-transpose
  (`object_normal_transform`). Cycles bakes single-user meshes for a final
  render (`SD_OBJECT_TRANSFORM_APPLIED`), the path a viewport must avoid.
  From here each batch names the Blender functions it ports.
- **Decided: the viewport goes Blender's way.** Solid, Wireframe and
  Material Preview are rasterized in Blender (Workbench, EEVEE) and ray
  traced here; batch 114 ports Workbench onto wgpu — see
  `animation-rendering-status.md`, "The viewport, and Blender's way".
- **Suites:** see the head of this file.
- **Not yet:** Female-01's journal; middle-mouse pan and Set Preview Range
  (P); the Graph editor's value axis under Frame All.

## Batch 112 — moving an object is a matrix, not a scene

- **Objects are instances in the tracer** (the user's third ask: instant
  move, rotate, scale). Each object's triangles stay in its own space
  under a tree of their own (`phiray/trace/trees.rs`: a BLAS per
  instance, built in parallel), and a small top tree spans the objects'
  world bounds; a face names its instance in the top half of its material
  word (`Scene::instance`, `Scene::face`), and the kernel walks each
  instance in object space, the ray taken there by the instance's inverse
  (`bvh.wgsl` `trace_blas`) and the hit placed back by its world matrix
  (`layout.wgsl` `tri_at`, normals by the inverse-transpose). phiber's
  scene builder hands each object as `out.instance(matrix_world)` with
  its raw vertices (`rendered/scene.rs`), primitives as translations.
- **A move is `Tracer::reinstance`**: the new matrices, the top tree
  rebuilt (a few kilobytes) and the instance table written in place —
  nothing of the meshes, their trees or the atlas moves. `place.rs`
  decides: the very same meshes, materials and textures under other
  matrices (`only_placed`, `Arc` identity) are *placed*; a mesh built
  again (a rig's frame, an edit) is *posed* as before (`repose`, every
  tree refitted, rebuilt once still); anything else builds. The journal
  says which: `Tracer: N objects placed in X ms` / `scene posed in X ms` /
  `trees rebuilt over the pose in X ms`.
- **Tests:** `phiray/tests/instances.rs` — an instanced scene (a scaled,
  turned wall) draws what the same scene baked in world space draws; a
  moved instance draws as a scene baked at the new place and starts its
  image over; a new tree after a move keeps the image's samples.
  `trees.rs`: the top tree leads, each instance's tree is shifted into
  place and ranges over its own faces; moving an instance changes only
  the top tree. `place.rs`: placed vs posed vs built, on the GPU.
  `a_moved_object_draws_as_a_scene_built_there` now runs the placed path.
- **The journal's warm-up line names each kernel's compile time**
  (`Tracer::warm`): the user's run showed the four kernels taking 21 s
  to compile on the 1660 Ti the first time (the driver caches them
  after) — see `frame-cost.md`.
- **Suites:** phiray 62, phiber 1023 passing / the same 7, phiphx 188 + 6.
- **Not yet:** Female-01's own numbers (the journal sent was a 10-minute
  default-cube session: no slow frames, no panics); Material Preview's
  soft shadows, reflections, AO; transparent glass under a transparent
  film.

## Batch 111 — the viewport refreshes on its own; one width for a popover's boxes

- **The slowness, named by the journal** (batch 108 build, Female-03: 322
  objects, 203 k triangles): the scene built in 200 ms (triangles 174,
  tree and upload 26), a full-size frame traced in 0–18 ms — and the first
  image showed **10.9 s** after the file opened, a shading change took
  1–2.7 s, and at start-up the default cube's image came **4.2 hours**
  later, with the first click. The viewport was never refreshing on its
  own. The ray ticker (`ensure_ray_ticker`, the task that draws the window
  when the render thread lands a frame) is armed from the last paint's
  verdict that a view is refining (`rendered::refining`); `render` asked
  **after** `drawing()` had cleared that flag for the draw in hand, so it
  never saw it, and every image — the first, and the last of each
  refinement — waited for some input to redraw the window. `render` now
  asks first, and a paint that leaves its view refining with no ticker
  running asks for one more frame itself (`wake::ticking`, set by the
  ticker). `tests/the_first_image_arrives_unasked.rs`: after the start-up
  frames, with no input and no notify, the ticker is running.
- **A deterministic view starts at full size when a full frame is quick**
  (`BCoarse::start_for`: the last full-size sample of that mode, per mode
  in `BSlicer`, predicts under 33 ms): a Solid, Material or Wireframe
  orbit no longer pixelates to an eighth and back; Rendered keeps its
  ladder.
- **The other kernels compile in the background** once the first image is
  up (`Tracer::warm`, a thread of its own, journaled): a change of shading
  no longer waits a second for its shader. phiray's pipeline cache is per
  device, so each compiles once.
- **A popover's checkboxes are one width.** The value column handed its
  width to the control (`ctx.filling()`), the row inside the control
  filled, but the wrapper around it hugged — so "Sync" sat in a pill a
  third the width of "Limit to Frame Range". The wrapper now fills too
  (`button.rs`, described with `BFlex`, within the raw-style budget).
  `a_popover_is_as_wide_as_its_rows.rs`: every box in the Playback popover
  measures the same.
- **Suites:** phiray 55, phiber 1020 passing / the same 7, phiphx 188 + 6.
- **Not yet:** Female-01's own numbers (the user's next run, now that the
  viewport refreshes); per-object transforms in the tracer; Material
  Preview's soft shadows, reflections, AO; transparent glass under a
  transparent film.

## Batch 110 — a crash writes itself into the journal

- `BJournal::arm` installs a panic hook: `PANIC on thread '…' at
  file:line: message`, written before the process's own hook runs, with a
  try-lock so a panic under the journal's lock cannot deadlock on it.

## Batch 109 — Viewport Denoising, on by default

- **The viewport is shown denoised while it refines.** Properties ▸ Render
  ▸ Sampling ▸ Viewport gains Blender's Denoise sub-panel
  (`CYCLES_RENDER_PT_sampling_viewport_denoise`: the toggle as its header,
  Start Sample below; `cycles.use_preview_denoising`,
  `preview_denoising_start_sample`), read into `BRenderSet` and carried on
  the shot as `denoise_from`. In Rendered mode the render thread filters
  every frame it publishes from that sample on (`Tracer::denoise_shown`),
  **leaving the linear sums as they are** — the denoiser's last pass used to
  replace them, right for a finished F12 and wrong for a viewport whose
  next sample must land on the truth, not on a filtered copy
  (`DenoiseParams::keep`). Deterministic modes are skipped by the tracer.
  `phiray/tests/denoise.rs`: the shown pixels change, the radiance does not,
  and a finished render's denoise still replaces it.
- **Phiber's default is on; Blender's is off** (`properties.py`:
  `use_preview_denoising` default False, start sample 1). The user asked
  for the best settings by default and a noisy view of plain cubes was the
  complaint, so the panel's own default is "on" — and because an edit is
  what differs from the panel's default, a file that never set it is saved
  as it was.
- **Suites:** phiray 55, phiber 1018 passing / the same 7, phiphx 188 + 6.
- **Not yet:** the user's 184 s after opening Female-01 and the per-pose
  re-upload — per-object transforms in the tracer (the user's third pick);
  Material Preview's soft shadows, reflections, AO; transparent glass under
  a transparent film.

## Batch 108 — Material Preview without noise, a sample that yields, and the window's X asks

- **The journal said where the slowness was** (the user's release build,
  GTX 1660 Ti, Female-01: 319 objects, 4.5 M vertices): the UI thread
  1.8–5 ms a frame; the tracer's first image **184 s** after the file was
  read; every orbit step **0.3–1.2 s** to its first image because a
  full-size sample of the previous view (2–6 s a view) ran to its end
  first; and Material Preview one noisy sample a frame, being path traced.
- **Material Preview is a deterministic kernel** (`Mode::Preview`,
  `trace/wgsl/preview.wgsl`), as EEVEE is to Cycles: irradiance SH and
  GGX-prefiltered copies of the studio HDRI precomputed into the atlas
  (`trace/ibl.rs`), one shadow ray per lamp, deterministic alpha and glass,
  mipmapped textures read at the pixel's footprint (`Atlas::add` builds the
  chains; the head's width carries the level count). One dispatch is the
  frame; no noise on a plain cube or anything else. `rendering.md` §22.
- **A sample in slices** (`Tracer::sample_in`): a full-size sample is cut
  into ~25 ms dispatches and abandoned between them when a newer view or
  scene has been posted (`BPosted`); while views keep arriving the view
  stays at its coarse size, as Cycles keeps a navigated viewport small, and
  grows once the hand rests. `phiray/tests/slices.rs`.
- **The tracer writes its own lines to the journal**: scene build split
  into triangles vs tree and upload, pose and retree times, a tally every
  64 samples (`rendered/pace.rs`). The 184 s gets named on the next run.
- **Closing the window asks about unsaved changes** as File ▸ Quit does:
  `BPhiberApp::may_close` behind gpui's `on_window_should_close` (the
  system's close, Alt F4, Windows' caption), and the drawn caption's X
  dispatches Quit on the other platforms (`op/close.rs`, `caption.rs`).
  Save / Don't Save / Cancel, the dialog that was already there.
- **Structure:** `rendered/worker.rs` split — `job.rs` (what is posted and
  published), `posted.rs`, `pace.rs` (slicing and the tally), `build.rs`;
  `phiray/src/trace/ibl/{mod,panorama}.rs`; `PI` and `unit_disk` moved to
  the kernels' shared prelude so every composition sees them.
- **Suites:** phiray 54 (+8: preview, slices), phiber 1018 passing / the
  same 7, phiphx 188 + 6.
- **Not yet:**
  - the 184 s scene build and the per-pose re-upload (a transform
    re-transforms and re-uploads every vertex): per-object transforms in
    the tracer — the user's third pick;
  - Material Preview's soft shadows, reflections of other objects, AO;
  - fast interaction in Rendered mode beyond the slices (half resolution
    while moving, denoise from the first sample) — the user's second pick;
  - transparent glass under a transparent film; "apply filters" and "add
    into a matrix" still need the user's meaning.

## Batch 107 — the session journal

- **One file says what the session did, with its times**:
  `target/release/phiber-session.log` beside the executable (temp dir as a
  fallback), started afresh each run (`app/journal.rs`, armed by
  `app::run`). Each line carries the seconds since launch and the gap since
  the line before: every Info-log line (actions, "Opening X", "Opened X … in
  N ms", the health lines), each read stage once (`Reading: …`), every
  frame over 50 ms with its tree/draw split, a frame summary every 120
  frames (average, split, worst, slow count — `trace::tick`, always on),
  the tracer's settle line (how many views an orbit posted, the last one's
  first image and finish — `rendered/clock.rs`), and F12's duration (from
  the render thread; the journal is a mutex, so any thread writes it). The
  slow-frame log in `%TEMP%` is gone; `BLogEntry` carries its `Instant`.
  The user does not paste anything: the file is in the workspace and is
  read from there (`cloud-workspace.md`).
- **Suites:** phiber 1016 passing / the same 7, phiphx 188 + 6, phiray 46.

## Batch 106 — the per-object frame cost, taken apart

- **Measured with a sampler, not a guess.** callgrind under the gpui test
  harness distorts a frame (the tracer's thread never delivers in time, so
  the CPU painter runs every frame); a poor-man's profiler — `gdb -batch -p
  <ui thread> -ex "bt 60"` 250 times over a long still run — does not. At
  500 objects, of a 25 ms debug frame: the hidden sets (`BHidden::now` and
  `for_select`, each a tree walk and two encoded key reads per object,
  built again every frame) 3 ms; the tracer's scene key (`BRayKey::of`,
  SipHash over 500 names and matrices) 0.8 ms; the mesh memo's fingerprint,
  the same hash again, 0.6 ms; the shown list cloned (500 `BObject3D`s, each
  a name and a materials list) and the unselectable names collected; and
  about 2.5 ms of GPUI laying out the Outliner's rows in view (~34 at this
  window height against 5 — the list's visible rows, legitimate).
- **Kept until what they read changes:**
  - `BState::scene_revision` names the thing both memos key on: the writes
    to every family but the viewport's camera. The Outliner's rows now use
    it too.
  - `BHidden` keeps each kind's set across frames by `(scene_revision,
    model)`; the pass cache stays as it was over it (`part/outliner/key.rs`,
    its tests).
  - `view3d/shown.rs` — `BShown`: the shown objects and the arrow-toggled
    names as one kept pair, by the identity of the objects list and the two
    hidden sets it was built from; the viewport's render takes it in place
    of its three per-object passes.
  - `view3d/identity.rs` — `BSceneId`: a kept list's identity, hashed once
    per list. `BRayKey::of_list` (the tracer) and `cache::fingerprint` (the
    mesh memo) read it; `BRayKey::of` and `shape` keep their uncached
    forms, the one identity hash shared.
  - Measured again: an orbit frame at 500 objects 24–28 → 16–17 ms, against
    15–16 ms at 3 objects — the per-object cost of an orbit is gone from the
    UI thread in debug. What remains per object is the Outliner's visible
    rows in GPUI's layout.
- **The user builds with `cargo run --release -p phiber`** — so the frame
  costs above (a few ms in release) are not what they feel. Asked for the
  Info editor's render-device line and `%TEMP%\phiber-frames.log`, and
  which action is slow. Checked meanwhile: Solid and Wire trace in one
  dispatch (`phiray::Mode::deterministic`), not progressively; the window
  wakes per landed frame, not per vsync (`rendered/wake.rs`). The answer
  decides batch 107.
- **The machine's tree** was exactly batch 105's: nothing to merge.
- **Not yet:**
  - whatever the logs say is slow in release;
  - transparent glass under a transparent film;
  - "apply filters" and "add into a matrix" from the report still need the
    user's meaning.

## Batch 105 — where a frame goes on a small scene and a long Outliner, and the Info log says what the session is

- **Measured again, after batches 99 and 101.** A scratch probe (the
  recipe in `cloud-workspace.md`) timed one UI frame in a debug build at
  1600 × 900 — a still, an orbit step, an orbit with an object selected, a
  zoom — over four scenes. Triangle count no longer matters: 500 objects
  of 10 million triangles and 500 of 4 000 cost the same frame (~28 ms in
  the test harness, of which ~13 ms is the harness itself). Two other
  things did:
  - **A small scene orbited slower than a huge one.** Three objects of
    60 000 triangles took 52 ms an orbit step against 13 ms for a still.
    The CPU painter stands in for a frame the tracer has not delivered,
    and its triangles were projected on every frame of every scene under
    its ceiling (400 000 triangles) *in case* the fallback was needed —
    every vertex of a Suzanne-sized scene, on the UI thread, on every
    mouse move, with the image already on screen. The projection now
    happens in the paint, the moment the tracer turns out to have no
    image, through the same memo (`view3d/mod.rs`, `fall_back`): 52 → 13 ms.
    `tests/the_viewport_always_draws_something.rs` pins the new shape.
  - **Five hundred Outliner rows were built every frame.** The Outliner
    lays out only the rows in view (batch 7x's `BRowWindow`), but the view
    model — five hundred `TreeRow`s with their toggles, badges and drag
    payloads — was built from the state on every frame: 6.3 ms of a debug
    frame's 9.5 ms tree. `BState` now counts writes per key family (the
    segment after `ctx`: `view3d`, `outliner`, `object` …;
    `revision_except`), and the Outliner keeps its rows until a family it
    reads changes or the loaded model does — an orbit writes `view3d` keys
    alone. The kept view is handed back as `BView::Shared(Rc<BView>)`, a
    new variant the renderer and the text renderer look through, so a hit
    costs no clone: 6.3 → 1.2 ms, and the 500-object frame 28 → 22 ms.
    `part/outliner` tests (the same rows across an orbit, new rows after a
    pick) and `state::revision_tests`.
  - What is left of the 500-object frame over the 3-object one (~9 ms in
    debug) is spread over the per-object passes that still run each frame
    (`for_select` + `hides` 1.4 ms, the hit list, the tree layout); next
    on the list.
- **The Info log says what the session is.** Three things decide where
  the time goes on a large file and none was on screen. `app/health.rs`
  writes, as the running app's window opens: "Debug build: the interface
  runs several times slower than `cargo run --release -p phiber`" (when it
  is one), where the slow-frame log is (`trace::log_path`, frames over 50
  ms), and — once phiray has its device — "Render device: NVIDIA … (Vulkan,
  discrete)", with a warning when the device is a software adapter
  (lavapipe, WARP, SwiftShader: "no GPU phiray could use was found …").
  Only the running app says them (`app::run` arms it); a test's window
  stays quiet, so the log tests still see their own lines alone.
- **The machine's tree** was exactly batch 104's (the 1 772 phiber source
  files the slow bridge left unchecked now pass too, in 27 s): nothing to
  merge.
- **Not yet:**
  - the remaining per-object frame cost above;
  - transparent glass under a transparent film;
  - "apply filters" and "add into a matrix" from the report still need the
    user's meaning.

## Batch 104 — the Render tab's enumerations reach the file, and string properties can be written

- **The denoiser and pixel filter type are saved and seeded.** Batch 102
  left the Render tab's enumerations out: the writer had no string
  property, and the panel's identifiers ("GAUSSIAN") looked like strings.
  They are not: Blender stores an add-on's `EnumProperty` as the item's
  *number* — checked with `bpy` (`pixel_filter_type` Box 0, Gaussian 1,
  Blackman-Harris 2; the denoiser's items are OptiX 2 and OpenImageDenoise
  4, from `cycles/blender/addon/properties.py`). A first attempt that
  wrote the identifier as a string read back as the default in Blender,
  because RNA never looked at a string where it kept an int.
  - `scene/render_edit.rs::BCyclesEnum` is the table both ways: a save
    writes the panel's identifier as the file's number, and seeding
    (`app/seed.rs`) turns the file's number back into the identifier the
    control shows — so a 5.0 file's `denoiser`, which seeding used to skip
    as an int under an enumeration control, now shows.
  - The two specs carry their RNA defaults (`.picked(…)`), so a panel left
    at the default writes nothing, as for every other Cycles value.
- **String ID properties can be written too** (`set_id_text`): a string
  keeps its text in a second block the property points at
  (`IDPropertyData.pointer`), `len` counting the terminator; a text that
  fits the block it has goes there, a longer one gets a new block. No
  Cycles setting the panel draws is a string, so this is checked by phiphx
  reading its own write back (`phiber_note`, which Blender keeps and
  ignores); the enumerations are checked by Blender reading Gaussian back
  from every fixture (`truth/saved.txt`).
- **Tests:** `phiphx/tests/blender_opens_saves.rs` (the two enumerations
  and a string, re-recorded with Blender 5.0.1), `scene/render_edit.rs`
  (a Gaussian pick is Blender's 1; the denoiser at its default is no
  edit), `tests/render_settings_are_saved.rs` (Gaussian through Ctrl S and
  seeded back as "GAUSSIAN").
- **On the slowness report.** Asked the user, mid-batch, for the two
  things that decide where the time goes — whether the build is a debug
  one (a debug UI thread spends 25–40 ms a frame laying out the window on
  a large scene; `cargo run --release` is the first check) and whether the
  Render tab's Device row names the card (Auto that never names one means
  phiray fell back to a software adapter) — and for the frame-cost log
  (`trace::log_path`) after a slow orbit. Nothing to change until that
  comes back.
- **The machine's tree.** The folder bridge was slow to the point of
  timing out for most of this batch (a bare `ls` of the tree took over two
  minutes — something heavy was running there). The eight files went over
  guarded as usual; the checksum pass after the copy confirmed phiphx,
  phiray, phiber's tests and phiber's `app`, `scene` and `part/props`
  trees match here (1 285 files), and timed out on the remaining 1 772
  phiber source files, which this batch does not touch. That remainder is
  the first thing the next batch checks.
- **Not yet:**
  - transparent glass under a transparent film;
  - "apply filters" and "add into a matrix" from the report still need the
    user's meaning.

## Batch 103 — Film ▸ Transparent reaches the tracer, the saves and the file

- **Film ▸ Transparent renders the background as alpha 0.** The Render
  tab's Film ▸ Transparent was a checkbox nothing read: F12 kept the sky
  behind the scene and every save was opaque. Now:
  - phiray's `Camera` gained `film: [transparent, _, _, _]`. Under a
    transparent film a camera ray that meets nothing leaves its pixel black
    at alpha 0 (Blender's render result) instead of the sky; a pixel's alpha
    is the share of its samples that met something, so an edge against the
    background is anti-aliased in alpha as it is in colour. The pixel
    buffer grew a word (`PX_WORDS` 25): the film coverage sum, which the
    packed pixel's alpha byte and the denoiser's final pass read
    (`util.wgsl::emit`, `view.wgsl::pack`, `denoise.wgsl`).
    `tests/solid.rs::a_transparent_film_cuts_the_background_away`: sky
    `[0, 0, 0, 0]`, the wall lit at alpha 255, the same after denoising, and
    the default film's sky at alpha 255.
  - The saves already took their alpha from that byte (batch 9x: PNG and
    WebP RGBA, 16-bit PNG, EXR's A channel), so a PNG saved with Color
    RGBA now has the background cut away; the Image editor's Color and
    Alpha channel mode shows the checker through it.
  - `render_set::BRenderSet.film_transparent` reads the panel, the render
    plan sets `Camera::film` from it — a final render's only: the viewport
    keeps its sky, as Blender's does — and it is seeded from the file
    (`Scene.r.alphamode == R_ALPHAPREMUL`; phiphx `BlendSceneProps::
    film_transparent`) and written back by Ctrl S (`BlendEdits::
    film_transparent`; `write/edit.rs`). Blender 5.0.1 reads it back from
    every fixture (`truth/saved.txt`, `film 1`).
  - `tests/render_settings_are_saved.rs` toggles it with the rest, saves,
    and checks the plan built from the saved file carries `film = [1, 0,
    0, 0]`.
- **The machine's tree** was exactly batch 102's at the sync: nothing to
  merge.
- **Not yet:**
  - Blender's transparent glass under a transparent film
    (`film_transparent_glass`, the roughness threshold) — a transparent
    BSDF or glass seen against nothing still reads the environment;
  - ~~string-valued Cycles settings (denoiser, pixel filter type)~~ —
    batch 104 (they are numbers, not strings);
  - "apply filters" and "add into a matrix" from the report still need the
    user's meaning.

## Batch 102 — the Render and Output tabs are written to the file, and a 5.0 file's Cycles settings are read at all

- **Ctrl S writes the Render and Output tabs.** The engine, the Cycles
  settings the Render tab draws (samples, viewport samples, adaptive
  sampling, bounces, denoise, caustics, volumes, the pixel filter width …)
  and Output's path, file format, colour depth and colour mode were seeded
  from the file (batch 90-odd) and read by F12 and the saves, but never
  written back: the file kept what it had. Now:
  - `scene/render_edit.rs::BRenderEdits` compares the panel with the file —
    a Cycles value is an edit when it differs from the file's, or, where the
    file carries none, from the property's RNA default, since Blender
    writes only the values that differ from it — and `save.rs` hands the
    engine, the changed Cycles values and the Output set to the writer.
  - phiphx `write/render.rs`: the engine into `Scene.r.engine`, the output
    into `Scene.r.pic` and `Scene.r.im_format` (imtype, the one-of-many
    depth flag, `color_mode` or the older `planes`, quality, compression,
    EXR codec; `BlendImageFormat::imtype` is the inverse of `of`), and the
    Cycles values as ID properties: one the file has is patched in place;
    one it lacks is a new `IDProperty` block at the end of the scene's run,
    linked at the end of the `cycles` group with its `len` one more
    (`IDP_AddToGroup`); a file without the group gets it, and one without
    ID properties at all gets the root group first ("IDProperties", as
    `IDP_EnsureProperties` names it). A bool is written as `IDP_BOOLEAN`
    unless the file already holds it as an int.
  - **Blender 4.5 split the ID properties in two.** `ID.system_properties`
    holds what registered RNA property groups store — the Cycles settings
    among them — and `ID.properties` the user's custom properties. A group
    written under the old root in a 5.0 file reads as a custom property and
    `scene.cycles.samples` never sees it: that is exactly what the first
    attempt did, and Blender read 4096 back from every 5.0 file while
    reading 48 from the 3.x and 4.2 ones (where the layout has one root and
    Blender migrates on open). The writer now puts the group under the root
    the file's layout has (`system_root_field`), and Blender reads every
    value back in all six fixtures.
  - That split also means **phiphx had never read a 5.0 file's Cycles
    settings**: `id_properties` read `ID.properties` alone, so the factory
    5.0 file's `sampling_pattern`, and any sample count a 5.0 Blender had
    saved, were invisible and the Render tab showed the defaults. Both
    roots are read into one group now, the system side on top.
  - Enumerations (the denoiser, the pixel filter type, the device) are
    string properties, which need a second block for the text; not written
    yet. The device is phiber's own setting anyway (batch 99).
- **The tests.** `phiphx/tests/blender_opens_saves.rs` saves the engine,
  four Cycles values and an EXR output into every fixture (3.x, 4.2, 5.0
  startups and the three scene files) and `truth/saved.txt`, re-recorded
  with Blender 5.0.1, shows Blender reading each back (`render` lines);
  `scene/render_edit.rs` (as seeded nothing is an edit; a value at its
  default is not one; the rest of Output stays the file's);
  `tests/render_settings_are_saved.rs` (the panel's values through Ctrl S
  into a 5.0 file and read again, with what the file carried kept).
- **The machine's tree** was exactly batch 101's at the sync: nothing to
  merge.
- **Not yet:**
  - string-valued Cycles settings (denoiser, pixel filter type);
  - ~~the Film ▸ Transparent toggle through the tracer and RGBA saves~~ —
    batch 103;
  - the Rendering workspace renders once; a scene edited since shows the
    old result until F12 (Blender does the same);
  - "apply filters" and "add into a matrix" from the report still need the
    user's meaning.

## Batch 101 — a cancelled render keeps its image, a running one stops redrawing the window, and a file's meshes are prepared as it loads

- **A cancelled render keeps the image as far as it got.** Batch 100's
  cancel finished the shot without an image. Blender keeps the tiles a
  cancelled render finished; `still_stepwise` now lays in the bands done and
  the one under way at its samples so far, and hands that to the job as
  the finished still, so the Render Result shows the partial render, Save
  As writes it, and the Info editor says "Render cancelled after N
  samples". An animation render stops at that frame (`render_job::
  cancelled`) instead of writing it and going on.
  `tests/render_job_runs_in_the_background.rs` checks the cancelled shot
  is drawn, not blank.
- **A final render no longer redraws the window every vsync.** The Image
  editor asked for a new frame every vsync while a render ran, so the whole
  window was laid out sixty times a second for the minutes a render takes
  — the same cost batch 99 took out of the viewport. The render job now
  flags each sample it publishes (`rendered::landed`) and the one task that
  serves the viewport (`BPhiberApp::ensure_ray_ticker`) draws the window
  when a sample lands, for a final render as for a refining view, ending
  with it. The status bar's progress bar moves on the same ticks.
  - That task could also run for ever: the viewport's "refining" flag kept
    its last value when no viewport was on screen (the Rendering
    workspace), so the task kept polling at 8 ms with nothing to draw.
    Each window draw now clears the flag first (`wake::drawing`) and any
    unsettled viewport sets it as it paints, so a window without a viewport
    is not refining.
    `tests/the_rendering_workspace_opens_on_a_render.rs` drives the clock
    through a render and checks the task runs while it renders and ends
    with it.
- **A file's large meshes are prepared as it loads.** The pick tree of a
  mesh (`pick/tree.rs`) and the extreme points its outline projects
  (`mesh/extreme.rs`) were made on first use on the UI thread: a second's
  pause on the first click over a two-million-triangle mesh in a debug
  build, another on the first orbit with it selected. Both caches now sit on
  one shared, thread-safe keep (`scene/keep.rs::BMeshKeep`, by the mesh's
  address, dropped with the mesh), and a thread started as a file loads
  (`scene/warm.rs::BWarm`, from `app/load.rs`) makes them for every mesh
  large enough to want one, largest first. The UI thread still makes what
  it asks for before the thread gets there — the lock is held for the
  lookup and the insert, never for the making — and the first made is the
  one kept. `scene/keep.rs` and `scene/warm.rs` carry the tests (made once
  per living mesh, found from another thread, the shared mesh listed once,
  both made by the time the thread joins).
- **The machine's tree** was exactly batch 100's at the sync: nothing to
  merge.
- **Not yet:**
  - the Rendering workspace renders once; a scene edited since shows the
    old result until F12 (Blender does the same);
  - "apply filters" and "add into a matrix" from the report still need the
    user's meaning;
  - the time-cost figures of batch 99's audit (`frame-cost.md`) have not
    been re-measured since; a run of the probe on a large file would say
    where the next second goes.

## Batch 100 — a render of any size renders whole, and a render can be stopped

- **Tiles: a frame larger than the GPU binds renders whole.** Batch 99
  stopped the render thread dying on a 1920 × 1080 F12 under lavapipe's
  128 MiB binding (about 1.4 million pixels) by rendering the largest size
  of that shape that fit. The frame is now rendered at its full size in
  horizontal bands and joined, so a 4K frame renders whole on that device
  (13 bands), and a 1920 × 1080 one in two:
  - phiray's `Camera` gained `tile: [x0, y0, full_w, full_h]` (all zero: the
    whole frame). Ray generation (`frame_point` in `util.wgsl`, used by
    `shade`, `solid` and `wire`) works out each pixel's place in the full
    frame — the pixel's offset, the film plane's aspect, the depth-of-field
    jitter and the wire's pixel width all come from the full frame — so a
    tile's rays are exactly the ones the whole frame casts through those
    pixels. `tests/solid.rs::tiles_laid_together_are_the_whole_frame`: two
    half-height tiles laid under each other equal the frame drawn whole,
    byte for byte, in Solid and in Wireframe.
  - `rendered/still.rs::still_stepwise` cuts the frame into bands as tall as
    `Tracer::max_pixels` allows (`BBands`), renders each to its sample count
    with the same denoise, and lays the bytes, display floats, radiance,
    albedo, normal and depth into one `BStill` (`lay`). Progress steps over
    the bands together, so the header still counts to `samples`.
    `a_still_rendered_in_bands_is_the_still_rendered_whole` forces three
    bands over a 40 × 40 frame and gets the whole frame's pixels and a depth
    in every row. `scene/render_fit.rs` keeps only the drop guard that
    clears a job whose thread died.
  - The viewport is unchanged: it never asks for more than the window has.
- **A render can be cancelled from the UI.** `render_job::cancel` existed
  and nothing called it. Now:
  - **Esc** in the render window stops the render (`render_view_cancel`),
    then — with no render under way — closes the window as before
    (`keymap/set.rs`; `OpKey::Render` takes `cancel`).
  - **The status bar** shows the render as a job while it runs — a progress
    bar reading "Rendering" with the same X as an import has
    (`WM_jobs_stop`), beside the import's own.
  - A cancelled render finishes without an image rather than staying
    half-reported: the shot is marked done with the samples it reached and
    the Info editor says "Render cancelled after N samples"; an animation
    render stops at that frame.
  - `tests/render_job_runs_in_the_background.rs` now also checks the
    cancelled shot and Esc's two meanings.
- **The suite's one flake.** `render_thread_refines_latest_view` asserted
  that the first frame it saw from a navigated view was the eighth-size one;
  the worker keeps only its newest frame and the test polls every 5 ms, so
  on a fast pass it saw the quarter first. It now asserts what a poll can
  know — the first frame seen is under full size and the sizes never shrink
  — and the exact ladder stays in
  `a_navigated_view_starts_at_an_eighth_and_doubles_to_full`.
- **The machine's tree.** Eight files differed from batch 99's record with
  modification times older than that batch's archive, so the `-newer` scan
  missed them and the full checksum pass found them: the other agent lit the
  popover anchors while open (`with_active(open)` on the filter, mode-bar,
  options, path-bar and status-log anchors — its `on_state_budget` pair now
  passes), finished `jobs.md`, and changed `phiecs`. Its copy of
  `part/filter/mod.rs` had come from before the X-ray toggle (batch 94), so
  that one was merged by hand: the toggle stays, the anchor lights. Seven
  tests fail on the merged tree, all the machine's work in progress: the
  overlay family's key, the Preferences palette override and its rows'
  actions, a region's host corners, the layout golden, the activity bar's
  treatment and the Outliner's guide lines. The golden re-recorded here is
  identical to batch 99's: nothing in this batch moves the layout (the
  status bar's render job shows only while a render runs).
- **Not yet:**
  - ~~the Rendering workspace's Image editor still asks for a window frame
    every vsync while a render runs~~ — batch 101;
  - "apply filters" and "add into a matrix" from the report are not
    understood yet — Blender has no filter step on a Render Result outside
    the Compositor, so this needs the user's meaning;
  - ~~the pick tree is still built on the first click, not as the file
    loads~~ — batch 101;
  - a scene edited after the Rendering workspace rendered shows the old
    result until F12.

## Batch 99 — why a large scene was slow to orbit and to click, and what now draws when

The scene in the report: 571 objects, 7.5 million vertices, 15 million
faces, in Rendered shading, a floor of millions of vertices selected.
Measured on a synthetic scene of that shape (`tests/an_orbit_does_not_
project_the_whole_mesh.rs`, and a scratch probe of 200 objects and 4
million triangles), four things made every mouse move or click slow. Each
is fixed and tested.

- **The selected object's outline projected every vertex, every move.**
  The outline round a selected object is the convex hull of its projected
  vertices (`mesh/hull.rs`). On each view change the floor's millions of
  vertices were transformed, projected and sorted on the UI thread, in a
  debug build, before the frame could be drawn.
  - A projection is linear, so the hull of the projected vertices is the
    hull of the projected *3D hull* vertices, and a vertex farthest along
    some direction is on the 3D hull. `mesh/extreme.rs` finds the farthest
    vertex along each of 192 directions once per mesh (kept by the mesh's
    address) and the outline projects those — a few hundred points, within
    a percent of the exact hull, under the outline's own width.
  - Measured: an orbit frame with a million-vertex object selected now costs
    the same as with a ten-vertex one (17 ms against 17 ms in the test; it
    was the whole projection before).
- **A click tested every triangle in the scene.** A pick transformed every
  vertex of every object into world space, then walked every triangle of
  each object whose box the ray crossed: seconds per click on this scene.
  - `pick/tree.rs` builds phiray's BVH (`Bvh::build_compact`, the tracer's
    own builder, which is optimised) over each mesh's triangles in the
    mesh's own space, once per mesh. A pick takes the ray into object space
    (`M⁻¹o + t·M⁻¹d`, the same `t`) and walks a few dozen nodes. Box select
    (`pick/occlude.rs`) uses the same trees.
  - Measured: on 200 objects of 20 000 triangles, the first click 6 ms (the
    tree), the next 0.1 ms. A mesh under 64 triangles is still walked whole.
- **Every view change traced the full image before anything new showed.**
  The tracer's first sample for a new view was at full size: on a large
  scene in Rendered shading, tens to hundreds of milliseconds before the
  first image, with the old image stretched in place (batch 95) meanwhile.
  - Cycles starts a navigated viewport small (`start_resolution`) and
    doubles it each pass. The render thread now does the same
    (`rendered/worker.rs`, `BCoarse`): a navigated view's first sample is
    an eighth of the size (a quarter for Solid and Wireframe, which are one
    dispatch), then a quarter, a half, full, where refinement goes on. The
    first image lands about sixty times sooner; the canvas stretches it.
- **The whole window redrew sixty times a second while the image refined.**
  While a view refined (up to 1024 viewport samples in Rendered), the
  viewport asked for a new frame every vsync to poll for the next sample, so
  the whole window — Outliner, Properties, headers — was laid out and
  painted sixty times a second for images that land a few times a second.
  In a debug build that is 25–40 ms a frame of the UI thread, which is where
  the clicks and drags queue.
  - The render thread now flags a landed frame (`rendered::FRESH`), and a
    task in the app (`BPhiberApp::ensure_ray_ticker`, 8 ms) draws the window
    only then, ending when the image settles. The window still redraws
    every frame while a resize waits (`settle.rs`), which is brief.
- **"Device: CPU" did not mean what it said.** The Render tab's Device row
  was Cycles' `cycles.device` read from the file (CPU / GPU Compute), which
  phiray never reads; the file said CPU. It now shows and sets the card
  phiray draws on — Auto, naming the card it takes, or any the machine has —
  over the same `ctx.gpu` key as Preferences ▸ System ▸ Render Devices
  (`pref/device.rs::BRenderDevices::control`).

Also in this batch:

- **The Rendering workspace opens on a render.** Switching to Rendering with
  a scene loaded and no Render Result yet renders the scene into the Image
  editor at once — through the active camera, at Output's size, with
  Render's samples and denoise — without F12's window
  (`BPhiberApp::render_on_entry`); so does opening a file while in it. A new
  file drops the old result (`render_job::forget`).
- **A render larger than the GPU binds no longer kills the render thread.**
  The job rendered the file's 1920 × 1080 whatever the device could bind as
  one image; past the cap (128 MiB on the software GPU here, more on most
  cards) the tracer failed a GPU validation check, the thread died and the
  job stayed "running" for ever. The frame is now rendered at the largest
  size of its shape that fits, the log says which (`scene/render_fit.rs`),
  and a thread that dies clears the job (`BRunning`). Tiles are still the
  real answer.
- **Color Management reaches the render.** View Transform, Exposure and
  Gamma on the Render tab were seeded from nothing and read by nothing; the
  viewport and F12 used the file's. They are now seeded from the file, the
  viewport and F12 look through them (`render_set::BColorManagement`), and
  Ctrl S writes them (`Scene.view_settings`, phiphx `BlendEdits.view`).
- **Auto keying works.** The record button and its Add & Replace / Replace
  popover stored a setting nothing read. A G, R or S kept in the viewport
  with it on now keys what it changed, at the shown frame, as Blender's
  `autokeyframe_object` does — checked against Blender 5.0.1 with `bpy`
  (`scene/action/autokey.rs`):
  - only the transform's channels: G keys Location, R the rotation the
    object's mode keeps, S Scale;
  - Only Insert Needed, on by default for auto keying: a curve already at
    the value gets no key, a curve with no keys always does — a second G
    along X keys X alone;
  - Replace only replaces a key standing on the frame; with none, nothing.
- **Delete Keyframes (Alt I) and Clear Keyframes** in Object ▸ Animation:
  the selected objects' keys on the shown frame, or all of them, through the
  Dope Sheet's steps (batch 98).
- **The tests:** `mesh/extreme.rs` (the extremes' hull reaches as far as the
  whole mesh's in three views, found once), `pick/tree.rs` (the tree finds
  what the whole walk finds through a moved and scaled mesh, misses past the
  edge and behind the ray), `rendered/tests.rs` (a navigated view's frames
  arrive small and grow: 6 → 12 → 24 → 48 px), `scene/action/autokey_tests.rs`
  (Blender's curves for G, R, S, Only Insert Needed, Replace, off),
  `tests/an_orbit_does_not_project_the_whole_mesh.rs`,
  `tests/the_rendering_workspace_opens_on_a_render.rs`.
- **The machine's tree.** Since batch 98 the other agent changed nineteen
  files — the Preferences controls (`props/pref/control/*`), the tree rows
  and their guides, buttons, the semantic set and its effect styles — added
  `one_answer_per_state.rs` and re-recorded the layout golden. None of them
  touch this batch's files, and each began from the copy batch 98 left.
  Nine tests fail on the merged tree, all in that work: the four of batch
  98 (the overlay family's key, the palette override, a region's host
  corners, its golden), its `on_state_budget` pair, and three new ones —
  `preference_rows_dispatch_canonical_edit_and_reset_actions`,
  `the_activity_bar_takes_the_toolbars_treatment` and
  `the_hierarchy_lines_run_the_whole_list`, whose code changed without
  their tests. Re-recorded here, the golden differs from batch 98's only in
  the Preferences and splash modals, the machine's controls.
- **Not yet:**
  - ~~tiles, so a frame larger than the device binds renders whole~~ —
    batch 100;
  - ~~the pick tree is built on the first click (one second on a two-million
    triangle mesh in a debug build); it could be built as the file loads~~ —
    batch 101;
  - the Rendering workspace renders once; a scene edited since shows the
    old result until F12;
  - Blender's auto-keying warning in the status bar, keying sets (K), and
    Alt I's confirmation.

## Batch 98 — the Dope Sheet edits keys: pick, move, box, delete, and Blender opens the result

- **What was missing.** The Dope Sheet drew every key but could not touch
  one. A press anywhere in it scrubbed. Select ▸ All, None and Invert, Key
  ▸ Transform ▸ Move and Key ▸ Delete Keyframes only said "not available
  yet". The first item on the animation list
  (`animation-rendering-status.md`) was this.
- **Picking** (`scene/action/dope/`, `BKeySelection`, kept in
  `ctx.keys.selected` so Undo steps it). A row's diamond stands for every
  key of the row's channels at one frame, so a pick is one object's key
  column. The summary's diamond picks every listed object's column.
  - **Click** (`action.clickselect`): the column under the pointer, within
    7 px, replaces the selection; Shift flips it in or out; a click on
    nothing deselects every key.
  - **Box** (`action.select_box`): a drag begun on nothing sweeps a box over
    the frames and rows it spans, drawn in the selection's shade, added to
    what was picked.
  - **A, Alt A, Ctrl I** and the Select menu's All, None, Invert.
  - Picked diamonds draw in `TH_KEYTYPE_KEYFRAME_SELECT` (the warm accent,
    `BAnimColors::key_select`); a summary diamond lights when any of its
    columns is picked, as `ActKeyColumn.sel` does.
- **Moving** (`transform.transform` in `TIME_TRANSLATE`). A drag begun on a
  key moves the picked keys (picking it first when it was not), and G does
  the same with the pointer until a click.
  - Whole frames, measured from where the move began, so nothing drifts.
  - Escape or the right button puts everything back; the whole move is one
    undo step, as a marker grab is.
  - Each moved key takes its handles with it. A key standing where a moved
    one lands is dropped (`posttrans_fcurve_deletedupes`), then each curve
    is sorted and its handles worked out again
    (`BKE_fcurve_handles_recalc`).
- **Deleting** (`action.delete`, X and Delete): the picked columns' keys go,
  and a curve left with no key goes with them
  (`ANIM_fcurve_delete_from_animdata`).
- **Over the Dope Sheet, G, X and Delete are its own.** The Marker menu's G
  and the Channel menu's X no longer take them there (`app/keymap/set.rs`);
  Escape cancels a key move first.
- **One ordered list of key edits** (`scene/action/retime.rs`, `BKeyStep`).
  A move and a delete are steps over a slot's curves, kept in
  `ctx.scene.keyed` with the inserts I makes, so an insert, a move and an
  insert again replay as they were made. A key move works on the keys as
  they stood when it began (`BActionScene::raw`), so the steps it replaces
  as the pointer moves never compound.
- **Rows read the slot the object is animated through** (`BKeyTarget`): its
  binding in each action it uses, else the action's only slot. Before, an
  object's row showed every key of every slot in its actions, so an action
  shared by two objects showed each the other's keys too. The Dope Sheet's
  summary is its rows' columns together.
- **Saving** (`BSceneActions::edits`, phiphx `write/keys.rs`). A save now
  writes every curve of a touched slot that differs from the action before
  the session's edits, and takes out of the slot's channelbag each curve the
  session emptied (`BlendKeyInsert::removed`; the curve's blocks are left
  unreferenced, which Blender's reader never reads).
- **Checked against Blender.** `tests/dope_sheet_edits_keys.rs` keeps each
  save, and `phiphx/tests/truth/blender_dope_keys.py` opens them in Blender
  5.0.1:
  - the key at 20 dragged to 30: three curves keyed at 1 and 30, and
    Blender puts the Cube at x 2.7632 at frame 25, as phiber's curve does
    (held to 0.0001 in `dope/tests.rs`);
  - the keys at 1 deleted: three curves keyed at 20 alone;
  - every key deleted: CubeAction with no curves;
  - in every file, Blender's own handle recalculation (`FCurve.update()`)
    moves no handle at all.
- **The tests:** `retime_tests.rs` (a move carries keys and handles and drops
  what it lands on; a delete closes up the slots' runs; the steps round-trip
  in order), `dope/tests.rs` (columns per slot, clicks, Shift, boxes, the
  Select menu, a move from where it began, Escape, delete), and
  `tests/dope_sheet_edits_keys.rs` (in the app: click, drag, Ctrl Z, A, Alt
  A, G and Escape, a box, the summary, X, Ctrl S twice).
- **The machine's tree.** Since batch 97 the other agent made a popover as
  wide as its rows (`view/prim/menu.rs`, `view/draw/split.rs`, new test
  `a_popover_is_as_wide_as_its_rows`), made the view snapshot print a
  button's and a menu row's on-state (`view/text.rs`) with a new budget
  test for controls that show it only by glyph (`on_state_budget`), and
  re-recorded the layout golden. Each of its files began from the copy batch
  97 left. Its budget test still finds five popover anchors drawn idle while
  open and one ticking control without its fill, and its golden still
  differs from its own tree by the same 4900 lines of work in progress as at
  batch 96; re-recorded here, the golden moves only in the Preferences and
  Profile modals' popover widths, the machine's change. Three more of its
  files (`theme/semantic/set.rs` and two tests) landed while this batch's
  suite ran; they touch none of this batch's files and come in with the
  next.
- **Not yet:**
  - the Graph editor's keys (selecting and moving keys and handles);
  - per-F-curve channel rows, so a key is picked per channel as in Blender;
  - X's confirmation menu (it deletes at once, as Delete does);
  - Extend (E), Scale (S), Slide, Snap, Mirror, Copy / Paste, Duplicate,
    keyframe and handle types, interpolation;
  - Box Select (B) armed from the keyboard, and Circle and Lasso;
  - saving key edits into a file older than Blender 4.4's layered actions
    (THYMUS is a 3.6 file): the writer still refuses those.

## Batch 97 — the viewport tips as well as turns; Output's resolution is what F12 renders and saves

- **Why the Z axis would not move.** A drag on the viewport read only the
  pointer's movement across (`canvas/view3d/orbit.rs`) and turned the view
  about the world's Z axis. Up and down did nothing, so the view could
  never tip: X and Y swung round the navigation gizmo while Z stood straight
  up. Only Numpad 8 / 2 and the gizmo's arrows tipped it, 15° at a time,
  and they stopped at the poles.
- **The turntable, as Blender orbits** (`scene/orbit.rs`, `BTurntable`;
  `view3d.rotate` with Orbit Method Turntable, `viewrotate_apply`):
  - across turns the view about the world's Z axis and up and down tips it
    about its own horizontal axis, half a degree a pixel. Dragging down
    brings the top of the scene into view, toward Top;
  - the tip goes on over a pole: past 90° the view is upside down, as
    Blender lets it be, and one turn wraps. Blender then turns the across
    direction round for that drag (`ViewOpsData::reverse`), so the scene
    still follows the pointer. Here too;
  - the pitch is no longer clamped to ±90° anywhere: the drag, Numpad 8 /
    2 and the gizmo's arrows all go over the top (`key.rs`, `view_op.rs`);
  - leaving the camera view, the drag starts from the camera's yaw and
    pitch.
- **A jump fixed on the way.** gpui hands a drag's start as an offset into
  the canvas, and the orbit measured the pointer from the window. Across,
  every orbit began with a jump of half a degree per pixel of the canvas's
  offset in the window. Up and down, the jump would have been tens of
  degrees. The pointer is now measured from the canvas too.
- **The Output panel's resolution is used** (`scene/render_set.rs`,
  `BResolution`). Format ▸ Resolution X, Y and % were seeded from the file
  and editable, but nothing read them back:
  - F12 rendered the file's size whatever the panel said. It now renders
    the panel's (`render_job::plan_at`);
  - the camera's frame in the camera view, and the camera's outline in the
    viewport, take their shape from the panel. The outline's memo counts the
    resolution (`extra/memo.rs`), so a change shows at once;
  - Ctrl S writes the three back (`Scene.r.xsch / ysch / size`, phiphx
    `BlendEdits.resolution`), only when they differ from the file's.
- **Checked against Blender.** A 3.6, a 4.2 and a 5.0 file saved at 300 ×
  400, 50 %, were opened in Blender 5.0.1 (`bpy`), and every one reads back
  300, 400, 50 with all its objects.
- **The tests:**
  - `scene/orbit.rs`: down tips toward Top, across turns about Z only, the
    view stays continuous through the pole and is upside down past it, the
    across drag reverses there, and a pitch wraps into one turn;
  - `tests/viewport_orbit_tips_and_turns.rs`: a drag in the app starts with
    no jump, turns 20° for 40 px across, tips 30° for 60 px down, and goes on
    past 90°;
  - `render_set.rs`: the panel over the file, and what a save writes;
  - `tests/output_resolution_reaches_the_render.rs`: the factory scene set
    to 300 × 400 at 50 % renders 150 × 200 on F12, scopes the camera frame to
    it, and Ctrl S writes it.
- **A full audit of the Animation and Rendering workspaces** is in
  `animation-rendering-status.md`, beside this file. It lists what works,
  what is on screen but inert, and what is missing, against Blender 5.0, and
  ranks the next ten tasks for each.
- **The machine's tree.** Since batch 96 the other agent folded the
  activity strip's tone and fill into one `panel_fill` (`card.rs`, back
  under its 200-line cap) and updated the three tests that read it. Four of
  batch 96's eight failures are gone. Each of its files began from the copy
  batch 96 left.
- **Not yet:**
  - dragging the navigation gizmo to orbit, as Blender's can be dragged;
  - Preferences ▸ Navigation: Trackball, sensitivity, Auto Depth and
    Orbit Around Selection;
  - the colour-management panel is still not read: F12 uses the file's view
    transform, exposure and gamma.

## Batch 96 — the 3D cursor moves: Shift + right-click, the Snap menu, Shift C, and it saves

- **What was missing.** Batch 94 drew the file's 3D cursor, but nothing
  moved it, and new objects always went to the world origin. In Blender the
  cursor is where Add puts things and what the Snap menu snaps to, so a
  scene could not be built up the way a Blender user builds one.
- **One owner for the cursor** (`scene/cursor.rs`, `BCursor`).
  - The cursor is its place and its turn. The file's cursor (phiphx
    `BlendCursor`, batch 94) is read with the file. Once the session moves
    it, it lives in one scene key over the file's (`ctx.scene.cursor`,
    `x,y,z,w,qx,qy,qz`), so Undo puts it back and the overlay draws it from
    there (`guide/mod.rs`).
  - `BCursorRule` runs the operators on `ctx.op.cursor`, after the scene
    edit rule.
- **Shift + right-click places it** (`view3d.cursor3d`, with Blender's
  defaults `use_depth` and `orientation='VIEW'`; `canvas/view3d/hit.rs`,
  `BHit::on_cursor`).
  - Over a mesh, it lands on the surface under the pointer, found with the
    same ray as a click-select.
  - Over empty space, it lands on the plane through the cursor that faces
    the view, so it keeps its depth.
  - Either way it turns to the view: its X, Y and Z become the view's
    right, up and back.
- **Object ▸ Snap** (`VIEW3D_MT_snap`, in Blender's order):
  - **Cursor to Selected** moves it to the selected objects' median point
    (`snap_curs_to_sel_ex`);
  - **Cursor to World Origin** and **Cursor to Active**;
  - **Selection to Cursor** moves each selected object's origin onto the
    cursor and keeps its turn and scale. The object's own part takes the
    difference (`local · world⁻¹ · world′`), so a child moves correctly
    under its parent. It goes through the Transform panel's key, so it
    saves and undoes like G;
  - a snap moves the cursor and never turns it;
  - Selection to Grid, Selection to Active and Cursor to Grid are listed
    and say they are not available yet.
- **View ▸ Align View ▸ Center Cursor and Frame All** (Shift C,
  `view3d.view_all(center=True)`) puts the cursor at the origin and frames
  everything.
- **Add puts new objects at the cursor** (`scene/edit/rule.rs`), aligned
  to the world, as `object.add` does with its default Align.
- **The cursor saves** (phiphx `blend/write/cursor.rs`,
  `BlendEdits.cursor`).
  - Ctrl S writes the cursor's place, and its turn in the rotation mode the
    file already keeps it in: a quaternion, an axis and angle, or Euler
    angles in any of the six orders (`View3DCursor::set_matrix`). So the N
    panel shows the cursor as the user left it set up.
  - A file whose cursor the session never moved is written as it was.
- **Checked against Blender.**
  - Five files, one per rotation mode (quaternion, axis-angle, XYZ, YZX,
    ZXY), were saved with a moved and turned cursor. Blender 5.0.1 (`bpy`)
    read every one back to within 5 × 10⁻⁷ of the matrix written.
  - phiphx `write/cursor.rs` writes and reads back the cursor of
    `fixtures/overlays.blend`, which keeps it in ZXY.
  - phiber `scene/cursor_tests.rs`, over the same fixture:
    - the file's cursor at (1.5, −2, 0.5) until the session moves it;
    - Cursor to Active puts it on Child at (2.999, 1.310, 0.5), as Blender
      poses it at the saved frame;
    - Cursor to Selected puts it at the median of Child and Track;
    - Cursor to World Origin keeps its turn;
    - Selection to Cursor puts both objects at (1, 2, 3);
    - an added cube lands at the cursor, and Shift C puts the cursor back;
    - a click on a cube's front face places it at (0, −1, 0) turned to
      the view, and a click past the cube places it on its own plane;
    - a save writes the cursor only once it has moved.
- **Should phiber be built optimised?** Measured while looking into the
  zoom lag: phiber built at opt-level 1 (its dependencies are already
  optimised) gives frames only about 10–30 % faster on THYMUS, and rebuilds
  take about four times as long. It stays at opt-level 0 for development.
  `cargo run --release` is still the way to judge speed.
- **The machine's tree.** Since batch 95 the other agent:
  - painted the welcome screen's backdrop from the theme's ladder, so it
    follows the theme and its mode (`view/draw/brand.rs`, new test
    `the_welcome_backdrop_follows_the_theme`);
  - changed the fullscreen modal (`modal/fullscreen.rs`);
  - fixed `host_header_refinement`'s popover test, one of batch 95's four
    failures;
  - painted the activity bar's secondary strip flat, the card shade
    throughout, since it fills its pane and has no surface to lie on as a
    raised card (`view/draw/card.rs`, new test
    `the_activity_strip_is_card_content`).

  The last two landed while this batch's suite ran and were taken into it
  before the test list was drawn up; `card.rs` began from the copy batch
  95 left. Listed side by side, the two trees hold the same files apart
  from this batch's three new ones.
- **Layout golden.** Re-recorded on the merged tree, it differs from
  batch 95's by exactly the machine's welcome backdrop (two lines, one per
  window size); the new menus are closed when the golden is taken. The
  golden the machine recorded agrees on that backdrop but differs from its
  own tree by about 4900 lines of work not in its files yet, so it is left
  for that work and fails until then.
- **Failing on the machine's own work.** Eight tests fail on the merged
  tree, none from batch 96:
  - `layout_fingerprint`, against the machine's golden (above);
  - the three from batch 95 that are still open: a region's host corners,
    the overlay family's key, and the Preferences palette override;
  - four from the activity-strip change, which landed on the machine
    without its tests: `file_browser_accordion_refinement` (two) and
    `host_header_refinement` read `card.rs` for the old
    `.fill(BFill::Tone(panel_tone(ctx)))`, and `renderer_decomposition`
    finds `card.rs` 8 lines over its 200-line cap.

  One failure was this batch's and is fixed: `viewport_click_selects`
  reads `orbit.rs` for the click pick spelled as it was, so the cursor's
  handler goes before it. On the way, Shift + Ctrl + right-drag (the
  lasso's deselect) no longer also moves the cursor.
- **Not yet:**
  - Cursor to Grid and Selection to Grid (they need the grid's step at
    the view's zoom), and Selection to Active;
  - the N panel's View ▸ 3D Cursor fields;
  - dragging the cursor (Shift + right-drag slides it in Blender);
  - Add always aligns a new object to the world; Blender's Align option
    (View, 3D Cursor) is not offered.

### Where each workspace stands (after batch 96)

| Area | Status | Missing |
|---|---|---|
| Layout | Mostly done | Solid cavity and shadows; snapping; bones hidden behind meshes, filled bones and B-Bones; Cursor to Grid |
| Rendering (F12, Image, settings) | Mostly done | Motion blur, film transparency, adaptive sampling, render border; Image ▸ New |
| Animation: Timeline | Done | – |
| Animation: Dope Sheet / Graph | Partial (display done) | Editing keys and channels; Graph view tools |
| Animation: NLA / Drivers | Partial | Moving, scaling and tweaking strips; drivers with several variables |
| Rigs and constraints | Mostly done | Spline IK, Shrinkwrap; Pose mode |
| Shading | Partial | Procedural shader nodes are folded to a flat colour; nodes cannot be edited |
| Outliner | Mostly done | All View Layers; reordering rows by dropping between them |
| Properties | Partial | Scene, World, Data, Modifier, Physics and Constraint tabs do not read the file yet |
| UV Editing | Partial | Selecting, moving and unwrapping UVs |
| Modeling / Sculpt / Texture Paint | Early | Edit, sculpt and paint tools do nothing yet |
| Compositing / Geometry Nodes / Scripting | Early | No node evaluation; no Python |
| Open / save | Mostly done | Materials of added objects; Link and Append |


## Batch 95 — playback shows the rig moving; zoom and resize keep the meshes in place

- **Why the character stood still during playback.**
  - The poser (`scene/pose/worker.rs`) poses rigged meshes on a thread of
    its own. The render pass used to draw a posed frame only when it was
    exactly the frame the playhead was on.
  - In a debug build THYMUS takes longer to pose than playback takes to
    step a frame, and a late tick steps several frames at once
    (`app::playback`). So by the time the poser finished a frame, the
    playhead had moved on.
  - The poser then dropped the finished frame as no longer wanted, and the
    pass kept drawing the meshes as read. The character never moved for as
    long as playback ran.
  - Now the shelf keeps the frame it posed last, and the pass draws that
    while the frame it wants is on its way, as Blender drops frames. The
    test `a_playhead_faster_than_the_poser_still_shows_the_rig_moving`
    (`tests/rig_poses_off_the_ui_thread.rs`) steps THYMUS's playhead seven
    frames every 5 ms. Before the fix it drew 0 poses in 20 seconds; now it
    reaches three different poses within the test's run.
- **Why the meshes lagged on a zoom or a window resize.** The tracer's
  image for a new view lands a frame or more after the view changes.
  - Until it landed, the old image was drawn over the canvas as it had been
    framed. A zoom showed the meshes at their old size, while the outlines,
    bones and other overlays were already at the new one.
  - During a window drag the old image was stretched to the new shape. The
    traced size is held until the drag stops (`settle.rs`), so the meshes
    were squashed for the whole drag.
  - Now every view posted to the tracer is noted with its zoom, its pan and
    the traced image's shape (`rendered/reframe.rs`). The image on screen is
    drawn where the current view puts it: one scale and one shift, because
    on the canvas a unit is `SCREEN · zoom · width` pixels across, centred
    on `0.5 + pan` of the width.
  - A zoom therefore resizes the meshes the moment it happens, a pan moves
    them, and a window resize keeps them in shape and in place. Each image
    is then replaced as soon as the tracer delivers the one for the current
    view.
  - The mapping is exact for an orthographic view and at the pivot's depth
    in a perspective one. An orbit cannot be reached by a scale, so for an
    orbit the old image is drawn as it was framed until the new one lands.
  - The canvas clips the image (`paint_image`'s visible bounds).
- **Frame cost still matters.** A debug build is unoptimised: THYMUS takes
  about 40–60 ms a frame here (`docs/frame-cost.md`). Every view change
  waits at least one such frame before the new trace is shown. A release
  build (`cargo run --release`) is roughly 5–10× faster.
- **The machine's tree.** Since batch 94 the other agent:
  - gave submenu rows an open state and checked menu entries the active
    fill;
  - let a menu popover take the height the window allows (`AppKey::WindowHeight`);
  - made F12 render on the theme's viewport shade and grid instead of two
    fixed greys;
  - extended the viewport-lightness guard
    (`tests/viewport_lightness_regression.rs`);
  - stopped the tracer's Material Preview and Rendered sky from going
    through the view transform (`shade.wgsl`), so a file with Filmic or
    AgX no longer darkens the theme's background;
  - adjusted the chrome geometry (`tests/chrome_geometry.rs`).

  Two of its files started from the copies before batch 94:
  `canvas/view3d/mod.rs`, without the overlays (`guide`), and `app/op/render.rs`,
  without `grid_hide`, so phiber no longer compiled. The merge
  (`git merge-file` against batch 92b's copy) keeps both and puts the
  machine's changes on top.
- **Failing on the machine's own work.** Two of the four failures are new:
  `preferences_show_effective_palette_default_and_valid_override`, after
  the palette changes, and `host_switch_popup_opens_inside_its_area`, which
  still expects the popover's old `max_height`. The layout golden the
  machine recorded predates its own tree: it has neither batch 94's Tool tab
  rows nor the X-ray button. Re-recorded here, it differs from the
  machine's by exactly those.
- **Not yet:**
  - an orbit still shows the old image until the new one lands;
  - the poser poses the frames after the playhead, not the frame the
    playhead will have reached when it finishes, so a slow build plays at
    the poser's rate.

## Batch 94 — the viewport's overlays: the popover takes effect, the 3D cursor, origins, relationship lines

- **What was wrong.**
  - The viewport header's Overlays and Gizmos popovers had no effect, and
    neither did the same rows in the Tool tab. Nothing read their keys, so
    unticking Floor, Extras, Bones or Overlays changed nothing, and Show
    Gizmo left the gizmo in place.
  - Blender draws three things over every scene that Phiber did not: the 3D
    cursor, the objects' origins and the relationship lines. It also draws
    the Z axis when asked.
  - Blender keeps these switches per 3D view in the file. A file opened in
    Phiber with Phiber's own switches, not the file's.
- **One reader for the switches** (`scene/overlay.rs`, `BOverlays`).
  - The canvas, the tracer, the picks and the read-out all read the
    popovers' keys through this one place. The master switch hides every
    overlay, as `V3D_HIDE_OVERLAYS` does, but leaves the gizmos, which have
    a master of their own.
  - New rows, with Blender's defaults: Text Info, Statistics (off), 3D
    Cursor, Outline Selected, Origins, Origins (All) (off), and the Navigate
    gizmo.
  - The overlay popover now lists its rows under Guides and Objects, as
    `VIEW3D_PT_overlay` does.
- **What each switch does:**
  - **Floor and each axis**, in the CPU painter (`floor.rs`) and in the
    tracer. The tracer gets them as `Camera::grid_hide`, which takes the
    place of a padding word. An axis turned off leaves an ordinary floor
    line where it was.
  - **Z axis.** The tracer draws it as a line through the origin, tested
    against what each ray hits, so meshes hide it. The CPU painter draws it
    with the floor.
  - **Text Info** shows the view's name, collection and frame lines.
  - **Statistics** shows the counts line. It ships off, as in Blender; the
    status bar still shows the counts.
  - **Extras and Bones** hide cameras, lights and empties, and bones.
    Whatever they hide can no longer be clicked or box-selected either.
  - **Outline Selected** turns off the selection outlines.
  - **Navigate** removes the orientation gizmo and the view buttons.
- **A file opens with its own view's switches.**
  - phiphx reads them from the 3D view Blender restores (`scene/overlay.rs`,
    `BlendOverlay`, keyed by RNA name). Blender keeps them as bits across
    six `View3D` fields: `overlay.flag`, `flag`, `flag2`, `gridflag`,
    `gizmo_flag` and `gizmo_show_object`.
  - Load UI writes them where the popovers read them.
- **The 3D cursor** (`guide/cursor.rs`, as `overlay_cursor.hh` draws it).
  - phiphx reads `Scene.cursor`: its place and its turn, in any rotation
    mode.
  - The viewport draws a ring 20 px across, in alternating red and white
    segments.
  - It also draws a crosshair along the cursor's own axes, in 3D, sized to
    one widget unit at the cursor's depth. Each axis is its axis colour at
    the tip and the overlay white toward the middle, with an empty centre;
    the negative halves are paler.
  - Nothing is drawn while the cursor is behind a perspective eye.
- **Origins** (`guide/origin.rs`, as `overlay_origin.hh` draws them).
  - Each origin is a 7 px dot ringed with one pixel of the overlay light.
  - The active object's dot is in the active colour, whether or not it is
    selected. Each selected object's dot is in the selection colour.
  - With Origins (All), every other object's dot is in the wire colour.
- **Relationship lines** (`scene/extra/relation.rs`, as
  `overlay_relation.hh` draws them).
  - phiphx's solve now records where each object hangs from its parent.
    That is the place of the parent matrix (`parent_display_origin`): the
    parent object, a bone's tail, a bone's deform with relative parenting,
    or the point on a curve parent's path.
  - It also records each open constraint's targets as the constraint reads
    them, in its target space: an object, a bone at `head_tail`, the point
    on a Follow Path curve, an Armature constraint's bones at their heads,
    and the custom-space object. Clamp To, a folded constraint and the
    Action constraint draw no line, as in Blender.
  - The viewport draws them dashed. A parent's line is in the wire colour
    and a constraint's in the Z axis colour.
  - A line to a hidden parent is left out, and a deleted object has none.
  - An object moved by G, R, S or the Transform panel carries its end of
    every line.
  - A line can never be clicked.
- **Checked against Blender.** `truth/blender_overlays.py` (Blender 5.0.1)
  makes `fixtures/overlays.blend`:
  - every overlay and gizmo switch set away from its default in the Layout
    view;
  - the cursor moved and turned in ZXY order;
  - children of an object, of a bone's tail, and of a bone's deform;
  - Track To, Copy Location at a bone's middle, Follow Path, a folded Damped
    Track, Clamp To, and a hidden parent;
  - its relationship lines at three frames.

  Blender has no RNA for the display origin. The truth takes it from
  `matrix_world · matrix_basis⁻¹ · matrix_parent_inverse⁻¹`, the parent
  matrix Blender used.

  The tests:
  - phiphx matches every switch, the cursor's matrix and every line to
    within 0.0001 (`blend/scene/overlay_tests.rs`);
  - phiber checks the floor and each axis, the tracer's mask, what is drawn
    and picked under each switch, the hidden parent, the moved parent, the
    cursor's place and size, the origins' colours, and that a file opens
    with its view's switches (`guide/tests.rs`, `scene/overlay.rs`,
    `app/load_ui.rs`);
  - phiray's `tests/grid.rs` leaves out the floor and each axis and draws a
    blue Z axis.
- **Layout golden.** Only the Tool tab's Options and Overlays cards changed:
  seven new rows, with everything below them moved down. Nothing else
  moved.
- **The machine's tree.** Nothing changed there since batch 93.
- **Not yet:**
  - moving the 3D cursor (Shift + right-click, Snap);
  - the Geometry section (Wireframe, Face Orientation), Motion Paths,
    Annotations and HDRI Preview;
  - relationship lines for Hook modifiers and rigid-body constraints;
  - a vertex parent's line runs to the parent's origin, not its vertices;
  - the CPU painter draws the Z axis over the meshes;
  - one set of switches serves every viewport, where Blender keeps one per
    view.

### Where each workspace stands (after batch 94)

| Area | Status | Missing |
|---|---|---|
| Layout | Mostly done | Solid cavity and shadows; snapping; moving the 3D cursor; bones hidden behind meshes, filled bones and B-Bones |
| Rendering (F12, Image, settings) | Mostly done | Motion blur, film transparency, adaptive sampling, render border; Image ▸ New |
| Animation: Timeline | Done | – |
| Animation: Dope Sheet / Graph | Partial (display done) | Editing keys and channels; Graph view tools |
| Animation: NLA / Drivers | Partial | Moving, scaling and tweaking strips; drivers with several variables |
| Rigs and constraints | Mostly done | Spline IK, Shrinkwrap; Pose mode |
| Shading | Partial | Procedural shader nodes are folded to a flat colour; nodes cannot be edited |
| Outliner | Mostly done | All View Layers; reordering rows by dropping between them |
| Properties | Partial | Scene, World, Data, Modifier, Physics and Constraint tabs do not read the file yet |
| UV Editing | Partial | Selecting, moving and unwrapping UVs |
| Modeling / Sculpt / Texture Paint | Early | Edit, sculpt and paint tools do nothing yet |
| Compositing / Geometry Nodes / Scripting | Early | No node evaluation; no Python |
| Open / save | Mostly done | Materials of added objects; Link and Append |

## Batch 93 — rigs drawn in the viewport: bones and their custom shapes

- **What was missing.** Blender draws an armature in Object mode as its
  shown bones: each one an octahedron, a stick, or the custom shape a rig
  gives it (`overlay_armature.cc`). Phiber drew no armature at all, so
  THYMUS's rig, which Blender shows as 54 Rigify controls (the root, the
  torso, the head box, the neck circle, the hand and foot shapes and so on),
  showed nothing, and neither did a plain octahedral rig.
- **Which bones show** (phiphx `rig/display.rs`, as
  `animrig::bone_is_visible`): a bone that is not hidden and is in a visible
  bone collection whose parents are visible too, or in no collection. While
  a collection is soloed, only the soloed collections' bones show.
  - The collections come from the `collections` list the file keeps.
    Blender saves the flat `collection_array` pointer as 0, so that pointer
    is not read.
  - A 3.x file has no collections (THYMUS is a 3.6 file). Blender makes them
    from the file's bone layers and bone groups as it opens it
    (`versioning_400.cc`): one per used layer, visible when the armature
    shows that layer, and one per bone group, hidden. Phiber does the same.
  - Blender 5.0 hides a bone in Object and Pose mode by its pose bone's flag
    (`PCHAN_DRAW_HIDDEN`), not the bone's own `hide`, which is Edit mode's.
    A file saved before 5.0.54 keeps that flag on the bone instead
    (`BONE_HIDDEN_P`), and Blender copies it across when it opens the file.
    Phiber reads it from wherever the file's version keeps it.
- **Where each bone's shape stands** (its display matrix, as
  `draw_bone_update_disp_matrix_default` / `_custom_shape` build it):
  - an octahedral bone is Blender's six-point octahedron and a stick is a
    line from head to tail, each scaled by the bone's length on its posed
    matrix;
  - a custom shape stands on the posed matrix of its own bone, or of the
    bone its Override Transform names (`custom_tx`). It is then moved,
    turned (XYZ) and scaled by the bone's custom-shape offset, rotation and
    scale, and scaled by the bone's length unless Scale to Bone Length is
    off. The shape is its mesh's edges, or the edges of its faces when the
    mesh stores none. The shape object's own transform does not count, as
    in Blender;
  - the armature's display type decides, and a bone's own display type
    overrides it. Stick and Wire bones draw as a line. B-Bones and envelopes
    draw as octahedra for now. An armature with Shapes off
    (`ARM_NO_CUSTOM`) draws octahedra in place of its custom shapes.
- **THYMUS's widgets.** THYMUS repeats addresses across its blocks. Read
  from the wrong block, the widget meshes came out as starbursts of lines
  across the rig. The shapes are now read inside the armature object's
  scope, then the mesh's, as the pose is (`BlendFile::scoped`). All 54 of
  its shapes now match Blender's edge counts and edge coordinates.
- **In the viewport** (`scene/extra/bone.rs`): every armature is drawn with
  the lines the viewport already strokes for cameras, lights and empties.
  - The bones stand where the poser has them at the shown frame. Before the
    poser's first result arrives, they stand in the pose the file was saved
    in. To make that possible, every armature is now a rig source, not only
    the ones that deform a mesh, and a rig that never moves is solved once.
  - The lines use the object's wire colour, or the selection colours when
    the armature is selected or active, as Blender's `const_color` does.
  - Clicking a bone picks its armature, as in Object mode. The armature's
    bones hide with its collection and with the object, and they follow a
    rename or a delete.
- **Checked against Blender.** `truth/blender_armature_display.py` (Blender
  5.0.1) makes `fixtures/armature_display.blend`: an octahedral rig with
  nested collections, a hidden collection, one pose bone hidden and one bone
  hidden only in Edit mode, custom shapes with an offset, a turn and a
  scale, one placed by another bone and one not scaled by its length, and a
  wire bone. Beside it is a stick rig with one collection soloed. The script
  writes each shown bone's display matrix at four frames. It also writes
  THYMUS's 328 bones at three frames and the edges of its 54 shapes. The
  tests match every shown bone to within 0.0001, and every shape's edges
  too (`rig/display_tests.rs`). Phiber's own tests draw the fixture's rigs
  with the right number of lines in the wire and selection colours, and
  draw none for a deleted rig.
- **Frame cost.** THYMUS's shown bones are 1366 lines.
  - They are built once per frame and pose, not once per caller: the
    viewport, its picks and the extras list share one copy until the frame,
    the pose or an edit changes (`scene_extras_now`, `BExtrasMemo`).
  - An object's name, hidden state and colour are resolved once per object,
    not once per line.
  - Each run of one colour is stroked as one path.
  - In a debug build, projecting, stroking and listing them for picks costs
    about 2 ms a frame (0.6 ms list, 0.25 ms project, 0.8 ms stroke). A
    still frame measures within the probe's noise of one without bones
    (`docs/frame-cost.md`).
- **The machine's tree.** Since batch 92 the other agent:
  - gave four more editor headers their box popover: the Image and UV
    editors' Overlays (`IMAGE_PT_overlay`), which replaces a toggle nothing
    read; the Movie Clip editor's Display (`CLIP_PT_display`); and the
    Spreadsheet's Filters (`SPREADSHEET_PT_filter`);
  - split `part/filter/overlay.rs` into `overlay/{mod,set}.rs`, one table
    per family;
  - showed those same boxes as folding cards in the Movie Clip editor's View
    tab and the Spreadsheet's Filter tab;
  - made the render-info test find the bar by its size and place rather
    than its colour.

  `part/filter/mod.rs` came across from an old copy again, without the
  X-ray button, for the fourth time. The merge keeps the button between the
  overlay popover and the shading radio, as `VIEW3D_HT_header` has it. The
  old `overlay.rs` is gone from both trees, and so are two files the
  machine had already retired: `tests/the_node_and_sequencer_headers_end_with_overlays.rs`
  (now `editor_headers_carry_their_box_panels.rs`) and
  `canvas/view3d/object.rs` (an object's name moved to `scene` in
  batch 92). The re-recorded layout golden differs from batch 92's by
  exactly what the machine describes: 10 lines left and 14 arrived. Eight of
  those are the Image and UV overlay button renamed in place, two are its
  anchor's quad, and four are the Spreadsheet's Filters control. Bones add
  nothing to the golden, because no scene is loaded when it is recorded.
  The machine's new `a_family_shares_one_table_and_one_scope` expects a
  box's key to encode as `overlay.<family>.<box>`, but `BPath::encode`
  starts every key with `ctx.`, so the test fails on both trees until one of
  them changes.
- **Not yet:**
  - B-Bone segments and envelopes, which draw as octahedra;
  - filled (solid) bones. Every bone draws as wire, which Blender does only
    for a wire object or a wire bone;
  - custom shapes that are not meshes (an empty, a curve);
  - depth: an armature not set to In Front should hide behind the meshes in
    front of it, but here it always draws on top;
  - bone colours (Pose mode only in Blender), and Pose mode itself.

### Where each workspace stands (after batch 93)

| Area | Status | Missing |
|---|---|---|
| Layout | Mostly done | Object origins; Solid cavity and shadows; snapping; bones hidden behind meshes, filled bones and B-Bones |
| Rendering (F12, Image, settings) | Mostly done | Motion blur, film transparency, adaptive sampling, render border; Image ▸ New |
| Animation: Timeline | Done | – |
| Animation: Dope Sheet / Graph | Partial (display done) | Editing keys and channels; Graph view tools |
| Animation: NLA / Drivers | Partial | Moving, scaling and tweaking strips; drivers with several variables |
| Rigs and constraints | Mostly done | Spline IK, Shrinkwrap; Pose mode |
| Shading | Partial | Procedural shader nodes are folded to a flat colour; nodes cannot be edited |
| Outliner | Mostly done | All View Layers; reordering rows by dropping between them |
| Properties | Partial | Scene, World, Data, Modifier, Physics and Constraint tabs do not read the file yet |
| UV Editing | Partial | Selecting, moving and unwrapping UVs |
| Modeling / Sculpt / Texture Paint | Early | Edit, sculpt and paint tools do nothing yet |
| Compositing / Geometry Nodes / Scripting | Early | No node evaluation; no Python |
| Open / save | Mostly done | Materials of added objects; Link and Append |

## Batch 92 — a quick Outliner that shows the pick, sixteen themes, Clamp To

- **Why the viewport was slow.** With THYMUS open, a debug build spent
  about 190 ms a frame in Layout, Animation and Shading, against 16 ms with
  nothing loaded (the machine's slow-frame log showed 75–115 ms). The time
  was the Outliner's, not the viewport's (`docs/frame-cost.md`):
  - listing a collection's objects looked for each one's children by
    scanning the whole level again — 288 rows made about 80 000 look-ups a
    frame. The children are now found once per level
    (`outliner/object_rows.rs`, `BLevel`);
  - every row was built and laid out each frame, though a dozen are in
    view. A long tree now lays out only the rows in view and eight either
    side (`view/draw/tree/window.rs`, `BRowWindow`). One spacer stands for
    the rows above and one for the rows below, so the scroll range, the
    wheel and the scroller are what they were; a list under 64 rows is
    drawn whole;
  - the viewport, its overlay, its extras and its picks each built the
    hidden set again — a walk of the collection tree and two keys per
    object. A render pass now shares one set per kind, answers included
    (`BHidden::begin_pass` / `end_pass` around the tree build and the draw);
    a click or a paint after the pass reads afresh.
  - THYMUS, debug build, 1600×900: a still frame went from 190 ms to
    41–45 ms in Layout, Animation and Shading, a frame change from 202 ms to
    66 ms (Layout) and 78 ms (Animation). What remains of a frame change is
    mostly the viewport's extras and GPUI's layout. Phiber itself is built
    unoptimised in a dev build, so a release build is about 5–10× faster
    again.
- **The Outliner shows the pick.** Picking an object in the viewport opens
  the rows above it — the collections down to the one holding it and the
  parents its row nests under (`part/outliner/reveal.rs`, as
  `outliner_show_active` opens them with `outliner_open_back`) — and
  scrolls its row to the top of the list. A click on the Outliner's own rows
  leaves the list where it is (the list knows when the pointer is over it),
  and so does a file opening. Every pick is counted (`ctx.outliner.reveal`),
  so picking the same object again after scrolling away brings it back.
- **Sixteen themes, and no more.** Moss, Ember and Plum are back with the
  seed, accent, radius scale, sweeps, shadow and seam gap they had before
  batch 40, and now a light mode each (the seed lifted, as every theme's
  is). Silver is back as a theme of its own: its light mode is the silver
  with Tesla's blue, its dark mode that silver lowered as far toward black
  as a dark seed is lifted toward white (`shade::lowered`, a `#313232`
  gunmetal). Apple Dark is back: macOS's dark grey and its dark-mode blue,
  its light mode that grey lifted with the same blue — so it differs from
  Apple only in its light mode. Grey keeps the silver as its light mode and
  Apple its two. `BPaletteName::MAX` is 16, and the tests hold the list to
  it.

  | theme | dark seed | light seed | accent | radii | sweeps | shadow | gap |
  | --- | --- | --- | --- | --- | --- | --- | --- |
  | Moss | `#3A4A3F` | lifted | `#D9A441` (straw) | Round | on, .05 | Large | 8 |
  | Ember | `#4A3A36` | lifted | `#4FB3A9` (teal) | Square | off | Small | 2 |
  | Plum | `#433B57` | lifted | `#1F8A5B` (emerald) | Pill | on, .03 | Extra Large | 10 |
  | Silver | `#313232` (lowered) | `#C4C6C9` | `#3E6AE1` (Tesla) | Medium | off | Small | 6 |
  | Apple Dark | `#38383A` | lifted | `#0A84FF` | Medium | off | Medium | 4 |

  The Preferences list is in the order it had before batch 40: Sphere,
  Graphite, Tokyo, Moss, Ember, Plum, Onyx, Grey, Silver, Apple, Apple Dark,
  Swift, Barbie, Fenty, Cyberpunk, Chanel.
- **Clamp To** (phiphx `rig/constraint/path.rs`, `clampto_evaluate`): the
  owner moves onto the target curve's path at the fraction its place makes
  along one axis of the curve's box — the box's longest unless an axis is
  named — held at the ends, or with Cyclic laid end to end past them. As in
  Blender, the owner's place in the world is measured against the box in
  the curve's own space, and the box is the control points (a Bézier
  point's handles too) grown by their radius (`BCurve::bounds`,
  `BKE_curve_minmax`). Only the owner's place changes.
  - Truth: `truth/blender_clamp_to.py` (Blender 5.0) makes
    `fixtures/clamp_to.blend`: an open Bézier rail (the automatic axis, and
    Y), a NURBS tower under a turning parent, a poly line whose radii widen
    its box, and a closed loop clamped flat and cyclically, one owner after
    Copy Location, each curve moved, turned and scaled, the owners keyed
    through and past the boxes. Every object is within 0.0001 of Blender's
    `matrix_world` at six frames and as read at the saved frame
    (`curve/tests.rs`).
- **The machine's tree.** Since batch 91 the other agent:
  - put the gizmo and overlay popovers in the viewport's header
    (`part/filter/view3d.rs`) over the Tool tab's own keys, whose Blender
    defaults now ship on (a fresh file draws its floor, axes and gizmo);
  - gave the shading radio Blender's four icons;
  - closed an enum picker when its value is chosen (`state/menu.rs`);
  - let the CPU painter stand in until the tracer delivers a frame
    (`tests/the_viewport_always_draws_something.rs`);
  - reshaped the tree row's parts (one slot for the arrow and the type
    icon, the row hover by its shade);
  - gave the status bar the scene's counts, the same line the viewport's
    read-out draws (`scene::stats_line`, `visible_metrics`), and moved an
    object's name to the scene (`scene::object_name`);
  - ended the node editors' and the Sequencer's headers with an overlay
    popover (`part/filter/overlay.rs`), and cleared the build's warnings.
  
  Nine of its files started from old copies: `phiphx/src/blend/write/mod.rs`
  without its `keys`, `nla` and `owned` modules (the four errors phiphx
  stopped compiling with on the machine), `tests/animation_editor_menus.rs`
  without batch 80's Action menu, `part/filter/mod.rs` without the X-ray
  button (twice more), `scene/mod.rs` and `scene/source.rs` without
  batch 86's posing thread (`scene::pose` was no longer compiled)
  and without `BXray`, `scene/shading.rs` without `BXray`,
  `view3d/rendered/mod.rs` without X-ray's see-through, the render-device
  switch and the scene key's shape, `view3d/boxsel.rs` with X-ray back to
  Wireframe only, and `tree/drag.rs` (batch 74's, with a drop colour the
  current one already takes from `BFill`). The merge keeps each of those
  and puts the machine's change on top; the header now reads gizmos,
  overlays, X-ray, shading, as `VIEW3D_HT_header` does. Re-recorded here,
  the layout golden differs from batch 91's by exactly the machine's
  changes — the two popovers (38 captures), the ten overlay toggles renamed
  by their new state (42), the four shading icons (38), the header's right
  pane wider (38) and the X-ray button beside it moved (7), the node
  editors' overlay popover and its pane (12), and the Spreadsheet's footer,
  which holds the status bar's part, 32px wider with the counts (2) — and
  nothing else moved. The machine's own golden also moves the timeline's
  frame fields and the Properties tab fill, from work not in its files yet.
- **A crash on smaller GPUs.** The tracer's image is one GPU binding of 96
  bytes a pixel, and many devices cap a binding at 128 MiB — about 1.4
  million pixels. A viewport traced wider than that (1600 × 966 here) was a
  validation error that ended the render thread; with the machine's CPU
  stand-in the viewport then drew only the CPU painter. The viewport now
  traces no more pixels than the device binds (`Tracer::max_pixels`) and
  stretches the image. `central_reuse`'s idle-painter test waits for the
  tracer's first image before it counts, since the painter now stands in
  until then (it still fails about one run in ten on the software GPU, when
  the tracer delivers nothing in the test's frames).
- **Not yet:**
  - an F12 render larger than that binding (1920 × 1080 is 2.1 million
    pixels) fails the same way on such a device: it needs tiles;
  - Clamp To on a curve that is no path (Blender puts the owner at the
    origin), and a curve's deform modifiers, shape keys and hooks.

### Where each workspace stands (after batch 92)

| Area | Status | Missing |
|---|---|---|
| Layout | Mostly done | Bones not drawn; object origins; Solid cavity and shadows; snapping |
| Rendering (F12, Image, settings) | Mostly done | Motion blur, film transparency, adaptive sampling, render border; Image ▸ New |
| Animation: Timeline | Done | – |
| Animation: Dope Sheet / Graph | Partial (display done) | Editing keys and channels; Graph view tools |
| Animation: NLA / Drivers | Partial | Moving, scaling and tweaking strips; drivers with several variables |
| Rigs and constraints | Mostly done | Spline IK, Shrinkwrap |
| Shading | Partial | Procedural shader nodes are folded to a flat colour; nodes cannot be edited |
| Outliner | Mostly done | All View Layers; reordering rows by dropping between them |
| Properties | Partial | Scene, World, Data, Modifier, Physics and Constraint tabs do not read the file yet |
| UV Editing | Partial | Selecting, moving and unwrapping UVs |
| Modeling / Sculpt / Texture Paint | Early | Edit, sculpt and paint tools do nothing yet |
| Compositing / Geometry Nodes / Scripting | Early | No node evaluation; no Python |
| Open / save | Mostly done | Materials of added objects; Link and Append |

## Batch 91 — objects on curve paths: THYMUS's turntable camera

- **Before.** Curves were not read at all. An object parented to a curve
  stood on the curve object, and Follow Path did nothing. THYMUS's
  turntable camera `turn_cam_02` rides the `Path` Bézier circle, and
  stood about 17 units from where Blender puts it.
- **Curves are read** (phiphx `blend/curve/read.rs`): the `Curve`
  datablock, its splines (`Nurb`) and their Bézier points (`BezTriple`)
  or NURBS / poly points (`BPoint`), with tilt, radius and weight. The
  file keeps a point's tilt under its old DNA name, `alfa`.
- **A spline becomes Blender's bevel list** (`bevel.rs`,
  `BKE_curve_bevelList_make`):
  - Bézier segments by forward differencing at the spline's resolution;
    a segment between two vector handles is one straight step;
  - NURBS by their basis over the saved knots (`BKE_nurb_makeCurve`);
    poly splines as they are;
  - tilt and radius between the points by the spline's interpolation
    (linear, cardinal, B-spline or ease);
  - duplicate points dropped, and a flat closed spline turned to run one
    way round, as Blender orders it.
- **Every point gets a rotation by the curve's twist method**
  (`orient.rs`): Minimum (with the closing correction for a closed
  spline), Tangent and Z-Up, then Smooth, then the tilt. For Tangent,
  Blender leaves a segment's last tangent in the next point's slot;
  a straight segment's point and the open end keep it, and so do they
  here.
- **The path** (`path.rs`): the first spline's running length
  (`BKE_anim_path_calc_data`), and a place on it by length
  (`BKE_where_on_path`): position, rotation, tilt and radius. A closed
  path wraps; an open one runs on past its ends unless it is clamped.
- **What rides it:**
  - **A child of the curve** (`ob_parcurve`) takes the place at the
    curve's evaluation time over its Path Animation frames. With Follow it
    also turns by its own track and up axes (`quat_apply_track`), and with
    Radius it scales by the curve's radius.
  - **The Follow Path constraint** (`constraint/path.rs`): an offset in
    frames or a fixed position, following or not, with or without the
    radius. The owner keeps its own scale.
  - **The evaluation time** comes from the curve data's own action,
    keys or a Generator (THYMUS's is a Generator over the frame).
- A curve whose time moves counts as moving. Its children are re-solved
  every frame, and the read places them at the saved frame.
- **Truth:**
  - phiphx `curve/tests.rs`, over `fixtures/curve_paths.blend` made by
    `truth/blender_curve_paths.py` (Blender 5.0). Five curves (a closed
    3D Bézier ring, an open NURBS path, a smoothed poly line, a flat
    closed Bézier drawn the wrong way round, an open Bézier with a vector
    corner and tangent twist) carry children and Follow Path objects.
    Every object is within 0.0001 of Blender's `matrix_world` at six
    frames, some past the path's ends, and as read at the saved frame.
  - THYMUS's `turn_cam_02` at five frames through the turntable
    (`truth/thymus_turntable.txt`) is within 0.0000005 of Blender.
- `save_truth` now expects an object that follows another to stand where
  the edited frame puts it: its test moves the frame to 42, and the
  turntable camera moves with it, as it would in Blender.
- **Not yet:**
  - Clamp To;
  - a curve's deform modifiers, shape keys and hooks;
  - a path through a curve's other splines (Blender reads only the first
    too, but it nests holes across all of them);
  - vertex parents.
- **The machine's tree.** Since batch 90 the other agent gave the Dope
  Sheet, Graph, Drivers and NLA editors their shared filter and channel
  search (`part::filter::dope`, one new test). Its `part/filter/mod.rs`
  again started from a copy without batch 87's X-ray button, and its
  layout golden was recorded on another tree. The merge keeps the X-ray
  button with the new filter. Re-recorded here, the golden gains exactly
  the new filter and search fields in the Animation and Compositing
  workspaces, and nothing moves.

## Batch 90 — objects that follow others: constraints on objects, bone parents

- **Before.** Constraints were solved on bones only. An object's own
  constraints were not read, so:
  - a camera or lamp with Track To pointed wherever its rotation said;
  - an object parented to a bone stood on the rig object, not on the bone;
  - in the app, a bone constraint aiming at an object saw the object at
    the world origin (the rig was posed with no scene around it).
- **Objects are solved in Blender's order** (phiphx `blend/anim/follow.rs`,
  `BKE_object_where_is_calc`). Each object waits for its parent and for
  whatever its constraints read. Then it takes:
  - its local transform at the frame;
  - its parent's matrix through `parentinv`. For a **bone parent**
    (`PARBONE`, `ob_parbone`) that is the bone's pose moved to its tail,
    or its deform with **relative parenting**;
  - its constraints.

  A rig is posed only when an object follows one of its bones, after its
  object and the objects its own bones aim at. A dependency cycle is cut
  where it closes, as the parent chain already was.
- **One constraint stack for bones and objects**
  (`blend/rig/constraint/stack.rs`):
  - `BConstraintStack` runs `BKE_constraints_solve` for any owner;
  - `BConstraintHost` supplies what differs: where targets are, the
    owner's spaces, animated settings. It has two implementations, the rig
    being posed (`bone.rs`) and an object (`object.rs`);
  - each kind's evaluator is written once, on the owner (`BCob`), and
    serves both.
- **What an object's constraints see** (`blend/rig/view.rs`,
  `BSceneView`):
  - other objects' world matrices and rotation modes;
  - an object's **local space**: its parent's world through `parentinv`,
    or with no parent its own rotation, without scale or location, as
    Blender defines it (`BObjectSpace`);
  - world and custom spaces;
  - **another rig's bones** (`BTarget::Bone`), in any of the bone's spaces
    (`BRigSpace`, the bone half of `BKE_constraint_mat_convertspace`),
    `head_tail` along it, or as the Armature constraint's deform. A
    missing bone falls back to the object, as in Blender.
- **Animated settings.** An object's constraint influence and mute can be
  keyed (`constraints["Name"].influence`).
- **Where it shows:**
  - **Opening a file.** Followers stand where their targets put them at
    the saved frame. THYMUS's two spotlights now aim at `cam_pivit`
    exactly as in Blender (to five decimals).
  - **Playback.** Cameras, lights and empties are placed at the frame
    with their constraints (`SceneMotion::at`). A scene counts as moving
    when an object follows a moving rig.
  - **Rigs.** A rig's bones aim at objects where those objects stand.
- **Found on the way.**
  - A rig object's action also holds its bones' curves, and they were
    written over the rig object's own location and rotation. Only the
    object's own channels move it now (`BFCurve::own_property`).
  - A rig whose bones read other objects now counts as moving. So does a
    rig an object follows, but only for placing that object: its meshes
    are not posed again unless it moves.
- **Truth:** phiphx `anim/follow_tests.rs`, over
  `fixtures/object_constraints.blend` made by
  `truth/blender_object_constraints.py` (Blender 5.0). The scene has 28
  objects:
  - a tracking camera, with a chain copying it;
  - an object under each kind in world, local and custom spaces, with
    and without a parent;
  - an animated influence;
  - Child Of, Copy Location at `head_tail`, Copy Transforms from a bone's
    local space, an Armature constraint, a bone parent and a relative bone
    parent, all on an animated rig.

  Every object at six frames is within 0.0001 of Blender's
  `matrix_world`, as read at the saved frame and as the scene model plays
  it.
- **Not yet:**
  - a **curve parent's path**. THYMUS's turntable camera `turn_cam_02` is
    parented to the `Path` curve, and Blender moves it along the path
    (`ob_parcurve`); it still stands off *(done in batch 91)*;
  - vertex parents, and a vertex group as a target;
  - the Action constraint on an object;
  - Follow Path *(done in batch 91)*, Clamp To and Shrinkwrap;
  - drivers on objects;
  - Spline IK.
- **The machine's tree.** Since batch 89 the other agent:
  - gave the Text editor its sidebar (Blender's one "Text" tab with Find
    and Properties);
  - made a canvas round the pixels it paints: the 3D viewport's raster no
    longer squares off its zone's corners;
  - changed twelve files in all, with one new test.

  Two of them needed repair in the merge:
  - Its new `view3d/rendered/mod.rs` was written over a copy older than
    batch 84, so the tree did not compile. It had also dropped three
    things: X-ray see-through in the render, the device-switch restart
    and the tracer's pose refit. The merge keeps those three, plus X-ray's
    `see_through` in `view3d/mod.rs`, and adds the machine's change on top
    (the frame painted with the zone's corners). `view3d/mod.rs` folds
    one call onto two lines to stay within 240.
  - Its layout golden was recorded on another tree: it lacks batch 87's
    X-ray button. Re-recorded here, the golden differs from batch 89's by
    exactly the six lines of the Text editor's second tab, as the machine
    describes.

  Batch 90 is otherwise phiphx only, and was tested on top of that work.

## Batch 89 — the rest of the matrix constraints

- **Before.** The pose solver (batch 85) evaluated the ten constraint kinds
  THYMUS uses. A bone under any other kind kept its channel's pose, as if
  the constraint were muted.
- **Ten more kinds, ported from Blender's `constraint.cc`** (phiphx
  `blend/rig/constraint/`):
  - **Child Of** (`parent.rs`, `childof_evaluate`): the target and the Set
    Inverse matrix, whole or filtered channel by channel (Euler angles in
    the target's and the owner's rotation orders). It also covers the flag
    Blender sets for itself (`CONSTRAINT_SPACEONCE` while it has a target)
    and an inverse still pending in the file.
  - **Track To** (`aim.rs`, `vectomat`): any track axis and up axis, world
    Z up or the target's Z.
  - **Locked Track** (`aim.rs`): every lock and track pair, the owner's
    heading kept.
  - **Limit Location**, **Limit Scale** and **Maintain Volume**
    (`limit.rs`). The file keeps Maintain Volume's axis under its old DNA
    name, `flag`.
  - **Transformation** (`transform.rs`):
    - from location, rotation or scale, to any of them, axis by axis, with
      or without extrapolation;
    - its rotation read as Euler angles in any order, as a quaternion's
      pseudo-angles, or as swing and twist
      (`BKE_driver_target_matrix_to_rot_channels`; swing and twist is not
      in the fixture yet);
    - every mix mode, and its own output Euler order.
  - **Floor** (`parent.rs`): any axis, with an offset, optionally in the
    target's rotation.
  - **Action** (`action.rs`): another action's pose of the bone.
    - The frame comes from a target channel (rotation in degrees, scale or
      location) or a fixed evaluation time.
    - The action is read for the slot the constraint names.
    - The pose is built on a fresh channel with the bone's rotation mode
      (`BKE_pchan_calc_mat`), and mixed in by any of the seven mix modes,
      sharing Copy Transforms' aligned-scale and split-channel products.
      The fixture has two of the modes.
  - **Pivot** (`parent.rs`): about the target, the owner or an absolute
    point. It honours the rotation-range switch and moves the pivot along
    the rotation axis.
- Every constraint kind now names its targets in one place
  (`BConstraintKind::targets`), which the solve order reads, so a new kind
  cannot be left out of it.
- **Truth:** phiphx `constraint/tests.rs`, over `fixtures/constraints.blend`
  made by `truth/blender_constraints.py` (Blender 5.0).
  - One rig, 24 bones, a bone under each kind in several settings, driven
    by two animated control bones (one in quaternions, one in YZX Euler)
    and an animated Empty. The rig object is moved and turned, so world,
    pose and local spaces all differ.
  - Every bone at six frames is within 0.000002 of Blender's pose matrix.
- Not yet:
  - Spline IK, Shrinkwrap, Clamp To and Follow Path (they need a mesh or a
    curve), and the motion-tracking kinds;
  - an Action constraint playing the rig object's own action;
  - constraints on objects rather than bones (a camera's Track To)
    *(done in batch 90)*.
- **The machine's tree.** Since batch 88 the other agent gave the Image
  editor and the node editors their sidebar panels and re-recorded the
  layout golden (eight files, two new tests). Batch 89 touches only phiphx
  and was tested on top of that work. The same three tests fail as at batch
  88, all in the rail and theme work still in progress there.

## Batch 88 — creases: THYMUS's head subdivides as Blender's does

- **Before.** phiphx's Subdivision Surface had no creases. THYMUS's head
  has 136 creased edges (crease 0.25 and 1.0), and it sat up to 2.7 mm off
  Blender's; every other mesh was within 0.01 mm (batch 85).
- **The creases are read** (phiphx `blend/mesh/crease.rs`): `crease_edge`
  and `crease_vert` with each edge's two vertices, from:
  - 4.x's named `CustomData` layers;
  - 5.x's attribute storage;
  - 3.x's unnamed `CD_CREASE` layers beside the `MEdge` array (THYMUS is
    a 3.6 file);
  - older files' `MEdge.crease` byte.

  They ride on the evaluated mesh (`BlendEvalMesh::edge_creases`,
  `vert_creases`) into the modifier stack and into a rig's skinned base,
  so a posed frame subdivides with them too.
- **They are applied as OpenSubdiv applies them**, with Blender's settings
  (phiphx `blend/modifier/crease.rs`):
  - a crease `c` is sharpness `c² · 10` (`crease_to_sharpness`), with
    uniform creasing (`CREASE_UNIFORM`);
  - an edge point is the smooth average, the midpoint at sharpness 1 or
    more, and a blend of the two for a fraction;
  - a vertex takes the smooth, dart, crease or corner rule from its own
    sharpness and how many sharp edges meet it. Where one level's step
    down changes that rule, the two masks are blended by how sharp the
    things that went smooth were (`ComputeFractionalWeightAtVertex`);
  - boundary and non-manifold edges are infinitely sharp, and so is a
    one-face corner when the modifier keeps corners. This reproduces the
    boundary rules phiphx already had;
  - the limit takes the crease curve's limit (`1/6, 4/6, 1/6`) or keeps a
    corner;
  - "Use Creases" off (`eSubsurfModifierFlag_UseCrease`) keeps the creases
    on the mesh but sharpens nothing.
- **Found against Blender: its `quality` setting matters.** Blender takes
  the limit surface from OpenSubdiv's patches, which isolate sharp
  features only to the modifier's `quality` level (3 by default).
  - A crease or vertex still semi-sharp at the grid's level is refined
    further, up to `quality`, and the sharpness it has left there is
    dropped; only infinite sharpness is kept.
  - phiphx now does the same. It refines positions only, past the grid,
    and reads the grid's vertices off the finer mesh, since a vertex keeps
    its index from level to level.
- Truth:
  - phiphx `tests/subsurf_creases.rs`, over `fixtures/crease.blend` made by
    `truth/blender_crease.py` (Blender 5.0, so the 5.x attribute path).
    Six cubes, every vertex within 0.000005 of Blender's (the test holds
    them to 0.0001):
    - a crease sharp for two levels and a half at the third;
    - a fractional crease, an infinitely sharp one and a creased vertex;
    - three sharp edges meeting at a corner, with the limit surface off;
    - "Use Creases" off;
    - sharpness left over at the `quality` level, on an edge and on a
      vertex;
  - `tests/rig_plays.rs`: the head is now held to 0.0001 like every other
    mesh. It is within 0.00001 at frames 1 and 66, down from 2.7 mm.
- Not yet: a grid finer than `quality` (`levels` above it). Blender
  evaluates it on OpenSubdiv's Gregory end caps round each extraordinary
  vertex, which only approximate the limit surface phiphx gives: 0.01 on
  a cube two units wide at four levels and quality 2 (a seventh cube in
  the same test, held to 0.02 until it is ported). THYMUS's meshes are all
  two levels at quality 3.
- **The machine's tree.** Since batch 87 the other agent changed nine
  files. It reworked the rail's open sidebar on top of batch 87's version
  (`BRailPart::WithPanels`), gave rails a fill of their own
  (`BFill::HostRail`, `theme/semantic/rail.rs`) and added
  `the_properties_strip_is_a_strip`. Batch 88 touches none of those files.
  Three tests fail on the merged tree, all in that work:
  - `region::tests::only_intentionally_painted_regions_claim_host_corners`
    (a rail region now paints `HostRail`);
  - `central_reuse` (the selected Properties tab's sweep);
  - `semantic_style_derivations` (a rail's sweep step).

  The machine's re-recorded layout golden was missing batch 87's header
  fixes and the X-ray button. It is re-recorded on the merged tree, and
  differs from the machine's only by those.

## Batch 87 — the failing UI tests: four real faults fixed, five tests brought up to date

Batch 86 left fourteen phiber tests failing, some since batch 79. Each was
traced to its cause. Where the app was wrong, the app is fixed. Where the
machine had changed the design on purpose, the test now checks the new
design. The suite passes whole again.

- **Real faults, fixed in the app.** In each case a rewrite on the machine
  had been started from an older copy of the file, so it lost what a later
  batch had added.
  - **Sidebars drew no panels** (batch 68's N panel). The rail renderer's
    rewrite (`view/draw/rail.rs`) dropped the column that opens beside the
    tab strip, so every sidebar was half-wired: N set the state and lit the
    tab, and the panels were built and thrown away.
    - Restored on the new rail as `BRail::beside`, with two new parts,
      `BRailPart::Beside` and `BRailPart::Strip`.
    - This fixes `viewport_sidebar` (two tests), the Action editor's
      Action and Slot panels (`action_editor_actions`) and the machine's own
      new test, `an_open_sidebar_paints_its_panels`.
  - **The view keys acted on no viewport** (batch 64). The area renderer's
    rewrite (`view/draw/area.rs`) dropped the per-area view slot. So in a
    workspace with two 3D Viewports, Numpad orbit, zoom and frame wrote
    the view every viewport shares, not the one under the pointer. The
    slot is restored (`viewports_keep_their_own_view`).
  - **Header controls lay on top of each other.** A header's centre cell
    could shrink below its controls, and they spilled over the panes on
    either side:
    - in the Timeline, the +5 jump button sat on the frame field's
      decrement arrow, so clicking +5 went from frame 100 to 104
      (`marker_box_and_transport_popovers`);
    - in the Image editor, Open sat on the gizmo and overlay toggles;
    - in the Dope Sheet's footer, at 1100 px wide, the transport's cell
      was squeezed to nothing and the frame fields were drawn over it.

    A header's centre is now never narrower than its controls
    (`view/draw/pane.rs`). A header too narrow for everything scrolls, as
    batch 60 made it, and Blender's does.
  - **The X-ray button was gone** (batch 79). The machine's new Properties
    options control rewrote `part/filter/mod.rs` from an older copy.
    X-ray leads the shading radio again (`xray_toggle`, `viewport_extras`).
- **Styling moved onto `BFlex`** (`style_budget`, `outliner_depth`). Four
  renderer files wrote raw GPUI style calls that the budget allows none
  of:
  - the tree row (`tree/mod.rs`, 36 calls) is now `BTreePart`: indent,
    row, icon, name and trail;
  - the disclosure cell (`tree/disclosure.rs`) shares the icon's part
    (from 12 calls to 6; the budget is lowered to 6);
  - a row dragged over lights through `BFill::Row` (`tree/drag.rs`);
  - the viewport's gesture layer is `BPlace::Overlay`
    (`canvas/view3d/hit.rs`).

  The indent helper nothing called (`prim::indent`, `render_default`) is
  gone.
- **Tests brought up to date with deliberate design changes:**
  - the style catalog holds 110 fields, with `dimensions.number_min_width`
    added (`theme::descriptor`);
  - a content header is its band plus the rule it closes with, 37 px; a
    footer and the file browser's execute bar are the band alone, 36 px.
    The machine made the same fix to `dope_sheet_footer_is_the_transport`
    and `file_browser_layout` while this batch ran, and its versions were
    kept;
  - the selection colour is a translucent accent tint and stays off the
    shade ladder, as the accent does, so its hover is a lightness nudge
    (`semantic_style_derivations`);
  - the render info bar is the theme's dim, the one a modal lays over the
    workspace, not a fixed quarter-black
    (`rendering_workspace_shows_the_render`);
  - the layout golden, re-recorded on the merged tree. It differs from the
    machine's own golden only where this batch moved something: the
    Timeline, Dope Sheet and Image editor headers no longer overlap, and
    the X-ray button is back.
- **The machine's tree, twice over.** The other agent changed 25 files
  while this batch ran, several of them the same failures (the rail foot,
  the strip heights, the style budget, the popover as `BPopoverPart`, and
  the Properties header's options popover). Its versions were taken
  wherever it had fixed the same thing. This batch's rail change and the
  X-ray button were laid on top of its files. Every file copied was checked
  first against the machine's current version.

## Batch 86 — the rig poses on its own thread, ahead of the playhead

- **Before.** Batch 85 posed THYMUS's rig and skinned its eleven meshes
  inside the render pass, on the UI thread. That cost 35 ms of every frame
  the playhead moved, on two cores. Blender does the same: it evaluates
  the depsgraph on the main thread and the viewport waits for it.
- **Now** (phiber `scene/pose/`):
  - a thread of its own (`BPoser`, `worker.rs`) poses the frames the UI
    asks for. Of what is queued it takes only the newest ask, as the
    tracer's thread does. It looks again between frames, so a scrub or a
    jump moves it straight to the frame wanted now;
  - the render pass asks it for the shown frame (`source::scene_objects`)
    and draws what it has. That is the frame itself when it is posed, or
    else the frame it drew last (the meshes as read, before the first).
    While the shown frame is still posing, the window draws again
    (`BPoser::waiting`, `request_animation_frame` in `app/mod.rs`) until
    it lands;
  - while playing, the poser also works on the next two frames after the
    playhead. In reverse it works backwards, and at the end of the range
    it wraps as looping playback does. So each frame is posed before the
    playhead reaches it. If the poser cannot keep up, the viewport shows
    the newest frame it has, as Blender's frame dropping does.
  - `current_scene` still poses in place, at the exact frame
    (`BPosed::exact`). F12 and anything else outside the render pass use
    it, so a render never takes a pose from another frame.
- **Measured** (debug build of phiber, dependencies optimised, two
  cores): a render pass at a frame the rig has not posed yet takes
  0.6 ms, down from 37 ms.
- Truth: phiber `tests/rig_poses_off_the_ui_thread.rs`.
  - Stopped at frame 66, the first pass does not wait. It draws the arm as
    read and asks for another pass.
  - Once the pose lands, the pass draws the same vertices as posing in
    place.
  - Playing from 66, frames 67 and 68 are posed without being asked for,
    and the pass at 67 has its pose at once.
  - Played in reverse from the start of the range, the frames ahead wrap
    to its end.
- Not yet:
  - while the viewport shows an older pose, the objects' own matrices are
    already at the shown frame. Nothing shows it on THYMUS, whose rig
    object does not move;
  - with no GPU, the CPU viewport still projects each posed frame on the
    UI thread (`view3d/cache.rs`).
- **The machine's tree.** Since batch 85 the machine has changed sixteen
  files and added one test (`scroller_fits_its_gutter`). The changes cover
  the header and footer strips, scrollers, cards, tabs, the Properties
  header and a re-recorded layout golden. Batch 86 touches none of those
  files, and was built and tested on top of them. Fourteen tests fail on
  the merged tree, none of them from batch 86:
  - the twelve from batch 85;
  - `dope_sheet_footer_is_the_transport` and `file_browser_layout`, which
    now measure a strip at 37 px against 36 px, after the machine's strip
    change.

## Batch 85 — the rig plays: THYMUS waves

- **A correction first.** Batch 84's notes said a playing armature was
  skinned on the GPU every frame. It was not: nothing rigged moved at all.
  Each mesh was skinned once, when the file opened, in the pose it was
  saved in.
- **Why it never moved.** On a Rigify rig the animator keys the control
  bones, but the bones that bend the mesh follow those controls through
  constraints and drivers. THYMUS's rig has:
  - 328 bones, 55 of which deform the mesh;
  - 330 constraints of ten kinds: Copy Transforms, Stretch To, Copy
    Scale, Copy Location, Copy Rotation, Damped Track, Armature, IK,
    Limit Rotation, Limit Distance;
  - 101 drivers, which switch IK and FK, parents and poles;
  - IK on the legs, and five bone inheritance modes.

  phiphx only evaluated the keyed bones (task 17 (b)).
- **A pose solver, ported from Blender** (phiphx `blend/rig/`, task 17
  (d)):
  - the action's curves over each channel and custom property, then the
    rig object's drivers (`driver/`). A driver can be a sum, an average,
    a minimum, a maximum or a simple expression. Simple expressions are
    evaluated as Blender evaluates them without Python
    (`BLI_expr_pylike_eval`, `driver/expr.rs`). What a driver writes can
    be a constraint's influence or mute, an Armature constraint's bone
    weight, a channel, or a property;
  - each bone after everything it depends on (`solve.rs`): its parent
    through the bone's inheritance (`BKE_bone_parent_transform_calc_from_matrices`,
    `space.rs`), then its channel, then its constraints;
  - each constraint in its own spaces (`BKE_constraint_mat_convertspace`:
    world, pose, local, local with parent, owner local, custom), blended
    in by its influence (`interp_m4_m4m4`, through a polar decomposition);
  - Blender's matrix and Euler math where it differs from glam's
    (`math.rs`). This covers the stable orthogonalisation, the choice
    between two Euler solutions, compatible Eulers, and the quaternion
    conversions;
  - an IK chain solved as one unit with Blender's legacy solver, ported
    step for step (`ik/`). The pieces are:
    - its segments (spherical, swing, elbow, revolute, translation for
      stretch);
    - the selectively damped least squares step over an SVD;
    - joint limits locked in turn;
    - the pole target;
    - the stretch and non-uniform scale laid back onto the bones.
- **Checked against Blender 5.0** (`tests/truth/thymus_rig.txt`, from
  `truth/blender_rig.py`):
  - every one of the 328 bones' pose matrices, at frames 1, 20, 40, 66,
    100 and 132, is within 0.00001 of Blender's. Without the IK port, 24
    of them are up to 0.36 off;
  - every driver's value at each of those frames matches.
- **The mesh follows** (phiphx `scene/rig.rs`):
  - at load, each mesh an Armature modifier deforms keeps three things:
    the mesh as the Armature receives it, the modifiers after it, and the
    triangles it was read with;
  - at a frame, the rig is posed and its deforming bones and B-Bones are
    worked out from that pose (`BlendRigSource::deform_bones`). The mesh
    is then skinned and its later modifiers run again;
  - the triangles never change, only the vertices. So the tracer takes a
    frame as a new pose (batch 84's `Tracer::repose`: the BVH refitted,
    not rebuilt).

  In phiber the object list at the shown frame carries the posed meshes
  (`scene::motion::skinned`). The tracer's scene key includes the mesh, so
  each frame reaches it.
- Measured on the cloud's two cores, release build, THYMUS's eleven
  meshes (218 000 vertices after subdivision):
  - the pose and the skinning together take 6 ms;
  - the whole frame, with two meshes subdivided again and every normal,
    takes 35 ms. It was 73 ms before two changes:
    - the subdivision now keeps its adjacency tables between frames;
    - a posed mesh works out its normals once for faces and corners.
- Truth:
  - phiphx `rig/tests.rs`: every bone at six frames, every driver;
  - `tests/rig_plays.rs`: at frames 1 and 66, every 25th vertex of each
    of the eleven meshes, as the Armature modifier leaves it:
    - the two meshes deformed before subdividing (the torso, the upper
      legs) are compared vertex for vertex, within 0.0001;
    - the nine subdivided first are compared by position, since their
      subdivision numbers vertices its own way. Each of Blender's
      vertices must have one of ours within 0.0001, or 3 mm on the head
      (see below);
    - frame 1 reproduces the meshes as read, subdivision and normals
      included;
    - the triangles are the same at every frame;
  - `rig/math.rs`, `driver/expr.rs`, `driver/path.rs` unit tests;
  - phiber `tests/rig_plays_in_the_viewport.rs`: the lower arm moves
    between frames 1 and 66 on the same triangles; a mesh no rig moves
    stays the mesh as read.
- Found on the way: phiphx's Subdivision Surface sits within 0.01 mm of
  Blender's on ten of THYMUS's meshes, but up to 2.7 mm off on the head.
  Edge creases are not ported yet *(done in batch 88: the head is within
  0.01 mm too)*.
- Not yet:
  - Spline IK, and the Action, Child Of, Track To, Locked Track,
    Transformation, Floor and Shrinkwrap constraints (a bone behind one
    keeps its channel's pose) *(all but Spline IK and Shrinkwrap done in
    batch 89)*;
  - B-Bone shaped constraint targets;
  - driver variables other than a single property of the rig, and
    drivers on the armature data (THYMUS's drive B-Bone ease from
    properties nobody animates, so the saved values hold);
  - NLA strips on a rig;
  - Preserve Volume (dual quaternions) and envelopes in the Armature
    modifier;
  - ~~posing runs on the UI thread in the render pass (35 ms a frame on
    two cores)~~ — on its own thread, ahead of the playhead, since
    batch 86.
- **The machine's tree.** Since batch 84 the machine has changed sixteen
  files and added two tests: a search field in the Properties header, the
  Image editor's canvas, menus, modals, popovers, regions, the semantic
  theme, and a re-recorded layout golden. Batch 85 touches none of them
  and was built and tested on top of them. Twelve tests fail on the merged
  tree, none from batch 85:
  - the eleven from before;
  - `the_rendering_workspace_is_the_image_editor_with_blenders_header`,
    whose render info bar no longer paints as the test expects after the
    machine's Image editor change.

  The layout golden still differs from the machine's own tree; batch 85
  draws nothing new, so it is left to that work.


## Batch 84 — a moved object no longer rebuilds the tracer

- **What a scene change cost.** Every change to the scene the viewport
  traces — an object moved with G, each frame of an animation playing, a
  scrub — built the tracer from nothing:
  - the BVH sorted and built on the CPU;
  - every buffer allocated and uploaded again;
  - the images and the world packed into the atlas again;
  - the bind groups made again, and the image started over.
- **A new pose goes up in place** (phiray `trace/pose.rs`,
  `Tracer::repose`). When only *where* things are has changed — the same
  objects, meshes and materials, which phiber tells apart by the scene's
  shape (`BRayKey::shape`) — the tracer keeps its tree and its buffers:
  - the vertices are written over the old ones;
  - the tree keeps its shape, and its bounds are refitted over the moved
    faces (`Bvh::refit_slots`). That is linear work with no sort, spread
    over all cores;
  - the atlas, the bind groups and the compiled kernels stay;
  - the light table follows the emissive faces.

  phiber builds the posed scene without the images or the world
  (`scene::build_pose`), since the tracer already holds them.
- **Once the motion stops, a fresh tree, and the image keeps its samples**
  (`Tracer::retree`). A refitted tree gets looser as things move apart.
  So 400 ms after the last change, the render thread builds the tree again
  over the pose it holds, and the faces move into the new tree's order.
  A tree decides only how fast a ray finds what it hits, never what it
  hits. That is why refinement carries on from the samples it has, rather
  than starting over.
- **The render thread takes only the newest.** A drag or a playing
  animation queues one scene per step. The thread now drains the queue and
  takes the newest scene and the newest view, instead of working through
  the backlog. A finished image is not read back again when nothing has
  changed.
- **A new image buffer is no longer uploaded.** The image holds 24 words
  a pixel, so at 1920×1080 about 200 MB of zeros went up at every resize
  or shading switch. wgpu clears a new buffer itself, so it is now only
  allocated (`GpuArray::zeroed`).
- **Measured** with `examples/scene_rebuild`, on the cloud's software
  Vulkan, with 204 800 triangles, half of them moved:

  | | Built again | Reposed |
  |---|---|---|
  | The CPU's part of a scene change | 34–83 ms | 17–22 ms |
  | To the first frame, Solid | 116–145 ms | 80–105 ms |
  | To the first frame, Rendered | 290–319 ms | 213–251 ms |

  Once the scene is still, the new tree takes 22–27 ms and does not
  restart the image.
- Truth:
  - phiray `tests/solid.rs`:
    - a new pose draws what a fresh build draws;
    - a scene with other faces is refused, so it is built again;
    - a new tree keeps the image and its sample count, and a new view
      over it draws what a fresh build draws;
  - phiber `rendered/tests.rs`: a moved object reaches the render thread
    as a pose of the same shape, and draws what a thread given the moved
    scene from the start draws.
- Not yet:
  - a new mesh, a vertex count a modifier changes, or a material change is
    a new shape, and still builds again;
  - ~~an armature playing is skinned on the GPU, read back to the CPU,
    then written up again to the tracer~~ — wrong: no armature played at
    all until batch 85. Skinning ran once, at load, in the saved pose;
  - the frame still comes back to the CPU to be shown;
  - there is no on-disk pipeline cache across runs;
  - the UI itself (GPUI's DirectX 11 renderer) still takes DXGI's first
    adapter (batch 83).
- **The machine's tree** had not moved since batch 83. The same eleven
  tests fail as then, none from batch 84.


## Batch 83 — the GPU: the right one, one device, kernels compiled once

- **Found why the GPU might not be the one you think.** phiray asked wgpu
  for "any adapter" (`RequestAdapterOptions::default()`). In wgpu 29 that
  means no power preference, and with none `request_adapter` does not rank
  at all: it takes the first adapter the first backend lists. On a laptop
  that is often the integrated GPU; on some machines, a software one.
- **phiray now ranks every adapter itself** (`gpu/adapter.rs`,
  `GpuAdapter::pick`):
  - a discrete GPU, then integrated, then virtual, then anything else, and
    software (`llvmpipe`, Microsoft Basic Render Driver) only when nothing
    else exists;
  - within that, Vulkan or Metal before DirectX 12 — whose shaders compile
    through FXC, slowly — before OpenGL;
  - then the one that binds the most.

  `PHIRAY_GPU=<part of a name>` picks a card by hand.
- **One device per process** (`Gpu::shared`). The viewport's tracer and GPU
  skinning each used to open their own device. That meant two adapter
  opens, and the UI thread waited for the second one on the first viewport
  frame. Now there is one device, opened at start-up on the skinning
  warm-up thread, and both use it.
- **Kernels compile once per device** (`Gpu::pipeline`). Every new tracer —
  every scene change: a moved object, each frame of an animation playing,
  a shading switch — used to compile its WGSL kernel again, because the
  pipeline cache lived in the tracer's own pass. The cache is now the
  device's. `examples/scene_rebuild` measured the same scene rebuilt three
  times on the cloud's software Vulkan, with its driver cache warm:
  - Solid: from 200 ms to 60 ms per rebuild;
  - Rendered: from 870 ms to 150 ms per rebuild;
  - the first build, cold, compiles as before.
- **Preferences ▸ System ▸ Render Devices** (`part/pref/device.rs`) lists
  every GPU the machine has, one row per card through its best backend,
  with its kind and backend:
  - Auto names the card it takes;
  - the line under the list says which device is drawing and what it binds;
  - picking a card opens it at once (`Gpu::switch`) and the viewport's
    render thread moves to it on its next frame (`Gpu::generation`). The
    skinning pass starts over on it. Blender asks for a restart here.

  The choice lasts the session: Phiber keeps no preferences file yet.
- The slow-frame log (`phiber-frames.log` in `%TEMP%`) now names the card
  and backend that drew the frame. `cargo run --release -p phiray --example
  limits` lists every adapter, best first.
- Truth:
  - phiray `gpu/adapter.rs`: a laptop as its drivers list it, integrated
    first — the discrete card through Vulkan wins, a name picks another,
    an unknown name falls back, one row per card, software only alone;
  - `tests/shared.rs`: one device through every handle, a switch opens a
    new one that compiles afresh;
  - `tests/solid.rs`: a second scene on the device compiles nothing and
    draws the same image;
  - phiber `pref/device.rs`: the panel on this machine, picking a card,
    and no reopen for the card already drawing.
- Not yet:
  - the UI itself (GPUI's DirectX 11 renderer on Windows) takes DXGI's
    first adapter; `EnumAdapterByGpuPreference` would put it on the
    discrete card too, but that is vendored Zed code and cannot be built
    here;
  - a moved object still rebuilds the BVH on the CPU — a refit on the GPU
    is the next speed-up for animation playback *(done in batch 84, as a
    refit over all the CPU's cores)*;
  - the frame still comes back to the CPU to be shown (a readback per
    refinement step);
  - no on-disk pipeline cache across runs.
- **The machine's tree** had not moved since batch 82. The same eleven
  tests fail as then, none from batch 83.


## Batch 82 — Add ▸ Action puts a strip in the NLA

- **Add ▸ Action** (`nla.actionclip_add`) lists every action, the file's
  and the session's (`BMenuKind::Live`: a submenu built from the state when
  it opens). Picking one adds a strip of it at the current frame, in each
  shown stack's active track (phiphx `BlendNla::add_clip`):
  - the strip is the clip `BKE_nlastrip_new` makes: through the slot the
    object would take (`generic_slot_for_autoassign`, then the NLA's
    fallback — the only slot, when it suits an object);
  - its range is the Manual Frame Range, else the slot's first key to its
    last;
  - it is moved to start at the frame, selected, and named uniquely after
    the action;
  - it goes in the active track when that has room; else a new NlaTrack
    on top, made active;
  - an empty stack whose action line is picked gets its track first.

  With no active, unlocked track nothing goes in, and the status bar says
  Blender's "No active track(s) to add strip to, select an existing track
  or add one before trying again".
- **Shift A** over the NLA editor opens the Add menu on Action ▸ — Blender
  opens a search over the actions there.
- The three live submenus — Open Recent, Undo History, Add ▸ Action — now
  share one popover (`BMenuItem::live_sub`) instead of each building its
  own.
- Truth:
  - phiphx `nla_edit.rs`: `nla.actionclip_add(action='Idle')` at frame 100
    on `nla_cube.blend`, as bpy 5.0 makes it:
    - Idle (keys 10–30) goes into the active Overlay before Walk Again,
      100–120 over 10–30, selected, not active;
    - again, into a new NlaTrack on top as Idle.001;
    - nothing with no active track, or a locked one.
  - `blender_opens_saves`: the nla_cube session adds Idle after Walk on
    Base Layer and on the new track. Blender reads the old and new strips
    in one track back as written.
  - phiber `tests/nla_editor_edits.rs`: Shift A, Action ▸ Idle; the menu
    again (a new track); the report with no active track.
- Not yet:
  - the other Add entries (Transition, Sound, Selected Objects);
  - moving, extending and scaling strips;
  - tweak mode.
- **The machine's tree.** Since batch 81 the machine changed the scroller
  (`view/draw/scroller.rs`: the thumb rides the container's gutter and
  swells in the accent under the pointer). Batch 82 was built and tested
  on top of it. That change moves every scroll thumb 3 px, which the
  machine's layout golden does not have yet. It is left for that work to
  re-record: the only difference is the scroller's own. Eleven tests fail
  on the merged tree, none from batch 82:
  - the ten from batch 81;
  - `layout_fingerprint` (the scroll thumbs).


## Batch 81 — the NLA editor edits the stack

- **Picking, as Blender 5.0's NLA editor picks** (phiphx
  `anim/nla_edit.rs`, ported from `nla_select.cc` and `nla_tracks.cc`):
  - a click in the strips picks the strip within 7 px of the pointer
    (`nla.click_select`); it alone is selected, it becomes the active
    strip, and its track is selected and made the active one;
  - Shift flips that strip instead;
  - a click on nothing clears every strip and track;
  - a click on a track's name in the channel list picks that track
    (`anim.channels_click`), and Shift flips it;
  - a press below the scrub band picks instead of scrubbing, as Blender's
    left-click keymap has it; the scrub band still scrubs.

  A selected track's name is drawn in the selection's colour, the one its
  selected strips have.
- **The operators**, from the menus and their keys over the editor:
  - Select ▸ All (A), None (Alt A), Invert (Ctrl I) — `nla.select_all`;
  - Strips ▸ Delete (X, `nla.delete`) deletes the selected strips, with a
    transition either side;
  - Strips ▸ Toggle Muting (H, `nla.mute_toggle`); a muted strip stops
    playing;
  - Tracks ▸ Add (`nla.tracks_add`) puts one track on top of each stack
    that has a selected track;
  - Tracks ▸ Add Above Selected puts one over each selected track;
  - Tracks ▸ Delete (`nla.tracks_delete`) removes the selected tracks with
    their strips.

  A new track is "NlaTrack" made unique (".001"), selected and active.
  Strips on a locked track are neither deleted nor muted. Tracks ▸ Add with
  nothing selected says "Select an existing NLA Track or an empty action
  line first", as Blender does.
- **The session keeps whole stacks.** `ctx.scene.nla` now holds each
  edited object's NLA stack as it stands (JSON), not a list of steps.
  Push Down and Stash (batch 80) lay onto it too. Undo steps it, the NLA
  editor and the viewport read it, and phiphx plays it.
- **The save writes each stack whole** (phiphx `write/nla.rs`):
  - a track or strip the file had keeps its block — with its control
    curves, modifiers and children, which phiphx does not model — and gets
    its name, flags and place updated;
  - a new one gets a new block;
  - one the session removed loses its block;
  - the lists are relinked, and `act_track` / `actstrip` let go of a
    block that is gone.

  Saving twice gives the same stack.
- Truth:
  - phiphx `nla_edit.rs`: each operator step for step against bpy 5.0 on
    `nla_cube.blend`:
    - Select None, Invert, All;
    - Toggle Muting;
    - Delete;
    - Add;
    - Add Above Selected (NlaTrack.001 / .002 / .003);
    - Delete tracks.

    Also covered: clicks and the 7 px reach. Headless Blender cannot
    place a click in the NLA editor, so click select is a straight port.
  - `blender_opens_saves`: the nla_cube case now carries an NLA editor
    session — a click on Walk, H, Add Above Selected, a click on Overlay's
    name, Tracks ▸ Delete. Blender reads the stack back flag for flag
    (strip mute is now compared too). An object disabled in viewports is
    left out of the motion lines, since Blender does not evaluate it.
  - phiber `tests/nla_editor_edits.rs`, on `nla_cube.blend`, through
    clicks, keys and menus:
    - the Cube reaches x 4 at frame 25 and stays put once Walk is muted;
    - Undo and Redo;
    - two saves write the same stack.
  - Unit tests: `scene/action/nla.rs`, `nla_editor.rs`, `row/mod.rs`
    (`index_at`, `track_at`).
- Not yet:
  - Add ▸ Action (Shift A) and the other Add entries;
  - moving, extending and scaling strips (G, E, S);
  - Duplicate, Split, Meta;
  - box select, and Select Before / After Current Frame;
  - clicking the action line or an object row in the channel list;
  - the tracks' mute, lock and solo toggles;
  - tweak mode (Tab);
  - renaming.
- **The machine's tree.** Since batch 80 the machine gained a floor under
  number fields (`dimensions.number_min_width`, `ctrl/number.rs`) and a
  re-recorded golden. Batch 81 was built and tested on top of it. The
  golden, recorded again without the X-ray button, was re-recorded on the
  merged tree; it differs from the machine's only by that button. Ten tests
  fail on the merged tree, none from batch 81:
  - the eight from before;
  - `theme::descriptor` (the style catalog counts one field more);
  - `marker_box_and_transport_popovers` (the time-jump field's drag),
    both from the number-field work.


## Batch 80 — the Action menu: Push Down and Stash

- **The Action menu** (Blender 5.0's `DOPESHEET_MT_action`, entry for
  entry):
  - Merge Animation;
  - Separate Slots;
  - Move Slots to new Action;
  - Push Down Action;
  - Stash Action.

  As in `DOPESHEET_MT_editor_menus`, it is drawn only in the Action Editor
  while the active object has an action. The Channel menu now follows the
  same rule: in the Action Editor it shows only over an action. A menu can
  now carry a poll (`BMenuSpec::shown`, built with `item::set::polled`),
  and the header's menu row draws it only while the poll holds. The first
  three entries say they are not available yet.
- **Push Down Action** (`action.push_down`, `BKE_nla_action_pushdown`) lays
  the action into the object's NLA as the clip Blender makes of it
  (`BKE_nlastrip_new_for_slot`):
  - its range is the Manual Frame Range when the action has one (then the
    strip does not sync its length), else the slot's first key to its last;
  - it cycles when the action is Cyclic;
  - it goes in the top track when that has room and isn't locked, else in
    a new track named after the action;
  - it is selected and active, named uniquely among the object's strips,
    and mixes with the action line's blend, influence and extrapolation;
  - then the action is unassigned.
- **Stash Action** (`action.stash`, `BKE_nla_action_stash`) puts the
  action in a muted, locked "[Action Stash]" track kept at the bottom of the
  stack. The strip is unselected and syncs its length. Stashing the same
  action and slot twice says "Action+Slot has already been stashed", as
  Blender does. Either way the action is unassigned.
- **The NLA plays** (phiphx `blend/anim/nla_eval.rs`, from
  `animsys_evaluate_nla`):
  - tracks play bottom to top; only the solo track plays when there is
    one, and muted tracks never do;
  - each track plays the strip the frame falls in; Hold carries the first
    strip backwards and every strip forwards unless its extrapolation is
    Nothing (`nlastrips_ctime_get_strip`);
  - the frame maps into the action with the strip's scale and repeats
    (`nlastrip_get_frame_actionclip`);
  - strips mix by blend mode and influence, eased by blend in / out
    (`nla_blend_value`);
  - the object's own action plays over the result.

  The file's NLA now moves objects too; before this, only the action did.
  A Cube pushed down keeps moving as it did, held at the strip's start
  before frame 5.
- **One owner for the laying.** phiphx `anim/nla_stack.rs`
  (`BlendNla::push_down` / `stash`) lays the strips both for the session's
  scene (`scene/action/nla.rs`, the NLA editor and the viewport read it)
  and for the save (`write/nla.rs`). So what shows is what saves.
- **The session keeps the steps in order** under `ctx.scene.nla`, so Undo
  steps them. (Since batch 81 it keeps the stacks they leave instead.)
- **The save writes Blender's stack:**
  - `NlaTrack` / `NlaStrip` blocks linked into `AnimData.nla_tracks`;
  - each strip points at its action and slot (`act`,
    `action_slot_handle`, `action_slot_name`);
  - the flags as Blender leaves them: one active track, one active strip.

  phiphx `NlaStrip` / `AnimData` reading also gained blend mode,
  influence, blend in / out and the slot.
- Truth:
  - phiphx `tests/blender_opens_saves.rs`, in which Blender reads the
    written NLA back:
    - it pushes down Spin on the Light, and stashes a cyclic
      manual-range CubeAction copy on the renamed Cube;
    - each track and strip is compared field by field;
    - the world Y axis at frames 1, 5, 12, 24, 30 and 42 matches Blender's
      own NLA evaluation.
  - phiber `tests/action_menu_push_down_and_stash.rs`, on the actions
    fixture:
    - the menu is hidden in the Dope Sheet and shown in the Action Editor;
    - Push Down gives the track CubeAction with its strip over 5–30, and
      the Cube at x 0.3429 / 1.3817 / 1.8523 / 3.0 at frames 0 / 10 / 12 /
      20;
    - Spin plays on top;
    - Stash, then the second Stash's report;
    - Undo and Redo;
    - Ctrl S.

    Every value is from bpy after the same operators on the same file.
    Blender reads the saved file back with the same stack and motion.
  - `nla_stack.rs`, `nla_eval.rs` and `scene/action/nla.rs` unit tests.
- Not yet:
  - Merge Animation, Separate Slots, Move Slots to new Action;
  - Stash's "Create New Action" (`action.stash_and_create`);
  - tweak mode (Tab on a strip);
  - the NLA editor's own operators (add, move and delete strips and
    tracks);
  - meta strips and transitions don't play;
  - Combine mixes a quaternion as Replace.

  After an action is unassigned, Blender keeps the values the animation
  last wrote on the object; phiber returns it to the file's rest values.
- **The machine's tree, again.** Since batch 79 the machine gained a seam
  change (`view/draw/resize.rs`: no seam line at rest, each area's own
  border draws the boundary) with its test (`visual_surface_refinement`,
  which now passes) and a re-recorded layout golden. That golden was
  recorded without batch 79's X-ray button, so it failed against the
  machine's own tree. Batch 80 was built and tested on the merged tree,
  and the golden re-recorded there. It differs from the machine's only by
  the X-ray button and the header's fold, and from the cloud's only by the
  seam. The eight failures left all predate batch 80:
  - `action_editor_actions` (the sidebar's card ids);
  - `outliner_depth`;
  - `rail_foot_stays_in_view`;
  - `semantic_style_derivations`;
  - `style_budget`;
  - `viewport_sidebar` (two);
  - `viewports_keep_their_own_view`.


## Batch 79 — X-ray (Alt Z) in Solid and Wireframe

- **The switch** (`view3d.toggle_xray`, Blender 5.0's `VIEW3D_HT_header`): an
  X-ray button before the shading radio, and Alt Z over the viewport, flip
  the switch of the shading in use — Solid's (`show_xray`, off in a new
  file, alpha 0.5) or Wireframe's (`show_xray_wireframe`, on, alpha 0) —
  each kept apart as `View3DShading` keeps them (`scene/shading.rs`,
  `BXray`). The button is pressed while the switch is on. In Material
  Preview and Rendered it is dimmed (a control's new `muted` state, drawn by
  the one button renderer) and Alt Z still flips the switch and says "X-Ray
  not available in current mode", as Blender does.
- **What a file says**: a file's viewport X-ray settings (`View3DShading.
  flag`, `xray_alpha`, `xray_alpha_wire`) are read on open with its view
  (phiphx `scene/ui.rs`, `SceneXray`).
- **Drawn as Workbench draws it** (phiray `solid.wgsl`): every surface
  along the ray, front and back faces alike, at the X-ray alpha, blended by
  weighted blended order-independent transparency — Blender's
  `workbench_transparent_accum` / `_resolve` — and the floor grid drawn
  through, faded to `1 − alpha` behind the nearest surface
  (`overlay_xray_fade`). Blender's own Workbench render of two walls,
  checked at X-ray off, 0.5 and 0.2 (`phiray/tests/truth/blender_xray.py`),
  matches to within one level in every spot (`tests/xray.rs`).
- **Wireframe X-ray** (`wire.wgsl`): on by default, so the wireframe now
  shows every edge, the ones behind faces too, as Blender's does; below
  alpha 1 the hidden ones fade; with the switch off, edges behind faces
  hide as before.
- **Box select sees through** whenever X-ray is on (`BXray::active`) — in
  Solid now too, not only in Wireframe.
- **Solid matched to Blender on the way.** The truth render showed three
  gaps in phiray's port of Workbench lighting, now closed:
  - the incident vector is per pixel in a perspective view
    (`drw_view_incident_vector`), not fixed;
  - the reciprocal in wrapped lighting is Blender's bit-trick `fast_rcp`,
    whose error its shading carries;
  - roughness is the square root, stored in a byte (`Material::pack_data`),
    taken from the material's viewport Roughness / Metallic
    (`Material.roughness`, 0.4 by default), not the Principled BSDF's
    (phiphx `viewport_pbr`).
  Opaque Solid now matches Blender's render to within one level too.
- Truth: phiray `tests/xray.rs` (against Blender, and the wireframe's
  hidden edge off, whole and faded), phiphx `tests/view_xray.rs` (the
  factory file's values, and a file Blender saved with all four changed,
  `fixtures/make_xray.py`), phiber `tests/xray_toggle.rs` (the button, Alt
  Z in Solid, Wireframe and Material Preview), `scene/shading.rs` tests.
  The layout golden moved only in the 3D Viewport header's right group.
  phiray's Solid and grid tests set their sky without the theme sweep they
  predated.
- Not yet:
  - the X-ray alpha fields (the Shading popover);
  - click, circle and lasso select through X-ray;
  - Pose Mode's bone X-ray (`show_xray_bone`);
  - the CPU fallback painter (no GPU) has no X-ray;
  - the wireframe draws quads' diagonals, which Blender leaves out.
- **The machine's tree had moved on.** Since batch 78, work landed on the
  machine outside these batches — the flex tokens (`BFill`,
  `flex/{fill,paint,place,space}.rs`), `theme/style/elevation.rs`
  (`BElevation`), `view/draw/tone.rs` gone, the tab shape, and new tests.
  Batch 79 was built and tested on top of it. Only this batch's own files
  were copied, each checked first to be unchanged on the machine since it
  was read. Nine tests fail on that tree without batch 79 as well:
  - `action_editor_actions`;
  - `outliner_depth` (a row never shrinks);
  - `rail_foot_stays_in_view`;
  - `semantic_style_derivations`;
  - `style_budget` (`canvas/view3d/hit.rs`, `tree/disclosure.rs`);
  - `viewport_sidebar` (two);
  - `viewports_keep_their_own_view`;
  - `visual_surface_refinement` (the gutter surface).

## Batch 78 — I inserts keyframes

- **I over the 3D Viewport** (Object ▸ Animation ▸ Insert Keyframe,
  `anim.keyframe_insert`; the Animation submenu is Blender 5.0's
  `VIEW3D_MT_object_animation`): every selected object's Location,
  Rotation (Euler, quaternion or axis-angle, as its mode keeps it) and
  Scale are keyed at the current frame, at their values now — where the
  animation puts the object, or where the Transform panel or G / R / S
  left it. An object with no action gets `<Object>Action`; one with no
  slot gets the slot named for it (`OBCube`). After the key the animation
  drives the object, as in Blender (`scene/action/keying.rs`).
- **I over the Dope Sheet** (Key ▸ Insert Keyframes,
  `action.keyframe_insert`) and the Graph editor's Key ▸ Insert ▸ All
  Channels key every curve the shown objects have, each at its value now.
- **Blender's handles, computed as Blender computes them**. phiphx
  (`blend/anim/handles.rs`, `smooth.rs`, `tridiagonal.rs`) inserts a
  Bézier key with auto-clamped handles and then recalculates the handles:
  - `calchandleNurb_intern`;
  - then the Continuous Acceleration smoothing
    (`BKE_nurb_handle_smooth_fcurve`, a tridiagonal system kept inside
    its neighbours' values).

  Blender keeps a file's handles as written, so these must be exact. They
  match bpy 5.0.1 on eight key sets to 2e-4 (`tests/handles_truth.rs`).
  Blender itself recalculates every curve phiphx wrote and finds the same
  handles.
- **The keys go in as Blender lays them out** (`write/keys.rs`, from a
  file Blender wrote):
  - the slot, if missing;
  - a "Layer" with one keyframe strip over all time, if missing;
  - the strip's channelbag for the slot;
  - each curve, found by path and index or made in "Object Transforms"
    (coloured XYZ → RGB);
  - the curve list in group order, and each group's range.

  A key that was already there keeps its keyframe type. The slot is made
  before the assignments are written, so the object can be given it.
- **What the editors read**: the session keeps each insert (the action,
  the slot, the frame and the values) under `ctx.scene.keyed` and replays
  them over the actions (`scene/action/keys.rs`). So Undo steps them, and
  the Timeline, Dope Sheet, Graph editor and viewport all show them. A
  save writes each keyed slot's curves.
- Truth:
  - `phiphx/tests/blender_opens_saves.rs`: the Cube's copy gets a key on
    its X curve and a new Z scale curve; the added Crate's empty action
    gets a slot, a layer, a strip and nine curves. Blender reads every
    key and handle, and its own recalculation agrees ("blenders").
  - `tests/keyframe_insert.rs`: I at frame 1 makes CubeAction; a move
    then I at 20 hands the Cube to the animation, which puts it where the
    curve does at frame 10; I over the Dope Sheet at 30 keys all nine
    curves; Undo; Ctrl S writes it all.
  - `scene/action/keys.rs` unit tests.
- Not yet:
  - Blender's Dope Sheet I asks which channels (All, Only Selected, In
    Active Group); phiber keys them all. The Graph editor's, the
    Timeline's and the Outliner's I are unbound.
  - Delete / Clear Keyframes, keying sets and auto keying.
  - A renamed object's file animation is not found under its new name.
  - An object the session added or duplicated saves its keys but does
    not move by them in the viewport.

## Batch 77 — the Dope Sheet, Graph, NLA and Drivers menus are Blender's

- **Every menu filled** (`part/menubar/editor/anim/`): the Dope Sheet's
  View, Select, Marker, Channel and Key; the Graph editor's View, Select,
  Marker, Channel and Key; the Drivers editor's (the Graph editor's less
  Marker and the playback entries, plus Delete Invalid Drivers); the NLA
  editor's View, Select, Marker, Add, Tracks and Strips — its menu row
  corrected from an Edit menu Blender 5.0 does not have. Entries, order,
  submenus (Transform, Snap, Mirror, Keyframe / Handle Type, Interpolation
  and Easing Mode, Extrapolation Mode, the channel settings, Move, Add
  F-Curve Modifier, Density / Blend / Smooth, Cache, Area) and labels are
  the operators' own and the shortcuts the default keymap's, read with bpy
  (`blender_default.py`).
- **The keys work over the editor**, since a menu entry's shortcut is its
  binding: N (View ▸ Sidebar) opens and closes the Dope Sheet's sidebar,
  Alt P clears the preview range, the Marker menu is the Timeline's — M
  adds, G moves, F2 renames, Ctrl B binds the camera.
- **An entry phiber cannot do yet says so** in the status bar and the Info
  editor when picked or keyed (`Key ▸ Copy Keyframes: not available yet`)
  — Select, Channel and Key's editing, the view entries (the animation
  window always shows the scene range), the NLA's strips and tracks.
- The menus' actions live on one struct (`BAnimMenu`).
- Truth: `tests/animation_editor_menus.rs` (each editor's menu row and
  entries against Blender's, the Dope Sheet's N, Alt P, M and Ctrl C).
- Not yet: the Dope Sheet's Action menu (Action Editor, with an action:
  Push Down, Stash, Merge Animation, Separate Slots) and the menus that
  change with the mode or tweak mode (NLA ▸ Stop Tweaking).

## Batch 76 — the UV editor draws the mesh's UVs and edits them

- **The UV layout** (`view/draw/canvas/image/uv.rs`, Blender's
  `overlay_edit_uv.cc`): the UV editor draws the active and selected mesh
  objects' UV maps over its square. In Edit Mode every face is washed in
  the face colour, every edge is drawn light on a dark outline
  (`edge_display_type` Outline) and every corner is a vertex dot; in Object
  Mode only the edges show, dim (`show_uv`, `uv_shadow`). Colours from the
  palette in use (`theme/style/uv.rs`, `BUvColors`).
- **The map, face by face**: phiphx keeps each mesh's active UV map as its
  faces' corners (`SceneModel::uv_maps`, `scene/uv.rs`) — the base mesh, as
  Blender's UV editor shows it; the tracer's per-vertex UVs lose the seams.
  The startup cube's 24 corners match Blender's exactly
  (`uv_editor_shows_uvs`).
- **Workspaces put the object in their mode** (`WorkSpace.object_mode`,
  `part/modebar/workspace.rs`, checked with bpy): UV Editing and Modeling
  in Edit Mode, Sculpting in Sculpt Mode, Texture Paint in Texture Paint,
  Animation in Pose Mode, the rest in Object Mode; a mode the active object
  cannot take (Pose on a mesh) leaves it as it was.
- **The UV header as Blender 5.0 draws it** (`IMAGE_HT_header`,
  `part/image/uv/`): no mode menu (the UV editor's `sima.mode` is UV); while
  editing, before the menus — UV Sync Selection (on in a new 5.0 scene),
  the select modes (the mesh's while syncing, the UV ones else), island
  selection, sticky selection; after the spacer — the pivot, snapping and
  its element, proportional editing and its falloff; then the image's New
  / Open and the gizmo and overlay toggles. Each dropdown is the one header
  dropdown every editor now shares (`view/prim/menu.rs`, `BPick`: the Image
  editor's slot / layer / pass, the Action Editor's action and slot).
- Truth: `tests/uv_editor_shows_uvs.rs` (the map against Blender's, Edit
  Mode on entering UV Editing, the header's settings, sync off to the UV
  select modes, the pivot menu, Object Mode showing without editing),
  `theme/style/uv.rs` and `part/modebar/workspace.rs` tests. The layout
  golden moved only in UV Editing's header.
- Not yet: selecting, moving or unwrapping UVs; the UV map selector at the
  header's end; the tool settings saved to or read from the file; a narrow
  header's centre group overlaps its right group while scrolled.

## Batch 75 — the Action Editor finds, assigns, makes and unlinks actions

- **The action selector** (`part/action/template.rs`, Blender's
  `DOPESHEET_HT_editor_buttons._draw_action_selector`): the Dope Sheet's
  header in **Action Editor** mode, after the menus. The browse menu lists
  every action in the file and every one the session made, sorted as
  Blender's list is, the active object's ticked; picking one animates the
  object with it. New copies the action in hand (`action.new`,
  `BKE_id_copy`: `Spin` → `Spin.001`, same slots) or, with none, makes an
  empty `Action` (no slot until a key is set); Unlink takes it off. The
  slot menu beside it lists the slots an object can use, and None.
- **Slots as Blender 5.0 assigns them** (`generic_slot_for_autoassign`,
  checked with bpy): the slot the object had in that action before, else
  the one named for it (`OBCube`), else none — an action assigned with no
  slot moves nothing, as in Blender.
- **The Dope Sheet sidebar's Action tab** has its panels
  (`DOPESHEET_PT_action`: the action's name, Manual Frame Range — turned on
  at the keys' span — Start / End, Cyclic Animation; `DOPESHEET_PT_action_slot`:
  the slot's name and type). **Properties ▸ Object ▸ Animation** has the
  same selector (`OBJECT_PT_animation`, closed until opened).
- **Everything follows**: the session's actions become part of the scene
  the app reads (`scene/action/set.rs`, `BActionScene`: built once per
  edit) — the object moves by its new action in the viewport and the
  camera view, the Dope Sheet and Timeline show its keys, the NLA editor's
  action line names it. Three keys (`ctx.scene.assigned`, `.made_actions`,
  `.action_ranges`), so Undo steps them; renames and deletes carry over.
- **Each editor keeps its own mode**: every editor wrote its mode to
  `ctx.mode`, so switching the Dope Sheet to Action Editor put the 3D
  viewport back in Object Mode. Only the 3D viewport's mode is `ctx.mode`
  now; the others are `<editor>.mode` (`SpaceAction.mode`,
  `SpaceImage.mode`).
- **Saves** (phiphx `write/action.rs`, `BlendEdits::actions`, `assigned`,
  `ranges`): a copy is the action's ID block and every data block it owns,
  re-addressed; a new action is an empty `bAction` with Blender's first slot
  handle; an object with no `AnimData` gets one. **Blender 5.0 opens the
  result** with the copy's slot, the assignments and the ranges as meant
  (`blender_opens_saves`, case `actions`, fixture `make_actions.py`).
- **Reading fixed**: phiphx never read Blender 4.4+'s layered actions — the
  field names are `**layer_array` (not `*layer_array`) and the file's DNA
  keeps `ActionChannelBag` and a slot's `name` — so it had been falling back
  to the legacy curve list Blender also writes. It now reads the layers,
  the slots and their identifiers (a legacy action's one slot is
  `OBLegacy Slot`, as Blender names it on load), each user's slot, Manual
  Frame Range and Cyclic.
- Truth: `tests/action_editor_actions.rs` (the menu, assign, the Cube moving
  by Spin, New copying, Unlink, New empty, Undo, the sidebar panels,
  Properties' Animation panel, Ctrl S read back), `scene/action` tests,
  `blender_opens_saves` (`actions`).
- Not yet: renaming an action in the selector, the fake-user toggle, a new
  slot (`anim.slot_new_for_id`), Push Down / Stash, linking a collection
  twice; the Action Editor lists the channels of every shown object, not
  only the action's.

## Batch 74 — collections drag in the Outliner, Ctrl links, a ghost follows

- **A collection dropped on another nests in it** (`outliner_collection_drop`,
  `BKE_collection_move`): last among its children, its objects and child
  collections along. Into itself or anything under it is refused ("Cannot
  move Props inside itself"), the Scene Collection stays put, and a drop on
  its own parent does nothing. The nestings are a history (`ctx.scene.nested`)
  replayed in order over the file's tree (`scene/links/collection.rs`,
  `BCollectionNode`), each collection keeping its file path as its id — so
  its row, eye, checkbox and the objects moved into it follow it.
- **Ctrl-drag links an object** into a second collection ("Link inside
  collection", `ctx.scene.linked`): it stays where it was and shows under
  both; a plain drag afterwards moves it out of all of them. An object in
  two collections is hidden only when both hide it — its base takes every
  holding collection's flags (`BKE_layer_collection_sync`), which also fixed
  objects the file itself links twice.
- **The drag ghost**: the row's icon and name on a lifted chip follow the
  pointer (`view/draw/tree/drag.rs`, styled once through `BFlex` and carried
  into gpui's drag view).
- The drop rule moved into `part/outliner/drop.rs` (`BOutlinerDrop`: Move,
  Link, Parent, Nest); `scene/links.rs` became `scene/links/` (the edits, the
  tree, the collection tree). Undo steps all four keys.
- **Saves**: phiphx `write/relink.rs` gained `link_object_to` and
  `nest_collection` (the `CollectionChild` node leaves its parent and joins
  the new one, in the master's case in the scene's data); `BlendEdits`
  carries `linked` and `nested`. **Blender 5.0 opens the result** with Lamps
  inside Props and the object in both collections (`blender_opens_saves`,
  case `collections`, fixture `make_collections.py`).
- Truth: `tests/outliner_collection_drops.rs` (drag, ghost, refusals, Undo /
  Redo, Ctrl link, hiding, Ctrl S read back), `scene/links/collection.rs`
  and `part/outliner/drop.rs` tests.
- Not yet: Ctrl-drag of a collection (a collection in two parents — said
  in the log, not made), dropping between rows to reorder.

## Batch 73 — G / R / S: typed values, planes, the read-out, the guide

- **A number typed drives the transform** (`scene/xform/typed.rs`,
  Blender's numeric input): G Z 1.5 Return moves one and a half metres up,
  R Z 90 Return turns a quarter about Z, S 2 Return doubles. Digits and a
  point build it, minus flips it, Backspace takes a character back; while
  there is a number the pointer does not take over. Free, a typed distance
  runs along X and a typed angle about the view axis, as Blender's first
  numeric field.
- **Shift X / Y / Z holds it to the plane of the other two**
  (`scene/xform/constraint.rs`, "locking" the axis named): a move slides in
  that plane, a scale spreads over the two axes; the same key again frees
  it. A new constraint or number applies at once, not on the next move.
- **The read-out** — Blender's header text while a transform runs — heads
  the viewport's info lines: `D: 1.2000 m (1.2000, 0.0000, 0.0000) along
  X`, `Rot: 45.00°`, `Scale: 2.0000 2.0000 2.0000`, `D: [1.5|] along Z`
  while typing.
- **R and S draw the dashed guide** from the pivot to the pointer
  (`transform_draw_cursor`), in the gesture inks, through the same stroke
  the lasso draws with (`canvas/view3d/lasso.rs::stroke`).
- The modal gained its pointer (`at`), constraint and typed text; the key
  handling and the operator moved into `scene/xform/rule.rs`.
- Truth: `tests/transform_typed_and_constrained.rs`, `scene/xform/typed.rs`
  and `constraint.rs` tests.
- Not yet: local / normal orientations (X X), snapping (Ctrl), Tab between
  typed fields.

## Batch 72 — click a camera, a light or an empty to pick it

- **A click on what the viewport draws for a camera, light or empty picks
  it** (`scene/pick/extra.rs::BPickExtra`) — they have no faces for the
  pick ray, so, as Blender's select buffer does with `overlay_extra`, a
  click within 6 px of one of their lines makes that object active and
  selected (Shift adds it), ahead of the mesh behind it; the lines are
  overlays. Hidden and unselectable ones are left out, renames followed.
- **Box, circle and lasso select take them by their origin**, as they take
  a mesh (box select does so whatever the X-ray setting: their lines draw
  over everything).
- **A selected camera, light or empty draws in the selection's ink** — the
  meshes' outline accent, the active one a step lighter (`TH_SELECT` /
  `TH_ACTIVE`) — so a pick shows (`BExtraPalette::selected` / `active`,
  taken from the mesh palette, one owner).
- Truth: `tests/pick_cameras_lights_empties.rs` (a click on the Light's
  lines, Shift-click on the Camera's, a box round the Camera's origin),
  `scene/pick/extra.rs` and `canvas/view3d/extra.rs` tests.

## Batch 71 — the camera view, the lamps and F12 follow moved objects

- **A moved camera is the one you look through** — Numpad 0's camera view
  and F12's render both take the active camera where the Transform panel
  or G / R / S put it (or where its moved parent took it), not where the
  file left it. **A moved light lights the scene from there** — in
  Rendered shading and in F12 — and so does a light whose parent moved.
- How: the render pass already scopes the loaded model and the shown
  frame for the readers that have no state in hand (the tracer's lamps,
  the camera view, a final render's camera); it now scopes the edits'
  camera / light / empty worlds with them (`source::set_loaded_at` →
  `motion::set_moved`, from `BTransforms::extra_worlds`), and
  `motion::cameras_at` / `lights_at` read them at that frame. A camera's
  and a light's `location` now follow their matrix (it kept the file's
  under animation too).
- Truth: `tests/moved_camera_and_lights.rs` (the camera turned and moved:
  the scene's camera, the camera view's camera and F12's eye; the Light
  following its parent; back to the file's once the edits go).
- Picking a camera, light or empty by clicking its gizmo landed in batch 72.

## Batch 70 — cameras, lights and empties move too

- **Children of every kind follow** (`scene/transform/world.rs`) — until
  now only meshes took the Transform panel's and G / R / S's edits: a
  light parented to a moved cube stayed behind, and so did a camera or an
  empty. Every object's world is now worked out in one place
  (`BTransforms::worlds`, over `BTransforms::posed` — the file's meshes,
  cameras, lights and empties where the animation puts them), so a light,
  camera or empty follows its parent, and a mesh follows an empty parent.
  The viewport's gizmos draw from it (`extra::scene_extras_now`).
- **G / R / S on a camera, light or empty** moves it (select it in the
  Outliner): the transform starts from every selected object's world, not
  only the meshes'.
- **The panel shows an object's own transform** — the Object tab's (and
  the N panel's) Location / Rotation / Scale come from the object's
  `matrix_local`, so a child shows its values relative to its parent, as
  Blender's panel does (it showed the world rotation, and typing into it
  took that as the local one); a camera, light or empty fills the panel
  too (it kept the last mesh's values). phiphx keeps each camera's,
  light's and empty's `matrix_local` (`SceneModel::extra_locals`).
- **Checked in Blender**: `blender_opens_saves` now moves and turns the
  light as well — Blender 5.0.1 reads the same location, rotation and
  world position.
- Not yet: picking a camera, light or empty by clicking its gizmo in the
  viewport (the camera view and the rendered lights follow since batch 71).
- Truth: `tests/extras_follow_edits.rs`, phiphx
  `tests/blender_opens_saves.rs`.

## Batch 69 — G, R and S in the viewport

- **G moves, R rotates, S scales the selected objects** with the pointer
  (`scene/xform`, Blender's `transform.translate` / `rotate` / `resize`;
  also Object ▸ Transform ▸ Move / Rotate / Scale). The change is measured
  from where the pointer first stands:
  - **Move** keeps the object's origin under the pointer, on the plane
    through the pivot facing the view;
  - **Rotate** turns about the view axis through the pivot by the angle the
    pointer swept round it on screen, counter-clockwise as seen;
  - **Scale** scales about the pivot by how much farther the pointer is
    from it than where it started.
  The pivot is the median of the objects' origins. A selected object whose
  parent also moves is left to follow its parent, as Blender skips it.
- **X / Y / Z** hold the change to that world axis (pressed again: free) —
  a move slides along the axis to the point nearest the pointer's ray, a
  turn is about the axis, a scale is along it. While a transform runs its
  keys come first (`app/keymap`), so X no longer deletes.
- **A click or Return keeps it; Escape or the right button puts
  everything back.** G, R or S during another switches mode and starts
  over. The whole transform is one undo step, named Move / Rotate / Scale
  (the history is held while it runs, as a marker grab holds it).
- The moves land in the Transform panel's own key (batch 66), so the
  children follow, the panel shows the new values once kept, and Ctrl S
  writes `loc` / `rot` / `size`.
- The viewport's pick view gained its axes (`BPickView::axes`), shared by
  the ray, the projection and the transform.
- Not yet: typed values (G 2 Return), Shift + axis (a plane), local /
  normal orientations, snapping, the dashed line from the pivot, the
  header read-out; transforming the built-in primitives of an empty
  session (only objects of a file or made in the session move).
- Truth: `tests/grab_rotate_scale.rs`, `scene/xform/set.rs` tests.

## Batch 68 — the viewport's N sidebar

- **N opens the sidebar** (View ▸ Sidebar, now a checkable entry bound to
  N: `SpaceView3D.show_region_ui`, `part/sidebar/show.rs::BSidebarShown`,
  `<host>.show.sidebar`). The region stands left of the floating tab strip
  (`view/draw/rail.rs::beside`): the same sidebar column the file
  browser's source list uses (`BColumn::Sidebar`, the width every sidebar
  shares), with its grab strip on its leading edge — dragged below a unit
  it closes the region (`resize::BPanelEdge::beside_strip`). The floating
  pane widens by it, so the navigation gizmo keeps clear of it.
- **Picking a tab opens the sidebar on it; picking the tab showing puts it
  away** (`BSidebarOpenRule`). It is shut until opened, as Blender's
  startup file has it.
- **Item ▸ Transform** (`VIEW3D_PT_transform`) is the Object tab's
  Transform (`props::object::TRANSFORM`) over the same keys, so typing a
  location, rotation or scale here moves the object and its children
  exactly as the Properties editor does (batch 66); it is there while an
  object is active. **Tool ▸ Active Tool** shows the tool. **View ▸ Focal
  Length** is the area's own `View3D.lens` (`View3dKey::Lens`, per
  viewport, `scene::view_slot`): each viewport's sidebar edits its own
  lens, and the projection follows (`camera::fov_of_lens`). An area's
  views are now built under its view slot (`area::BAreaItem`), so a control
  built there edits that area's view.
- **A file's saved screen decides whether it opens** — phiphx reads the
  3D view area's `RGN_TYPE_UI` region and its `RGN_FLAG_HIDDEN`
  (`BlendView3d::sidebar` → `SceneView::sidebar`), and Load UI opens or
  shuts the sidebar to match. Fixture: `tests/fixtures/sidebar_open.blend`
  (`make_sidebar_open.py`, Blender 5.0.1, every viewport's sidebar shown).
- The sidebar tab tables declare what each tab draws
  (`BSidebarTabSpec::body`: `Object` / `Props` panels or the `View` panel);
  a host whose tabs draw nothing keeps its strip alone.
- Not yet: View ▸ Clip Start / End and the 3D Cursor panel; the sidebar in
  the other editors (Image, node editors, sequencer); G / R / S in the
  viewport.
- Truth: `tests/viewport_sidebar.rs`, `app/load_ui.rs` tests.

## Batch 67 — one ground: no dark strips, seam-wide gaps, pinned rail feet

Asked for by the user from a screenshot; each item names what caused the
inconsistency.

- **The Outliner's rows sit on the Outliner** (`view/draw/tree/mod.rs`) —
  cause: batch 52 painted every row as a Properties card (a leaf the card
  surface, a parent the card heading), one step over the region, with a
  gap between rows, so the list read as strips on a darker parent. A row
  at rest now paints its editor's body (`host_body_surface`), the region's
  own shade: content and ground are one. Hover, selection and the active
  row still mark a row.
- **The file browser's execute bar is a strip** (`view/draw/region`) —
  cause: `region_fill` gave Header, ToolHeader and Footer the header
  surface but left Execute clear, so the bottom strip showed the body
  while the two strips above it were header shade. Execute now takes the
  header surface and owns its corners like a footer.
- **The file browser's column gap is the zone seam** — cause: the source
  list and the file list were spaced by a content pad (`pad_md`, 8 px), and
  an empty N-sidebar tab strip (the File host's `Bookmarks` rail, one strip
  wide) stood right of the list, so the right gap was 28 px; the workspace
  spaces its areas by the seam (`resize.gap`, 4 px). The Center region of a
  host with panel tools now pads its shared edge by `BSpace::Seam`; the
  path strip's centre pane follows so the directory field still starts
  where the list does; the empty rail is gone from the File and Asset
  hosts (their bookmarks and options are the source list's cards). The
  outer edges keep the content inset, the same on both sides.
- **No dark vertical strips** — cause: the Properties / Preferences navbar
  painted `HostRail` (the chrome shade `bg_activity`), the activity bar's
  area card took `bg_activity` as its host background, and the channel
  lists painted the strip shade. All three now paint nothing of their own:
  the area's body is their surface, and only a tab's hover / active fill
  marks it. `BFill::HostRail` and `strip_gradient_soft` are gone.
- **A rail's foot is pinned** (`view/draw/rail.rs`) — cause: a strip
  rail laid its entries and its expand / paging cluster out as one column
  of groups, so a strip shorter than its entries pushed the cluster out
  of the region (the Properties navbar's arrow sat at y = 1010 in a
  900 px window). The entries now scroll inside the strip and the cluster
  is pinned under them, a cap gap (`menu_top_gap`, 2 px) off the bottom
  edge — the docked rails' region pads its foot by it, and the viewport's
  floating toolbar and sidebar end 2 px off the canvas's bottom
  (`BPlace::Float { foot }`).
- **Groundwork for the N sidebar** — the viewport's field of view key is
  now its focal length (`View3dKey::Lens`, mm, `View3D.lens`; the
  projection derives the width per distance from it), so the sidebar's
  View ▸ Focal Length can edit it directly.
- Truth: `rail_foot_stays_in_view.rs`, `file_browser_layout.rs` (the seam
  between the columns, equal outer insets), `region/tests.rs` (rails
  clear, Execute a strip), `semantic_style_derivations.rs` (the activity
  bar is the panel shade).
- Open: the palette's `bg_activity` token is still listed in the theme
  editor but no longer read by anything.

## Batch 66 — the Transform panel moves the object

- **Location, Rotation and Scale typed into the Object tab move the object**
  (`scene/transform.rs`) — until now the Transform panel only showed the
  active object's values. A value typed in moves it in the viewport at
  once, and its children follow it: an edited object's world matrix is its
  old one with its own part replaced (`world · local⁻¹ · local′`, the
  parent's kept — phiphx now keeps each object's `matrix_local`,
  `SceneModel::locals`), and a child's world moves by what its parent's
  did. One scene key (`ctx.scene.transformed`), so Undo steps it; a rename
  follows it, a delete forgets it, and picking the object again shows the
  typed values.
- **Saved, and checked in Blender** — Ctrl S writes `loc`, `rot` (XYZ
  Euler, `rotmode` set to XYZ) and `size` (phiphx `BlendObjectEdit::
  transform`), for file objects and the session's own. `blender_opens_saves`
  now moves, turns (45° about Z) and doubles the duplicate before
  parenting it: Blender 5.0.1 reads the same location, rotation, scale and
  world position.
- Not yet: the viewport's N sidebar (Item tab) with the same fields; G / R
  / S in the viewport; an empty, camera or light child following its
  moved parent (they are drawn from their own lists); the panel's
  Rotation for a parented object shows its world rotation, so typing into
  it takes that as the local one.
- Truth: `tests/transform_panel_moves_objects.rs`, `scene/transform.rs`
  tests, phiphx `tests/blender_opens_saves.rs`.

## Batch 65 — Outliner drag and drop: move to a collection, parent to an object

- **Drag an object row onto a collection row: it moves there**
  (`outliner_collection_drop`, Move) — out of every collection that held
  it, into that one. **Onto another object row: it is parented** to it
  (`parent_drop_exec`), keeping where it stands. A drop that would make an
  object its own ancestor is refused ("Loop in parents"), as Blender does.
  The target row lights while something is dragged over it
  (`view/draw/tree/drag.rs`, the tree primitive's `BTreeDrag`).
- **The Outliner shows the session's scene** (`scene/links.rs::BSceneTree`)
  — until now a loaded file's tree was the file as read: objects added,
  duplicated or renamed in the session were missing from it. It is now
  the file's collections with the session's renames, deletions, additions
  (in the Scene Collection, where `object.add` puts them), duplicates
  (beside their source), moves and parents applied; a collection's eye,
  camera and checkbox hide what it holds now.
- **Saved, and checked in Blender** — the moves and parents are two scene
  keys (`ctx.scene.moved`, `ctx.scene.parented`; Undo steps them, a rename
  follows them, a delete forgets them) written on save by phiphx
  (`write/relink.rs`: the object's collection links replaced; `*parent`,
  `partype = PAROBJECT` and `parentinv = parent⁻¹ · world · local⁻¹` so
  the child stays put). `blender_opens_saves` now moves the added cube
  and parents the duplicate to it: Blender 5.0.1 reads the same
  collections, the same parent, and the child's world position unchanged.
- **H deselects what it hides** (`object.hide_view_set`), like the eye and
  the monitor since batch 63.
- The layout golden gained the 120 rows' drag ids (`outliner-row-…`),
  nothing moved; the style budget gained the drop highlight (1).
- Not yet: dragging a collection onto another (nesting / reordering), Ctrl
  to link rather than move, and a drag ghost under the pointer.
- Truth: `tests/outliner_drag_and_drop.rs`, phiphx
  `tests/blender_opens_saves.rs`.

## Batch 64 — every 3D Viewport keeps its own view

- **One view per 3D Viewport** (`scene/view_slot.rs::BViewSlot`) — in
  Blender each 3D Viewport is its own `RegionView3D`; here the Animation
  workspace's two viewports turned, zoomed and framed together. Each view
  field — yaw, pitch, zoom, perspective, pan, pivot, field of view — is now
  kept per area (`ctx.view3d.<workspace>.<area>.<field>`,
  `View3dKey::Slot`) and read through the area it is asked for: the area
  being drawn (the area renderer scopes its host), else the viewport the
  pointer last moved over (what the Numpad keys, the View menu and the
  wheel act on). An area that never changed its view reads the shared one,
  which is what a file's saved view writes — so every viewport opens on the
  file's view (each area's own view is dropped on load) and goes its own
  way from there.
- **Orbits start from the view as drawn** — Numpad 4/6/8/2 turned from 0°
  when the view had never been orbited, while the viewport drew Blender's
  30° / 30°: the first press jumped the view. They start from what is
  drawn now.
- **Framing clears the pan in that viewport only** (it used to remove the
  shared keys).
- The log reads one viewport's key as the shared one ("Orbited to …",
  "Zoomed to …").
- Truth: `tests/viewports_keep_their_own_view.rs`, `scene/view_slot.rs`
  tests, `tests/gizmo_interaction_regression.rs`.

## Batch 63 — Blender's reading of collections and visibility; duplicates and hidden objects as Blender has them

- **The Blender check covers more** (`blender_opens_saves`) — besides
  batch 62's edits, each saved file now has an object hidden with the eye
  and the monitor, a collection with its eye closed and its camera off, and
  the added cube put into that collection; Blender reads every collection
  (its objects, exclude, eye, camera) and every object's viewport and eye
  state as phiphx does.
- **A duplicate lands in its source's collection** (`write/link.rs::
  collections_holding`) — Shift D in Blender puts the copy where the
  original is (`BKE_collection_object_add_from`); the writer put it in the
  Scene Collection. (A source linked into several collections gives the
  copy the first of them.)
- **Hidden means deselected** — Blender drops the selection of an object it
  cannot see when it evaluates the view layer (`BKE_base_eval_flags`), so a
  file saved hidden-and-selected opened deselected. The writer saves it
  that way, and in the session hiding an object with the eye or the
  monitor takes it out of the selection (`outliner::op::BHideDeselectsRule`);
  the camera and the arrow do not.
- Truth: phiphx `tests/blender_opens_saves.rs` (`truth/saved.txt`, 44
  lines from Blender 5.0.1), `write/edit.rs` test, phiber
  `part/outliner/op.rs` test.

## Batch 62 — Blender opens what phiber saves

- **Checked in Blender itself** (phiphx `tests/blender_opens_saves.rs`,
  `tests/truth/blender_saved.py`, `truth/saved.txt`) — the open question
  since batch 44 was whether Blender reads what the writer writes, not only
  phiphx. Four files (the 3.6, 4.2 and 5.0 startup files and `nla_cube`)
  are saved with a session's edits — frame and range, a rename, the camera
  toggle, selection and active object, markers, Playback settings, a cube
  added, an object duplicated, one deleted — and Blender 5.0.1 (`bpy`)
  opens each and reports what it sees. The test compares that with
  phiphx's reading of the same bytes, line by line: they agree.
- **A crash found and fixed** — Blender segfaulted opening any file with a
  **duplicated** object, in every layout. The copy kept its source's
  `totcol` with its `**mat` array cleared, and Blender read that many
  slots from a null array. A duplicate now gets its own copy of the
  source's `**mat` and `*matbits` arrays and the same `totcol` / `actcol`
  (`write/object.rs::copy_material_slots`), so it carries the source's
  materials as Blender's linked duplicate does; an added object starts with
  none (`fresh_object`, shared by both).
- To re-check after a writer change: run the test (it writes the files to
  `target/tmp/blender_opens_saves/`), then
  `python3 tests/truth/blender_saved.py target/tmp/blender_opens_saves tests/truth/saved.txt`
  with `bpy` installed, and run the test again.
- Truth: phiphx `tests/blender_opens_saves.rs`, `write/object.rs` tests.

## Batch 61 — middle-drag pans a header, Playback leads the footer

- **Middle-drag pans a header** (`view/draw/region/pan.rs::BStripPan`) —
  Blender's `view2d.pan` on a header region: with the middle button held,
  a strip too narrow for its contents follows the pointer sideways, from
  its start to its end and no further. The wheel and the drag share one
  offset (a `ScrollHandle` per strip, kept across frames and told apart by
  area — `BRenderCtx::area`, set by the area walker — so two 3D Viewport
  headers pan on their own). The press is heard before the buttons under
  it, and the drag window-wide, so it goes on past the strip's edge.
- **The Playback popover leads every other transport** — `playback_controls`
  opens with `TIME_PT_playback`; the Timeline carries it among its menus,
  and now the Dope Sheet's and Graph Editor's footers lead their transport
  with the same entry (`menubar::editor::set::PLAYBACK`, one spec for
  both). The layout golden's Action footers moved by that one button, in
  the Animation and Compositing workspaces, and nothing else.
- **The file list's column heading** was already a card heading (top
  radius, heading fill — `table.rs::BTableHeader`); open item 5 is closed.
- Truth: `tests/header_scrolls.rs`, `tests/dope_sheet_footer_is_the_transport.rs`,
  `region/pan.rs` tests.

## Batch 60 — a narrow header scrolls

- **Headers scroll sideways** (`view/draw/region/mod.rs::view`,
  `flex/stateful.rs::strip_scroller`) — an editor's header, tool header,
  footer or execute bar too narrow for what it holds is a horizontal
  scroller: the wheel pans it sideways, as Blender's header regions do, and
  no bar is drawn. The top bar keeps wrapping onto a second line instead.
  Menus and popovers opened from a scrolled header still draw over
  everything (they are deferred, outside the strip's clip).
- **No more overflow onto the next area** — until now a header's right end
  was painted past its own edge, over the neighbouring area's header (the
  Animation workspace's left 3D Viewport drew its shading buttons into the
  centre one's). It is clipped to its area now: the layout golden lost the
  12 quads that were painted outside their header and every header quad
  gained its header's clip, nothing moved.
- Middle-drag panning landed in batch 61.
- Truth: `tests/header_scrolls.rs`, `tests/golden/layout_fingerprint.txt`.

## Batch 59 — playhead snapping

- **The magnet and its Playhead popover** (`part/playback/snap.rs::
  BPlayheadSnap`, `panel::snap_body`) — `playback_controls` ends the
  transport with `use_snap_playhead` and `TIME_PT_playhead_snapping`: Snap
  Distance (px), Snap Target (Frames, Seconds, Markers, Keyframes — Keyframes
  on in the factory settings) and, for frames and seconds, their step. A
  scrub with the magnet on snaps as `anim_ops.cc::apply_frame_snap` does: a
  marker or key column within the distance wins, else the nearest frame or
  second step from the start frame, which is always taken. Ctrl held while
  scrubbing flips the magnet. What a scrub can snap to is read once per
  render of the window (`BSnapTargets`).
- **In the file** (phiphx `blend/scene/playback.rs`, `write/playback.rs`) —
  `ToolSettings.snap_flag_playhead`, `snap_playhead_mode`, `snap_step_frames`,
  `snap_step_seconds`, `playhead_snap_distance`, read and written where the
  file's DNA has them (Blender 5.1 on; 5.0's startup file has none, so
  there the settings stay in the session).
- **A magnet icon** — the placeholder circle in `assets/icons/magnet.svg`
  is a drawing now.
- The transport grew by the magnet and its chevron, so the layout golden's
  transport buttons moved (only those).
- Truth: `tests/playhead_snap.rs`, `part/playback/snap.rs` tests,
  phiphx `write/playback.rs` test.

## Batch 58 — box select sees what is in front

- **Box select skips what is hidden** (`scene/pick/occlude.rs::BPickScene`)
  — Blender reads the depth buffer under the box with X-ray off, so an
  object behind another is not taken. Here the box is sampled with rays
  (a grid as fine as a triangle budget allows, up to 32 × 32, plus one ray
  through each candidate's centre so a small object in a big box is not
  missed), and an object whose projected bounds overlap the box counts
  only when some ray meets it first. An object with its arrow toggle set is
  out of the pass, so what is behind it shows through, as in Blender's
  selection pass. The scene goes into world space once per box, not once
  per ray.
- **Wireframe is X-ray** (`BShading::sees_through`) — the factory file has
  `show_xray_wireframe` on, so in Wireframe the box keeps everything it
  overlaps.
- **One ray test** (`pick/triangle.rs::bounds`, `hits_bounds`;
  `pick::nearest_triangle`) — the pick and the box share the box cull and
  the triangle walk.
- Not yet: an X-ray toggle (Alt Z) for Solid — it would also have to draw
  the surfaces see-through; the samples are a grid, not every pixel, so a
  sliver thinner than a sample step can be missed.
- Truth: `scene/pick/occlude.rs` tests.

## Batch 57 — lasso select

- **Ctrl right-drag: Lasso Select** (`scene/lasso.rs::BLasso`,
  `canvas/view3d/lasso.rs`) — `view3d.select_lasso` as the default keymap
  binds it: the outline is drawn while the button is down (a point every
  4 px, `ctx.view3d.lasso.path`), dashed and closed back to where it
  began; on release every object whose origin is inside is added, or with
  Shift Ctrl taken away. A right-drag without Ctrl draws nothing.
- **One owner per gesture piece** — where an object's origin lands on the
  canvas is `BPickView::origin_on_canvas` (circle and lasso both ask it);
  the add / take-away op a gesture fires is `BBoxMode::fire` (was
  `BCircle::stroke`); and what the gestures paint over the canvas — box,
  brush, outline — is one `canvas/view3d/gesture.rs::BGestureInk`, read
  once per frame instead of a line per gesture in the canvas renderer.
- Box select stops taking hidden objects in batch 58.
- Truth: `tests/lasso_select.rs`, `scene/lasso.rs` tests.

## Batch 56 — circle select

- **C: Circle Select** (`scene/circlesel.rs::BCircle`,
  `canvas/view3d/circlesel.rs`) — `view3d.select_circle` as Blender runs
  it: a brush follows the pointer as a dashed ring; the left button (press
  or drag) selects every object whose origin is under it, Shift or the
  middle button deselects, the wheel resizes it (±2 px plus a tenth, never
  under 1 px; the radius is kept between uses, 25 px at first), and the
  right button or Escape puts it away. It stays on across strokes. A
  stroke only fires when it changes the selection (`BCircle::stroke`), so a
  drag records an undo step per change, not per pointer move. On the
  View3D Select menu beside Box Select, so the keymap has C.
- **One gesture layer** (`canvas/view3d/hit.rs::layer`) — the orbit, the
  box and the circle surfaces are the same whole-canvas layer with their
  own cursor; the styling moved out of `orbit.rs` and `boxsel.rs` into it
  (style budget −2).
- Lasso landed in batch 57. Blender tests the object's
  origin for circle select, as here — box select still counts projected
  bounds, not what is hidden behind another object.
- Truth: `tests/circle_select.rs`, `scene/circlesel.rs` tests.

## Batch 55 — Object Children, type boxes by file, Revert asks

- **Object Children** (`part/outliner/object_rows.rs`,
  `use_filter_children`, on by default) — the Outliner stays a table: a
  child's row is placed straight after its parent's (after the parent's
  data row) with the parent's depth + 1, and is not listed again at its
  collection's level. A parent with children gets the disclosure arrow even
  when it holds no mesh, and its closed row counts them in the icon row.
  Off, or while searching, the objects are one flat list. phiphx now keeps
  who is parented to whom (`SceneModel::parents`, `parent_of`, from each
  object's `*parent` pointer).
- **Type boxes only for what the file has** (`BOutlineFilter::file_has`) —
  as `OUTLINER_PT_options_filter` draws them: Meshes, Armatures, Lights,
  Cameras and Others appear when the open file holds that kind; Empties
  always.
- **Revert asks first** (`modal/revert/`) — File ▸ Revert opens Blender's
  confirm, "Revert to the Saved File" / "Any unsaved changes will be
  lost.", with Revert and Cancel in the platform's order; the same alert
  card as "Save changes before closing?". Only Revert reads the file again.
- Truth: `tests/outliner_children.rs` (fixture `tests/fixtures/parented.blend`,
  made by `make_parented.py` with Blender 5.0), `tests/revert_asks_first.rs`.

## Batch 54 — auto keying and time jump in the file, the Outliner's Filter

- **Auto Keyframing and Time Jump save** (phiphx `blend/scene/playback.rs`,
  `blend/write/playback.rs`) — the record button and its mode come from
  `ToolSettings.autokey_mode` (`AUTOKEY_ON`, Add & Replace / Replace) and
  Only Active Keying Set from `keying_flag` (`autokey_flag` in older files);
  Jump Unit and Delta from `RenderData.time_jump_unit` / `time_jump_delta`
  where the file's DNA has them. Opening a file seeds all five
  (`settings::seed`), Ctrl S writes them back (`settings::now`).
- **The Outliner's Filter popover, all of it** (`part/outliner/filter/`) —
  laid out like Playback: the restriction Columns, Sort Alphabetically,
  **Search** (Exact Match, Case Sensitive) and, in View Layer mode,
  **Filter**: Collections, Objects, Object Contents, the object state
  (All / Visible / Selected / Active / Selectable) with Invert, and the
  type boxes (Meshes, Armatures, Lights, Cameras, Empties, Others). Each
  one works: `BOutlineFilter` is read once per flatten and asked per row —
  a type or state drops the object, Objects off leaves the collections,
  Object Contents off makes an object a leaf, Collections off hangs every
  object straight off the Scene Collection, and search matches whole names
  or by case when asked. It is a panel: ticking a box keeps it open.
- **One owner for popover rows** (`view/prim/split.rs`: `BView::prop_row`,
  `check_row`, `select_row`) — the Playback, Time Jump, Auto Keyframing and
  Filter popovers build their rows from the same three constructors; the
  Outliner's popover used to be a list of icon buttons of its own.
- Not yet: All View Layers (phiphx reads the one view layer the file
  opens with). Object Children and the type boxes by file landed in 55.
- Truth: `tests/outliner_filter.rs`, `tests/playback_popover.rs`,
  phiphx `blend/write/playback.rs` tests.

## How much of the mock the app applies *(batch 40)*

The mock is *Sphere Shades* (`scratchpad/sphere_shades.html`, artifact v17):
eleven themes in both modes, the Rendering and Layout workspaces, the
Preferences sheet, the modals. Per area, what the app draws from the same
rules the mock draws, and what still differs. The percentage is the share of
that area's mock rules the app holds by a test.

| area | applied | what the app holds | what still differs |
| --- | --- | --- | --- |
| Ladder & themes | 100 % | eighteen steps per seed, eleven themes, dark/light per theme, every token a fixed step (`shade_ladder`, `theme_selection`) | — |
| Sweeps | 100 % | left → right, one step, back on the slot by half way; canvases and the tracer's sky (`semantic_style_derivations`) | — |
| Headers & strips | 95 % | 36 px strip, header shade, one centre line, switcher/pulldowns/toggles as 26 px buttons, footer a strip (`chrome_geometry`, `central_reuse`) | a card heading's right padding (open 1) |
| Cards & headings | 100 % | 30 px module, heading a step over its card, sub-panels a recessed, guided tree, grip and fold gestures (`sub_panels_are_a_tree`, `layout_gutters`) | — |
| Properties navbar & rails | 90 % | rail entries one module, active tab hands its shade to the panel, actions bar painted (`preferences_rail_shares_the_row_grid`) | entries centred on the activity rail's axis and its chat icon (open 4) |
| Viewport strips & gizmo | 100 % | bare 30 px buttons, theme-shade icons, accent when active, gizmo cluster incl. the camera button (`viewport_extras`, `camera_view_draws_like_blender`) | — |
| Outliner | 98 % | depth-indented table, one guide backdrop in the opposite shade, rows on the region's own shade (batch 67), selection tone, restriction toggles, count badges, exclude checkbox, the Filter popover (`outliner_tree_geometry`, `outliner_depth`, `outliner_view_layer`, `outliner_filter`) | All View Layers, drop-between reordering (open 3) |
| File Browser | 95 % | strip of columns, headed list on the card surface, column heading the card heading (`file_browser_layout`, `host_visual_fixes`) | — |
| Timeline | 100 % | transport in the header/footer, pause two slots wide, reverse play, frame fields, the Playback panel (`timeline_scrubs_like_blender`, `dope_sheet_footer_is_the_transport`, `playback_popover`) | — |
| Dope Sheet / Graph / NLA / Drivers | 85 % | the canvases, channels, modes and footers; the Action Editor's action and slot selector, the sidebar's Action panels, Properties' Animation panel (batch 75); every menu entry Blender has, with its shortcut (batch 77) | editing keys and channels (most entries report "not available yet"), the header's filters, snapping and proportional editing, the Action menu |
| UV Editor | 80 % | the layout, the View / Select / Image / UV menus, the tile grid, the mesh's UV layout in Edit and Object Mode, the header's selection, pivot, snapping and proportional settings (batch 76) | selecting and moving UVs, the UV map selector, the tool settings in the file |
| Rendering | 90 % | the layout, the Render menu and keys, the Render Result's slot / layer / pass selectors, Render and Output properties | Image ▸ New (falls back to Open) |
| Menus & popovers | 85 % | card surface, role radii, hover shade, shortcuts dim (`welcome_refinement`) | every item with an icon slot (open 2) |
| Fields & buttons | 100 % | `pad_md` margins, clipped text, 26 px, control shade at rest (`field`, `style_budget`) | — |
| Modals & Preferences | 95 % | the same headings and contents as the hosts, Theme card with Dark/Light, Family choice (`theme_selection`, `typeface_choice`) | the mock's Preferences shows the rail's actions bar at the foot of a scrolled rail — verify in the app at short heights |
| Canvases (tracer) | — | frames from phiray, blank frames guarded | the user's machine still shows black frames: awaiting `limits` / `color_render` output |

## Open — the queued layout list

1. **Header wrap** — the top bar wraps since batch 43, a narrow editor
   header scrolls under the wheel since batch 60 and pans under a
   middle-drag since 61; open: a card heading's default right padding.
2. **Menu items** — done: every row reserves the icon slot and labels line up (`view/draw/menu.rs`); rows round as top-only caps since batch 49.
3. **Outliner rows** — toggles and the icon row landed in batch 43, the
   exclude checkbox in batch 46, the badges' tooltip in batch 50, the
   Filter popover in batch 54, Object Children in 55, drag to move and
   parent in 65, collection drags and Ctrl link in 74; open: All View
   Layers, reordering by dropping between rows.
4. **Activity bar** — the chat icon; the channel and stream lists wait on
   the collaboration service writing `collab.*` (the panels are in place
   since batch 41).
5. **File list** — done: the Name / Date Modified / Size row is a card
   heading of the file column (confirmed in batch 61).
6. **Tracer** — the viewport renders on the user's machine since batch 40
   (the screenshot shows the cube and sphere); the scope fix in batch 41
   is what F12 was missing.
7. **Hotkeys** — the keymap is the menus' since batch 42, and acts on the
   viewport under the pointer since batch 64; the viewport's N sidebar
   landed in batch 68; still open: the other editors' sidebar panels, a
   Preferences ▸ Keymap sheet to rebind.
8. **Animation beyond objects** — a rig's pose per frame, its drivers,
   its constraints and IK, and the meshes it deforms landed in batch 85
   (THYMUS waves), ten more constraint kinds in batch 89, and constraints
   on objects and bone parents in batch 90, curve paths (parents and
   Follow Path, THYMUS's turntable camera) in batch 91; open: Spline IK and
   the constraints that need a mesh or a curve's nearest point
   (Shrinkwrap, Clamp To), NLA strips on a rig, drivers reading anything
   but a property. (Easing
   and linear extrapolation landed in batch 42; the tracer refits a new
   pose instead of rebuilding since batch 84; the rig poses on its own
   thread, ahead of the playhead, since batch 86.)
9. **Saving** — Save, Save As, Save Copy, Save Incremental, added /
   duplicated / deleted objects (batch 45) and a New File through the
   shipped `startup.blend`, with primitives as quads (batch 46), all
   write; the dirty flag, the "Save changes before closing?" prompt and a
   startup file per Blender version (3.6 / 4.2 / 5.0, matched to the open
   document) landed in batch 47. Open: the **materials** of an added
   object (none are written: it saves grey); **Link and Append** from
   another `.blend` (the Revert confirmation landed in batch 55); and — checked in batch 62 — Blender itself opens what the writer saves
   (`blender_opens_saves`), which found and fixed a crash on duplicates.
10. **Selection** — box select and F2 landed in batch 43, marker select
    and drag in batch 46, circle select in 56, lasso in 57, box select
    that skips hidden objects in 58; open: the X-ray toggle (Alt Z) in
    Solid.
    (G and F2 on markers landed in batch 51, box select over the marker
    band in 53.)

## Open — UV, Rendering and Animation *(audit, batch 74)*

Checked against Blender 5.0's `bl_ui` and its factory workspaces (bpy
5.0.1): the three layouts match area for area. What differs:

**Animation**
1. The Action Editor has no action selector (`DOPESHEET_HT_editor_buttons.
   _draw_action_selector`: `template_action` + the slot's `template_search`);
   the file's actions (`scene::source::scene_actions`) reach only the
   channel rows. Nothing lists them, assigns one, makes a new one
   (`action.new` copies the current action, else an empty "Action") or
   unlinks (`action.unlink`). — **done in batch 75**
2. The Dope Sheet sidebar's Action tab is empty (`DOPESHEET_PT_action`:
   name, Manual Frame Range, Start / End, Cyclic; `DOPESHEET_PT_action_slot`).
   — **done in batch 75**
3. Properties ▸ Object has no Animation panel (`OBJECT_PT_animation`, the
   same action and slot selector, closed by default). — **done in batch 75**
4. The Dope Sheet, Graph, NLA and Drivers menus are titles over empty lists
   (`part/menubar/editor/set.rs`), against `DOPESHEET_MT_view / select /
   channel / key` and their Graph and NLA peers. — **done in batch 77**
5. Nothing inserts a key (`anim.keyframe_insert`, I). — **done in batch 78**
   The Action Editor has no Action menu (`DOPESHEET_MT_action`: Push Down,
   Stash, …), and the file's NLA never moves anything. — **done in batch
   80** (the slot operators still say they are not available yet)

**UV Editing**
6. The UV canvas draws the 0–1 grid only (`canvas/image/set.rs::
   paint_uv_grid`); the mesh's UV layout — which phiphx reads
   (`blend/mesh/uv.rs::read_loop_uvs`) for the tracer's textures — is
   never drawn. — **done in batch 76**
7. The UV header's controls are missing: UV sync selection, sticky
   selection, pivot, snapping, proportional editing (`IMAGE_HT_header`).
   — **done in batch 76**

**Rendering**
8. Image ▸ New falls back to Open (`part/image/header/mod.rs`).

## Palette — decided: the shade ladder

One seed per theme, eighteen shades (`theme/palette/shade/`): shades 1–10 are
the seed at 10 % … 100 % over black, 11–18 white at 10 % … 80 % over the seed.
The default theme's seed is the sphere's mid-tone, `#3B4657`; the table it was
handed over as is `shade/set.rs::SPHERE_TABLE` and the guard holds the derived
hexes to it.

### The eleven themes, dark and light *(batches 31, 34, 35, 40)*

Preferences › Appearance (`BSettingsCategory::Appearance`, slug `theme`) leads with the *Theme* section — an accordion like the
sections under it, folding on the same key, one row per theme with its *Use*
button and the one in force lit (`part/pref::palette_card`). The Radii,
Effects and Strokes sections under it show that theme's own values
(`tests/theme_selection.rs`); the rail's caps sit on its rows' grid.

Every theme has a **Dark** and a **Light** mode (`BThemeMode`, `ctx.theme_mode`,
the switch in the Theme heading's preset slot), so the light themes are no
longer themes of their own: Silver became Grey's light mode, Apple Dark and
Apple became one Apple. Moss, Ember and Plum were removed to land on eleven
— redirect if other three were meant. A light mode reads the same ladder the
other way up (`BShadeLadder::light`) on a light seed: step 1 is the whitest
shade, step 18 the darkest, so every slot keeps its number and "one step
inward" is a step darker. The light seed is the dark seed lifted three
quarters toward white (`shade::lifted`, `LIFT_TO_LIGHT` = 75) — `#3B4657`
seeds `#CED1D5` — except Grey (the silver `#C4C6C9`, accent Tesla's
`#3E6AE1`) and Apple (macOS's `#CBCBCD`, accent `#007AFF`), which keep their
own. Tests compare inward, not lighter. `step_of` picks the nearest step,
not the first within tolerance — a near-black ladder's steps sit closer
than the tolerance.

| theme | dark seed | light seed | accent (dark / light) | radii (xs/sm/md/lg/xl) | sweeps | shadow | gap |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Sphere (default) | `#3B4657` | lifted `#CED1D5` | `#478AD1` | Soft 4/7/10/14/18 | on, .035 | Extra Large | 4 |
| Graphite | `#3D3D3D` (Blender's panel grey) | lifted | Blender's orange | Tight 1/2/4/6/8 | off | None | 4 |
| Tokyo | `#3B4261` | lifted | `#7AA2F7` | Medium 2/4/6/9/12 | on, .025 | Medium | 6 |
| Onyx (black) | `#2A2A2A` | lifted | `#D6247A` (the concept car's magenta) | Medium | on, .02 | Large | 4 |
| Grey | `#7A7A7A` | `#C4C6C9` (the silver) | `#E8912B` (Boldland's orange) / `#3E6AE1` (Tesla's UI blue) | Round 6/10/14/20/26 | on, .03 | Medium | 6 |
| Apple | `#38383A` | `#CBCBCD` | `#0A84FF` / `#007AFF` | Medium | off | Medium | 4 |
| Swift | `#3F3532` | lifted | `#FF5933` | Soft | on, .03 | Large | 4 |
| Barbie | `#4D2A3E` | lifted | `#ED1C8C` | Pill 8/12/16/22/30 | on, .04 | Extra Large | 8 |
| Fenty | `#4E3F33` | lifted | `#D18C59` | Round | on, .035 | Large | 6 |
| Cyberpunk | `#2C1C42` | lifted | `#00FFE6` | Square 0/0/2/3/4 | off | None | 2 |
| Chanel | `#3E3A30` | lifted | `#CCB88C` | Tight | on, .02 | Medium | 4 |

Removed in batch 40: Moss (`#3A4A3F`/`#D9A441`), Ember (`#4A3A36`/`#4FB3A9`),
Plum (`#433B57`, accent emerald `#1F8A5B` since batch 36), Silver (now Grey ·
Light) and Apple Dark (now Apple · Dark).

Brought back in batch 92 as themes of their own, each with a dark and a
light mode, for sixteen (`BPaletteName::MAX`): see Batch 92.

Swift, Barbie, Fenty, Cyberpunk and Chanel are the `phi` appkit's theme
variants (`phi/crates/appkit/src/theme/variants/*.rs`), each surface brought
up to a canvas seed and its accent kept; the appkit's `RadiusStyle`
(Razor · Tight · Standard · Soft · Round · Pill) maps onto the six scales.

`BShadowStyle` is None · Small · Medium · Large · Extra Large, mapped onto
the backend's rungs in one place (`flex/apply.rs::shadowed`); the popover and
every lifted container take it from there. The seam gap is the theme's
`resize.gap`, which every `BSpace::Seam` in the window reads.

Every token is a fixed step (`palette/set.rs::BPalette::on`), the same in
every theme, climbing inward as Blender's greys do (window `#181818`, header
and region `#303030`, panel `#3D3D3D`, field `#545454`):

| step | slot |
| --- | --- |
| 1 | modal dim, shadows |
| 2 | app background, seams, gutters |
| 3 | top bar, status bar, activity rail, Properties nav rail, modal title bar |
| 4 | every header, tool header, footer, scrub strip |
| 5 | every body — regions, sidebars, channel regions, modal bodies; the active Properties tab lands on it; a sub-panel's well (`Nested(1)`), one more step back per level |
| 6 | cards, the file list (a headed body), alternate rows; a sub-panel's heading (a step over its well) |
| 7 | card headings and the file list's column heading (`heading_surface(Card)`: a step over the card, swept 8 → 7 or flat 7); fields, dropdowns, buttons, menus (`bg_control`); a floating strip's entries and the rail's actions bar at rest |
| 8 | hover |
| 9 | active, selected, sorted column, outlines |
| 10 | every canvas — viewport, nodes, image, timeline, graph, NLA, drivers |
| 11 | grid lines on every canvas (`viewport_grid`; the timeline, dope sheet, graph, NLA and driver grids step one in from their canvas, the minor lines at half alpha; the node grid one more) — light on every graph host, as Blender's `#545454` grid over `#3D3D3D`; disabled text and icons |
| 12 | dim text |
| 13 | placeholder text |
| 14 | icons |
| 15 | field values |
| 16 | body text |
| 17 | header and heading text |
| 18 | highlighted text |

Off the ladder, because they mean something: the accent (`#478AD1` in Sphere,
Blender's orange in Blender Dark), the axis colours, the active-wire orange,
the status green, the search-match yellow, Blender's node-class colours.

The semantic layer (`theme/semantic/`) moves in whole steps: `step(color, n)`
along the ladder, `shade(color, lift)` as the nearest number of steps, a
gradient — when a theme turns them on — one step up at the left edge, back
on the slot's own shade by half the width (`SWEEP_STOP`).
A colour off the ladder keeps a lightness nudge, having no ladder to move
along. `host_background` is the panel shade for every editor and the chrome
shade for the bars; every strip is the header shade; hover and active are the
two shades every host shares. `tests/shade_ladder.rs` holds the table, the
token steps in every theme, the animation editors' surfaces, and that no file
under `view/`, `part/`, `host/` or `app/` calls `.shade(`.

## Assets
`phi\_to_delete\phiphx_assets` (924MB) was **deleted by the user** and is not
recoverable from here. 30 icons were recovered from `phi\assets\icons` and
`phi\crates\appkit\assets\icons`. ~130 placeholder icons remain, all
unreferenced; the guard test fails the moment one is used.
