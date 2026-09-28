# Phiber UI — status

**Suite: 780 passing, 0 failing** (phiber; phiphx 156 passing, 6 known
asset-dependent). Batches 17–73 are extracted on the machine; every item
below is covered by a test.

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
8. **Animation beyond objects** — armature deform per frame (`poses_at` →
   `modifier/armature.rs`), NLA strips, drivers, constraints; a tracer
   refit instead of a rebuild per frame for big scenes. (Easing and linear
   extrapolation landed in batch 42.)
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

**UV Editing**
5. The UV canvas draws the 0–1 grid only (`canvas/image/set.rs::
   paint_uv_grid`); the mesh's UV layout — which phiphx reads
   (`blend/mesh/uv.rs::read_loop_uvs`) for the tracer's textures — is
   never drawn. — **done in batch 76**
6. The UV header's controls are missing: UV sync selection, sticky
   selection, pivot, snapping, proportional editing (`IMAGE_HT_header`).
   — **done in batch 76**

**Rendering**
7. Image ▸ New falls back to Open (`part/image/header/mod.rs`).

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
