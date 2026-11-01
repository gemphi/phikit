# NSAlert — Modal Dialogs

## Apple Header: `NSAlert.h`

## Overview

`NSAlert` displays modal dialog boxes to the user. Mirrors Apple's [`NSAlert`](https://developer.apple.com/documentation/appkit/nsalert).

## Architecture Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    NSAlert Creation                          │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              Configure Alert Content                         │
│  ├─ setMessageText() - Bold title                          │
│  ├─ setInformativeText() - Detailed description            │
│  ├─ setAlertStyle() - Warning/Info/Critical                │
│  └─ addButtonWithTitle() - Action buttons                  │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              runModal() - Display Alert                    │
│  ├─ Blocks until user responds (in real AppKit)           │
│  ├─ Returns button index (NSAlertFirstButtonReturn = 1000)│
│  └─ Current stub logs to console                           │
└─────────────────────────────────────────────────────────────┘
```

## Alert Styles (NSAlertStyle)

| Style | Icon | Use Case |
|-------|------|----------|
| `Warning` | ⚠️ Yellow triangle | Potential issues |
| `Informational` | ℹ️ Info icon | Neutral information |
| `Critical` | 🛑 Stop icon | Data loss, errors |

## Connection to Other Modules

```
NSAlert
├── Uses: NSWindow (modal presentation)
├── Uses: NSButton (alert buttons)
├── Uses: NSImage (style icons)
└── Called by: Application error handlers
```

## Usage Examples

### Simple Confirmation
```rust
use rust_appkit::prelude::*;
use rust_appkit::alert::{NSAlert, NSAlertStyle};

let mut alert = NSAlert::new();
alert.set_message_text("Delete Project?");
alert.set_informative_text("This action cannot be undone.");
alert.set_alert_style(NSAlertStyle::Warning);
alert.add_button_with_title("Delete");
alert.add_button_with_title("Cancel");

let response = alert.run_modal();
if response == 1000 {
    // First button (Delete) clicked
    perform_delete();
}
```

### Error Alert
```rust
let mut alert = NSAlert::new();
alert.set_message_text("Export Failed");
alert.set_informative_text("Could not write to disk. Check permissions.");
alert.set_alert_style(NSAlertStyle::Critical);
alert.add_button_with_title("OK");
alert.run_modal();
```

### Save Changes Dialog
```rust
let mut alert = NSAlert::new();
alert.set_message_text("Save changes to 'Untitled.phiproject'?");
alert.set_alert_style(NSAlertStyle::Informational);
alert.add_button_with_title("Save");
alert.add_button_with_title("Don't Save");
alert.add_button_with_title("Cancel");

match alert.run_modal() {
    1000 => save_project(),
    1001 => discard_changes(),
    _ => {}, // Cancel
}
```

## Response Codes

| Code | Constant | Meaning |
|------|----------|---------|
| 1000 | `NSAlertFirstButtonReturn` | First button added (typically default) |
| 1001 | `NSAlertSecondButtonReturn` | Second button |
| 1002 | `NSAlertThirdButtonReturn` | Third button |

## Visual Layout

```
┌──────────────────────────────────────┐
│  [Icon]  Message Text (Bold)          │
│          Informative text (regular)   │
│                                       │
│          [Button 1]  [Button 2]      │
└──────────────────────────────────────┘
```

## Testing & Verification

```rust
let mut alert = NSAlert::new();
alert.set_message_text("Test");
alert.add_button_with_title("OK");
alert.add_button_with_title("Cancel");

assert_eq!(alert.run_modal(), 1000); // Stub returns first button
```

## Current Implementation Status

⚠️ **Partial Implementation**: Currently a stub that logs to console. Full modal window presentation requires `NSWindow` modal session support.

## File Structure

```
src/alert/
└── mod.rs          # NSAlert, NSAlertStyle
```

## Missing from Apple's NSAlert.h

- **Modal session**: True blocking `runModal()`
- **Sheet presentation**: `beginSheetModalForWindow`
- **Help button**: `setShowsHelp`
- **Suppression**: "Don't ask again" checkbox
- **Accessory views**: Custom content in alert
- **Sound**: Alert sounds

These require full `NSWindow` modal implementation.
