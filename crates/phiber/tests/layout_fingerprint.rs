//! The layout fingerprint: where every selector-tagged element lands, and
//! every quad, shadow and sprite the frame paints — in every space, at two
//! window sizes, and with each main modal open.
//!
//! This is the oracle for refactors of the render layer. Moving layout
//! decisions from one place to another must not move a single element, and
//! this is what proves it didn't: it compares the whole window against the
//! recorded golden, not a handful of hand-picked bounds.
//!
//! Record a new golden (only when a layout change is intended):
//!     UPDATE_LAYOUT_GOLDEN=1 cargo test -p phiber --test layout_fingerprint
//!
//! Re-recorded for the node editor. 74 golden lines no longer hold and **0 are
//! new** — the asymmetry is the whole account: elements were removed and none
//! appeared.
//!
//! The Shader/Node canvas used to draw two hardcoded cards, "Principled BSDF"
//! and "Material Output", at two hardcoded fractions, with a hardcoded link
//! between them — the same three elements for every file, including no file at
//! all. It draws the material's real node tree now (`phiphx`'s `shade/graph.rs`
//! reads `bNodeTree.nodes` and `.links`), and every scenario in this golden
//! loads no file, so there is no tree and the canvas correctly paints nothing
//! over its grid.
//!
//! The 74 are those three mock elements and their text, across the scenarios
//! that show a node editor. The canvas, its grid and every surrounding region
//! are untouched: no geometry moved, nothing was re-coloured, and nothing that
//! belongs to a real graph has appeared because no scenario here has one.
//!
//! Re-recorded for the vertical strips. 48 golden lines no longer hold and 418
//! are new; every one of them is one of two things, and the two are the same
//! change seen from both sides.
//!
//! - The 48 are the strip backgrounds, one per space and per modal at both
//!   window sizes. Their fill moved from `l: 0.91078` to `l: 0.89294` — off
//!   `step(body, -1)` and onto the body itself. Blender paints a nav bar in
//!   `TH_BACK`, the same colour as the region beside it
//!   (`region_background_color_id`, `editors/screen/area.cc:3012`, which
//!   returns `TH_BACK` for everything that is not a header or a preview). No
//!   geometry moved: same origins, same sizes, fill only.
//! - The 418 are resting tab chips, which did not exist before. A rail entry
//!   at rest painted nothing and showed the strip through; with the strip now
//!   level with the panel, an unpainted tab would be invisible, so each one
//!   paints `rail_tab_resting` — and that lands at `l: 0.91078`, which is
//!   exactly the shade the strip used to be. The two colours swapped roles,
//!   which is the whole of this change: Blender's `wcol_tab` has
//!   `inner = 0x1d1d1d` against a `back` of `0x303030`, so the chip is the
//!   recessed one and the strip is not.
//!
//! Nothing else moved. No element appeared or vanished from the tagged set —
//! the new lines are all `*-background` paint layers of entries that were
//! already there, and the Profile column's card, which is the same container
//! every other secondary activity column already had.
//!
//! Re-recorded for the header rule and the nested-accordion flattening. The
//! drift was read element by element before it was absorbed: 15134 tagged
//! elements in, 15134 out — nothing appeared and nothing vanished — and every
//! move is one of four things:
//!
//! - a content header now closes with a rule along its bottom edge, drawn
//!   below its band (the strip is a stroke taller, so nothing inside it
//!   moves). 6690 elements slid down exactly 1px because they sit under one
//!   header; 188 under two (a header and a tool header); 652 regions took the
//!   same 1px off their height. The global bars are not ruled — the top bar
//!   draws its one line on the area itself — so the window's own grid is
//!   unchanged;
//! - a sub-panel lost its `pad_sm` indent and its left guide, so 1364
//!   elements start 4px left and run 8px wider: a nested accordion now sits
//!   on its parent's own row column, with the same space on all four sides;
//! - the Properties host is labelled "Inspector" rather than
//!   "Controls/Inspector", which takes 59px off its switcher and its Left
//!   pane (84 elements narrower, 42 shifted left);
//! - 5662 elements did not move at all.
//!
//! No element moved that those three changes do not explain.
//!
//! Re-recorded again for the rail's pinned cluster. 15134 elements in, 15134
//! out; 15018 did not move. The 116 that did are the Properties navbar's and
//! the tool rail's expand / paging clusters, which came UP by 198 and 298:
//! they had been pushed past the foot of their strip — off the bottom of the
//! window in a short one — because dividing a strip's height only pins the
//! end group while the entries fit. The entries scroll now and the cluster
//! keeps its size (`view/draw/rail.rs`, `tests/rail_foot_stays_in_view.rs`).
//! Nothing else moved at all.
//!
//! And again for the Properties header's Options panel
//! (`PROPERTIES_PT_options`, `tests/properties_header_carries_its_options.rs`):
//! 84 elements appeared — the control and the `pane-Right-Buts` it stands in,
//! in every space at both sizes — nothing vanished, and the only movement is
//! the 42 search fields that gave up 34px to make room for it. That host's
//! header right end had been empty; it is the one editor where Blender's is
//! not.
//!
//! And for the Timeline's sidebar (`tests/the_timeline_carries_a_sidebar.rs`):
//! 60 elements appeared — the collapsed strip and its one tab, in every
//! space at both sizes — 60 gave up the 20px it takes, and nothing vanished.
//! The remaining 30 rows are all `pane-Right-Time`, and they did not move:
//! that selector is `pane-<kind>-<host>`, so the Timeline's header right
//! pane and its new content-region right pane share it, and the capture
//! records whichever painted last. Both are still drawn; only which one the
//! id reports changed.
//!
//! And for the rail's own shade (`tests/the_properties_strip_is_a_strip.rs`):
//! nothing appeared or vanished and 14958 elements did not move. A vertical
//! strip paints a surface now — a step UNDER the host's body, the one rung
//! of the ladder that goes down — so the active tab, which is painted in the
//! body's shade, is a hole cut in it rather than the same flat field. That
//! is 48 new quad lines and no movement at all. The elements that did move
//! are the expand / paging cluster and what sits beside it: the bar is now
//! inset 4px from each of the strip's edges — a bar is a control and keeps a
//! button's gap, where a tab spans the rail and meets the pane — and that
//! inset comes OUT of its width (it shifted 4 and narrowed 8), because a
//! full width plus a margin overflows, which is what ran the expand arrow
//! past the edge it was being held off.
//!
//! And for the Text editor's tab set: 6 lines went and none arrived. Its
//! sidebar had a "Text" tab and a "Properties" tab, which is one more than
//! Blender has — `space_text.py` registers `TEXT_PT_find` and
//! `TEXT_PT_properties` under one `bl_category`, "Text" — so the second tab
//! and its icon are gone from the Scripting workspace at both sizes
//! (`sidebar::text`). Nothing moved.
//!
//! And for the animation editors' filter and search
//! (`tests/the_animation_editors_filter_and_search.rs`): 8 elements arrived,
//! nothing left, nothing moved. They are the Dope Sheet's filter control and
//! its search field, in the Animation and Compositing workspaces at both
//! sizes — that header's middle and right end had both been empty, and
//! Blender fills them from one `DopeSheet` shared with the Graph, Drivers and
//! NLA editors.
//!
//! And for the viewport's two header popovers
//! (`part::filter::view3d`): 76 elements arrived, nothing left, 15216 did
//! not move. `VIEW3D_HT_header` ends with the gizmo popover, the overlay
//! popover and the shading radio; only the radio was here, so two of the
//! three were missing from the editor you spend the most time in. The 66
//! elements that moved gave up the 68px the pair takes.
//!
//! And for the viewport's overlay defaults
//! (`tests/the_viewport_always_draws_something.rs`): nothing moved at all —
//! 14938 elements identical, 14744 quads in and 14744 out. The 420 that
//! changed are the same ten toggles in 42 captures, and what changed is
//! their NAME: `show_floor-true-square` became `show_floor-false-check_square`
//! because the id encodes the state, and they ship on now as Blender ships
//! them. The 118 quads that differ are those ticks. A fresh file had no
//! floor, no axes and no gizmo because every one of these defaulted off.
//!
//! And the shading radio's four icons: 152 lines in, 152 out, positions
//! identical. `circle_dashed` / `circle` / `sphere` / `sun` became `box` /
//! `shading_solid` / `material` / `render` — two of the four had been the
//! same circle with a different dash, which is no way to tell a mode apart.
//!
//! And the node editors' overlay popover
//! (`tests/editor_headers_carry_their_box_panels.rs`): 12 elements arrived,
//! nothing left, nothing moved. `NODE_HT_header` ends with `NODE_PT_overlay`
//! and this had nothing there at all, so the control and the `pane-Right-*`
//! it stands in appear in the Shading, Compositing and Geometry Nodes
//! workspaces at both sizes. The Sequencer's is the same panel over its own
//! table and is not in any captured space.
//!
//! And for the four headers that followed them (same test): 14 lines arrived,
//! 10 left, 30308 are identical — and eight of the ten are a RENAME at the
//! same pixel. The Image and UV editors already had an overlay button, but it
//! was `ctx.image.overlay`, a toggle nothing read and nothing drew; it is
//! `IMAGE_PT_overlay` now, a popover of seven boxes over the same
//! `SpaceImageEditor.overlay` those two editors share in Blender. Same
//! 30 × 26 at the same x and y in the UV Editing, Texture Paint, Shading and
//! Rendering workspaces — one control replaced in place, not a second one
//! added, which is why `part::filter` is kept off those two headers
//! (`draws_its_own_right_end`). The two quads that changed did so at the same
//! bounds: a popover anchor paints a flat surface where the toggle painted a
//! gradient. What is genuinely new is the Spreadsheet's `Filters` control and
//! its quad, in the Geometry Nodes workspace at both sizes — four lines. The
//! Movie Clip editor's `Display` popover is real too, but its editor is in no
//! captured space.
//!
//! And re-recorded for the theme mode the app opens in: `BThemeMode::DEFAULT`
//! is Light. Not one element moved — 15372 tagged elements in, 15372 out, and
//! every one of them at the same bounds. 14193 quads in and 14193 out, 198
//! shadows in and 198 out; what changed is 12119 quad fills and all 198 shadow
//! colours, which is precisely a ladder read from white instead of from black.
//! A theme mode is a colour change and nothing else, and the fingerprint is
//! what says so: the same window, repainted.
//!
//! The recording absorbed a drift that was already here, and it was NOT the
//! theme's. Against the golden as committed, 957 lines went and 398 arrived;
//! against a recording of the same source without the theme change, 0 and 0.
//! So the golden had been left behind by an earlier change and the 559-line
//! shrink belongs to that, not to Light. The 218 elements it moves are the
//! Timeline's frame cluster — `ctx.frame`, `ctx.frame_start`, `ctx.frame_end`,
//! `ctx.preview_range` and their chevrons, 262px to the right in 19 captures —
//! together with the Image and UV headers' right ends (`ctx.image.gizmo`, the
//! `Overlays` popover, `pane-Right-Image`, `pane-Right-Uv`, `pane-Right-Action`,
//! `pane-Center-Action`, `pane-Center-Spreadsheet`). Nothing appeared or
//! vanished in that drift either. Whoever moved those knows why; this file only
//! records that the re-record took it in, and that it is separable from the
//! theme flip because the baseline was measured both ways before recording.
//!
//! And re-recorded for one row height (`tests/a_row_is_one_height.rs`). Again
//! nothing appeared or vanished: 15372 elements, 14193 quads, 198 shadows, the
//! same counts as before, and 27101 lines identical. The 2662 that changed are
//! the tree's rows and what sits on them. `outliner-row-*` went from 26 to 30
//! — `row_height` is `BUTTON_HEIGHT + HEAD_ROOM` now, the sum
//! `card_head_height` had been computing privately — so a tree row, a card row
//! (already 30) and a section's heading row (already 30) are one height. The
//! tree's row PITCH was already 30, from a 26 row plus a 4px gap; it is 34 now,
//! a 30 row plus the same gap, which is the visible cost of the change and the
//! reason it is worth stating rather than absorbing quietly.
//!
//! `section-<name>` is the row and reads 30; its `-label` (18) and `-value`
//! (26) are a caption and a control standing inside it, and those are not rows
//! — which is the distinction `a_row_is_one_height` has to make too.
//!
//! No menu row appears in this drift, because no menu is open in any capture.
//! The squashing fix that came with it — a popover's row caps now keep their
//! size, where a long menu used to shrink them to 17px — is measured by opening
//! a menu, in `a_row_is_one_height::a_menus_rows_are_rows`.
//!
//! And re-recorded for the Welcome backdrop
//! (`tests/the_welcome_backdrop_follows_the_theme.rs`): TWO lines changed, both
//! the same quad — the backdrop's full-window ground, in the splash capture at
//! both sizes. It went from L 8.6% to L 92.9%, which is a dark slab becoming a
//! light one, because the ground is a ladder rung now instead of a frozen hex
//! and the ladder is read from white. 29761 lines are identical; nothing
//! appeared, vanished or moved.
//!
//! Only the ground shows here. The four `brand/welcome_bg_*` layers over it go
//! through `paint_svg`, which this fingerprint does not record, so their shades
//! are checked by the rung arithmetic in that test instead — including that
//! each one still stands clear of the ground it sits on, in all 22 theme-and-
//! mode pairs.
//!
//! And re-recorded for Blender's property split
//! (`tests/a_popover_is_as_wide_as_its_rows.rs`): 15372 elements in and out,
//! 14193 quads, 198 shadows — the same counts, so nothing appeared or vanished
//! — with 26709 lines identical and 3054 changed. Every one of the 3054 is a
//! `section-*` row or something standing in one, which is the whole of what the
//! change touches.
//!
//! Both columns were `fills()`, which divides a row in half. `use_property_split`
//! does not: the label takes 40% and the value 60% (`UI_ITEM_PROP_SEP_DIVIDE`).
//! Half a row spent 110px on the word "Sync" while the value column opposite it
//! was too narrow for its own control, so the Playback popover read "Limit to
//! Frame R" and "Follow Current F". A heading is one or two words; what faces it
//! is a tick and a sentence, and they were being given the same room.
//!
//! And re-recorded twice more. First for the Outliner's band and the resting
//! control alpha: 3021 lines changed and NOT ONE of them an element — pure
//! colour. `BRowShade::Alt` had been falling into a catch-all that painted the
//! host body, so the Outliner banded every other row and the renderer threw it
//! away; and `control_rest_alpha` was declared, valued and never read, so a
//! resting ground was as solid as the panel under it.
//!
//! Then for Preferences being Properties (`part/props/pref/control`): 384 lines
//! left, 628 arrived, 29379 identical, and every one of them in `modal:settings`
//! at both sizes — nothing outside the Preferences modal moved at all. Elements
//! went 15372 → 15440 and quads 14193 → 14369, which is the point: a number was
//! a text field with a hand-built "Decrease" and "Increase" either side, and
//! `BCtrlKind::Number` is a field with a real step on each side. The new parts
//! are the controls the Properties host has had all along.
//!
//! And again for two controls that were drawn but not painted: 128 lines moved,
//! 293 arrived, nothing vanished and no existing quad changed. Both changes are
//! a control that had been asking for a treatment it never got, and between
//! them they account for every line:
//!
//! - the X-ray button has an asset (`assets/icons/xray.svg`). `icon_coverage`
//!   was naming it as the one icon the UI asks for and cannot draw, and an icon
//!   button with no asset paints nothing and registers no bounds — so the button
//!   was in the View3D header, clickable, invisible. It appears in all 38
//!   captures now (19 × 2 sizes), and the 128 moves are the room it takes: one
//!   button, 30px and a 4px gap, so every move is exactly ±34 and in the
//!   direction its row's justification implies. In the 31 captures whose header
//!   is pinned to the shading group, Gizmos and Overlays slide 34 left (62) and
//!   the shading radio does not move; in the 7 that flow from the left, the four
//!   shading buttons slide 34 right (28) and Gizmos and Overlays do not move;
//!   and `pane-Right-View3D` is 34 wider in all 38, growing leftward in the 31
//!   and rightward in the 7 (38). 62 + 28 + 38 = 128;
//! - a zone toggle paints its state (`view/draw/toggle.rs` through
//!   `chrome::paint`). 96 of the 127 new quads are the 96 zone-toggle buttons in
//!   the capture, one each, exactly at their bounds: a shown zone's toggle rested
//!   on no fill at all before, so the only thing separating it from a hidden
//!   one was the brightness of a 16px glyph.
//!
//! The remaining 31 quads are the Gizmos button's own fill at its new x, in the
//! 31 captures where it moved. Nothing went away because nothing was vacated:
//! each position a button left was taken by the next button along, painting the
//! same fill, so those lines are identical and only Gizmos' new home is new.
//!
//! And once more for a Preferences control showing its own default: 168 lines
//! left, 168 arrived, all of them in `modal:settings`, 84 at each size — nothing
//! moved, no quad changed, and no element appeared or vanished. The changed lines
//! are number steppers, and what changed is the value each one steps TO, which
//! the selector carries: `dimensions.activity_width-0-chevron_left` and `-1-`
//! became `-63-` and `-65-`. `BCtrl::read`'s fallback had been shaped by the
//! control's type rather than the preference — `Float(min)` for a number,
//! `Text("")` for a colour — so an unset width sat at 0 and offered to step to 0
//! and 1, where its declared default is 64. It steps around 64 now
//! (`part/props/pref/control/mod.rs`). The geometry is identical because nothing
//! about the layout changed: only what the controls were saying.
//!
//! And for a muted control being faded rather than recoloured: 240 quads left,
//! 240 arrived, every one of them at the same position with the same hue,
//! saturation and lightness, and nothing else in the capture touched — no
//! selector moved, appeared or vanished. The only difference is alpha, 0.5 to
//! 0.25, which is `control_rest_alpha` times the new `control_muted_alpha`: six
//! controls per capture across all 40 are there but not in force, and their
//! resting ground now fades with the rest of them. "Off" and "not available" had
//! been one look — `button.rs` passed `active && !muted`, dropping the fill, and
//! swapped the ink for `text_dim`, which is what an inactive tab's label takes.
//!
//! And for the font chooser becoming openable: 4 lines left, 8 arrived, all in
//! `modal:settings`, which opens on Typography — and that section's one row is
//! `typography.family`. The chooser had its eight families attached all along but
//! no `open_key`, and a `Select` with nowhere to record open-ness renders as a
//! label-only button with no action, which is why it had no selector of its own to
//! appear in a capture. It is an anchor now
//! (`btn-label-toggle-ctx.prefs.open.typography.family-text`, 2 appeared), the
//! reset beside it narrowed to make room (2 moved), and the two of them paint two
//! fills where there was one (2 quads became 4). Nothing vanished. The colour
//! swatches are not in this drift because this capture shows Typography, not
//! Appearance.
//!
//! And for the alternating row being Blender's: 212 quads left, 206 arrived, no
//! selector moved, vanished or appeared. 120 go 91.08 → 89.29 and 6 stop recording a
//! fill at all — those are the rows I had banded off the body, returning to the plain
//! background, because Blender paints every OTHER row and leaves the rest alone. The
//! other 86 go 87.51 → 87.89: the banded rows moving from a fixed ladder rung to the
//! body blended 4/255 toward the readable extreme, which is `row_alternate =
//! RGBA(0xffffff04)` through `immUniformThemeColorBlend(TH_BACK, TH_ROW_ALTERNATE,
//! …)` in Blender 5.2.0. 1.78 points became 1.40, against Blender's own 1.33.
//!
//! Re-recorded for the Properties tab strip, which was reading inside out. An
//! unselected tab drew a darker chip and the selected tab drew nothing, so the
//! chips were the loudest thing in the editor and the selection — the thing the
//! chips exist to point at — was the one tab with no mark on it. The step that
//! chip used now belongs to the editor's body (`host_body_surface` for
//! `BHostType::Buts`), so the strip is flat, a resting tab is strip, and the
//! selected tab carries the body's shade and sinks into the panel it opens.
//!
//! 454 golden lines no longer hold and 84 are new. Every one is accounted for:
//!
//! - **370 removed outright** — every resting tab's chip: 328 at 76x60 and 42
//!   at 60x60, across every space, every modal and both window sizes. Nothing
//!   replaces them, which is the point: a resting tab now paints nothing.
//! - **84 changed fill and nothing else** — same origin, same size, every one
//!   `l: 0.89294` → `l: 0.91078`, which is the one ladder step moving from the
//!   chip to the body. 42 are the Properties body quad (560 wide, four heights
//!   across the scenarios) and 42 are the selected tab (76x60), which take the
//!   same shade because they read the same token and are meant to be one
//!   surface.
//!
//! No geometry moved anywhere: not one origin, size or clip differs, and the
//! only quads that changed are the strip's own and the body it opens onto.
//!
//! Re-recorded again, for square icon buttons. An icon-only button's height was
//! always `button_height`; its WIDTH was its horizontal padding twice plus an
//! icon, and that padding differs inside a strip and out (`pad_md` against
//! `pad_sm`). So one transport button came out one width in the Timeline's
//! header and another in a panel, and no two icon rails agreed on how wide an
//! icon is. Blender has no such drift — `UI_UNIT_X` and `UI_UNIT_Y` are both
//! `U.widget_unit`, so an icon button is a unit square by construction — and it
//! is decided in one place here now (`view/draw/button.rs`).
//!
//! 4371 golden lines no longer hold and 4392 are new, and the shape of that is
//! the account:
//!
//! - **Not one colour changed.** Every `l:` and every `hsla(...)` in the lines
//!   that left appears in the lines that arrived and the reverse. This is a
//!   pure geometry re-record; nothing was re-shaded, re-themed or re-inked.
//! - **Not one height changed.** The quads that moved are 52 and 60 tall before
//!   and after, in the same counts. Only widths and origins differ.
//! - **1298 quads went from 60x52 to 52x52** — the icon buttons themselves,
//!   square now (52 is `button_height` at the golden's 2x capture scale).
//! - **502 went from 52x52 to 28x52** — the padding box inside each of those,
//!   collapsing onto the icon's own slot now that the square owns the width.
//! - **Everything else is downstream shift.** A row holding a button that lost
//!   8px re-measures (162x60 → 170x60 and so on) and every sibling after it on
//!   that row moves by the same amount. That is why the count is four thousand
//!   and not two thousand: one width change per button, and one re-measure for
//!   every element sharing its row.
//!
//! The net is +21 quads, which is the rows that now fit one more element
//! without clipping it away.
//!
//! Re-recorded for the strip's pitch. The Properties icons are meant to line up
//! with the headings they open, row for row, and two of the three things that
//! needs were already true: `rail_lead` puts the first icon's top on the first
//! heading's top, and a rail entry is drawn at `card_head_height`, a panel
//! heading's own height. The third was not — the strip stepped by
//! `menu_top_gap` (2) and the region beside it by `gap_xs` (4), so the columns
//! started level and drifted two pixels a row. Invisible at the first icon,
//! most of a row out by the ninth.
//!
//! 480 golden lines no longer hold and 480 are new — the same count, and the
//! account is that they are the same elements:
//!
//! - the multiset of (scenario, x, width, height) is **identical** on both
//!   sides, so nothing changed size, nothing changed column, and nothing
//!   appeared or vanished;
//! - **not one colour changed**, by the same check as the two re-records above;
//! - only `y` origins differ, and every difference is a multiple of the gap's
//!   own change. Fourteen x columns are touched, which is the rails across the
//!   spaces and modals at both window sizes.
//!
//! Re-recorded for one control height and one heading height
//! (`tests/a_row_is_one_height.rs` carries the reasoning). Two numbers moved
//! and everything in the window is measured from them, so this is the largest
//! re-record in the file's history: 21143 golden lines no longer hold and 20766
//! are new. The account is that it is two numbers:
//!
//! - **a control went from 26 to 32** — the module less its room, rather than a
//!   size of its own. In the golden's 2x capture that is every `…x52.00`
//!   becoming `…x64.00`: 3324 out, and 64 is the commonest height arriving.
//! - **a row and a heading went from 30 to 36** — the module itself, so a
//!   card's heading, a row of controls and the bar above them are one height.
//!   That is `…x60.00` becoming `…x72.00`: 2617 out.
//!
//! Everything else is those two cascading. A row that grew six pixels moves
//! every row under it; a panel whose headings each grew six is taller, so the
//! column it sits in reflows. The line count falls by 377 because taller rows
//! fit fewer of them in a fixed-height region before it clips.
//!
//! **Not one colour changed.** Every `l:` and every `hsla(...)` in the lines
//! that left appears in the lines that arrived, and the reverse — checked the
//! same way as the three re-records above. Nothing was re-shaded, re-themed or
//! re-inked; this is geometry only, and two constants' worth of it.

