# DELETE builder.rs - App-Specific Code in Wrong Layer

## Problem
`builder.rs` contains Reality Composer Pro-specific UI code in the reusable view component layer.

## Evidence
- Hardcoded tab names: "Project Browser", "Shader Graph", "Timelines", "Audio Mixer", "Statistics"
- App-specific icons: `IconId::BottomPanel`, `IconId::ShaderGraph`, etc.
- App-specific layout and styling
- Function name: `build_rcp_bottom_tabs` (RCP = Reality Composer Pro)

## Why This is Wrong
**rust-appkit** = Reusable AppKit-style components (like Apple's AppKit)
**rust-kit-pro** = Reality Composer Pro app-specific code

This file is app code in the framework layer!

## Solution
1. **DELETE** `src/views/tab_view/builder.rs` from rust-appkit
2. **MOVE** to `rust-kit-pro/src/panels/bottom_panel/builder.rs`
3. Keep generic `NSTabView` component in rust-appkit
4. App-specific configuration goes in rust-kit-pro

## Correct Pattern

### rust-appkit (Generic)
```rust
// src/views/tab_view/mod.rs
pub struct NSTabView {
    tabs: Vec<NSTabViewItem>,
    selected_index: usize,
}

impl NSTabView {
    pub fn add_tab(&mut self, item: NSTabViewItem) { ... }
    pub fn select_tab(&mut self, index: usize) { ... }
}
```

### rust-kit-pro (App-Specific)
```rust
// src/panels/bottom_panel/view_controller.rs
pub struct BottomPanelController {
    tab_view: NSTabView,
}

impl BottomPanelController {
    fn build_tabs() -> NSTabView {
        let mut tabs = NSTabView::new();
        tabs.add_tab(NSTabViewItem::new("Project Browser", IconId::BottomPanel));
        tabs.add_tab(NSTabViewItem::new("Shader Graph", IconId::ShaderGraph));
        // ... RCP-specific tabs
        tabs
    }
}
```

## Action Required
1. Move builder.rs to rust-kit-pro
2. Update imports in rust-kit-pro
3. Delete from rust-appkit
4. Verify NSTabView component is generic
