# Button — How to Use

## Files
- `mod.rs` — `NSButton`, `ButtonState` state machine, types, constructors
- `actions.rs` — action dispatch helpers
- `drawing.rs` — `NSButtonViewDelegate` renders button + handles mouse events

## Quick Start
```rust
use rust_appkit::controls::button::NSButton;

// Auto-sized button (title determines width)
let mut save_btn = NSButton::new("Save");
save_btn.set_action(|| save_file());

// Chaining API
let btn = NSButton::new("Run")
    .with_action(|| run_project())
    .with_bezel_style(NSBezelStyle::Rounded)
    .with_accent_color(NSColor::system_blue());

// Add to parent
btn.add_to(&mut toolbar_view);

// Checkbox / Radio
let toggle = NSButton::new("Show Grid")
    .with_button_type(NSButtonType::Switch);
```

## Key APIs
- `NSButton::new(title)` — auto-sized push button
- `NSButton::new_with_frame(rect)` — manual sizing
- `btn.set_action(closure)` — click handler
- `btn.set_button_type(type)` — MomentaryPushIn, Toggle, Switch, Radio
- `btn.set_bezel_style(style)` — Rounded, Toolbar, Circular, etc.
- `btn.interaction_state()` — Idle, Hover, or Pressed
- `btn.current_background_color()` — resolved from theme + state

## State Machine
```
Idle → Hover (mouse enter)
Hover → Idle (mouse exit)
Hover → Pressed (mouse down)
Pressed → Hover (mouse up inside → fires action)
Pressed → Idle (mouse up outside → no action)
```

## Rules
- Corner radius comes from `theme::keys::RADIUS_BUTTON`
- Background color resolves from state: `CLR_BG_BUTTON`, `_HOVER`, `_PRESSED`
- Drawing is a separate delegate struct — keeps rendering isolated
- Use `.add_to(parent)` for ergonomic view tree construction