use phiber::v2::state::{CtxKey, ModalKey};
use gpui::{TestAppContext, px, size};
use phiber::v2::app::BPhiberApp;
use phiber::v2::space::BSpaceType;
use phiber::v2::state::{BState};

const GOLDEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/layout_fingerprint.txt");
const SIZES: &[(f32, f32)] = &[(1600.0, 900.0), (1100.0, 700.0)];
const MODALS: &[&str] = &["settings", "file_browser", "splash", "about", "render", "spaces", "collab", "profile", "command", "help", "progress", "reader"];

fn frame(cx: &mut gpui::VisualTestContext, label: &str, out: &mut String) {
    // Sorted, and without `image` lines: the viewport's traced frame arrives
    // from the render thread whenever it is ready, so whether it is in this
    // capture — and where the quads fall around it in paint order — varies
    // run to run. It is scene content, not chrome; everything else is exact.
    let mut painted: Vec<String> = cx
        .scene_fingerprint()
        .into_iter()
        .filter(|line| !line.starts_with("image "))
        .collect();
    painted.sort();
    for line in painted {
        out.push_str(&format!("{label} | {line}\n"));
    }
    for (selector, b) in cx.all_debug_bounds() {
        out.push_str(&format!(
            "{label} | {selector} | {:.2} {:.2} {:.2} {:.2}\n",
            f32::from(b.origin.x),
            f32::from(b.origin.y),
            f32::from(b.size.width),
            f32::from(b.size.height)
        ));
    }
}

