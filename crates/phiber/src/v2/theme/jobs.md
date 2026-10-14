# Jobs — `phiber/v2/theme`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 1 | 3 |

## Todo

- [ ] Spatial look: one elevation scale (0 base · 1 zone · 2 popover · 3 modal) with soft depth shadows, translucent "glass" materials over the viewport, continuous rounded corners, and motion (modal scale-in, zone expand/collapse, hover lift) — all from theme tokens (tokens side)

## Done

- [x] Silver is the show's theme (`palette::shade::SILVER_SEED`, `theme::SILVER`): the costume's blue-grey `#484C74` as the ladder's seed — the dominant shade of every surface — and its dark blue (`#060750`, brought up to `#0F12BD` so it reads on the blue-grey) as the accent, in both modes; the light mode is that blue-grey lifted toward white. It was the Tesla silver lowered to a gunmetal with Tesla's UI blue — one more dark grey. (`palette::shade` tests; `theme_selection`, `viewport_lightness_regression` sweep it)
- [x] Top bar follows the theme (native title bar hidden)
- [x] The Properties strip is the column beside it (`semantic::set`): `rail_surface` is the host's body and the body is `bg_panel` like every editor's — the one-step-darker Properties body is gone, so no colour sits behind the strip that the panel beside it lacks — and the selected tab is painted as the accordion heading level with it (`tab_active_gradient(Buts)` = `heading_surface(Card)`, its own sweep kept rather than turned towards the pane), so the tab wears the colour of the panel it opens and sits in the same row as the first head. Blender marks a tab by `wcol_tab.inner_sel` against `TH_BACK`; this marks it with the panel head's own surface — deliberate, recorded. (`tests/the_properties_strip_is_a_strip.rs`, `the_strips_read_as_one_surface.rs`, `semantic_style_derivations.rs`, `central_reuse.rs`)

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
