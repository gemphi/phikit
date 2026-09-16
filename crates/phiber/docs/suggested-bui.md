# Suggested BUI — Unified View Structure

The target `src/bui/` layout that makes every level consistent, covers **all** Blender
source views (all 28 editors/hosts), and puts **one view trait** behind every view.

Principles carried over from the current code:
- **One-way declarative pipeline** — builders produce a `BView` tree; `BView::into_elm()`
  produces GPUI. No GPUI types above the `BView` line.
- **store / state / bui split** — specs are static data in `store/`, runtime in `state/`,
  views in `bui/`.
- **A view is a folder if it has parts** — leaves are single files; composite views are
  `mod.rs` + one file per part.

Companion docs: `bspec.md` (spec abstraction), `bui-flow.md` (event flow),
`render-path-layout.md` (the concrete Layout file map).

---

## 1. The view trait

One trait, implemented by every builder. Builders hold only their *parameters* (host,
region, …); the runtime state is passed in. This keeps the trait object-safe and free of
lifetimes.

```rust
// src/bui/view.rs
use super::state::BState;
use super::bview::BView;

/// Every view in the tree builds itself into a `BView` node.
pub trait BViewBuilder {
    /// Build the declarative node for this view from runtime state.
    fn build(&self, state: &BState) -> BView;
}
```

Naming rules:
- The **tree node** stays `BView` (enum) — unchanged.
- The **builder trait** is `BViewBuilder`.
- Each concrete builder is `<Level>View` (e.g. `SpaceView`, `HostView`, `RegionView`).
- Render is separate: `BView::into_elm(&BPalette, on_action) -> AnyElement` (unchanged).

> Alternative considered: `fn build(&self) -> BView` with `state: &'a BState` stored on the
> struct. Rejected — it forces a lifetime onto the trait and every impl. Passing `state`
> in keeps the trait clean.

---

## 2. Full folder structure

A view **is a folder** (`name/mod.rs`) whenever it has parts; otherwise a single file.
Leaf render elements live in `core/`; the universal Card lives in `card/`.

