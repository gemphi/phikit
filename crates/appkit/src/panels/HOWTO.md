# Panels — How to Use

## Files
- `mod.rs` — module registry, re-exports all panel implementations
- `traits.rs` — `EditorPanel` trait, `PanelConfig` trait
- `defaults.rs` — default panel configurations
- `floating_panels.rs` — floating/detachable panel support
- Subdirectories: `viewport/`, `hierarchy/`, `inspector/`, `timeline/`, etc.

## Quick Start
```rust
use rust_appkit::panels::EditorPanel;

struct MyPanel { view: NSView }

impl EditorPanel for MyPanel {
    fn view(&self) -> &NSView { &self.view }
    fn view_mut(&mut self) -> &mut NSView { &mut self.view }
    fn on_selection_changed(&mut self, entity_id: Option<u64>) {
        // React to global selection
    }
    fn name(&self) -> &str { "MyPanel" }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
```

## Key APIs
- `EditorPanel::view()` — returns the panel's NSView
- `EditorPanel::on_selection_changed(id)` — global selection notification
- `EditorPanel::name()` — unique panel identifier
- `EditorPanel::reload_data()` — refresh from data source
- `EditorPanel::min_size()` / `max_size()` / `preferred_size()` — layout hints
- `EditorPanel::set_collapsed(bool)` — collapse/expand
- `PanelConfig::title()` / `icon_name()` / `default_size()` — display metadata

## Rules
- Every workspace region implements `EditorPanel`
- `on_selection_changed` is the primary inter-panel communication channel
- Collapse = `view.set_hidden(true)` — space is reclaimed by split view
- `as_any()` enables downcasting to specific panel types
- Panel controllers live here; `rust-kit-pro` provides thin app wrappers
