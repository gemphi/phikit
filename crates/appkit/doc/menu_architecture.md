# Menu System Architecture

> Reference: Apple `NSMenu.h` (AppKit), Reality Composer Pro project window layout

---

## The Three Menu Bar Types

Apple's AppKit provides three distinct menu interaction patterns.
Our `rust-appkit` must implement all three **independently** of any specific application.

### 1. Application Menu Bar (`NSMenuBar`)

The persistent horizontal bar at the top of the window.
Reality Composer Pro uses: `File | Edit | Insert | Component | Object | View | Theme | Window | Help`

```
┌─────────────────────────────────────────────────────────────────────┐
│  File   Edit   Insert   Component   Object   View   Theme   Help   │
├─────────────────────────────────────────────────────────────────────┤
│                          (editor content)                           │
```

**Apple API:** `NSApplication.mainMenu` → `NSMenu` with child `NSMenu` submenus.
**Our API:** `NSApplication.set_main_menu(bar: NSMenuBar)`.
**Ownership:** The `NSApplication` singleton owns the menu bar. The application (rust-kit-pro) *builds* it, but the framework *manages* it.

### 2. Context Menu (Right-Click)

A popup menu anchored at the mouse position, shown on right-click.
Reality Composer Pro shows these on entities (Duplicate, Delete, Group, etc.)

```
         ┌──────────────┐
         │ Duplicate    │
         │ Delete       │
         │ ─────────── │
         │ Group        │
         │ Ungroup      │
         └──────────────┘
```

**Apple API:** `NSMenu.popUpContextMenu(menu, withEvent:, forView:)`.
**Our API:** `NSMenu::popup(rect, items, on_dismiss)`.
**Ownership:** Created ephemerally. The caller builds the `NSMenu`, calls popup, and forgets about it. The framework handles positioning, rendering, interaction, and auto-dismissal.

### 3. Popup/Dropdown Button Menu

A menu attached to an `NSPopUpButton` or toolbar control.
Reality Composer Pro uses these for environment pickers, camera modes, etc.

```
  ┌─── Environment ▾ ───┐
  │ Studio              │
  │ Kitchen - Day       │
  │ Kitchen - Night     │
  │ Living Room - Day   │
  └─────────────────────┘
```

**Apple API:** `NSPopUpButton.menu`.
**Our API:** `NSPopUpButton::new(rect, items)` — wraps an `NSMenu`.
**Ownership:** The button owns the menu. Clicking the button shows/hides the dropdown.

---

## Clean Separation: Framework vs Application

### What belongs in `rust-appkit` (the framework)

| File | Responsibility |
|------|---------------|
| `menu/mod.rs` | `NSMenu`, `NSMenuItem` data structures |
| `menu/bar.rs` | `NSMenuBar` state (active index, hover) |
| `menu/view.rs` | `NSMenuBarView` — renders the bar, handles hover/click |
| `menu/dropdown.rs` | `NSMenuDropdownView` — renders a dropdown panel |
| `menu/overlay.rs` *(new)* | `NSMenuOverlay` — full-screen dismiss catcher |
| `menu/popup.rs` *(new)* | `NSMenu::popup()` — static method for context menus |

> **RULE:** The framework **never** knows about `Action`, `EditorApp`, themes, or any application concept.
> It only knows: titles, key equivalents, action closures, separators, submenus, enabled/disabled.

### What belongs in `rust-kit-pro` (the application)

| File | Responsibility |
|------|---------------|
| `Editor/Menu.rs` | `build_menu_bar(dispatch)` — creates the full menu tree |
| `Shared/ActionHandler.rs` | `Action` enum + `handle()` dispatch |

> **RULE:** The application creates `NSMenu`/`NSMenuItem` with closures that call
> `dispatch(Action::SetTheme("Cyberpunk"))` etc. The framework fires the closure.
> The application handles the action. **No back-references.**

---

## The Correct Event Flow

```
User clicks "Theme" on menu bar
  │
  ├─→ NSMenuBarView::mouse_down()
  │     └─→ sets NSMenuBar.active_menu_index = 5
  │     └─→ pushes pending_action: "add overlay + dropdown"
  │
  ├─→ Event loop processes pending_action
  │     └─→ Creates NSMenuOverlay (full-screen, z-top)
  │     └─→ Creates NSMenuDropdownView (child of overlay)
  │
User clicks "Dark Mode" in dropdown
  │
  ├─→ NSMenuDropdownView::mouse_down()
  │     └─→ NSMenuItem::perform_action()
  │           └─→ closure fires: dispatch(Action::SetMode("dark"))
  │     └─→ pushes pending_action: "dismiss overlay + dropdown"
  │           (NOT locking NSApplication — already in dispatch)
  │
  ├─→ Event loop processes pending_action
  │     └─→ Removes overlay + dropdown from view tree
  │     └─→ NSMenuBar.active_menu_index = None
  │
User clicks outside dropdown
  │
  ├─→ NSMenuOverlay::mouse_down()
  │     └─→ pushes pending_action: "dismiss overlay + dropdown"
  │     └─→ NSMenuBar.active_menu_index = None
```

---

## The Overlay Pattern (macOS Native Behavior)

When a dropdown is shown, macOS creates an invisible event-tracking window that:
1. Catches all clicks outside the dropdown → dismisses it
2. Passes hover events to the menu bar → enables scrub-switching
3. Is removed when the dropdown is dismissed

### Our implementation:

```
root_view
  ├── NSMenuBarView (persistent, z-index: low)
  ├── EditorContent (panels, etc.)
  └── NSMenuOverlay (temporary, z-index: top)
        └── NSMenuDropdownView (the actual dropdown)
```

