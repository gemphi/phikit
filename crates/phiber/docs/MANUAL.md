
# Phiber Manual

Phiber's own manual, mirroring [Blender's manual](./blender-manual/manual/index.rst)
section-for-section, keyed to the v2 store/state/view code tree at
`crates/phiber/src/v2/`.

Every table maps a Blender concept to the exact file path in the codebase.
When a section says "sourced from X", `X` is the Blender source of truth
that authored the static data.

---

## 1. Hierarchy Overview

Phiber's UI is a strict 7-tier hierarchy plus a peer overlay tier:

```
Space → Zone → Area → Host → Region → Pane → Part
                                                 (widget composition)
Modal (overlay peer of Space)
```

Every tier lives in one folder under `v2/` with the same six files:

| File               | Purpose                                                        |
|--------------------|----------------------------------------------------------------|
| `type.rs`          | The enum: `B<Level>Type`                                       |
| `spec.rs`          | Static per-item descriptor: `B<Level>Spec`                     |
| `set.rs`           | `pub static SET: &[B<Level>Spec]` — all static data lives here |
| `item.rs`          | `B<Level>Item: BViewItem` — emits one `BView::<level>` variant |
| `list.rs`          | `B<Level>List: BViewList` — iterates children                  |
| `mod.rs`           | Re-exports                                                     |

Blender parallel: `source/blender/makesdna/DNA_screen_types.h`
(`bScreen`, `ScrArea`, `ARegion`, `SpaceLink`).

---

## 2. Space — Workspace Tabs

The topmost tier. Each `BSpaceType` is one Blender workspace.

| Space           | v2 file                                | Blender ref                                    |
|-----------------|----------------------------------------|------------------------------------------------|
| Layout          | `v2/space/layout.rs`                   | `bl_app_templates_system/General/startup.blend` |
| Modeling        | `v2/space/modeling.rs`                 | ditto                                          |
| Sculpting       | `v2/space/sculpting.rs`                | ditto                                          |
| UV Editing      | `v2/space/uv_editing.rs`               | ditto                                          |
| Texture Paint   | `v2/space/texture_paint.rs`            | ditto                                          |
| Shading         | `v2/space/shading.rs`                  | ditto                                          |
| Animation       | `v2/space/animation.rs`                | ditto                                          |
| Rendering       | `v2/space/rendering.rs`                | ditto                                          |
| Compositing     | `v2/space/compositing.rs`              | ditto                                          |
| Geometry Nodes  | `v2/space/geometry_nodes.rs`           | ditto                                          |
| Scripting       | `v2/space/scripting.rs`                | ditto                                          |

Blender manual: [`interface/window_system/workspaces.rst`](./blender-manual/manual/interface/window_system/workspaces.rst).

Root coordinator: `v2/space/list.rs` — reads `ctx.space`, emits the active
space and any active modal overlay.

---

## 3. Zone — Grid Slot

The zone tier divides a space into six regions of the screen.

| Zone   | Toggleable | v2 file           |
|--------|-----------:|-------------------|
| Top    |         no | `v2/zone/set.rs`  |
| Left   |    yes (T) | `v2/zone/set.rs`  |
| Center |         no | ditto             |
| Right  |    yes (N) | ditto             |
| Bottom |        yes | ditto             |
| Full   |         no | ditto             |

Blender parallel: `ScrAreaMap` + `AZone`. Toggle keys `T` (Left) and `N`
(Right) are documented in
[`interface/window_system/areas.rst`](./blender-manual/manual/interface/window_system/areas.rst).

---

## 4. Area — Host Slot

An area is a slot inside a zone that shows one host at a time.

| Level path        | Type            | State key                    |
|-------------------|-----------------|------------------------------|
| `v2/area/type.rs` | `BAreaType`     | `Top / Left / Center / Right / Bottom` |
| `v2/area/spec.rs` | `BAreaSpec`     | `{ kind, zone, default_host, hosts }` |
| `v2/area/item.rs` | `BAreaItem`     | reads `ctx.area.<host_slug>` to override the default |

