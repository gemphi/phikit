# NSTextField — Text Input and Labels

## Apple Header: `NSTextField.h`

## Overview

`NSTextField` provides text input and display. Supports editable fields, labels, secure text, and number scrubbing. Mirrors Apple's [`NSTextField`](https://developer.apple.com/documentation/appkit/nstextfield).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 NSTextField Creation                         │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Text Field Modes                                │
│  ├─ Label: Non-editable, display only                       │
│  ├─ Input: Editable with keyboard                           │
│  ├─ Secure: Password masking (•••)                         │
│  └─ Scrubbable: Drag to change numbers                     │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 User Interaction                             │
│  ├─ Click to focus (become first responder)                  │
│  ├─ Keyboard input captured                                  │
│  ├─ Drag on numbers → scrub mode (delta tracking)           │
│  └─ Action callback on value change                         │
└─────────────────────────────────────────────────────────────┘
```

## Text Alignment (NSTextAlignment)

| Alignment | Description |
|-----------|-------------|
| `Left` | Default, text starts at left |
| `Right` | Numbers, right-aligned |
| `Center` | Titles, centered |
| `Justified` | Full justification |
| `Natural` | Follows locale direction |

## Field Types

| Type | Behavior | Use Case |
|------|----------|----------|
| **Label** | `editable=false, bordered=false` | Static text display |
| **Input** | `editable=true, bordered=true` | Form fields |
| **Value** | `alignment=Right, editable=true` | Number entry |
| **Scrubbable** | Drag changes value | Inspector panels |

## Connection to Other Modules

```
NSTextField
├── Inherits: NSControl
│   └── Inherits: NSView
├── Used by: NSAlert (message text)
├── Used by: Inspector panels (value editing)
├── Used by: Forms (labeled inputs)
└── Uses: NSFont (text rendering)
```

## Usage Examples

### Label (Actual API)
```rust
use rust_appkit::prelude::*;

// Factory method for labels
let label = NSTextField::label("Entity Name:");
// Uses: text_field.set_text(text), editable=false by default
```

### Input Field (Actual API)
```rust
let mut input = NSTextField::new(NSRect::new(0.0, 0.0, 200.0, 22.0));
input.set_editable(true);    // Actual method from mod.rs
input.set_bordered(true);     // Actual method from mod.rs
input.set_placeholder("Enter name...");  // Stub in mod.rs
input.set_action(|text| {    // Actual method
    println!("User entered: {}", text);
});
```

### Text Editing (Actual API)
```rust
// Direct text manipulation (from mod.rs)
input.set_text("new text");           // Set text
let text = input.text();              // Get text
input.insert_text("insert");          // Insert at caret
input.delete_backward();              // Backspace
input.delete_forward();               // Delete key
input.move_left();                    // Caret left
input.move_right();                   // Caret right
input.move_to_beginning();            // Ctrl+Home
input.move_to_end();                  // Ctrl+End
input.set_caret_position(5);          // Set caret
let pos = input.caret_position();     // Get caret pos
input.insert_newline();               // Enter/Return

// First responder
input.become_first_responder();
input.resign_first_responder();
let is_fr = input.is_first_responder();
let can_fr = input.accepts_first_responder(); // checks editable
```

### Number Field (Scrubbable)
```rust
let mut x_field = NSTextField::new(NSRect::new(0.0, 0.0, 60.0, 22.0));
x_field.set_text("0.0");
x_field.set_text_alignment(NSTextAlignment::Right);
// Drag left/right to scrub value
```

### Labeled Row Pattern
```rust
fn labeled_row(label: &str, field: NSTextField) -> NSStackView {
    let mut row = NSStackView::horizontal();
    row.set_spacing(8.0);
    
    let mut label_view = NSTextField::label(label);
    label_view.view_mut().set_frame(NSRect::new(0.0, 0.0, 80.0, 22.0));
    
    row.add_arranged_subview(label_view.view_clone());
    row.add_arranged_subview(field.view_clone());
    row
}

// Usage
let name_field = NSTextField::new(NSRect::new(0.0, 0.0, 200.0, 22.0));
name_field.set_editable(true);
let row = labeled_row("Name:", name_field);
```

## Inspector Panel Pattern (rust-kit-pro)

```rust
// Transform property editing
fn create_transform_row(label: &str, initial: f32) -> NSStackView {
    let mut row = NSStackView::horizontal();
    row.set_spacing(8.0);
    
    let label = NSTextField::label(label);
    label.view_mut().set_frame(NSRect::new(0.0, 0.0, 20.0, 22.0));
    
    let mut value = NSTextField::new(NSRect::new(0.0, 0.0, 60.0, 22.0));
    value.set_text(&format!("{:.1}", initial));
    value.set_text_alignment(NSTextAlignment::Right);
    // Scrubbing enabled by default for numbers
    
    row.add_arranged_subview(label.view_clone());
    row.add_arranged_subview(value.view_clone());
    row
}

// X, Y, Z position
let x_row = create_transform_row("X:", 0.0);
let y_row = create_transform_row("Y:", 0.0);
let z_row = create_transform_row("Z:", 0.0);
```

## First Responder Chain (Actual Implementation)

Actual methods from `mod.rs`:
- `become_first_responder()` → sets `is_first_responder = true`
- `resign_first_responder()` → sets `is_first_responder = false`
- `is_first_responder()` → returns bool
- `accepts_first_responder()` → returns `editable` state

```
User clicks text field
        │
        ▼
become_first_responder() called
        │
        ▼
state.is_first_responder = true
        │
        ▼
Mouse drag triggers scrubbing (mod.rs:73-126)
        │
        ▼
Text mutated, action callback fired
```

## Testing & Verification

```rust
// Test label
let label = NSTextField::label("Test");
assert!(!label.is_editable());

// Test input
let mut input = NSTextField::new(NSRect::new(0.0, 0.0, 100.0, 22.0));
input.set_text("Hello");
assert_eq!(input.text(), "Hello");

// Test alignment
input.set_text_alignment(NSTextAlignment::Right);
```

## File Structure

```
src/text_field/
└── mod.rs          # NSTextField, NSTextAlignment, NSTextFieldDelegate
```

## Missing from Apple's NSTextField

- **Secure text entry**: Password field dots (partial)
- **Attributed strings**: Rich text formatting
- **Spell checking**: NSSpellChecker integration
- **Undo manager**: Full undo/redo support
- **Delegate methods**: `textShouldBeginEditing`, `textDidChange`
- **Formatter**: NSFormatter for number/date validation

These are stubbed for current use cases.
