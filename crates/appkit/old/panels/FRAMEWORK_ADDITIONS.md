# Framework Additions Needed

This document describes what needs to be added to `rust-appkit` to support
the simplified panel architecture.

## 1. NSPropertyGrid Component

**Purpose:** Inspector panel needs a reusable property grid instead of rebuilding views.

**Location:** `rust-appkit/src/controls/property_grid.rs`

**API:**
```rust
pub struct NSPropertyGrid {
    view: NSView,
    rows: Vec<NSPropertyRow>,
    entity_id: Option<u64>,
}

pub struct NSPropertyRow {
    label: NSLabel,
    control: PropertyControl,
}

pub enum PropertyControl {
    Text(NSTextField),
    Number(NSSlider),
    Color(NSColorWell),
    Dropdown(NSPopupButton),
    Bool(NSCheckButton),
}

impl NSPropertyGrid {
    pub fn new(rect: NSRect) -> Self;
    
    /// Clear and rebuild for new entity type
    pub fn set_entity_type(&mut self, descriptor: &EntityDescriptor);
    
    /// Update values without rebuilding UI
    pub fn update_values(&mut self, entity: &dyn Reflect);
    
    /// Set callback for property changes
    pub fn on_change<F>(&mut self, callback: F)
    where F: Fn(&str, PropertyValue) + 'static;
}
```

## 2. Enhanced NSHierarchyViewController

**Purpose:** Support proper data source pattern for hierarchy panel.

**Add to:** `rust-appkit/src/panels/hierarchy/view_controller.rs`

**Additions:**
```rust
impl NSHierarchyViewController {
    /// Set data source callback that provides hierarchy tree.
    pub fn set_data_source<F>(&mut self, provider: F)
    where F: Fn() -> Vec<HierarchyEntity> + 'static;
    
    /// Set callback when selection changes.
    pub fn set_selection_callback<F>(&mut self, callback: F)
    where F: Fn(Option<u64>) + 'static;
    
    /// Set callback for drag-drop reordering.
    pub fn set_drag_handler<F>(&mut self, handler: F)
    where F: Fn(DragOperation) + 'static;
}

pub struct DragOperation {
    pub dragged_id: u64,
    pub target_id: u64,
    pub position: DropPosition, // Before, After, Inside
}
```

## 3. NSViewportViewController.refresh_entities()

**Purpose:** Allow viewport to update entities without full rebuild.

**Add to:** `rust-appkit/src/panels/viewport/`

**API:**
```rust
impl NSViewportViewController {
    /// Update entity transforms without recreating GPU buffers.
    pub fn update_transforms(&mut self, transforms: &[(u64, [f32; 3], [f32; 4], [f32; 3])]);
    
    /// Update materials without full rebuild.
    pub fn update_materials(&mut self, materials: &HashMap<u64, MaterialData>);
    
    /// Full entity replacement (current behavior).
    pub fn update_entities(&mut self, entities: Vec<ViewportEntity>, materials: &HashMap<u64, MaterialData>);
}
```

## 4. Missing EditorPanel Implementations

**Purpose:** Make all framework panels work with workspace system.

**Add to each panel module:**

```rust
// In rust-appkit/src/panels/activity_bar/
impl EditorPanel for NSActivityBarViewController {
    fn view(&self) -> &NSView { &self.view }
    fn view_mut(&mut self) -> &mut NSView { &mut self.view }
    fn on_selection_changed(&mut self, _entity_id: Option<u64>) {}
    fn reload_data(&mut self) {}
    fn name(&self) -> &str { "ActivityBar" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

// Same for: toolbar, project_browser, asset_library, etc.
```

## 5. ActivityBarPanel Mode Switching

**Purpose:** Allow activity bar to switch sidebar content.

**Add to:** `rust-appkit/src/panels/activity_bar/`

**API:**
```rust
pub enum SidebarMode {
    Hierarchy,
    Search,
    Extensions,
    Bookmarks,
}

impl NSActivityBarViewController {
    /// Set callback when mode changes.
    pub fn on_mode_change<F>(&mut self, callback: F)
    where F: Fn(SidebarMode) + 'static;
    
    /// Programmatically set active mode.
    pub fn set_mode(&mut self, mode: SidebarMode);
}
```

## Implementation Priority

| Priority | Feature | Panel Unblocked |
|----------|---------|-----------------|
| **P0** | NSHierarchyViewController data source | Hierarchy |
| **P0** | NSViewportViewController.update_entities() | Viewport |
| **P1** | NSPropertyGrid | Inspector |
| **P1** | EditorPanel impls for framework panels | All panels |
| **P2** | ActivityBar mode switching | Navigator |