Blender parallel: `ScrArea` in `DNA_screen_types.h`.

---

## 5. Host — Editor Kind

26 host types, one per Blender editor. Each host declares its region list
inline in its `host/<name>/mod.rs`. Regions are sourced from the
`RGN_TYPE_*` declarations in each `space_<name>.cc`.

| Host        | Regions                                   | Blender source                 |
|-------------|-------------------------------------------|--------------------------------|
| Topbar      | Header                                    | `space_topbar.cc:307-`         |
| View3D      | ToolHeader, Header, Center, AssetShelf+H  | `space_view3d.cc:1620-`        |
| Outliner    | Header, Center                            | `space_outliner.cc:678-`       |
| Buts        | Header, Center (NavBar + Panel)           | `space_buttons.cc:1105-`       |
| Time        | Header, Center (Channels + Custom)        | `space_action.cc` (Timeline)   |
| Statusbar   | Header                                    | `space_statusbar.cc:150-`      |
| File        | Header, Tools, Center, Execute            | `space_file.cc:1043-`          |
| Image       | ToolHeader, Header, Center, AssetShelf+H  | `space_image.cc:1343-`         |
| Info        | Header, Center                            | `space_info.cc:271-`           |
| Seq         | ToolHeader, Header, Channels, Scrubbing, Center | `space_sequencer.cc:1210-` |
| Text        | Header, Center, Footer                    | `space_text.cc:470-`           |
| Action      | Header, Center, Footer                    | `space_action.cc:987-`         |
| Nla         | Header, Center, Footer                    | `space_nla.cc:680-`            |
| Node        | Header, Center, AssetShelf+H              | `space_node.cc:1745-`          |
| Logic       | Header, Center                            | (legacy; 5.2 removed)          |
| Console     | Header, Center                            | `space_console.cc:370-`        |
| Uv          | ToolHeader, Header, Center                | shares `space_image`           |
| Compositor  | Header, Center                            | shares `space_node`            |
| Shader      | Header, Center                            | shares `space_node`            |
| GeoNode     | Header, Center                            | shares `space_node`            |
| Clip        | Header, Center                            | `space_clip.cc:1256-`          |
| Drivers     | Header, Center, Footer                    | shares `space_graph`           |
| Asset       | Header, Tools, Center                     | shares `space_file`            |
| Spreadsheet | Header, Center, Footer                    | `space_spreadsheet.cc:795-`    |
| Ipo         | Header, Center, Footer                    | `space_graph.cc:1009-`         |
| Prefs       | Header, Center, Execute                   | `space_userpref.cc:419-`       |

Blender manual: [`editors/`](./blender-manual/manual/editors/) has one page per host.

Content: `v2/host/<name>/mod.rs` for the region declaration;
widget content in `v2/part/custom/<name>/`.

---

## 6. Region — Region Kind

DNA-authored region kinds: `Header / Center / Footer / ToolHeader / Tools /
UI / ToolProps / AssetShelf / AssetShelfHeader / Channels / HUD / NavBar /
Execute / Scrubbing / Preview / Temporary / XR`.

Files: `v2/region/type.rs`, `v2/region/spec.rs`, `v2/region/item.rs`,
`v2/region/list.rs`.

Blender parallel: `enum eRegion_Type` in `DNA_screen_types.h`.

---

## 7. Pane — Left / Center / Right Slot

Panes divide a region into aligned slots. Each pane owns a `&[BPartType]`.

Files: `v2/pane/type.rs`, `v2/pane/spec.rs`, `v2/pane/item.rs`, `v2/pane/list.rs`.

Pane layouts per host+region are declared inside `v2/host/<name>/mod.rs`
(the `HEADER_PANES`, `CENTER_PANES`, `FOOTER_PANES` statics).

---