#[gpui::test]
async fn the_whole_window_lands_where_the_golden_says(cx: &mut TestAppContext) {
    let (app, cx) = cx.add_window_view(|_, _| BPhiberApp::new());
    let mut out = String::new();
    for &(w, h) in SIZES {
        cx.simulate_resize(size(px(w), px(h)));
        for space in BSpaceType::ALL {
            app.update(cx, |app, cx| {
                app.state = BState::new();
                app.state.set(CtxKey::Space, space.slug());
                cx.notify();
            });
            cx.run_until_parked();
            frame(cx, &format!("{w}x{h} {}", space.slug()), &mut out);
        }
        for modal in MODALS {
            app.update(cx, |app, cx| {
                app.state = BState::new();
                app.state.set(CtxKey::Space, BSpaceType::Layout.slug());
                app.state.set(ModalKey::Open, *modal);
                if *modal == "progress" {
                    // The progress screen is only up while a file is loading;
                    // without a load in flight it dismisses itself mid-capture.
                    app.state.set(&phiber::v2::scene::load_progress_key(), "0.4");
                    app.state.set(&phiber::v2::scene::load_stage_key(), "Reading objects 431/2104");
                }
                cx.notify();
            });
            cx.run_until_parked();
            frame(cx, &format!("{w}x{h} modal:{modal}"), &mut out);
        }
    }
    if std::env::var("UPDATE_LAYOUT_GOLDEN").is_ok() {
        std::fs::write(GOLDEN, &out).expect("write golden");
        eprintln!("recorded {} bounds to {GOLDEN}", out.lines().count());
        return;
    }
    let golden = std::fs::read_to_string(GOLDEN).expect("no golden — record one with UPDATE_LAYOUT_GOLDEN=1");
    if golden == out {
        return;
    }
    let (g, o): (Vec<_>, Vec<_>) = (golden.lines().collect(), out.lines().collect());
    let gs: std::collections::BTreeSet<_> = g.iter().copied().collect();
    let os: std::collections::BTreeSet<_> = o.iter().copied().collect();
    let gone: Vec<_> = gs.difference(&os).take(15).collect();
    let new: Vec<_> = os.difference(&gs).take(15).collect();
    panic!(
        "layout moved: {} golden line(s) no longer hold, {} new\n--- expected (first 15)\n{}\n--- got (first 15)\n{}",
        gs.difference(&os).count(),
        os.difference(&gs).count(),
        gone.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n"),
        new.iter().map(|s| s.to_string()).collect::<Vec<_>>().join("\n")
    );
}
