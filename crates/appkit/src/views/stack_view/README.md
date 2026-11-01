# NSStackView — Automatic Layout Container

## Apple Header: `NSStackView.h`

## Overview

`NSStackView` arranges subviews in a horizontal or vertical line. Mirrors Apple's [`NSStackView`](https://developer.apple.com/documentation/appkit/nsstackview).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                 NSStackView Creation                         │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Configure Orientation                           │
│  ├─ NSUserInterfaceLayoutOrientation.Horizontal          │
│  │   └── Subviews arranged left-to-right                  │
│  └─ NSUserInterfaceLayoutOrientation.Vertical            │
│      └── Subviews arranged top-to-bottom                   │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Add Arranged Subviews                        │
│  ├─ addArrangedSubview() adds to stack                     │
│  ├─ Spacing applied between views                          │
│  └─ Views sized based on distribution                       │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                 Layout Pass                                  │
│  ├─ Calculate total available space                          │
│  ├─ Apply spacing between views                            │
│  ├─ Size views based on distribution                       │
│  └─ Position views in line                                  │
└─────────────────────────────────────────────────────────────┘
```

## Stack Orientation

| Orientation | Layout Direction |
|-------------|------------------|
| `Horizontal` | Left → Right |
| `Vertical` | Top → Bottom |

## Distribution Modes

| Mode | Behavior |
|------|----------|
| `Fill` | Views fill space, first view gets priority |
| `FillEqually` | All views same size |
| `FillProportionally` | Based on intrinsic content size |
| `EqualSpacing` | Equal gaps between views |
| `EqualCentering` | Equal spacing between centers |

## Connection to Other Modules

```
NSStackView
├── Contains: NSView[] (arranged subviews)
├── Used by: NSButton (button groups)
├── Used by: NSBox (nested layouts)
├── Used by: NSSplitView (pane content)
└── Used by: NSToolbar (item layout)
```

## Usage Examples

### Horizontal Button Group
```rust
use rust_appkit::prelude::*;

let mut toolbar = NSStackView::horizontal();
toolbar.set_spacing(8.0);

let save = NSButton::button_with_title("Save", || save());
let open = NSButton::button_with_title("Open", || open());
let export = NSButton::button_with_title("Export", || export());

toolbar.add_arranged_subview(save.view_clone());
toolbar.add_arranged_subview(open.view_clone());
toolbar.add_arranged_subview(export.view_clone());
```

### Vertical Form Layout
```rust
let mut form = NSStackView::vertical();
form.set_spacing(12.0);

let name_row = create_labeled_row("Name:", name_field.view_clone());
let email_row = create_labeled_row("Email:", email_field.view_clone());

form.add_arranged_subview(name_row);
form.add_arranged_subview(email_row);
```

### Nested Stacks
```rust
// Horizontal row with vertical stacks inside
let main = NSStackView::horizontal();

let left_column = NSStackView::vertical();
left_column.add_arranged_subview(button1.view_clone());
left_column.add_arranged_subview(button2.view_clone());

let right_column = NSStackView::vertical();
right_column.add_arranged_subview(label1.view_clone());
right_column.add_arranged_subview(label2.view_clone());

main.add_arranged_subview(left_column.view_clone());
main.add_arranged_subview(right_column.view_clone());
```

## Visual Layout

```
Horizontal Stack (spacing=10)
┌─────────────────────────────────────────────────────┐
│  [View 1]──10px──[View 2]──10px──[View 3]          │
└─────────────────────────────────────────────────────┘

Vertical Stack (spacing=8)
┌─────────────────────┐
│    [View 1]         │
│      8px            │
│    [View 2]         │
│      8px            │
│    [View 3]         │
└─────────────────────┘
```

## rust-kit-pro Usage

```rust
// Inspector panel uses vertical stack
let inspector_stack = NSStackView::vertical();
inspector_stack.set_spacing(16.0);

// Add sections
inspector_stack.add_arranged_subview(transform_section.view_clone());
inspector_stack.add_arranged_subview(material_section.view_clone());
inspector_stack.add_arranged_subview(physics_section.view_clone());
```

## Testing & Verification

```rust
let mut stack = NSStackView::horizontal();
stack.set_spacing(10.0);

let v1 = NSView::new(NSRect::new(0.0, 0.0, 50.0, 30.0));
let v2 = NSView::new(NSRect::new(0.0, 0.0, 50.0, 30.0));

stack.add_arranged_subview(v1.view_clone());
stack.add_arranged_subview(v2.view_clone());

// After layout, v2.x should be v1.x + 50 + 10 = 60
```

## File Structure

```
src/stack_view/
└── mod.rs          # NSStackView, NSStackViewDistribution
```

## Missing from Apple's NSStackView

- **Edge insets**: Custom spacing from edges
- **Alignment**: Horizontal/vertical alignment
- **Custom spacing**: Per-view spacing overrides
- **Detached views**: Hidden but measured views
- **Visibility priority**: Which views hide first

These are stubbed for current use cases.
