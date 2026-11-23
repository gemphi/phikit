# Panel Refactoring — Use Views, Not Raw NSView

## The Problem

**Current (WRONG):** Panels manually build UI with raw `NSView`
```rust
// panels/activity_bar/view_controller.rs
let mut btn = NSView::new(frame).with_name("ActivityBar");  // WRONG!
btn.set_background_color(if is_active { selected } else { none }); // Manual!
btn.set_hover_color(Some(hover)); // Manual!
// Then manually add NSImageView inside
```

**Should Be (CORRECT):** Use `views/` components
```rust
// panels/activity_bar/view_controller.rs
use crate::views::toolbar::NSToolbar;  // USE THIS!
let mut toolbar = NSToolbar::new("ActivityBar");
toolbar.add_item(NSToolbarItem::with_icon(icon, action));  // Button handles itself!
```

## Architecture Mapping

| Panel | Currently Doing | Should Use From views/ |
|-------|-----------------|------------------------|
| **Toolbar** | Manual NSView construction | `NSToolbar` |
| **ActivityBar** | Manual NSView buttons | Custom but use `NSButton`, not NSView |
| **Hierarchy** | Manual tree rendering | `NSOutlineView` |
| **Inspector** | Manual stack construction | `NSStackView` + property controls |
| **Project** | Partial manual construction | `NSTabView` + `NSTableView` |
| **Viewport** | ✅ Uses `NSViewportViewController` | ✅ Already correct |

## Files to Refactor

### 1. panels/toolbar/view_controller.rs
**Current:** Manually building toolbar with NSView
**Should:** Use `views/toolbar/NSToolbar.rs`

```rust
// CORRECT implementation
use crate::views::toolbar::{NSToolbar, NSToolbarItem, NSToolbarDisplayMode};

pub struct NSToolbarViewController {
    pub toolbar: NSToolbar,
    pub playback_state: PlaybackState,
    pub editor_mode: EditorMode,
}

impl NSToolbarViewController {
    pub fn new(width: f32, height: f32) -> Self {
        let mut toolbar = NSToolbar::new("EditorToolbar");
        toolbar.set_display_mode(NSToolbarDisplayMode::IconOnly);
        
        // Add items using NSToolbarItem
        toolbar.add_item(Self::create_play_button());
        toolbar.add_item(Self::create_mode_segmented());
        toolbar.add_item(Self::create_search_field());
        toolbar.add_item(Self::create_layout_toggles());
        
        Self {
            toolbar,
            playback_state: PlaybackState::Stopped,
            editor_mode: EditorMode::Scene,
        }
    }
    
    fn create_play_button() -> NSToolbarItem {
        // Use NSButtonDefaults for consistent styling
        let btn = NSButtonDefaults::play_control(IconId::Play, false, || {});
        NSToolbarItem::with_view("play", "Play", btn.view_clone())
    }
}
```

### 2. panels/activity_bar/view_controller.rs
**Current:** Manual NSView icon buttons
**Should:** Use `NSButton` from `controls/button/`

```rust
use crate::controls::button::NSButtonDefaults;

fn build_icon_button(...) -> NSButton {
    // NOT NSView!
    NSButtonDefaults::activity_bar(icon_id, is_active, action)
}
```

### 3. panels/hierarchy/view_controller.rs
**Current:** Manual tree with NSView rows
**Should:** Use `views/outline_view/NSOutlineView.rs`

```rust
use crate::views::outline_view::{NSOutlineView, NSOutlineViewDataSource};

pub struct NSHierarchyViewController {
    pub outline_view: NSOutlineView,  // USE THIS!
    pub view: NSView,
}

impl NSHierarchyViewController {
    pub fn new(rect: NSRect) -> Self {
        let outline_view = NSOutlineView::new(rect);
        // Let NSOutlineView handle its own rendering!
        Self { outline_view, view: outline_view.view().clone() }
    }
    
    pub fn set_data_source(&mut self, ds: impl NSOutlineViewDataSource) {
        self.outline_view.set_data_source(ds);
    }
}
```

### 4. panels/inspector/view_controller.rs
**Current:** Manual NSStackView with manual property rows
**Should:** Use `NSStackView` + proper controls

```rust
use crate::views::stack_view::NSStackView;
use crate::controls::{NSSlider, NSColorWell, NSTextField};

pub struct NSInspectorViewController {
    pub stack_view: NSStackView,
    pub view: NSView,
}

impl NSInspectorViewController {
    fn add_property_row(&mut self, name: &str, control: impl Into<NSControl>) {
        let row = NSStackView::horizontal();
        row.add_arranged_subview(LabelDefaults::row_label(name).view_clone());
        row.add_arranged_subview(control.into().view_clone());
        self.stack_view.add_arranged_subview(row.view_clone());
    }
}
```

## Missing View Controllers

Need to create these in `panels/`:

| View | Panel Controller | Status |
|------|-----------------|--------|
| `NSToolbar` | `NSToolbarViewController` | ✅ Exists but not using NSToolbar |
| `NSOutlineView` | `NSHierarchyViewController` | ✅ Exists but manual rendering |
| `NSTableView` | `NSProjectBrowserViewController` | ⚠️ Partial |
| `NSTabView` | `NSProjectPanelController` | ✅ Uses it |
| `NSStackView` | `NSInspectorViewController` | ❌ Missing! |
| `NSViewportGPU` | `NSViewportViewController` | ✅ Good |

## Priority Order

1. **ActivityBar** (Easiest) - Replace NSView with NSButton
2. **Toolbar** - Use NSToolbar instead of manual construction
3. **Hierarchy** - Use NSOutlineView data source pattern
4. **Inspector** - Create NSInspectorViewController with proper controls

## Consistency Checklist

Each panel should:
- [ ] Use `views/*` components, not raw NSView
- [ ] Use `controls/*` for interactive elements
- [ ] Implement `EditorPanel` trait
- [ ] Have thin wrapper in rust-kit-pro
- [ ] Under 200 lines