```
src/bui/
├─ mod.rs                 // impl Render for BView (debug); re-exports
├─ view.rs                // BViewBuilder trait + BViewCtx (current space/host/mode/region)
├─ bview.rs               // BView enum + fluent constructors + BAction + CardShade
├─ render.rs              // BView::into_elm dispatch (tree -> AnyElement)
├─ theme.rs               // BTheme, BPalette, BPalette::from_state
├─ modal.rs               // ModalView (overlay wrapper)  [implements BViewBuilder]
├─ icon.rs                // IconView, IconSize (leaf icon renderer)
├─ palette.rs             // palette helpers (gradients, fills)
│
├─ state/                 // runtime state (has parts)
│  ├─ mod.rs              // BState
│  ├─ key.rs              // BKey  (ctx::*, ui::*)
│  ├─ set.rs              // default_state()
│  └─ def.rs              // WireframeMesh
│
├─ space/                 // SpaceView  [implements BViewBuilder] — has parts
│  ├─ mod.rs              // SpaceView: activity bar + zone layout + modal overlay
│  ├─ zones.rs            // zone row/column composition
│  └─ togglers.rs         // Left/Right/Bottom zone toggle buttons
│
├─ zone/                  // ZoneView   [implements BViewBuilder] — leaf (no parts)
│  └─ mod.rs
│
├─ area/                  // AreaView   [implements BViewBuilder] — has parts
│  ├─ mod.rs              // AreaView: resolve active host, render sized
│  └─ switch.rs           // area-switch seam (reads/writes BKey::AreaHost)
│
├─ host/                  // HostView   [implements BViewBuilder] — has parts (one per editor)
│  ├─ mod.rs              // HostView: region split (header/center/footer + overlays)
│  ├─ view3d/             // View3dView  [BViewBuilder] — has parts
│  │  ├─ mod.rs           // regions: Header, Center, Footer
│  │  ├─ header.rs        // Menubar | Modebar | Switcher
│  │  ├─ viewport.rs      // center canvas (Spatial)
│  │  ├─ toolbar.rs       // left tools overlay
│  │  ├─ sidebar.rs       // right sidebar overlay
│  │  └─ footer.rs        // footer strip
│  ├─ topbar/             // TopbarView [BViewBuilder]
│  │  ├─ mod.rs
│  │  └─ header.rs        // Menubar | Spacebar | Statusbar
│  ├─ outliner/           // OutlinerView [BViewBuilder]
│  │  ├─ mod.rs
│  │  ├─ header.rs
│  │  └─ tree.rs          // scene collection tree
│  ├─ properties/         // PropertiesView (Buts) [BViewBuilder]
│  │  ├─ mod.rs
│  │  ├─ header.rs
│  │  └─ panel.rs         // vtab bar + card accordion
│  ├─ timeline/           // TimelineView (Time) [BViewBuilder]
│  │  ├─ mod.rs
│  │  ├─ header.rs
│  │  └─ channels.rs      // dope channels + scrubber
│  ├─ file/ image/ info/ sequencer/ text/ action/ nla/ node/ logic/ console/
│  │  uv/ compositor/ shader/ geo_node/ clip/ drivers/ asset/ spreadsheet/
│  │  graph_editor/ prefs/ statusbar/ empty/   // … one folder per remaining host
│  │     mod.rs            // <Name>View [BViewBuilder]
│  │     header.rs         // header region composition
│  │     body.rs           // center region content
│  │     (footer.rs)       // if the host has a footer region
│  └─ ...                 // full 28-editor coverage, table in §4
│
├─ region/                // RegionView [implements BViewBuilder] — has parts
│  ├─ mod.rs              // RegionView: render region's panes by alignment
│  └─ alignment.rs        // BRegionAlignment -> layout mapping
│
├─ pane/                  // PaneView   [implements BViewBuilder] — has parts
│  ├─ mod.rs              // PaneView: Left/Center/Right, sizes Toolbar/Sidebar
│  └─ container.rs        // PartContainer (shared part wrapper)
│
├─ part/                  // PartView   [implements BViewBuilder] — dispatcher + one folder per part
│  ├─ mod.rs              // PartView: BPartType -> concrete part view
│  ├─ activity/           // ActivityView  [BViewBuilder] — has parts
│  │  ├─ mod.rs
│  │  ├─ brand.rs
│  │  ├─ primary.rs
│  │  └─ drawer.rs
│  ├─ menubar/            // MenubarView   [BViewBuilder]
│  │  ├─ mod.rs
│  │  ├─ topbar.rs
│  │  └─ header.rs
│  ├─ modebar/            // ModebarView   [BViewBuilder]
│  │  ├─ mod.rs
│  │  └─ item.rs
│  ├─ switcher/           // SwitcherView  [BViewBuilder] — has parts
│  │  ├─ mod.rs
│  │  ├─ trigger.rs
│  │  ├─ hosts_col.rs
│  │  ├─ modes_col.rs
│  │  └─ popup.rs
│  ├─ spacebar/           // SpacebarView  [BViewBuilder]
│  │  ├─ mod.rs
│  │  ├─ tabs.rs
│  │  └─ actions.rs
│  ├─ statusbar/          // StatusbarView [BViewBuilder]
│  │  ├─ mod.rs
│  │  └─ stats.rs
│  ├─ toolbar/            // ToolbarView   [BViewBuilder]
│  │  ├─ mod.rs
│  │  └─ item.rs
│  ├─ sidebar/            // SidebarView   [BViewBuilder]
│  │  ├─ mod.rs
│  │  └─ tabs.rs
│  ├─ panel/              // PanelView     [BViewBuilder]
│  │  ├─ mod.rs
│  │  ├─ tabs.rs
│  │  └─ accordion.rs
│  ├─ channels/           // ChannelsView  [BViewBuilder]
│  │  ├─ mod.rs
│  │  └─ item.rs
│  └─ custom/             // CustomView    [BViewBuilder]
│     ├─ mod.rs
│     └─ canvas.rs
│
├─ card/                  // Universal Card (has parts)
│  ├─ mod.rs              // CardView
│  ├─ header.rs
│  ├─ body.rs
│  ├─ footer.rs
│  ├─ item.rs
│  └─ list.rs
│
└─ core/                  // leaf elements — the last UI bit (each BView variant)
   ├─ mod.rs              // re-exports
   ├─ row.rs              // RowView
   ├─ column.rs           // ColumnView
   ├─ panel.rs            // PanelView (BPanelRole container)
   ├─ dropdown.rs         // DropdownView
   ├─ tab.rs              // TabView
   ├─ htab_bar.rs         // HTabBarView, VTabBarView
   ├─ icon_list.rs        // IconButtonView, IconListView
   ├─ section.rs          // SectionView
   ├─ separator.rs        // SeparatorView (resize grip)
   ├─ scroll.rs           // ScrollView
   ├─ spatial.rs          // SpatialView (3D viewport)
   ├─ prop_row.rs         // PropRowView
   ├─ label.rs            // LabelView      ← the smallest UI bit
   └─ empty.rs            // EmptyView
```

---

## 3. Per-level contract

Each builder file follows one shape. Parameters are fields; `state` is passed in.

