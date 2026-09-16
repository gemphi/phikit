# Spaces - Documentation Index

> **Blender references:**
> - Blender Manual: [`interface/window_system/workspaces.rst`](../../blender-manual/manual/interface/window_system/workspaces.rst)
> - `DNA_screen_types.h` - `bScreen`, workspace tabs

---

## BSpaceType Enum

```rust
pub enum BSpaceType {
    Layout,
    Modeling,
    Sculpting,
    UvEditing,
    TexturePaint,
    Shading,
    Animation,
    Rendering,
    Compositing,
    GeometryNodes,
    Scripting,
}
```

Each space defines a `SpaceSpec` with:
- `space_type` - the enum variant
- `label` - display name
- `areas` - `&[BArea]` (zone, default_host, BHosts, fraction)
- `columns` - `&[f32]` grid fractions

### Hierarchy

```
Space → Zone → BArea → Host → Region → Pane → Part → BView
```

- **Zone** (`BZoneType`) - grid position (Top, Left, Center, Right, Bottom, Full)
- **BArea** (`BArea`) - slot at a zone position, holds switchable hosts
- **Host** (`BHostType`) - editor type (View3D, Outliner, Properties, ...)

---

## Space List

| Space | Columns | Primary Hosts | Docs |
|-------|---------|---------------|------|
| **Layout** | 18% / 64% / 18% | Topbar, Outliner, View3D, Properties, Timeline | [layout/](layout/README.md) |
| **Modeling** | 100% | Topbar, View3D, Timeline | [modeling/](modeling/README.md) |
| **Sculpting** | 78% / 22% | Topbar, View3D, Properties | [sculpting/](sculpting/README.md) |
| **UV Editing** | 50% / 50% | Topbar, ImageEditor, View3D | [uv_editing/](uv_editing/README.md) |
| **Texture Paint** | 50% / 50% | Topbar, ImageEditor, View3D | [texture_paint/](texture_paint/README.md) |
| **Shading** | 18% / 47% / 35% | Topbar, Outliner, View3D, ShaderEditor | [shading/](shading/README.md) |
| **Animation** | 18% / 82% | Topbar, Outliner, View3D, DopeSheet | [animation/](animation/README.md) |
| **Rendering** | 65% / 35% | Topbar, View3D, Properties | [rendering/](rendering/README.md) |
| **Compositing** | 100% | Topbar, Compositor | [compositing/](compositing/README.md) |
| **Geometry Nodes** | 55% / 45% | Topbar, View3D, GeoNode | [geometry_nodes/](geometry_nodes/README.md) |
| **Scripting** | 18% / 47% / 35% | Topbar, Outliner, TextEditor, Info | [scripting/](scripting/README.md) |

---

## Shared Architecture

All spaces share the same hierarchy:

```
Space → Zone → BArea → Host → Region → Pane → Part → BView
```

- **Zone** (`BZoneType`) - grid position
- **BArea** (`BArea`) - slot at a zone, holds switchable hosts

- **Store** specs define what CAN exist (static, per-space)
- **State** tracks what IS happening (runtime, per-instance)
- **Context** tracks what's FOCUSED (hierarchical path)
- **Views** render by walking the hierarchy and matching enums

### Reusable Parts

Parts are shared across all spaces. A `Toolbar` part in Sculpting uses the same
`part/toolbar.rs` view as a `Toolbar` part in Layout - only the data (store specs)
differs.

| Part | Used In |
|------|---------|
| `Menubar` | Every host's Header region |
| `Modebar` | View3D, ImageEditor (mode-switching hosts) |
| `Switcher` | Every host's Header region (editor type dropdown) |
| `Toolbar` | View3D, ImageEditor, UvEditor |
| `Sidebar` | View3D, ShaderEditor, Compositor, GeoNode |
| `Panel` | Properties, View3D, ShaderEditor, Compositor |
| `Channels` | DopeSheet, GraphEditor, Nla |
| `Statusbar` | Topbar (footer region) |
| `Custom` | Every host's Center pane (viewport, node graph, text editor, ...) |

### Layout Space - Detailed Docs

The Layout space has the most comprehensive documentation. All other spaces
follow the same patterns.

| Doc | Description |
|-----|-------------|
| [layout/README.md](layout/README.md) | Layout space overview and index |
| [layout/overview.md](layout/overview.md) | New hierarchy, enum definitions, concrete example tree |
| [layout/store.md](layout/store.md) | Static store specs |
| [layout/state.md](layout/state.md) | Per-instance state (mod.rs/def.rs/ctx.rs/use.rs per level) |
| [layout/ctx.md](layout/ctx.md) | Hierarchical BContext |
| [layout/views.md](layout/views.md) | View layer: render walk, part reuse, BHostEffect trait |
| [layout/flow.md](layout/flow.md) | Boot, render, data, and state mutation flow diagrams |
| [layout/specs.md](layout/specs.md) | Spec-driven pattern matching strategy |
| [layout/tasks.md](layout/tasks.md) | Implementation phases and task checklist |

### Modals — Space-like Overlays

Modals are a separate level (`BModalType`) because they can be triggered from any space, but they are still *space-like*: a `BModalSpec` implements `BSpec` and `BSpaceDef`, so the same store/state/view pipeline applies. They render in a modal layer on top of the current workspace.

The Help modal is special: it renders the internal `docs/` markdown (e.g. `bspec.md`, `spaces/README.md`) with breadcrumbs, scrolling, and sections, so the app is self-documenting and works offline.

| Doc | Description |
|-----|-------------|
| [modals/README.md](../modals/README.md) | Modals overview, architecture, Zed comparison |
| [modals/store.md](../modals/store.md) | Modal store specs (BModalType, all modal specs) |
| [modals/state.md](../modals/state.md) | Modal state (ModalState enum, per-modal state, ctx sync) |
| [modals/views.md](../modals/views.md) | Modal view rendering, Zed ModalLayer comparison |
| [modals/settings.md](../modals/settings.md) | Settings modal (categories, rows, Zed settings_ui comparison) |
| [modals/help.md](../modals/help.md) | Help modal (docs browser, breadcrumbs, offline) |
| [modals/profile.md](../modals/profile.md) | Profile modal (account info, session actions) |
