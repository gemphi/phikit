# Rust AppKit

## Overview
`rust-appkit` is a high-performance, Swift-inspired native UI framework for Windows, providing an experience mirroring macOS CoreGraphics and AppKit APIs. 

## Component Flows
This crate drives the primary UI composition. Its key flows include:
1. **Window Management:** Uses `winit` to spin up native OS windows while keeping the API surface native to AppKit's `NSWindow`.
2. **View Hierarchy:** Uses an `NSView` tree structure that handles rendering, hit-testing, auto-layout, and event delegation using the `NSViewDelegate` trait.
3. **Hardware Acceleration:** Leverages `wgpu` (`RenderContext`) to rasterize UI elements, shapes, gradients, and custom themes (such as the RCP frosted glass effect).
4. **Controls & Panels:** Implements native-style controls (e.g., `NSButton`, `NSTextField`, `NSOutlineView`) mapping directly to Apple's Objective-C/Swift factory methods.

## Connectivity with Other Crates
- **rust-kit-pro:** Directly consumes `rust-appkit` to build the full 3D editor panels (Viewport, Inspector, Hierarchy).
- **phx:** Feeds 3D render-pass textures into AppKit’s `wgpu` surfaces for hybrid 2D/3D integration.

## Example Flow: Button Instantiation
Instead of relying on boilerplate-heavy structs, `rust-appkit` mirrors Apple's AppKit interfaces.
```rust
use rust_appkit::button::NSButton;

// Apple-style UI instantiation
let save_btn = NSButton::button_with_title("Save", || {
    println!("Save action triggered!");
});

// Adding it to a stack view or layout
my_stack_view.add_arranged_subview(save_btn.view_clone());
```
