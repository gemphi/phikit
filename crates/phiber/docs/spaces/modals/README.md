# Modals - Overview

> **Hierarchy position:** Modals are space-display types - they overlay the workspace
> like spaces but have their own store/state/view triple. They use the same
> store/state/view pattern as spaces but render in a modal layer on top.

```
Modal (BModalType)
  ├─ Settings  - preferences editor (categories sidebar + content)
  ├─ Help      - help links + version info
  └─ Profile   - account info + session actions
```

> **Note:** Host switching is NOT a modal. The `Switcher` is a menu part
> (`BPartType::Switcher`) that lives in the header region's left pane,
> next to the Modebar. It shows a dropdown menu of available host types for
> the current area's zone.

---

## UI Hierarchy (Modal-specific)

Modals don't follow the `Space → Zone → BArea → Host` grid. Instead:

```
BModalType
  └─ Modal view function (store specs + state → BView tree)
       └─ BView nodes (Row, Column, Label, Button, ...)
```

The modal itself is a container with:
- **Header** - title + dismiss button
- **Body** - BView tree from the modal's view function
- **Footer** - optional action buttons

---

## Architecture (parallel to spaces)

```
Store (static specs)          State (runtime)              View (render)
─────────────────             ─────────────                ────────────
store/modals/                 state/modals/                buikit/src/modal.rs
  mod.rs                        mod.rs                       BModalType enum
  settings.rs                   settings.rs                  b_modal() container
  help.rs                       help.rs
  profile.rs                    profile.rs
```

### Store → State → View flow

```
BModalType (enum)              → identifies which modal
  ↓
Store specs (static)           → modal content data (categories, links, actions)
  ↓
State (ModalState)             → tracks open/closed + internal selection
  ↓
View function                  → reads store + state, produces Vec<BView>
  ↓
App view layer                 → wraps in b_modal(), renders with entity
```

---

## Zed Comparison

| Concept | Phiber | Zed |
|---------|--------|-----|
| Modal type enum | `BModalType` (Settings, Help, Profile) | `ModalView` trait (any type implementing it) |
| Modal content | Store specs + state → view function → BView | `ModalView` → `Render` impl |
| Modal layer | App manages `ModalState` | `ModalLayer` entity on workspace |
| Toggle/show/hide | `modal_state.set_toggle(BModalType)` | `workspace.toggle_modal()` → `ModalLayer` |
| Dismiss | App dismisses | `on_before_dismiss() → DismissDecision` |
| Settings UI | `BModalType::Settings` → categories + rows | `SettingsWindow` - separate window with navbar, pages, search |
| Settings store | `SettingIdx` enum + static rows | `SettingsStore` global, JSON files, `SettingsContent` |
| Focus management | App manages | `ModalLayer` tracks previous focus, restores on dismiss |
| Stash/reopen | Not implemented | `stashed_modal` + `reveal_stashed_modal()` for pickers |

### Key differences

1. **Zed's settings** open as a **separate window** (`SettingsWindow`), not a modal.
   Phiber treats Settings as a modal - simpler, but Zed's approach allows the settings
   to stay open while interacting with the workspace.

2. **Zed's `ModalLayer`** is a workspace-level entity that manages one active modal at a time.
   It handles focus restoration, dismiss decisions, and picker stashing.
   Phiber uses `ModalState` - simpler, just tracks open/closed + internal state.

3. **Zed's `ModalView` trait** requires `ManagedView` (a GPUI concept).
   Phiber uses store specs + state + view functions - pure data, app wraps it.

4. **Zed's settings store** is a full JSON-backed `SettingsStore` with file watchers,
   schema validation, and profile support. Phiber's settings store is static enum rows.

---

## Files

- [store.md](store.md) - Modal store specs (BModalType, all modal specs)
- [state.md](state.md) - Modal state (ModalState enum, per-modal state, ctx sync, set methods)
- [views.md](views.md) - Modal view rendering, Zed ModalLayer comparison
- [settings.md](settings.md) - Settings modal full doc (categories, rows, Zed settings_ui comparison)
- [help.md](help.md) - Help modal full doc
- [profile.md](profile.md) - Profile modal full doc

---

## Blender References

- `editors/preferences/index.rst` - Blender Preferences editor sections
- `bpy.types.SpacePreferences` - Preferences is a space type in Blender
- `bpy.ops.screen.userpref_show` - `Edit → Preferences...` (Ctrl-Comma)
- Blender treats Preferences as a **space type** (`SpacePreferences`) - same pattern as our modals