```rust
pub struct RegionView { pub host: BHostType, pub region: BRegionType }

impl BViewBuilder for RegionView {
    fn build(&self, state: &BState) -> BView {
        // read region spec from store, map panes to PaneView, compose into BView
    }
}
```

| Level | Builder | Params (fields) | Reads from state | Produces |
|---|---|---|---|---|
| Space | `SpaceView` | — | `CtxSpace`, `Zone(*)`, `Modal`, `ActivityTab` | App panel |
| Zone | `ZoneView` | `zone: BZoneType` | `Zone(zone)` visibility | zone slot |
| Area | `AreaView` | `area: BAreaSpec` | `AreaHost(default)`, dims | sized host |
| Host | `HostView` / `<Name>View` | `host: BHostType` | region visibility | region split |
| Region | `RegionView` | `host`, `region: BRegionType` | — | pane row/col |
| Pane | `PaneView` | `host`, `pane: BPaneType` | pane widths | part stack |
| Part | `PartView` / `<Part>View` | `host`, `part: BPartType` | part-specific keys | `BView` node |
| View | — | — | — | `BView` enum node |
| Core | `*View` (leaf) | variant data | palette only | `AnyElement` |

The trait is the same at every level; only the parameters and produced node differ.

---

## 4. Host coverage — all 28 Blender editors

One folder per host under `bui/host/`. Each `<Name>View` implements `BViewBuilder` and owns
its region composition. `header.rs` / `body.rs` / `footer.rs` files exist only where the
host actually has that region (per `store/hosts/<host>/set.rs::REGIONS`).

| Host (`BHostType`) | Folder | Regions | Notes |
|---|---|---|---|
| Empty | `host/empty/` | — | placeholder |
| View3D | `host/view3d/` | Header, Center, Footer | Toolbar/Sidebar overlays |
| Outliner | `host/outliner/` | Header, Center | tree |
| Buts (Properties) | `host/properties/` | Header, Center | vtab + accordion |
| File | `host/file/` | Header, Center | browser |
| Image | `host/image/` | Header, Center | toolbar/sidebar |
| Info | `host/info/` | Header, Center | report log |
| Seq | `host/sequencer/` | Header, Center | strips |
| Text | `host/text/` | Header, Center | editor |
| Action | `host/action/` | Header, Center | dope sheet |
| Nla | `host/nla/` | Header, Center | tracks |
| Time | `host/timeline/` | Header, Center | scrubber |
| Node | `host/node/` | Header, Center | node canvas |
| Logic | `host/logic/` | Header, Center | bricks |
| Console | `host/console/` | Header, Center | REPL |
| Uv | `host/uv/` | Header, Center | UV canvas |
| Compositor | `host/compositor/` | Header, Center | node canvas |
| Shader | `host/shader/` | Header, Center | node canvas |
| GeoNode | `host/geo_node/` | Header, Center | node canvas |
| Clip | `host/clip/` | Header, Center | tracking |
| Drivers | `host/drivers/` | Header, Center | driver editor |
| Asset | `host/asset/` | Header, Center | asset shelf |
| Spreadsheet | `host/spreadsheet/` | Header, Center | table |
| Ipo (Graph) | `host/graph_editor/` | Header, Center | curves |
| Prefs | `host/prefs/` | Header, Center | settings |
| Topbar | `host/topbar/` | Header | app bar |
| Statusbar | `host/statusbar/` | Header | status strip |

---

## 5. What changes vs. today

- **Consistency** — the flat `space.rs / area.rs / host.rs / region.rs / pane.rs` become
  folders only where they have parts (`space/`, `area/`, `host/`, `region/`, `pane/`), so the
  whole tree obeys "folder if it has parts."
- **One trait** — every builder implements `BViewBuilder::build(&self, &BState) -> BView`;
  today they are ad-hoc `.new(...).build()` structs.
- **Host dispatch moves down** — `HostView` keeps generic region splitting; each host folder
  holds its own `<Name>View` for host-specific content instead of a single `part/custom/`
  placeholder.
- **Store untouched** — `store/` specs (`panes_for`, `REGIONS`, `SWITCHABLE_HOSTS`) are the
  data the views read; this restructure only reorganizes the `bui/` view layer.

---

## 6. Migration order

1. Add `bui/view.rs` (`BViewBuilder` trait).
2. Convert level views (`space/ zone/ area/ host/ region/ pane/`) to the trait.
3. Convert `part/*` to the trait (dispatcher `PartView` + per-part folders).
4. Add `host/<name>/` folders, moving host-specific content out of `part/custom/`.
5. Wire `HostView` to dispatch to `<Name>View` per `BHostType`.
6. Keep `core/`, `card/`, `bview.rs`, `render.rs`, `theme.rs` as-is (already leaves).
