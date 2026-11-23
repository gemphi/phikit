# Panel Architecture Fix

## Problem

Panels are **recreating view components manually** instead of using the provided view classes in `src/views/`.

This violates the separation of concerns:
- `src/views/` = Reusable view components (NSToolbar, NSOutlineView, NSTableView, etc.)
- `src/panels/` = Editor-specific panels that **compose** those views

## Current Issues

### 1. Toolbar Panel
**Current:** `panels/toolbar/view_controller.rs` manually creates NSView and builds everything from scratch
**Should:** Use `views/toolbar/NSToolbar` and configure it

### 2. Hierarchy Panel  
**Current:** Manually builds tree structure with NSView
**Should:** Use `views/outline_view/NSOutlineView`

### 3. Activity Bar
**Current:** Manually positions icon buttons
**Should:** Use a vertical stack or custom NSActivityBar view component

### 4. Inspector Panel
**Current:** Manually builds property rows
**Should:** Use `views/table_view/NSTableView` or property list component

## Refactoring Strategy

### Phase 1: Use Existing View Components
1. **Toolbar** → Use `NSToolbar` from `views/toolbar/`
2. **Hierarchy** → Use `NSOutlineView` from `views/outline_view/`
3. **Inspector** → Use `NSTableView` or create `NSPropertyListView`

### Phase 2: Create Missing View Components
1. **Activity Bar** → Create `views/activity_bar/NSActivityBar`
2. **Timeline** → Use `views/timeline/NSTimeline`
3. **Node Editor** → Use `views/shader_graph_view/`

### Phase 3: Clean Panel Controllers
Panels should only:
- Hold state (model)
- Configure view components
- Handle callbacks/delegates
- NOT manually build UI primitives

## Example Refactor: Toolbar

### Before (panels/toolbar/view_controller.rs):
```rust
fn build_view(width: f32, height: f32, state: &NSToolbarState) -> NSView {
    let mut bar = NSView::new(NSRect::sized(width, height));
    
    // Manually create traffic lights
    let mut dot = NSView::new(...);
    dot.set_background_color(...);
    bar.add_subview(dot);
    
    // Manually create play buttons
    let mut btn = NSView::new(...);
    btn.set_background_color(...);
    bar.add_subview(btn);
    
    // ... 100+ lines of manual UI construction
}
```

### After:
```rust
fn build_view(width: f32, height: f32, state: &NSToolbarState) -> NSToolbar {
    let mut toolbar = NSToolbar::new("EditorToolbar");
    
    // Use toolbar API
    toolbar.add_item(NSToolbarItem::button("play", IconId::Play, || play()));
    toolbar.add_item(NSToolbarItem::button("pause", IconId::Pause, || pause()));
    toolbar.add_flexible_space();
    toolbar.add_item(NSToolbarItem::search_field("search"));
    
    toolbar
}
```

## Benefits

1. **Less code** - Panels become thin configuration layers
2. **Reusability** - View components can be used anywhere
3. **Consistency** - All toolbars/outlines/tables behave the same
4. **Maintainability** - Fix bugs in one place
5. **Testability** - Test view components independently

## Action Items

- [ ] Fix inspector brace error first
- [ ] Refactor toolbar panel to use NSToolbar
- [ ] Refactor hierarchy panel to use NSOutlineView  
- [ ] Create NSActivityBar view component
- [ ] Refactor activity_bar panel to use NSActivityBar
- [ ] Document view component usage patterns
