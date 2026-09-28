# Panels — `BCard` against Blender's `Panel`

Checked against the Blender source in `REFS/blender/blender/source/blender`.
Each row names the Blender code it was read from. "Done" rows are covered by
a test; "Gap" rows are what the other agent can pick up next, in order.

## Reference

| What | Where in Blender |
|---|---|
| Panel data | `makesdna/DNA_screen_types.h` — `struct Panel`, `ePanel_Flag` (`PNL_CLOSED`, `PNL_PIN`, `PNL_POPOVER`) |
| Panel type | `blenkernel/BKE_screen.hh` — `struct PanelType` (`draw_header`, `draw_header_preset`, `draw`, `poll`, `parent`, `order`, `category`), `PANEL_TYPE_*` flags |
| Layout panels (`layout.panel()`) | `BKE_screen.hh` — `LayoutPanelHeader`, `LayoutPanelBody`, `LayoutPanels`; `DNA_screen_types.h` — `LayoutPanelState` |
| Drawing | `editors/interface/interface_panel.cc` — `draw_aligned_panel` (L1347), `panel_draw_aligned_backdrop` (L1266), `panel_draw_aligned_widgets` (L1114), `panel_draw_border` (L1085), `draw_layout_panels_backdrop` (L1209) |
| Input | `interface_panel.cc` — `handle_panel_header` (L2277), `handle_layout_panel_header` (L2252), `panel_drag_collapse` (L2118), `panels_collapse_all` (L643), `do_drag` (L2042) |
| Sizes | `interface_intern.hh` — `PNL_HEADER = UI_UNIT_Y * 1.25`; `UI_interface_c.hh` — `UI_PANEL_MARGIN_X = 0.4 unit`, `UI_PANEL_MARGIN_Y = 0.1 unit` |
| Region resize | `editors/screen/screen_ops.cc` — `SCREEN_OT_region_scale` (L3648), `RegionMoveData`, `region_scale_validate_size`, hide below `UI_UNIT_Y / 4` |
| Region scroll | `editors/include/UI_view2d.hh` — `V2D_SCROLL_WIDTH = 0.45 unit + 2px`, `V2D_SCROLL_HANDLE_*`, `V2D_SCROLL_THUMB_SIZE_MIN = 30px`; `DNA_view2d_types.h` — `V2D_SCROLL_*` flags |

## Header

| Blender | Phiber | |
|---|---|---|
| Header strip `PNL_HEADER` tall | `BDim::HeaderHeight` | Done |
| Fold triangle at the left, `ICON_RIGHTARROW` / `ICON_DOWNARROW_HLT` | `card-fold-*`, `triangle_right` / `triangle_down` | Done |
| Title after the triangle, `paneltitle` font | `card-title-*`, `BType::Body` | Done |
| `draw_header` controls, before the title (`labelofs`) | `BCardHead.controls` ← `BSectionSpec.header` | Done — Motion Blur, Film ▸ Transparent, Sampling ▸ Render ▸ Denoise |
| `draw_header_preset` pushed right | `BCardHead.preset` | Done (no user yet) |
| Grip `ICON_GRIP` at the right, alpha 0.5, top-level panels only | `card-grip-*`, `effects.grip_alpha` | Done |
| Header backdrop `TH_PANEL_HEADER`, top-level only | `BFill::Tone(Header)` for `BCardKind::Panel` | Done |
| Closed: header rounded on all corners; open: top corners | `BRadius::Role(Card)` / `BRadius::Top` | Done |
| Title dims when a region search filter doesn't match (`panel_title_color_get`) | — | Gap |
| Triangle and grip fade as the header narrows below 4–5 units | — | Gap |
| `PNL_PIN` pin icon, Shift-click to pin (`show_pin`) | — | Gap |

## Body

| Blender | Phiber | |
|---|---|---|
| Body backdrop `TH_PANEL_BACK`; sub-panels `TH_PANEL_SUB_BACK` | `BSurfaceTone::Card` / `Body` in Properties; `Section` for `Sub` | Done |
| Header backdrop `TH_PANEL_HEADER` (Blender: top-level only) | `BFill::CardHead` → `heading_surface`: the section sweep, or its top flat — on `Panel` and `Sub` alike, as the approved mock-ups lift both | Done |
| Outline `TH_PANEL_OUTLINE`, top-level only | `BEdge::All` for `Panel` | Done |
| Active outline `TH_PANEL_ACTIVE` (`active_property`) | `BCard.active` is declared | Gap: not drawn — needs a `BEdge` colour token |
| `PANEL_TYPE_NO_HEADER` / `HIDE_HEADER` | `BSectionSpec.headless` / `BCard::headless()` — `RENDER_PT_context` (engine, device), the Preferences theme picker, dialog groups | Done |
| Sub-panels (`PanelType.parent`), title 0.7 unit in | `BSectionSpec.children` → `BCardKind::Sub` cards at the foot of the parent's body; keys `ctx.buts.<tab>.<parent_child>.expanded` | Done for Render (Sampling, Light Paths, Film, Performance) and Object ▸ Transform ▸ Delta Transform; Gap: Output, Material, World, View Layer, Object ▸ Visibility children (see below) |
| Layout panels drawn inside a body (`layout.panel()`, `draw_layout_panels_backdrop`) | a `Sub` card inside a body | Gap: needs a user |
| `PANEL_TYPE_DEFAULT_CLOSED` | `BSectionSpec.closed`; `BPanelItem::is_open` = the user's choice, else Blender's default | Done for every Properties panel Phiber carries (39 closed) — `tests/properties_panel_parity.rs` pins each to its Blender class. Preferences sections are Phiber's own and open |

## Input

