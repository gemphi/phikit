# Inspector Panel Refactoring Plan

## Current Issues
1. **Duplicate code** - Two `format_value` functions, legacy build methods
2. **Brace mismatch** - Extra closing brace from incomplete refactoring
3. **Manual UI construction** - Not using NSTableView or property list components

## Refactoring Steps

### Phase 1: Fix Compilation (IMMEDIATE)
- Remove duplicate `format_value` function
- Remove legacy `build_view_legacy` method
- Fix brace mismatch

### Phase 2: Use Proper View Components
- Use `NSStackView` for property rows (already partially done)
- Consider using `NSTableView` for property list
- Or create dedicated `NSPropertyListView` component

### Phase 3: Clean Architecture
- Model: `NSInspectorState` (already exists)
- View: Use view components, not manual NSView construction
- Controller: Thin layer that configures views

## Target Architecture

```rust
pub struct NSInspectorViewController {
    pub view: NSStackView,  // or NSPropertyListView
    pub state: NSInspectorState,
}

impl NSInspectorViewController {
    fn rebuild(&mut self) {
        let mut stack = NSStackView::vertical();
        
        for section in &self.state.sections {
            stack.add_arranged_subview(self.build_section(section));
        }
        
        self.view = stack;
    }
    
    fn build_section(&self, section: &NSInspectorSection) -> NSView {
        // Use property row components
    }
}
```

## Action: Fix Now
Remove lines 180-305 (legacy build_view code) and keep only the clean NSStackView-based implementation.
