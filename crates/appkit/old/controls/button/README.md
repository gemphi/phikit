# NSButton — User Action Controls

## Apple Header: `NSButton.h`

## Overview

`NSButton` provides clickable controls for user interaction. Mirrors Apple's [`NSButton`](https://developer.apple.com/documentation/appkit/nsbutton).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    NSButton Creation                         │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│         Factory Method: buttonWithTitle()                    │
│  ├─ Creates NSControl (base)                               │
│  ├─ Sets NSButtonState (type, title, bezel)                │
│  ├─ Assigns NSButtonViewDelegate (drawing + events)        │
│  └─ Returns fully configured button                         │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    User Interaction                        │
│  ├─ mouseDown → highlight(true)                             │
│  ├─ mouseUp → highlight(false) + trigger action            │
│  └─ Action closure invoked with captured context           │
└─────────────────────────────────────────────────────────────┘
```

## Button Types (NSButtonType)

| Type | Behavior | Apple Parity |
|------|----------|--------------|
| `MomentaryPushIn` | Press and release, no state | Default push button |
| `MomentaryChange` | Press and release, text swaps | Link buttons |
| `Toggle` | Click toggles on/off | State persists |
| `Switch` | Checkbox behavior | ☑ / ☐ |
| `Radio` | Radio button behavior | ● / ○ |

## Bezel Styles (NSBezelStyle)

| Style | Visual | Use Case |
|-------|--------|----------|
| `Rounded` | Pill shape | Primary actions |
| `Inline` | Flat, minimal | Toolbars |
| `Circular` | Perfect circle | FABs, icons |
| `TexturedRounded` | Metal look | Dark toolbars |
| `HelpButton` | Small "?" | Help buttons |
| `Recessed` | Pressed-in look | Scope bars |

## Factory Methods (Apple Parity)

```rust
// Apple's +buttonWithTitle:target:action:
let save = NSButton::button_with_title("Save", || save_file());

// Apple's +checkboxWithTitle:target:action:
let toggle = NSButton::checkbox_with_title("Show Grid", || toggle_grid());

// Apple's +radioButtonWithTitle:target:action:
let option = NSButton::radio_button_with_title("Option A", || select_a());

// Extended factory methods
let primary = NSButton::primary_button_with_title("Export", || export());
let destructive = NSButton::destructive_button_with_title("Delete", || delete());
let toolbar = NSButton::toolbar_button_with_title("+", || add_item());
```

## Connection to Other Modules

```
NSButton
├── Inherits: NSControl
│   └── Inherits: NSView
├── Uses: NSImage (for icons)
├── Uses: NSColor (for accents)
├── Used by: NSToolbar
├── Used by: NSAlert
└── Used by: NSStackView (for button groups)
```

## State Properties

```rust
let mut btn = NSButton::button_with_title("Click Me", || action());

// High-fidelity Apple properties
btn.set_alternate_title("Clicked!");
btn.set_key_equivalent("\r"); // Enter key
btn.set_has_destructive_action(true); // Red styling
btn.set_bezel_color(Some(NSColor::system_accent()));
bn.set_content_tint_color(Some(NSColor::white()));
btn.set_tint_prominence(NSTintProminence::Primary);
btn.set_image_hugs_title(true);
btn.set_allows_mixed_state(true); // For tri-state checkboxes
```

## Visual Hierarchy

```
NSButton Visual Stack (from back to front)
├── Bezel (border + background)
│   ├── borderColor (if set)
│   └── bezelColor (tint)
├── Content
│   ├── Image (left or right of title)
│   └── Title text
└── Highlight overlay (when pressed)
```

## Usage Examples

### Basic Button
```rust
use rust_appkit::prelude::*;

let mut toolbar = NSStackView::horizontal();

let play = NSButton::button_with_title("Play", || player.play());
let pause = NSButton::button_with_title("Pause", || player.pause());
let stop = NSButton::button_with_title("Stop", || player.stop());

toolbar.add_arranged_subview(play.view_clone());
toolbar.add_arranged_subview(pause.view_clone());
toolbar.add_arranged_subview(stop.view_clone());
```

### Icon Button
```rust
let icon = svg_icons::get(IconId::Settings);
let settings = NSButton::button_with_image(icon, || open_settings());
settings.set_bezel_style(NSBezelStyle::Inline);
```

### Checkbox
```rust
let show_bounds = NSButton::checkbox_with_title("Show Bounding Boxes", || {
    viewport.toggle_bounds_visibility();
});
```

### Radio Group
```rust
let group = NSStackView::vertical();

let option1 = NSButton::radio_button_with_title("Wireframe", || set_mode(Mode::Wireframe));
let option2 = NSButton::radio_button_with_title("Shaded", || set_mode(Mode::Shaded));
let option3 = NSButton::radio_button_with_title("Textured", || set_mode(Mode::Textured));

group.add_arranged_subview(option1.view_clone());
group.add_arranged_subview(option2.view_clone());
group.add_arranged_subview(option3.view_clone());
```

### Destructive Action
```rust
let delete = NSButton::destructive_button_with_title("Delete Project", || {
    if confirm_delete() {
        project.delete();
    }
});
```

## Event Flow

```
User clicks button
        │
        ▼
NSButtonViewDelegate.mouse_down()
        │
        ▼
State.is_pressed = true
View redraws with highlight
        │
        ▼
User releases mouse
        │
        ▼
NSButtonViewDelegate.mouse_up()
        │
        ▼
State.is_pressed = false
Action closure invoked
View redraws normal state
```

## Testing & Verification

```rust
// Test button creation
let btn = NSButton::button_with_title("Test", || println!("clicked"));
assert_eq!(btn.state(), 0); // Off by default

// Test checkbox toggle simulation
let mut check = NSButton::checkbox_with_title("Check", || {});
check.set_state(1);
assert!(check.state() > 0);

// Test destructive action flag
let mut del = NSButton::destructive_button_with_title("Del", || {});
assert!(del.has_destructive_action());
```

## File Structure

```
src/button/
├── mod.rs          # NSButton struct, state, properties
├── drawing.rs      # NSButtonViewDelegate (rendering)
├── actions.rs      # Action closure handling
└── templates.rs    # Factory methods (Apple parity)
```

## Missing from Apple's NSButton.h

- **NSCell**: Full cell-based architecture
- **Sound**: `setSound:` for click sounds
- **Periodic delay**: Auto-repeat on hold
- **Spring loading**: Deep press actions
- **Accelerator buttons**: Multi-level pressure
- **Attributed titles**: Rich text support
- **Help buttons**: Contextual help integration

These are stubbed for current use cases.
