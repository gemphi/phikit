# Render Path — Layout Space, Top to Bottom

This document traces **every file** used to render the `Layout` workspace, from process
entry down to the last GPUI element. It is the concrete reference we build the
trait-per-level unification against. Companion to `bspec.md` (the abstraction) and
`bui-flow.md` (the event flow); this doc is the *file map* of a single, real render.

The pipeline is a pure one-way declarative walk:

```
main() → app::run() → PhiberApp::render()
   → SpaceView.build() → … → PartView.build() → BView tree
   → BView::into_elm() → core element structs → gpui::AnyElement
```

The same three layers recur at every level:
- **store/** — static spec (plural modules), the *what*. Read during build.
- **state/** — runtime `BState`/`BContext` (singular modules), the *now*. Read during build, written by actions.
- **bui/** — view builder producing `BView`, then `into_elm` producing GPUI. The *how it looks*.

---

## 1. Entry (app/)

| File | Role |
|---|---|
| `@crates/phiber/src/main.rs` | `main()` → panic hook → `app::run()` |
| `@crates/phiber/src/app/mod.rs` | `run()` — opens the GPUI window, `PhiberApp::new()` |
| `@crates/phiber/src/app/interactive.rs` | `PhiberApp { state: BState }`, `Render::render` → `SpaceView { state }.build()` |
| `@crates/phiber/src/app/render.rs` | render bridge — `tree.into_elm(&palette, on_action)` → `AnyElement` |
| `@crates/phiber/src/app/handle.rs` | `handle_action` — `BAction` → `BState` mutation → `cx.notify()` |

Boot state comes from `@crates/phiber/src/bui/state/set.rs` (`default_state()`): space=`Layout`,
host=`View3D`, theme=`BlenderDark`.

---

## 2. Space level

| File | Role |
|---|---|
| `@crates/phiber/src/bui/space.rs` | `SpaceView` — layout coordinator; builds zones, the full-height Activity bar, the modal overlay |
| `@crates/phiber/src/store/spaces/mod.rs` | `BSpaceType` impl — `label/icon/all/with_set/zones/columns` |
| `@crates/phiber/src/store/spaces/def.rs` | `BSpaceType` enum + `LAYOUT_LABEL/ICON`, `BObjectMode`, `default_object_mode()` |
| `@crates/phiber/src/store/spaces/set.rs` | `ALL`, `ALL_SPECS`, `LABELS`, `ICONS` |
| `@crates/phiber/src/store/spaces/layout/set.rs` | **`LAYOUT_AREAS`, `ZONES`, `COLUMNS`** — the concrete Layout grid |

`SpaceView.build()` reads `BKey::CtxSpace`, calls `space_type.with_set()` to get `LAYOUT_AREAS`,
then walks zones: Top → [Left|Center|Right] → Bottom, plus `ActivityView` at the far left.

---

## 3. Zone level

| File | Role |
|---|---|
| `@crates/phiber/src/store/zones/mod.rs` | `BZoneType` impl — `label/icon/tooltip/is_toggleable/with_set` |
| `@crates/phiber/src/store/zones/def.rs` | `BZoneType { Full, Top, Left, Right, Center, Bottom }` |
| `@crates/phiber/src/store/zones/set.rs` | `ALL_ZONES`, `ALL_SPECS`, `TOGGLEABLE` |

Zones are grid slots (no children). `SpaceView::zone_visible` reads `BKey::Zone(zone)`;
`zone_togglers` emits toggle buttons for Left/Right/Bottom.

---

## 4. Area level

| File | Role |
|---|---|
| `@crates/phiber/src/bui/area.rs` | `AreaView` — resolves the area's active host, renders it sized |
| `@crates/phiber/src/store/areas/mod.rs` | `BAreaSpec { idx, zone, default_host, items }` |
| `@crates/phiber/src/store/areas/def.rs` | `BAreaType { Top, Left, Right, Center, Bottom }`, `panel_role()` |
| `@crates/phiber/src/store/areas/set.rs` | `ALL_AREAS`, `ALL_SPECS`, `AREA_LABELS` |

`AreaView.build()` reads `BKey::AreaHost(default_host)` to pick the **active** host, falling back
to `default_host`. This is the **area-switch** seam — the Layout Right area holds
`[Outliner, File, Asset]` and `[Buts, Spreadsheet, Asset]`.

---

## 5. Host level

| File | Role |
|---|---|
| `@crates/phiber/src/bui/host.rs` | `HostView` — renders regions; splits Toolbar/Sidebar into overlays |
| `@crates/phiber/src/store/hosts/mod.rs` | `BHostType` impl, `BHostSpec`, `BHostDef` trait, `host_spec` |
| `@crates/phiber/src/store/hosts/def.rs` | `BHostType` enum (28 editors), labels, icons, `dna_space_type()` |
| `@crates/phiber/src/store/hosts/set.rs` | `base_regions`, `build_spec`, `all_specs` (OnceLock-leaked), `SWITCHABLE_HOSTS` |

Layout touches these hosts — each has a `set.rs` with a static `REGIONS` slice:

| Host | set.rs | Regions |
|---|---|---|
| `Topbar` | `@crates/phiber/src/store/hosts/topbar/set.rs` | `[Header]` |
| `View3D` | `@crates/phiber/src/store/hosts/view3d/set.rs` | `[Header, Center, Footer]` |
| `Outliner` | `@crates/phiber/src/store/hosts/outliner/set.rs` | `[Header, Center]` |
| `Buts` | `@crates/phiber/src/store/hosts/properties/set.rs` | `[Header, Center]` |
| `Time` | `@crates/phiber/src/store/hosts/timeline/set.rs` | `[Header, Center]` |

`BHostState` (runtime region visibility/active-region trait) lives in
`@crates/phiber/src/state/host/mod.rs`.

---

## 6. Region level

| File | Role |
|---|---|
| `@crates/phiber/src/bui/region.rs` | `RegionView` — renders the panes of one region |
| `@crates/phiber/src/store/regions/mod.rs` | `BRegionType` impl, `BRegionSpec { idx, alignment, items }` |
| `@crates/phiber/src/store/regions/def.rs` | 17 `BRegionType` (Blender `eRegion_Type`), `BRegionAlignment` |
| `@crates/phiber/src/store/regions/set.rs` | `ALL`, `ALL_SPECS`, `LABELS` |

`RegionView.build()` looks the region up in the host spec, maps its panes to `PaneView`.
The region set is a direct port of Blender's `ARegion`/`eRegion_Type`/`eRegion_Alignment`
(`@REFS/blender/.../DNA_screen_types.h`).

---

## 7. Pane level

| File | Role |
|---|---|
| `@crates/phiber/src/bui/pane.rs` | `PaneView` + `PartContainer` — maps parts to part views, sizes Toolbar/Sidebar |
| `@crates/phiber/src/store/panes/mod.rs` | `BPaneType { Left, Center, Right }`, `BPaneSpec { idx, items }`, `panel_role()` |
| `@crates/phiber/src/store/panes/def.rs` | pane labels |
| `@crates/phiber/src/store/panes/set.rs` | **`panes_for(host, region)`** — the static pane-layout table |

`panes_for` is the Layout's per-host wiring. Relevant slices:
`TOPBAR_HEADER` (Menubar|Spacebar|Statusbar), `VIEW3D_HEADER` (Menubar|Modebar|Switcher),
`VIEW3D_CENTER` (Toolbar|Viewport|Sidebar), `OUTLINER_CENTER` (Viewport),
`BUTS_CENTER` (Panel), `TIME_CENTER` (Channels|Viewport), `SIMPLE_HEADER`.

---

## 8. Part level

| File | Role |
|---|---|
| `@crates/phiber/src/bui/part/mod.rs` | **`PartView`** — the dispatcher: `BPartType` → concrete `*View.build()` |
| `@crates/phiber/src/store/parts/mod.rs` | `BPartType` impl, `BPart`, `BPartSpec` trait |
| `@crates/phiber/src/store/parts/def.rs` | 18 `BPartType` + component-name consts |
| `@crates/phiber/src/store/parts/set.rs` | `ALL`, `ALL_SPECS`, `LABELS`, `COMPONENTS` |

Each part view is a module under `@crates/phiber/src/bui/part/` with the same
`.new(host, state).build() -> BView` shape. Parts reached by Layout:

| Part | View module | Reads |
|---|---|---|
| Activity | `bui/part/activity/mod.rs` (+`brand`/`primary`/`drawer`) | `ActivityTab/Drawer/Expanded`, `CtxSpace` |
| Menubar | `bui/part/menubar/mod.rs` (+`topbar`/`header`) | menus |
| Modebar | `bui/part/modebar/mod.rs` (+`item`) | `CtxMode`, `Dropdown{Modebar}` |
| Switcher | `bui/part/switcher/mod.rs` (+`trigger`/`hosts_col`/`modes_col`/`popup`) | `Dropdown{Switcher}`, `SwitcherPreviewHost` |
| Spacebar | `bui/part/spacebar/mod.rs` (+`tabs`/`actions`) | `CtxSpace` |
| Statusbar | `bui/part/statusbar/mod.rs` (+`stats`) | — |
| Toolbar | `bui/part/toolbar/mod.rs` (+`item`) | `Expanded{Toolbar}`, `CtxTool` |
| Sidebar | `bui/part/sidebar/mod.rs` (+`tabs`) | `Expanded{Sidebar}`, `Tab{Sidebar}` |
| Panel | `bui/part/panel/mod.rs` (+`tabs`/`accordion`) | `PanelExpanded`, `Tab{Panel}` |
| Channels | `bui/part/channels/mod.rs` (+`item`) | — |
| Custom | `bui/part/custom/mod.rs` (+`canvas`) | viewport placeholder |

Part content data comes from the matching `@crates/phiber/src/store/parts/<part>/set.rs`
(`with_idx(host)`).

---

## 9. View level — the BView tree

| File | Role |
|---|---|
| `@crates/phiber/src/bui/bview.rs` | **`BView` enum** (Card, List, Item, Row, Column, Panel, Dropdown, Tab, Section, Spatial, Switcher, …) + fluent constructors + `BAction` + `CardShade` |
| `@crates/phiber/src/bui/mod.rs` | `impl Render for BView` (debug render) |
| `@crates/phiber/src/bui/render.rs` | **`BView::into_elm()`** — dispatches each variant to a core element struct |

Every part's `build()` returns a `BView` node. `BView` is the single declarative
intermediate representation — no GPUI types leak above this line.

---

## 10. Core level — the last UI bit

| File | Role |
|---|---|
| `@crates/phiber/src/bui/core/mod.rs` | re-exports the element structs |
| `@crates/phiber/src/bui/core/panel.rs` | `PanelView` → themed container (`BPanelRole`) |
| `@crates/phiber/src/bui/core/row.rs` / `column.rs` | `RowView` / `ColumnView` |
| `@crates/phiber/src/bui/core/dropdown.rs` | `DropdownView` |
| `@crates/phiber/src/bui/core/tab.rs` | `TabView` |
| `@crates/phiber/src/bui/core/htab_bar.rs` | `HTabBarView`, `VTabBarView` |
| `@crates/phiber/src/bui/core/icon_list.rs` | `IconButtonView`, `IconListView` |
| `@crates/phiber/src/bui/core/section.rs` | `SectionView` |
| `@crates/phiber/src/bui/core/separator.rs` | `SeparatorView` (resize grip) |
| `@crates/phiber/src/bui/core/scroll.rs` | `ScrollView` |
| `@crates/phiber/src/bui/core/spatial.rs` | `SpatialView` (3D viewport) |
| `@crates/phiber/src/bui/core/label.rs` / `prop_row.rs` / `empty.rs` | leaf text / rows / nothing |
| `@crates/phiber/src/bui/card/mod.rs` (+`body`/`footer`/`header`/`item`/`list`) | `CardView`, `CardListView`, `CardItemView` |

Each core struct exposes `into_elm(&BPalette, on_action) -> gpui::AnyElement`. This is the
**last UI bit** — the boundary where `BView` becomes a live GPUI element with click handlers.

---

## 11. Cross-cutting support

| File | Role |
|---|---|
| `@crates/phiber/src/bui/theme.rs` | `BTheme { BlenderDark, AppleDark }`, **`BPalette`** (colors, radii, metrics), `BPalette::from_state` |
| `@crates/phiber/src/bui/state/mod.rs` | `BState` (flat `HashMap<String,String>` + scene) |
| `@crates/phiber/src/bui/state/key.rs` | `BKey` — hierarchy-bound string keys (`ctx::*`, `ui::*`) |
| `@crates/phiber/src/bui/state/set.rs` | `default_state()` |
| `@crates/phiber/src/bui/state/def.rs` | `WireframeMesh` |
| `@crates/phiber/src/state/context/` | `BContext`, `BMode` (typed context, mirrors `bpy.context`) |
| `@crates/phiber/src/state/host/` | `HostState`, `BHostState` trait |
| `@crates/phiber/src/state/modal/` | `ModalState`, `BModalState` trait, `modal_body()` |
| `@crates/phiber/src/store/modals/` | modal specs |
| `@crates/phiber/src/store/menus/` | menu data for the menubar |
| `@crates/phiber/src/store/panel_role.rs` | `BPanelRole` → per-role styling |

---

## 12. Layout space — the concrete tree it produces

With `default_state()` (space=Layout, host=View3D), `SpaceView.build()` yields:

```
App (panel)
├─ Activity bar                      ← ActivityView           [activity/mod.rs]
└─ CenterArea (panel)
   ├─ Top zone  → Topbar host        ← AreaView→HostView      [topbar/set.rs]
   │   Header   → TOPBAR_HEADER      ← RegionView→PaneView    [panes/set.rs]
   │             Menubar | Spacebar | Statusbar               [part/{menubar,spacebar,statusbar}]
   ├─ Work (panel)
   │   ├─ Center → View3D host        ← AreaView→HostView     [view3d/set.rs]
   │   │   Header → VIEW3D_HEADER     Menubar|Modebar|Switcher [part/{menubar,modebar,switcher}]
   │   │   Center → VIEW3D_CENTER     Toolbar|Viewport|Sidebar [part/{toolbar,custom,sidebar}]
   │   │   Footer → (empty)
   │   └─ Right col → Outliner + Buts hosts                    [outliner,set + properties,set]
   │       Outliner: Header(SIMPLE)+Center(Viewport)
   │       Buts:     Header(BUTS)+Center(Panel)                [part/panel]
   └─ Bottom → Time host             ← AreaView→HostView       [timeline/set.rs]
       Header(SIMPLE) + Center(Channels|Viewport)               [part/channels]
```

## 13. Where we take it from here

The file map above is complete for Layout. The unification gap this doc feeds:

1. Every `*View` in §2/§4/§5/§6/§7/§8 is an ad-hoc struct with `.build()`. Introduce one
   `BViewBuilder` trait (`fn build(&self, state: &BState) -> BView`) and implement it at each level.
2. Extend the store's `BSpec` marker into a behavior trait (`label/icon/children/default_mode`) so
   the store answers questions the view layer currently re-derives.
3. Add mode→part/region visibility (a `poll(mode) -> bool` on `BRegionSpec`) so `BObjectMode`
   drives the Modebar/Toolbar/ToolProps — currently `default_object_mode()` is set but never consumed.
4. Wire the `Switcher` part's apply action to `BKey::AreaHost` so area switching is a write, not just a read.
