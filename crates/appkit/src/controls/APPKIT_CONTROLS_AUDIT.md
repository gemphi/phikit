# Apple AppKit Controls Audit — What's Missing/Inconsistent

## Current State (INCONSISTENT)

### ❌ Problems Identified:

1. **ButtonDefaults scattered**: Referenced in `panels/` but should be in `controls::button`
2. **No NSButtonDefaults**: The `ButtonDefaults` in panels doesn't match Apple's NSButton factory methods
3. **Manual view construction**: ActivityBar building `NSView` instead of using `NSButton`
4. **Inconsistent factory patterns**: `LabelDefaults::header()` vs `NSButton::button_with_title()`

### ❌ Current Anti-Patterns:

```rust
// BAD: ActivityBar manually building NSView buttons
let mut btn = NSView::new(frame);  // Should be NSButton!
btn.set_background_color(if is_active { selected } else { none });
btn.set_hover_color(Some(hover));  // Manual hover!

// BAD: Using LabelDefaults for everything
LabelDefaults::header("Title")  // Good
LabelDefaults::row_value("123") // Good
// But buttons have no equivalent!
```

## Apple's AppKit Pattern (What We Should Have)

### NSButton Factory Methods (Apple's Design)

| Apple AppKit | Our Current | Status |
|--------------|-------------|--------|
| `+buttonWithTitle:target:action:` | ❌ Missing | Needs impl |
| `+buttonWithImage:target:action:` | ✅ Partial | `NSButton::button_with_image()` |
| `+checkboxWithTitle:target:action:` | ❌ Missing | Needs impl |
| `+radioButtonWithTitle:target:action:` | ❌ Missing | Needs impl |
| `+toolbarButtonWithTitle:target:action:` | ❌ Missing | Needs impl |
| `+pushButtonWithTitle:target:action:` | ❌ Missing | Needs impl |

### NSButton State Management (Apple's Design)

```objc
// Apple: NSButton manages its own state
NSButton *btn = [NSButton buttonWithTitle:@"Click" target:self action:@selector(clicked)];
[btn setState:NSControlStateValueOn];  // Built-in!
[btn setBezelStyle:NSBezelStyleToolbar]; // Styling!
```

Our button HAS this internally (`NSButtonState`) but we're not using it!

## Missing Apple Controls

| Apple Control | Our Status | Used In |
|---------------|-----------|---------|
| **NSButton** (complete) | ⚠️ Partial | ActivityBar (wrong!), Toolbar |
| **NSTextField** | ✅ Good | Inspector, labels |
| **NSSlider** | ❓ Check | Inspector needs it |
| **NSColorWell** | ❓ Check | Inspector needs it |
| **NSPopUpButton** | ✅ Has popup_button | Project panel? |
| **NSStepper** | ❓ Check | Inspector number inputs |
| **NSSegmentedControl** | ✅ Has it | Toolbar? |
| **NSProgressIndicator** | ✅ Has it | Statistics? |
| **NSImageView** | ✅ Has it | ActivityBar icons |
| **NSSwitch** (macOS 10.15+) | ✅ Has switch | Inspector booleans |
| **NSComboBox** | ❌ Missing | Search/filter? |
| **NSOutlineView** | ✅ Has it | Hierarchy panel |
| **NSTableView** | ✅ Has it | Project browser |
| **NSCollectionView** | ✅ Has it | Asset library |
| **NSTabView** | ✅ Has it | Project panel |
| **NSSplitView** | ✅ Has it | Main layout |
| **NSStackView** | ✅ Has it | Inspector |
| **NSBox** | ✅ Has it | Separators |

## Recommended Reorganization

### 1. Create `NSButtonDefaults` in `controls::button`

```rust
// controls/button/defaults.rs
pub struct NSButtonDefaults;

impl NSButtonDefaults {
    /// +buttonWithTitle:target:action:
    pub fn with_title(title: &str, action: impl Fn() + 'static) -> NSButton;
    
    /// +buttonWithImage:target:action:
    pub fn with_image(image: NSImage, action: impl Fn() + 'static) -> NSButton;
    
    /// +checkboxWithTitle:target:action:
    pub fn checkbox(title: &str, action: impl Fn(bool) + 'static) -> NSButton;
    
    /// +radioButtonWithTitle:target:action:
    pub fn radio(title: &str, action: impl Fn() + 'static) -> NSButton;
    
    /// Toolbar button (icon only)
    pub fn toolbar_icon(icon: IconId, action: impl Fn() + 'static) -> NSButton;
    
    /// Activity bar button (selectable)
    pub fn activity_bar(icon: IconId, is_active: bool, action: impl Fn() + 'static) -> NSButton;
    
    /// Play control button (circular)
    pub fn play_control(icon: IconId, is_active: bool, action: impl Fn() + 'static) -> NSButton;
}
```

### 2. Fix ActivityBarViewController

```rust
// OLD: Manual NSView construction
let mut btn = NSView::new(frame);  // WRONG

// NEW: Use NSButton with proper state
let btn = NSButtonDefaults::activity_bar(
    item.icon_id, 
    is_active,
    item.action
);
```

### 3. Consistent Defaults Pattern

| Type | Factory | Example |
|------|---------|---------|
| Labels | `LabelDefaults` | `LabelDefaults::header("Title")` |
| Buttons | `NSButtonDefaults` | `NSButtonDefaults::toolbar_icon(...)` |
| Panels | `PanelDefaults` | `PanelDefaults::sidebar()` |
| Separators | `SeparatorDefaults` | `SeparatorDefaults::horizontal()` |

### 4. Panel Consistency Checklist

| Panel | Uses Controller | Uses Defaults | Has EditorPanel | Clean |
|-------|----------------|---------------|-----------------|-------|
| Toolbar | ✅ | ⚠️ Partial | ❌ | Needs work |
| ActivityBar | ✅ | ⚠️ Manual | ❌ | Needs work |
| Hierarchy | ❌ | ❌ | ❌ | Needs full refactor |
| Inspector | ❌ | ✅ | ✅ | Needs controller |
| Viewport | ✅ | ✅ | ✅ | ✅ Good |
| Project | ⚠️ Partial | ✅ | ✅ | Needs cleanup |
| Navigator | ✅ | ⚠️ Manual | ❌ | Needs EditorPanel |

## Action Plan

1. **Create `NSButtonDefaults`** — proper factory for all button types
2. **Fix ActivityBarViewController** — use NSButton, not NSView
3. **Fix ToolbarViewController** — use NSButtonDefaults
4. **Add EditorPanel to all panels** — missing in 3 panels
5. **Create `NSPropertyInspectorViewController`** — for inspector panel

## Current Button State (From button_types.rs)

Your `NSButton` already has:
- `NSButtonState` with `is_hovered`, `is_pressed`, `interaction_state`
- `bezel_style`, `state` (NSControlStateValue)
- `image`, `alternate_image`
- `action` callback

**We're just not using it properly!** The framework has the right internals but the panels are bypassing them.
