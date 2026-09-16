# Icon Mapping - Blender `UI_icons.h` → BUIKit `IconIdx` → SVG

> **Reference**: `REFS/blender/blender/source/blender/editors/include/UI_icons.h`

## Blender Editor Icons (EDITORS section, line ~152)

| Blender `DEF_ICON` | `BSpaceType` | `IconIdx` | SVG file | Status |
|---|---|---|---|---|
| `VIEW3D` | `View3D` | `Mesh` | `cube.svg` | ✓ |
| `IPO` | `Ipo` | `DopeSheet` | `dopesheet.svg` | ✓ (shared) |
| `OOPS` (Outliner) | `Outliner` | `Outliner` | `outliner.svg` | ✓ |
| `BUTS` (Properties) | `Buts` | `Properties` | `sliders.svg` | ✓ |
| `FILESEL` | `File` | `File` | `file.svg` | ✓ |
| `IMAGE_COL` | `Image` | `Image` | `image.svg` | ✓ |
| `INFO` | `Info` | `Info` | `info.svg` | ✓ (new) |
| `SEQUENCE` | `Seq` | `Sequencer` | `sequencer.svg` | ✓ |
| `TEXT` | `Text` | `Text` | `text.svg` | ✓ |
| `ACTION` | `Action` | `DopeSheet` | `dopesheet.svg` | ✓ (shared) |
| `NLA` | `Nla` | `DopeSheet` | `dopesheet.svg` | ✓ (shared) |
| `TIME` | `Time` | `Timeline` | `timeline.svg` | ✓ (new) |
| `NODETREE` | `Node` | `Nodes` | `nodes.svg` | ✓ |
| `LOGIC` | `Logic` | `Mesh` | `cube.svg` | placeholder |
| `CONSOLE` | `Console` | `Console` | `console.svg` | ✓ |
| `PREFERENCES` | - | `Settings` | `settings.svg` | ✓ |

## Blender Object/Data Icons (OUTLINER section)

| Blender `DEF_ICON` | `IconIdx` | SVG file | Used in |
|---|---|---|---|
| `OUTLINER_OB_MESH` | `Mesh` | `cube.svg` | Outliner tree |
| `OUTLINER_OB_CAMERA` | `Camera` | `camera.svg` | Outliner tree |
| `OUTLINER_OB_LAMP` | `Light` | `light.svg` | Outliner tree |
| `OUTLINER_DATA` | `Collection` | `collection.svg` | Outliner tree |

## How Blender Handles Icons (Data-Driven)

Blender's icon system is **data-driven**:

1. **`UI_icons.h`** - `DEF_ICON(NAME)` macro defines icon IDs as an enum.
   Each icon has a numeric ID auto-assigned by the macro order.

2. **`icons.svg`** - A single SVG sprite sheet in `release/datafiles/icons/`.
   Each icon is a `<g id="icon_NAME">` element. Blender loads this at startup
   and rasterizes individual icons on demand.

3. **`BKE_icon_get_event`** / `UI_icon_draw` - Runtime lookup by `IconIdx`.
   The icon is rendered with the current theme color (tinted via
   `TH_TEXT` / `TH_TEXT_HI`).

4. **Space type registration** - Each `ED_spacetype_*()` sets `st->iconid`:
   ```c
   // In ED_spacetype_view3d():
   st->iconid = ICON_VIEW3D;  // from UI_icons.h enum
   ```

## BUIKit Equivalent

- `IconIdx` enum = Blender's `DEF_ICON` enum
- Individual `.svg` files = Blender's sprite sheet (simpler, one file per icon)
- `svg_map.rs` = Blender's icon lookup table
- `BSpaceType::icon()` = Blender's `st->iconid`
- `BSpaceHandler::icon()` = delegates to `BSpaceType::icon()` (overridable)
- Icons render with `currentColor` → theme-aware tinting (same as Blender's `TH_TEXT`)
