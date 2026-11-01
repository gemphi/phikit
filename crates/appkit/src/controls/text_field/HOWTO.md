# TextField — How to Use

## Files
- `mod.rs` — `NSTextField`, `NSTextAlignment`, internal delegate with scrub + keyboard support

## Quick Start
```rust
use rust_appkit::controls::text_field::NSTextField;

// Editable text field
let mut field = NSTextField::new(NSRect::sized(200.0, 24.0));
field.set_text("Hello");
field.set_editable(true);

// Action fires on text change or enter
field.set_action(|new_text| {
    println!("Text changed: {}", new_text);
});

// Label (non-editable)
let label = NSTextField::label("Position:");

// Keyboard editing
field.insert_text("world");
field.delete_backward();
field.move_left();
field.move_to_end();
```

## Key APIs
- `NSTextField::new(frame)` — editable text field
- `NSTextField::label(text)` — non-editable label shortcut
- `field.set_text(str)` / `field.text()` — get/set content
- `field.set_action(|text| {...})` — callback on change/commit
- `field.set_editable(bool)` — toggle editing
- `field.set_bordered(bool)` — toggle border
- `field.set_text_alignment(align)` — Left, Right, Center, etc.
- `field.set_font_size(f32)` — override font size
- `field.insert_text(str)` — insert at caret
- `field.delete_backward()` / `field.delete_forward()` — delete chars
- `field.become_first_responder()` — accept keyboard focus

## Rules
- Supports drag-to-scrub for numeric values (mouse drag changes value)
- Caret position tracked internally for keyboard input
- Action fires on both text mutation and Enter key
- Colors come from theme tokens (`CLR_BG_CONTROL`, `CLR_BORDER`, `CLR_ACCENT`)
- Font size defaults to `theme::keys::SIZE_FONT_UI`
