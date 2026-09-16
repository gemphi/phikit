# Blender Framework Architecture & BUIApp Mapping

## 1. Blender Window Layout System

Blender tiles every application window into a strict hierarchy defined in `source/blender/makesdna/DNA_screen_types.h`:

```
bScreen (Window layout root)
  └─ ScrArea (Editor tile area)
       └─ ARegion (Region panel strip: Header, Tools, Main, Properties, Footer)
            └─ Panel (Collapsible control card)
                 └─ uiBut (Leaf interactive control / button)
```

## 2. BUIKit Structural Parity (Unified `BView` Hierarchy)

BUIKit mirrors Blender's structural layout hierarchy directly on top of `AppKit` using a purely data-driven, retained `BView` enum, avoiding intermediate structural objects:

| Blender Concept (C struct) | BUIKit `BView` Variant | Underlying AppKit Base |
|---|---|---|
| `bScreen` (Window layout) | `BView::Screen` | Grid container hosting multiple tiled spaces |
| `ScrArea` (Editor view) | `BView::BArea` | Layout block hosting space regions |
| `ARegion` (`RGN_TYPE_*`) | `BView::Region` | Sub-region view wrapper (Header, Main, UI) |
| `Panel` (Collapsible card) | `BView::Panel` | native vertical toggle button section wrapper |
| `UI_UNIT_X/Y` (Scalable DPI unit) | `BUnit` / `unit()` | `appkit::theme` registry metrics |
| Theme Color Palette | `BTheme` / `b_theme()` | `appkit::theme` token system |

## 3. Region Roles (`BRegionType`)

BView supports standard region roles defined by `BRegionType`:
- **Header**: Top or bottom horizontal menu strip containing workspace pickers, menus, status indicators.
- **Window (Main)**: Central context canvas representing the main area of the space (e.g., 3D view canvas, spreadsheet grid).
- **Tools**: Left-side vertical toolbar panel for modal active tools.
- **UI**: Right-side collapsible properties sidebar (also known as the N-panel).
- **Footer**: Bottom status and operator log region.

## 4. Scalable UI Unit System (`UI_UNIT`)

Blender never hardcodes pixel dimensions. Every button height, padding, gap, font size, and layout offset derives from a single scalable unit (`UI_UNIT`).

In BUIKit:
- `u.unit(1.0)` -> 20.0px default (scales dynamically with display DPI or user preferences).
- Button height: `u.btn_h` -> `unit(1.0)`
- Header height: `u.header_h` -> `unit(1.2)`
- Gap sm / md / lg: `u.gap_sm` (4px), `u.gap_md` (8px), `u.gap_lg` (12px)