## 8. Part — Leaf Widget Kind

18 part variants, each with its own folder:

| Part          | Folder                    | What it renders                   |
|---------------|---------------------------|-----------------------------------|
| Menubar       | `v2/part/menubar/`        | Row of top menus with cascades    |
| Modebar       | `v2/part/modebar/`        | Row of mode / transport tabs      |
| Switcher      | `v2/part/switcher/`       | Editor-type dropdown              |
| Spacebar      | `v2/part/spacebar/`       | Workspace tabs                    |
| Toolbar       | `v2/part/toolbar/`        | Column of tool icons              |
| Sidebar       | `v2/part/sidebar/`        | N-panel vertical tabs             |
| Panel         | `v2/part/panel/`          | Properties accordion              |
| Channels      | `v2/part/channels/`       | Animation channels                |
| Statusbar     | `v2/part/statusbar/`      | Bottom status segments            |
| NavBar        | `v2/part/navbar/`         | Properties context tabs           |
| HUD           | `v2/part/hud/`            | Floating overlay                  |
| Execute       | `v2/part/execute/`        | Footer action buttons             |
| AssetShelf    | `v2/part/assetshelf/`     | Brush / preset tiles              |
| ToolProps     | `v2/part/toolprops/`      | Tool-header settings              |
| Activity      | `v2/part/activity/`       | Phiber-native left nav strip      |
| Custom        | `v2/part/custom/`         | Per-host center canvas dispatcher |

Every part follows the same four-file shape: `mod.rs / set.rs / item.rs /
list.rs`. `mod.rs` declares a `B<Part>` struct that implements `BViewItem`;
it composes primitives (`Row` / `Column` / `Panel`) around its `<Part>List`.

Blender parallel: `RGN_TYPE_*` slot content, e.g. `VIEW3D_HT_header`,
`OUTLINER_HT_header`, `BUTTONS_HT_header`, `TOPBAR_MT_editor_menus`.

---

## 9. Menubar — Cascaded Menus

The menubar supports arbitrary-depth recursive sub-menus:

```rust
pub enum BMenuKind {
    Op,
    Toggle,
    Sub(&'static [BMenuItem]),   // recursion via nested items
    Sep,
}
```

Ground truth: `v2/part/menubar/set.rs`; Blender source
`bl_ui/space_topbar.py` (`TOPBAR_MT_editor_menus`, `TOPBAR_MT_file`,
`TOPBAR_MT_file_import`, `TOPBAR_MT_file_export`, etc.).

Topbar menu layout matches Blender exactly: **Blender / File / Edit /
Render / Window / Help**, with recursive sub-menus for **Recover / Import /
Export / External Data / Clean Up / Defaults / About Blender**.

Blender manual: [`interface/window_system/topbar.rst`](./blender-manual/manual/interface/window_system/topbar.rst).

---

## 10. Properties Editor — Contexts and RNA

The Properties editor (`BHostType::Buts`) hosts 15 context tabs. Each context
lives in `v2/part/panel/<name>/set.rs`.

| Context     | Folder                              | Sections | Blender source                       |
|-------------|-------------------------------------|---------:|--------------------------------------|
| Active Tool | `v2/part/panel/tool/`               |        3 | `bl_ui/properties_tool.py`           |
| Render      | `v2/part/panel/render/`             |        8 | `bl_ui/properties_render.py` + Cycles |
| Output      | `v2/part/panel/output/`             |        5 | `bl_ui/properties_output.py`         |
| View Layer  | `v2/part/panel/view_layer/`         |        4 | `bl_ui/properties_view_layer.py`     |
| Scene       | `v2/part/panel/scene/`              |        5 | `bl_ui/properties_scene.py`          |
| World       | `v2/part/panel/world/`              |        4 | `bl_ui/properties_world.py`          |
| Collection  | `v2/part/panel/collection/`         |        3 | `bl_ui/properties_collection.py`     |
| Object      | `v2/part/panel/object/`             |        9 | `bl_ui/properties_object.py`         |
| Modifiers   | `v2/part/panel/modifier/`           |        2 | `bl_ui/properties_data_modifier.py`  |
| Particles   | `v2/part/panel/particles/`          |        5 | `bl_ui/properties_particle.py`       |
| Physics     | `v2/part/panel/physics/`            |        4 | `bl_ui/properties_physics_*.py`      |
| Constraints | `v2/part/panel/constraint/`         |        2 | `bl_ui/properties_object_constraint.py` |
| Data        | `v2/part/panel/data/`               |        7 | `bl_ui/properties_data_mesh.py`      |
| Material    | `v2/part/panel/material/`           |        5 | `bl_ui/properties_material.py`       |
| Texture     | `v2/part/panel/texture/`            |        6 | `bl_ui/properties_texture.py`        |

