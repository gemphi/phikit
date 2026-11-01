# Panel Naming Conventions & Structure

## 🎯 Apple Naming Conventions

All panel controllers follow **Apple's Cocoa/AppKit naming patterns**:

### Controller Naming
- **Pattern:** `NS{PanelName}ViewController`
- **Examples:**
  - `NSHierarchyViewController` (like `NSOutlineViewController`)
  - `NSInspectorViewController` (like `NSPropertyInspectorViewController`)
  - `NSToolbarViewController` (like `NSToolbarController`)
  - `NSViewportViewController` (custom, but follows pattern)

### Method Naming (Apple Delegate Pattern)
- **Setters:** `set_{property}:` → `set_data_source()`, `set_state()`
- **Actions:** `{action}Item:` → `expand_item()`, `collapse_item()`, `select_item()`
- **Queries:** `{property}` → `selected_item()`, `is_expandable()`
- **Callbacks:** `set_on_{event}` → `set_on_selection_changed()`, `set_on_drag_drop()`

### Delegate Callbacks (Apple Pattern)
```rust
// Apple: outlineViewSelectionDidChange:
pub fn set_on_selection_changed<F>(&mut self, callback: F)

// Apple: outlineView:acceptDrop:item:childIndex:
pub fn set_on_drag_drop<F>(&mut self, callback: F)

// Apple: outlineViewItemDidExpand:
pub fn set_on_expand_collapse<F>(&mut self, callback: F)
```

---

## 📁 Consistent Folder Structure

**Every panel folder follows this exact structure:**

```
panel_name/
├── mod.rs              # Module exports
├── model.rs            # Data structures
└── view_controller.rs  # UI controller (Apple pattern)
```

### Optional Files (Only When Needed)
```
panel_name/
├── mod.rs
├── model.rs
├── view_controller.rs
└── data_source.rs      # ✅ ONLY for data source traits (like hierarchy)
```

---

## ✅ Current Panel Structure

All panels now follow the consistent pattern:

### Standard Panels (3 files each)
```
activity_bar/
├── mod.rs
├── model.rs
└── view_controller.rs

asset_library/
├── mod.rs
├── model.rs
└── view_controller.rs

inspector/
├── mod.rs
├── model.rs
└── view_controller.rs

material_inspector/
├── mod.rs
├── model.rs
└── view_controller.rs

node_editor/
├── mod.rs
├── model.rs
└── view_controller.rs

project_browser/
├── mod.rs
├── model.rs
└── view_controller.rs

statistics/
├── mod.rs
├── model.rs
└── view_controller.rs

timeline/
├── mod.rs
├── model.rs
└── view_controller.rs

toolbar/
├── mod.rs
├── model.rs
└── view_controller.rs

tools_sidebar/
├── mod.rs
├── model.rs
└── view_controller.rs

viewport/
├── mod.rs
├── model.rs
└── view_controller.rs
```

### Special Case: Hierarchy (4 files)
```
hierarchy/
├── mod.rs
├── model.rs
├── view_controller.rs
└── data_source.rs      # ✅ Trait for data binding (Apple pattern)
```

**Why data_source.rs?** 
- Follows Apple's `NSOutlineViewDataSource` protocol pattern
- Separates data interface from UI controller
- Allows flexible data binding (Scene, mock data, etc.)

---

## 📋 File Responsibilities

### mod.rs
```rust
//! Panel name - brief description.
//! 
//! Follows Apple's NS{Name}ViewController pattern.
pub mod model;
pub mod view_controller;

pub use model::*;
pub use view_controller::*;
```

### model.rs
```rust
//! Data structures for the panel.

#[derive(Debug, Clone)]
pub struct PanelState {
    // State fields
}

#[derive(Debug, Clone)]
pub enum PanelValue {
    // Value types
}
```

### view_controller.rs
```rust
//! # NS{Panel}ViewController
//!
//! Follows Apple's NSViewController pattern.

pub struct NS{Panel}ViewController {
    pub view: NSView,
    // State and callbacks
}

impl NS{Panel}ViewController {
    // Apple-style methods:
    pub fn new(rect: NSRect) -> Self { }
    pub fn set_state(&mut self, state: State) { }
    pub fn reload_data(&mut self) { }
    pub fn set_on_event<F>(&mut self, callback: F) { }
}
```

