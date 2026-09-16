# BUIKit ↔ Blender Control Coverage Matrix

## 1. Interactive Control Parity

| Blender Control (`uiBut` Type) | RNAWidget Parameter | BUIKit Type | Implementation Status | Native AppKit Backing |
|---|---|---|---|---|
| `LABEL` | - | `BLabel` | ✅ Native | `NSTextField` (label style) |
| `BUT` (Operator button) | - | `BButton` | ✅ Native | `NSButton` |
| `SEPR` / `SEPRLINE` | - | `BSeparator` | ✅ Native | `NSView` (1px hairline view) |
| `COLOR` (Color field) | `color` | `BColorField` | ✅ Native | `NSView` (rounded color fill) |
| `LISTBOX` | - | `BList` | ✅ Native | `NSScrollView` + `BColumn` |
| `LISTROW` | - | `BListRow` | ✅ Native | `BRow` |
| Spacer | - | `BSpacer` | ✅ Native | Flexible `NSView` |
| `NUM` / `NUMSLI` | `float` / `int` | `BSlider` | ✅ Native | `NSSlider` + field delegate |
| `TEX` (Text input) | `string` | `BTextField` | ✅ Native | Editable `NSTextField` |
| `TOG` / `ICONTOG` | `boolean` | `BToggle` | ✅ Native | `NSButton` (state fill toggle) |
| `MENU` / `PULLDOWN` | `enum` | `BEnum` | ✅ Native | `NSPopUpButton` |
| `SEARCH_MENU` | - | `BSearch` | ✅ Native | `NSSearchField` |
| Tree / Disclosure | - | `BTree` / `BTreeNode` | ✅ Native | Indented `BRow` forest + disclosure |
| Icon Glyph | - | Glyph label | ✅ Native | `IconIdx` glyph renderer |
| Region Header | - | `BHeader` | ✅ Native | `BRow` + `BLabel` |
| Collapsible Panel | - | `BPanel` | ✅ Native | `BColumn` + `BHeader` |

## 2. Layout Containers (`uiLayout`)

| Blender Layout Concept | BUIKit Type | Underlying AppKit Container |
|---|---|---|
| Row layout | `BRow` | `NSStackView` (Horizontal) |
| Column layout | `BColumn` | `NSStackView` (Vertical) |
| Scroll region | `BScroll` | `NSScrollView` |
| Composition contract | `BContent` | Native layout trait |

## 3. Overall Parity Summary

**Core Control Coverage: 24 / 24 (100%).**
All core Blender interface elements - structural containers, region headers, collapsible panels, input controls, enum menus, search fields, and Outliner trees - are 100% native in BUIKit.