Every property is a typed `BPropSpec { key, label, kind }` — `key` is the
RNA identifier from `source/blender/makesrna/intern/rna_*.cc`; `kind` is
one of `Bool / Int / Float / Vec3 / Vec4 / Enum / Str / Color / Pointer`.

The property system schema lives at `v2/part/panel/prop.rs`.

---

## 11. Modal — Overlay Peer of Space

Modals open on top of the active space by setting `ctx.modal`.

| Modal        | Folder                              | Opens via | Blender parallel                          |
|--------------|-------------------------------------|-----------|-------------------------------------------|
| Settings     | `v2/modal/settings/`                | `ctx.modal = "settings"` | `space_userpref` in a temp window |
| Help         | `v2/modal/help/`                    | `ctx.modal = "help"`     | Help menu items                    |
| Profile      | `v2/modal/profile/`                 | `ctx.modal = "profile"`  | Phiber-native                      |
| Command      | `v2/modal/command/`                 | `ctx.modal = "command"`  | F3 Menu Search / Operator Search   |
| Splash       | `v2/modal/`                         | `ctx.modal = "splash"`   | Splash screen at startup           |
| About        | `v2/modal/`                         | `ctx.modal = "about"`    | Blender menu > About Blender       |
| FileBrowser  | `v2/modal/file_browser/`            | `ctx.modal = "file_browser"` | File > Open / Save As / Import / Export |

FileBrowser reuses the sidebar / column / filter / navigation sets from
`v2/part/custom/file/set.rs` — same widget, two entry points (host area or
modal overlay).

Overlay wiring: `v2/space/list.rs` reads `ctx.modal` and emits the modal
as a sibling `BView` alongside the active space.

Blender manual: [`interface/window_system/tabs_panels.rst`](./blender-manual/manual/interface/window_system/tabs_panels.rst)
covers modal-style overlays.

---

## 12. Theme — Central Tokens

Every metric and color lives in `v2/theme/`.

| File                              | What it holds                                             |
|-----------------------------------|-----------------------------------------------------------|
| `v2/theme/set.rs`                 | `METRICS: BMetric` — gaps, paddings, header/footer heights, widths, radii, font sizes |
| `v2/theme/palette/set.rs`         | `BLENDER_DARK`, `TOKYO_NIGHT` — 20 HSLA colors each       |
| `v2/theme/palette/mod.rs`         | `BPaletteName` enum + `palette_for(name)` lookup          |

No inline color or padding numbers exist anywhere else. Access:
`theme::METRICS.h_header`, `theme::palette_for(BPaletteName::BlenderDark)`.

Blender parallel: `theme` blocks in `bl_ui/themes/*.xml` and `ThemeSpace`
subtypes in `DNA_userdef_types.h`.

---

## 13. State — Path-Based, Host-Scoped

Every runtime value is a key/value pair in `BState.inner: HashMap<String, String>`.

Every key encodes as `ctx.<seg>.<seg>...`. There is no `ui.*`, `host.*`,
or `area_host.*` prefix — the segments themselves indicate the hierarchy.

