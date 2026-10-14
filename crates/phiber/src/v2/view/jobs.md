# Jobs — `phiber/v2/view`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 5 | 10 |

## Todo

- [ ] Spatial look: one elevation scale (0 base · 1 zone · 2 popover · 3 modal) with soft depth shadows, translucent "glass" materials over the viewport, continuous rounded corners, and motion (modal scale-in, zone expand/collapse, hover lift) — all from theme tokens
- [ ] One menu look: every popover body uses `MenuItem` rows
- [ ] One zone look: all zone / area / host frames from the same surface treatment
- [ ] GPU viewport: renderer 2 (appkit wgpu, Material Preview) and renderer 3 (WebGPU path tracer) in the View3D area
- [ ] Viewport interaction on the real camera: orbit yaw + pitch, Shift pan of the pivot, dolly, numpad views, frame selected

## Done

- [x] Node cards paint again (`canvas::node::set::card`): each node's width and height are a fraction of the canvas, but they sat on the inner card, whose percentages resolve against its auto-sized absolute wrapper and so collapsed to a point — the Compositor/Shader/Geometry graphs drew their sockets, labels and wires but no card bodies at all. The canvas fractions now ride the absolute wrapper (which resolves against the sized canvas, `surface::base`) and the inner card fills it, splitting its own height into header and body. (`canvas::node::tests`)
- [x] The activity bar's secondary column wears the floating bar's material (`card::panel_fill` → `BFill::Floating`): the same lighter, near-solid shade the viewport's toolbar and sidebar bars take, so every column standing beside a rail is one surface. (`tests/the_activity_strip_is_card_content.rs`)
- [x] A strip's icon buttons are tiles (`draw::button`): an icon-only button in a header, footer or tool header paints the control shade whole at rest — the floating strip's fill — where a docked control's half-weight wash vanished on a header, so a transport read as loose glyphs beside one lit square. Every square button shows its square by default; a labelled control and a list row's toggles keep the wash, as Blender's restriction columns are bare icons. (`tests/a_strip_icon_is_a_tile.rs`)
- [x] Floating strips are bars (`pane::BPanePart::Float`, `floating_bg`): the viewport's toolbar and sidebar — and the gizmos, which share the fill — take one step over the panel shade at `floating_alpha` 0.96 (was 0.36, bare buttons on the canvas), rounded as a panel, their buttons a pad in from the bar's edges; the rail's foot cluster drops its own margin on a floating strip so its button is the same square as the tools above it (`rail::bar`), and the gizmo's clearance grows by the pad. Blender's floating regions are `TH_BACK` at its alpha; a lighter bar is Phiber's — deliberate, recorded. (`viewport_chrome`, `viewport_overlays`, `rail_foot_stays_in_view`, `central_reuse`)
- [x] Reading text is sized for reading (`draw::prose`, `BType::Prose*`): a modal's paragraphs, items, terms and notes at a quarter over the body size, headings a third over the header, titles a half over it — every role sized, where a paragraph had no size and took the element default while its title took the 12 px header and a note the 10 px caption, so the page read smallest at the top. The scales ride the Typography preferences. (`draw::prose` tests)
- [x] The render's data passes read for the eye (`canvas::image::set::pixels`): Depth is spread from the nearest hit (black) to the farthest (white) — raw it clipped to white past one unit, so every pixel more than a metre away was white — and Depth, Normal and Diffuse Color show a mid grey, clear with Color & Alpha, where the ray met nothing, where the Normal buffer used to show the kernel's leftover sky as a gradient. Combined is the bytes as rendered. Blender shows a raw Z scaled by the camera's clip range; this scales by the hits — deliberate, recorded. (`canvas::image::set` tests)
- [x] Real orbit camera (`scene/camera.rs`), floor grid in 3D, view-relative studio light
- [x] Silhouettes match Blender renders (`tests/viewport_vs_blender.rs`)
- [x] Sortable headers: label left, arrow at the end, slot reserved
- [x] Window caption controls (`WindowControls`)

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