- **`NSMenuOverlay`**: Full-window view with `hit_test() → true`.
  On `mouse_down`: dismiss everything. On `mouse_moved`: forward to menu bar for scrub.
- **`NSMenuDropdownView`**: The dropdown panel. Because it's a child of the overlay, it
  gets checked first (front-to-back). Clicks inside hit the dropdown. Clicks outside hit
  the overlay → dismiss.

---

## CRITICAL BUG: Deadlock in `on_dismiss`

The current `on_dismiss()` callback tries to lock `NSApplication::shared()` from inside
a dropdown `mouse_down` handler, which is itself called from inside `NSApplication`'s
event dispatch loop. **This deadlocks.**

```rust
// WRONG (deadlocks):
let on_dismiss = Arc::new(|| {
    if let Ok(mut app) = NSApplication::shared().lock() {
        // ← Already locked by the dispatch loop that called us!
        app.pending_actions.push(...);
    }
});
```

### Fix: Signal-based dismiss

Instead of locking the app, the dismiss should set a flag that the event loop checks:

```rust
// The dismiss_flag lives on the dropdown delegate
pub(crate) dismiss_requested: Arc<AtomicBool>,

// In mouse_down, after performing the action:
self.dismiss_requested.store(true, Ordering::Relaxed);

// In the event loop, after dispatch_event_to_view returns:
if dismiss_flag.load(Ordering::Relaxed) {
    // Safe to mutate the view tree here — we're not inside a lock
    root_view.subviews_mut().retain(|v| v.name != "NSMenuOverlay");
    menu_bar.active_menu_index = None;
}
```

---

## What Must Change

### 1. Extract overlay into `menu/overlay.rs`

Move `NSMenuDropdownOverlayDelegate` out of `view.rs` into its own file.
This keeps view.rs focused on the menu bar rendering.

### 2. Add `NSMenu::popup()` static method

```rust
impl NSMenu {
    /// Shows a context menu at the given position.
    /// Automatically creates an overlay, positions the dropdown,
    /// and dismisses when the user clicks outside or selects an item.
    pub fn popup(menu: NSMenu, location: NSPoint) {
        // Push to pending_actions from caller's context
    }
}
```

### 3. Remove hardcoded dimensions

All of these should come from `RCPLayout` / theme tokens:

| Current hardcode | Replace with |
|---|---|
| `240.0` dropdown width | `RCPLayout::DROPDOWN_WIDTH` |
| `7.5` char width | Computed from font metrics |
| `MENU_START_X = 20.0` | `RCPLayout::MENU_BAR_PADDING_X` |
| `MENU_TITLE_PADDING_X = 12.0` | `RCPLayout::MENU_TITLE_PADDING` |
| `4.0` corner radius | Theme token `RADIUS_CONTROL` |
| `22.0` item height | Theme token `SIZE_CONTROL_HEIGHT` |

### 4. Context menus should use the same overlay pattern

Currently `OutlineView.rs` and `EditorWindow.rs` create dropdowns manually:

```rust
// WRONG: no overlay, no dismiss, no state reset
root_view.add_subview(dropdown.view.clone());
```

They should call:

```rust
NSMenu::popup(menu, event.location_in_window());
```

---

## Apple Reference: `NSMenu.h` Key API Surface

From: `phi/apple-source/ruskit/Versions/C/Headers/NSMenu.h`

| Apple Pattern | Our Equivalent | Status |
|---|---|---|
| `NSMenu.initWithTitle:` | `NSMenu::new(title)` | ✅ |
| `NSMenu.addItem:` | `NSMenu::add_item(item)` | ✅ |
| `NSMenuItem.initWithTitle:action:keyEquivalent:` | `NSMenuItem::new(title, action)` | ✅ |
| `NSMenuItem.submenu` | `NSMenuItem::set_submenu(menu)` | ✅ |
| `NSMenuItem.separatorItem` | `NSMenuItem::separator()` | ✅ |
| `NSMenu.popUpContextMenu:withEvent:forView:` | ❌ Missing | **Needed** |
| `NSMenuDelegate.menuWillOpen:` | ❌ Missing | Nice-to-have |
| `NSMenuDelegate.menuDidClose:` | ❌ Missing | Nice-to-have |
| `NSMenu.cancelTracking` | ❌ Missing | **Needed** |
| `NSMenu.highlightedItem` | Partial (hover_index) | ✅ |
| `NSApplication.mainMenu` | `NSApplication.menu_bar` | ✅ |
| `NSMenu.performActionForItemAtIndex:` | `NSMenuItem::perform_action()` | ✅ |
| `NSMenu.delegate` | ❌ Missing | Nice-to-have |

---

## Apple Reference: Reality Composer Pro Window

From `configure-the-reality-composer-pro-project-window`:

> "The toolbar provides quick access to actions and settings. You can configure it
> by adding and removing tools. To edit, control-click and select Customize Toolbar."

Key takeaways:

1. The menu bar is **part of the window chrome**, not the editor content
2. Toolbars are **separate** from menus — they live below the menu bar
3. Context menus appear on **control-click** (right-click) anywhere
4. The menu system is **completely independent** of the editor panels

---

## Action Plan (Priority Order)

1. **Fix the deadlock** — replace `on_dismiss` Arc callback with `AtomicBool` dismiss flag
2. **Extract overlay** — move into `menu/overlay.rs`
3. **Add `NSMenu::popup()`** — single entry point for all context menus
4. **Replace hardcoded values** — use theme tokens / `RCPLayout` constants
5. **Clean up `view.rs`** — reduce to < 90 lines by extracting overlay + popup logic
6. **Update `OutlineView.rs` + `EditorWindow.rs`** — use `NSMenu::popup()` instead of manual dropdown creation