| Key                                  | Meaning                                    |
|--------------------------------------|--------------------------------------------|
| `ctx.space`                          | Active space (e.g. `Layout`)               |
| `ctx.frame`                          | Current frame                              |
| `ctx.theme`                          | Active theme name                          |
| `ctx.modal`                          | Active modal slug (empty = closed)         |
| `ctx.zone.right`                     | Right zone visibility (`true`/`false`)     |
| `ctx.area.outliner`                  | Which host the (Right/Outliner) slot shows |
| `ctx.view3d.tool`                    | View3D active tool                         |
| `ctx.view3d.toolbar.expanded`        | View3D T-panel expansion state             |
| `ctx.view3d.sidebar.tab`             | View3D N-panel active tab                  |
| `ctx.buts.navbar.tab`                | Properties active context tab              |

Files: `v2/state/mod.rs` (BState + BHostState), `v2/state/path.rs` (BPath),
`v2/state/seed.rs` (boot defaults).

Blender parallel: `bContext` in `source/blender/windowmanager/wm_context.h`.

---

## 14. View — BView Tree + Traits

The declarative view tree has one variant per store-level plus primitives.

```rust
pub enum BView {
    Space  { kind, children },   Zone   { kind, children },
    Area   { kind, child    },   Host   { kind, children },
    Region { kind, children },   Pane   { kind, children },
    Part   { kind, body     },   Modal  { kind, body     },
    Row, Column, Panel, Label, Icon, Button, Empty,   // primitives, only inside Part.body
}

pub trait BViewItem { fn build(&self, s: &BState) -> BView; }
pub trait BViewList { fn items(&self, s: &BState) -> Vec<BView>; }
```

Every level and every part is a `BViewItem` implementer. No standalone
`pub fn build(...)` free functions.

File: `v2/view/mod.rs`.

Renderer entry point: `v2/view/render/` (per-level files) with a debug
`BAsciiRenderer` in `v2/view/render/ascii.rs`.

---

## 15. Icons

**Status: string-only, no assets yet.** Every set carries icon *names*
(`"cursor"`, `"select"`, `"folder"`, `"home"` …), but there is no SVG /
atlas layer that resolves those names to bitmap glyphs. The renderer will
need an icon atlas mapping keyed on those names.

Blender's atlas: `datatoc_icons.png` (from `source/blender/editors/datafiles/`).
Phiber will need an equivalent under `crates/phiber/assets/icons/`.

---

## 16. What's needed to run on Windows

Honest gap analysis:

- [ ] **Renderer**: `v2/view/render/{space,zone,area,host,region,pane,part}.rs`
      files that map `BView::<kind>` to GPUI elements. Empty today.
- [ ] **Icon atlas**: a resolver for the icon strings in the sets — SVG
      files or a packed atlas keyed by the names in `sidebar/set.rs`,
      `toolbar/set.rs`, `menubar/set.rs`, etc.
- [ ] **Real widget bodies**: `part/custom/<host>/item.rs` currently emits
      `BView::label("3D Viewport")` placeholders. Real content needs the
      scene bridge and drawing code.
- [ ] **Wire `pub mod v2;` into `crates/phiber/src/lib.rs`** and run
      `cargo check` to catch any linkage gaps.
- [ ] **Bootstrap**: `crates/phiber/src/main.rs` opens a GPUI window and
      renders `BSpaceList::new().items(&state)` on the root view.

Once those five are done, `cargo run -p phiber` on Windows will open a
Layout workspace showing all wired parts.

---

## 17. Cross-references

- [Composition Hierarchy — the ≤60 line rule](./composition-hierarchy.md)
- [Layout Space Mock](./spaces/layout/mock.md)
- Blender manual root: [`docs/blender-manual/manual/index.rst`](./blender-manual/manual/index.rst)
- Blender source root: `phiace/REFS/blender/blender/source/blender/`
