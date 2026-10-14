# Jobs — `phiber/v2/part`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 4 | 5 |

## Todo

- [ ] File and Asset browser as reusable, resizable hosts with per-instance state (not one global `file.*`)
- [ ] End-aligned numeric header (Size) flush with its numbers
- [ ] Timeline channels from the file's F-Curves (after `phiphx` animation)
- [ ] Declarative audit: every part built from its `set.rs` table

## Done

- [x] View ▸ Toolbar (T) hides and shows the tool rail (`toolbar::BToolbarShown`, `tog_host_on`): Blender's `SpaceView3D.show_region_toolbar`, shown by default, each editor flipping its own — the third of the region-toggle family beside Sidebar (N) and Maximize Area (Ctrl Space). The floating pane draws nothing when its parts are empty, so a hidden toolbar leaves no empty bar on the canvas. Replaces two `op()` placeholders. (`tests/viewport_toolbar_toggle.rs`)
- [x] The Outliner's header search stops clipping on a side panel (`region::complete_header` + `outliner::filter::popover`): in a side zone (Left/Right) the switcher, View Layer mode dropdown and View menu fill the strip, so the inline field crushed to a bordered sliver at the right end. The field moves to the Filter popover there — which now carries it, as Blender's `OUTLINER_PT_filter` opens with `filter_text` — and the inline field returns when the Outliner is maximized into the Center zone. (`tests/outliner_header_search.rs`)
- [x] The Image editor's Render tab (`sidebar::image::RENDER`): a fourth sidebar tab beside Blender's Image, Tool and View, with the rows a render is customised by — Sampling (samples), Denoise, Film ▸ Transparent, Format (resolution, %, frame rate) and Output (path, file format, RGB/RGBA) — drawn from the SAME tables Properties ▸ Render / ▸ Output draw, so one key per setting. Blender's Image editor has no such tab; it keeps the render's settings beside the render — deliberate, recorded. (`tests/the_image_editor_sidebar_has_its_panels.rs`)
- [x] Activity bar wired (every entry opens its destination)
- [x] Caption parts, Load UI option, thumbnails, Info log, command palette rows

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
