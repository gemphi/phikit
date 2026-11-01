# Systematic Refactoring — All Controls, Views, Panels

## Architecture Principles

1. **Controls** (`controls/`): Interactive elements with built-in state
   - `NSButton`, `NSTextField`, `NSSlider`, etc.
   - Factory methods directly on type: `NSButton::toolbar_icon()`

2. **Views** (`views/`): Layout containers and display components
   - `NSStackView`, `NSToolbar`, `NSOutlineView`, `NSTableView`, etc.
   - Panels should USE these, not recreate with `NSView`

3. **Panels** (`panels/`): Editor-specific view controllers
   - Thin wrappers around `views/` components
   - Implement `EditorPanel` trait
   - Delegate to data sources, don't render manually

## Refactoring Checklist

### Phase 1: Controls (Factory Methods)

| Control | Current | Target | Status |
|---------|---------|--------|--------|
| **NSButton** | `NSButtonDefaults::` | `NSButton::` | ✅ Fixed |
| **NSTextField** | Check if needs factories | `NSTextField::label()`, `::header()` | ⬜ |
| **NSSlider** | Check current | `NSSlider::with_range()` | ⬜ |
| **NSColorWell** | Check current | `NSColorWell::new()` | ⬜ |
| **NSStepper** | Check current | `NSStepper::with_range()` | ⬜ |

### Phase 2: Views (Ensure Complete)

| View | Status | Notes |
|------|--------|-------|
| **NSStackView** | ✅ | Good, panels should use |
| **NSToolbar** | ⚠️ | Check if panels use it |
| **NSOutlineView** | ✅ | Hierarchy should use |
| **NSTableView** | ✅ | Project browser should use |
| **NSTabView** | ✅ | Project panel uses |
| **NSSplitView** | ✅ | Main layout uses |
| **NSCollectionView** | ✅ | Asset library |

### Phase 3: Panels (Major Refactoring Needed)

| Panel | Current Architecture | Target Architecture | Priority |
|-------|----------------------|---------------------|----------|
| **ActivityBar** | ✅ NSStackView + NSButton | ✅ Done | - |
| **Toolbar** | Manual NSView construction | Use `NSToolbar` | **P0** |
| **Hierarchy** | Manual tree with NSView | Use `NSOutlineView` + data source | **P0** |
| **Inspector** | Manual NSStackView rows | `NSStackView` + controls:: factories | **P1** |
| **Project** | Partial NSTabView | Full `NSTabView` + `NSTableView` | **P1** |
| **Viewport** | Uses `NSViewportViewController` | ✅ Good | - |
| **Navigator** | Simple wrapper | Add `EditorPanel` impl | **P2** |

## File-by-File Action Plan

### 1. panels/toolbar/view_controller.rs
**Problem:** Manually constructing toolbar with NSView
```rust
// WRONG: Manual construction
let mut toolbar = NSView::new(frame);
// Manually add buttons, search, etc.

// CORRECT: Use NSToolbar
let mut toolbar = NSToolbar::new("EditorToolbar");
toolbar.add_item(NSButton::toolbar_icon(...));
```

### 2. panels/hierarchy/view_controller.rs
**Problem:** Manual tree rendering
```rust
// WRONG: Manual NSView rows
for entity in entities {
    let row = NSView::new(frame);
    // Manual rendering
}

// CORRECT: NSOutlineView with data source
let outline = NSOutlineView::new(frame);
outline.set_data_source(scene_data_source);
```

### 3. panels/inspector/view_controller.rs
**Problem:** Doesn't exist! Manual construction in rust-kit-pro
```rust
// NEED: NSInspectorViewController
pub struct NSInspectorViewController {
    stack_view: NSStackView,
    // Property grid support
}
```

### 4. panels/project/view_controller.rs
**Problem:** Partial implementation
- Uses NSTabView ✅
- But manual content construction ❌
- Should use NSTableView for browser

## Execution Order

1. **Toolbar** — Most visible, high impact
2. **Hierarchy** — Core functionality, complex refactor
3. **Inspector** — Create missing controller
4. **Project** — Complete partial implementation

## Testing Checklist

After each panel refactor:
- [ ] Compiles without errors
- [ ] Uses views/* components, not raw NSView
- [ ] Uses controls/* factories
- [ ] Implements EditorPanel trait
- [ ] Under 200 lines
- [ ] Proper documentation comments
