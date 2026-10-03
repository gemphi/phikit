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