| Blender | Phiber | |
|---|---|---|
| Click anywhere on the header (outside the grip) folds | the whole head is one `on_click` with `BFold::flip` | Done |
| Header controls take their own clicks | the controls run `occlude()`s | Done |
| Drag across headers: open/close every panel the pointer crosses to the first one's new state (`panel_drag_collapse`) | `view/draw/fold.rs`: a `BFoldSweep` drag flips the first header and `BFold::sweep_to` sets each header it crosses | Done — `tests/header_gestures.rs` |
| Ctrl-click: collapse every other panel (`panels_collapse_all`) | `BFold::solo`; the region's fold keys come from `with_peers` (Properties, Preferences, File Browser sidebars) | Done — `tests/header_gestures.rs` |
| Drag the grip to reorder (`PANEL_STATE_DRAG`, `sortorder`) | grip is drawn only | Gap |
| Enter / A on a hovered header folds it | — | Gap (keyboard) |
| Open/close animates (`ANIMATION_TIME 0.30`) | instant | Gap: `BFlex::div_unfolding` exists for the status log and could drive it |

## Region: resize and scroll (the card's standalone traits, next)

| Blender | Phiber | |
|---|---|---|
| Region edge drag (`SCREEN_OT_region_scale`) clamps to `maxsize - UI_UNIT_Y` | `zone/size.rs`, `area/size.rs` | Partly: seams resize; no max clamp against the area |
| Dragging a region below one unit hides it and keeps its size; a click on its edge brings it back (`region_scale_modal`: `collapse_size` = `UI_UNIT_X`, `region_scale_toggle_hidden`) | `view/size.rs` (`scale`, `ctx.sidebar.hidden`); the hidden sidebar leaves a one-row strip with the arrow | Done for sidebars — `tests/region_scale.rs`; area/zone seams still clamp without hiding |
| `RGN_FLAG_NO_USER_RESIZE` | `zone::is_zone_resizable` | Done for zones |
| View2D scrollbars: 0.45 unit wide, fade to `V2D_SCROLL_MIN_ALPHA` when idle, thumb ≥ 30px | `view/scroll.rs` (geometry) + `prim/scroller.rs` (painted from a persistent `ScrollHandle`) on panel bodies and sidebars | Done — thumb drawn, follows scrolling, brightens under the pointer. Gap: dragging the thumb, horizontal bars, the other scroll containers (column, popover, thumbnails) |
| Ctrl-click a header resets the view (`view2d_offset`) | — | Gap |

## Where cards are used

| Producer | Blender | Card |
|---|---|---|
| Properties sections (`part/props/item.rs`) | `properties_*.py` panels | folds, grip |
| File browser bookmarks (`part/custom/file/sidebar.rs`) | `FILEBROWSER_PT_bookmarks_*` | folds, grip |
| Preferences sections (`part/custom/pref/item.rs`) | `USERPREF_PT_*` (window region) | folds, grip |
| Preferences theme picker | `USERPREF_PT_theme` (`HIDE_HEADER`) | headless |
| Welcome, About, Help, Profile, Collab dialogs | popup blocks (no panels in Blender) | titled or headless, no fold |

## Render tab against Cycles' `ui.py` (sync121)

Panel tree now as Blender draws it with Cycles. Properties Blender draws that
Phiber does not carry yet — each needs an RNA key and a row:

| Panel | Missing properties |
|---|---|
| Sampling ▸ Viewport | `use_preview_adaptive_sampling`, `preview_adaptive_threshold`, `preview_adaptive_min_samples`; Viewport ▸ Denoise (`preview_denoiser`, …) |
| Sampling ▸ Render ▸ Denoise | `denoising_input_passes`, `denoising_prefilter`, `denoising_quality`, `denoising_use_gpu` |
| Sampling | Path Guiding, Advanced, Lights sub-panels |
| Light Paths | Clamping (`sample_clamp_direct`, `sample_clamp_indirect`), Fast GI Approximation |
| Volumes | `volume_biased` |
| Motion Blur | `rolling_shutter_type`, `rolling_shutter_duration`; Shutter Curve |
| Film | `film_exposure`; Transparent ▸ `film_transparent_glass`, `film_transparent_roughness` |
| Performance | Memory (`tile_size`), Texture Cache, Acceleration Structure, Viewport, Compositor |
| Color Management | Working Space, Advanced, Curves, White Balance sub-panels |

Other tabs are flat where Blender nests: Output ▸ Output (Views, Color
Management, Pixel Density, Encoding ▸ Video/Audio), Output ▸ Metadata (Note,
Burn Into Image), Object ▸ Transform ▸ Parent Inverse Transform, Object ▸
Instancing ▸ Scale by Face Size, Object ▸ Visibility (Ray Visibility,
Culling), Material ▸ Settings (Surface, Volume).

The full Blender panel list (670 Properties panels: label, context, parent,
options, class) was extracted with `ast` from the source and is kept at
`assets/logo/blender_panels.json` for the next pass.

## Next, in order

1. ~~Move every `BPanelRole::Card` + `CardHeader` user to `BCard`~~ — done
   (sync120). This also fixed the Preferences window region, which declared
   the Properties panel list and so showed an empty body.
2. ~~`BSectionSpec { closed, headless, header, children }`~~ — done
   (sync121): Blender's defaults, header toggles and sub-panels, Render
   and Object ▸ Transform nested. Remaining nesting and properties above.
3. ~~Drag-collapse and Ctrl-click collapse-all on headers~~ — done (sync122).
4. Standalone `size` — done for sidebars (sync123, `view/size.rs`); drawn
   `scroll` — done for panel bodies and sidebars (sync124, `view/scroll.rs`).
5. ~~Fold the remaining `prim/panel/*` roles into `BCard` and delete them~~ —
   done (sync125–128): strips, region columns and `BView::Split`.