### data_source.rs (Optional - Only for data-driven panels)
```rust
//! Data source trait for {Panel}.
//! 
//! Follows Apple's NS{Panel}DataSource protocol pattern.

pub trait {Panel}DataSource: Send + Sync {
    fn root_items(&self) -> Vec<Item>;
    fn children_of(&self, id: u64) -> Vec<Item>;
}
```

---

## 🔍 Apple Pattern Examples

### NSOutlineView Pattern (Hierarchy)
```rust
// Apple: NSOutlineView + NSOutlineViewDataSource
pub struct NSHierarchyViewController {
    pub outline_view: NSOutlineView,
    data_source: Option<Arc<Mutex<dyn HierarchyDataSource>>>,
}

// Apple: setDataSource:
pub fn set_data_source(&mut self, source: Arc<Mutex<dyn HierarchyDataSource>>)

// Apple: reloadData
pub fn reload_data(&mut self)

// Apple: selectRowIndexes:byExtendingSelection:
pub fn select_item(&mut self, entity_id: u64)

// Apple: expandItem:
pub fn expand_item(&mut self, entity_id: u64)
```

### NSViewController Pattern (Inspector)
```rust
// Apple: NSViewController
pub struct NSInspectorViewController {
    pub view: NSView,
    pub state: NSInspectorState,
}

// Apple: setRepresentedObject:
pub fn set_state(&mut self, state: NSInspectorState)

// Apple: viewDidLoad equivalent
pub fn new(rect: NSRect) -> Self

// Apple: delegate pattern
pub fn set_on_property_changed<F>(&mut self, callback: F)
```

---

## ❌ Anti-Patterns (What NOT to Do)

### ❌ Don't Create Extra Files
```
hierarchy/
├── mod.rs
├── model.rs
├── view_controller.rs
├── enhanced_controller.rs  ❌ NO! Merge into view_controller.rs
└── property_inspector.rs   ❌ NO! Merge into view_controller.rs
```

### ❌ Don't Use Non-Apple Naming
```rust
// ❌ Bad
pub struct HierarchyPanelController { }
pub struct EnhancedHierarchyView { }
pub struct PropertyInspectorWidget { }

// ✅ Good (Apple pattern)
pub struct NSHierarchyViewController { }
pub struct NSInspectorViewController { }
```

### ❌ Don't Mix Patterns
```rust
// ❌ Bad - inconsistent method names
pub fn update_selection(&mut self, id: u64)
pub fn changeState(&mut self, state: State)
pub fn on_drag(&mut self, callback: F)

// ✅ Good - consistent Apple pattern
pub fn select_item(&mut self, id: u64)
pub fn set_state(&mut self, state: State)
pub fn set_on_drag_drop<F>(&mut self, callback: F)
```

---

## 📚 Reference: Apple Documentation

### NSViewController
- Manages a view hierarchy
- Owns the view
- Driven by model/state
- Provides lifecycle methods

### NSOutlineView + DataSource
- Hierarchical data display
- Separate data source protocol
- Delegate callbacks for events
- Selection, expand/collapse, drag/drop

### Delegate Pattern
- Callbacks for events
- `set_on_{event}` pattern
- Closure-based in Rust (vs protocol in Objective-C)

---

## ✅ Checklist for New Panels

When creating a new panel:

- [ ] Create folder: `panel_name/`
- [ ] Create `mod.rs` with exports
- [ ] Create `model.rs` with data structures
- [ ] Create `view_controller.rs` with `NS{Name}ViewController`
- [ ] Use Apple naming: `set_*`, `*_item`, `reload_data`
- [ ] Add delegate callbacks: `set_on_*`
- [ ] Only add `data_source.rs` if needed (data-driven panels)
- [ ] Document with Apple pattern references
- [ ] Export from `panels/mod.rs`

---

## 🎯 Summary

**Key Principles:**
1. **Consistent Structure** - All panels have mod.rs, model.rs, view_controller.rs
2. **Apple Naming** - NS{Name}ViewController, set_*, *_item patterns
3. **Minimal Files** - Only add data_source.rs when truly needed
4. **Delegate Pattern** - Use set_on_* for callbacks
5. **Documentation** - Reference Apple patterns in comments

**Result:** Clean, consistent, professional codebase that follows industry-standard patterns.

---

**Last Updated:** 2026-05-27
**Status:** ✅ All Panels Consistent
