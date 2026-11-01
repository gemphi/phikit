# Windows AppKit Tasks & Roadmap

## Current Status
**Version:** 0.2.0  
**Status:** UI shell complete, egui fully purged, SceneKit gap analysis done  
**SceneKit Parity:** 6% implemented, 17% partial, 78% missing (across 4 core headers)

---

## Completed Tasks ✅

### Core Framework
- [x] Create crate structure (`rust-appkit`)
- [x] Implement `NSColor`, `NSRect`, `NSPoint`, `NSSize`
- [x] Implement `NSEdgeInsets`, `NSAutoresizingMask`
- [x] Create `NSView` base class with hierarchy & delegate pattern
- [x] Create `NSControl` base class with actions
- [x] Implement `NSButton` with all styles
- [x] Implement `NSTextField` with scrubbing & action callbacks
- [x] Implement `NSSlider` with vertical/horizontal
- [x] Implement `NSStackView` for layout
- [x] Implement `NSScrollView`, `NSClipView`, `NSScroller`
- [x] Implement `NSSplitView` with controller
- [x] Implement `NSWindow` with delegate
- [x] Implement `NSApplication` with winit integration
- [x] Implement `NSEvent` with all types + NSTrackingArea
- [x] Create `RenderContext` for software rendering (softbuffer + fontdue)

### Editor Components (Reality Composer Pro Layout)
- [x] `NSTabView` — Bottom tab panel (4 tabs)
- [x] `NSOutlineView` — Scene hierarchy tree (read-only, ECS-driven)
- [x] `NSCollectionView` — Project browser asset grid
- [x] `NSShaderGraphView` — Node-based shader editor canvas with draggable nodes
- [x] `NSTimelineView` — Animation timeline with playhead, keyframes, tracks
- [x] `NS3DViewport` — Center viewport (2D software grid, bounding box visualization)
- [x] `ecs.rs` — Shared Scene state via `Arc<Mutex<Scene>>`

### Architecture
- [x] Full egui/eframe purge from `rust-kit-pro` and `rust-appkit`
- [x] Retained-mode rendering via NSViewDelegate responder chain
- [x] Event routing: winit → NSApplication → NSWindow → NSView hit-test
- [x] Two-way data binding scaffold (NSTextField scrub → ECS mutation)
- [x] WGSL compiler stub in shader graph (compile_wgsl → fragment source text)
- [x] NSDragging protocol stub in NSCollectionView

### Theme System
- [x] `ThemeRegistry` with 6 predefined themes
- [x] Constant-based key system (`keys` module)
- [x] Legacy `ThemeManager` backward compatibility

### Documentation
- [x] `flow.md` — Architecture documentation
- [x] `task.md` — This task list
- [x] `scenekit_gap_analysis.md` — Full SceneKit header parity matrix
- [x] Inline documentation on all modules

### Testing
- [x] Unit tests for themes, colors, windows, buttons, stack views
- [x] Demo `main.rs` with full editor layout
- [x] Zero compiler errors, warnings suppressed via `#![allow(...)]`

---

## Blocking Tasks 🔴 (Must fix before program is usable)

### B1: Keyboard Input
- [ ] Handle `winit::WindowEvent::KeyboardInput` in `application.rs`
- [ ] Route character events to focused `NSTextField`
- [ ] Implement caret cursor rendering in text field
- [ ] Handle backspace, enter, arrow keys
- [ ] Basic text selection (stretch goal)

### B2: Hierarchy Selection
- [ ] Add `mouse_down` hit-testing to `NSOutlineView` rows
- [ ] Calculate row index from mouse Y position
- [ ] Update `scene.selected_entity` on click
- [ ] Visual highlight for selected row
- [ ] Trigger Inspector refresh when selection changes

### B3: Menu Bar
- [ ] Create `menu.rs` — `NSMenu`, `NSMenuItem` structs
- [ ] Render horizontal menu bar at top of window
- [ ] File → New / Open / Save (via `rfd::FileDialog`)
- [ ] Wire to `Project::load()` and `Project::save()`
- [ ] Edit → Undo / Redo (command pattern stack)

---

## Phase 3: SceneKit Math Foundation 📐

- [ ] Create `scenekit_types.rs` module
- [ ] `SCNVector3` struct (x, y, z) with math ops
- [ ] `SCNVector4` struct (x, y, z, w)
- [ ] `SCNQuaternion` (alias to SCNVector4)
- [ ] `SCNMatrix4` (4x4 matrix, bridged from CATransform3D)
- [ ] `SCNMatrix4MakeTranslation`, `MakeRotation`, `MakeScale`
- [ ] `SCNMatrix4Mult`, `SCNMatrix4Invert`
- [ ] `SCNActionTimingMode` enum (Linear, EaseIn, EaseOut, EaseInEaseOut)
- [ ] `SCNBoundingVolume` trait
  - [ ] `get_bounding_box_min_max() → (SCNVector3, SCNVector3)`
  - [ ] `set_bounding_box_min_max()`
  - [ ] `get_bounding_sphere_center_radius() → (SCNVector3, f32)`
- [ ] Bridge all types to `glam` (Vec3, Vec4, Mat4, Quat)

---

## Phase 4: Animation Runtime 🎬

- [ ] `SCNAnimatable` trait
  - [ ] `add_animation(animation, key)`
  - [ ] `add_animation_player(player, key)`
  - [ ] `remove_all_animations()`
  - [ ] `remove_animation_for_key(key)`
  - [ ] `animation_keys() → Vec<String>`
  - [ ] `animation_player_for_key(key)`
- [ ] `SCNAnimation` struct
  - [ ] `duration: f64`
  - [ ] `key_path: Option<String>`
  - [ ] `timing_function: SCNTimingFunction`
  - [ ] `blend_in_duration / blend_out_duration`
  - [ ] `removed_on_completion / applied_on_completion`
  - [ ] `repeat_count / autoreverses`
  - [ ] `start_delay / time_offset`
  - [ ] `fills_forward / fills_backward`
  - [ ] `uses_scene_time_base`
  - [ ] `animation_did_start / animation_did_stop` callbacks
  - [ ] `animation_events: Vec<SCNAnimationEvent>`
  - [ ] `additive / cumulative`
- [ ] `SCNAnimationPlayer` struct
  - [ ] `play()`, `stop()`, `stop_with_blend_out_duration()`
  - [ ] `speed`, `blend_factor`, `paused`
- [ ] `SCNTimingFunction` (easing curves)
- [ ] `SCNAnimationEvent` (trigger at key time 0.0–1.0)
- [ ] Wire timeline UI keyframes → actual property evaluation

---

## Phase 5: Real 3D Viewport (WGPU) 🖥️

- [x] Add `wgpu` dependency to `Cargo.toml`
- [x] Mount `wgpu::Surface` inside `NS3DViewport` view
- [x] Create render pipeline (vertex + fragment shader)
- [x] Camera struct (position, target, up, fov, aspect)
- [x] Arcball orbit controller (mouse drag → camera rotation)
- [x] Middle mouse → pan
- [x] Scroll wheel → zoom
- [x] Camera focus methods (focus_on, frame_bounding_box, reset)
- [x] Render 3D grid on GPU
- [x] Load meshes from PHX/GLTF → GPU vertex buffers
- [x] Per-entity transform from ECS → model matrix uniform
- [x] Material system integration with viewport
- [ ] Bounding box rendering from `SCNBoundingVolume`
- [ ] Selection highlighting (outline shader on selected entity)

---

## Phase 6: Shader Pipeline 🔧

- [x] `SCNProgram` struct
  - [x] `vertex_shader: Option<String>`
  - [x] `fragment_shader: Option<String>`
  - [x] `vertex_function_name` / `fragment_function_name`
  - [x] `is_opaque: bool`
  - [x] WGSL compile with modifier injection
- [x] `SCNShadable` trait
  - [x] `shader_modifiers: HashMap<EntryPoint, String>`
  - [x] `handle_binding_of_symbol()` / `handle_unbinding()`
- [x] 4 shader modifier entry points
  - [x] `SCNShaderModifierEntryPointGeometry` (vertex deformation)
  - [x] `SCNShaderModifierEntryPointSurface` (material properties)
  - [x] `SCNShaderModifierEntryPointLightingModel` (custom lighting)
  - [x] `SCNShaderModifierEntryPointFragment` (final color)
- [x] Built-in uniforms (u_time, u_modelTransform, u_projectionTransform, etc.)
- [x] `SCNBufferFrequency` (PerFrame, PerNode, PerShadable)
- [x] `SCNMaterial` struct implementing `SCNShadable`
- [x] Shader compiler with unlit, diffuse, and PBR modes
- [x] WGSL templates (BASIC_VERTEX, BASIC_FRAGMENT, PBR_FRAGMENT)

---

## Platform & Polish

### Phase 7: Platform Features
- [ ] High DPI display support
- [ ] Multiple monitor support
- [ ] System tray integration
- [ ] File drag from explorer

### Phase 8: Additional Controls
- [ ] `NSImageView` — Display images
- [ ] `NSProgressIndicator` — Progress bars
- [ ] `NSSegmentedControl` — Tab-like buttons
- [ ] `NSTableView` — Table/grid view
- [ ] `NSComboBox` — Dropdown selection

### Phase 9: Performance
- [ ] Dirty rect tracking (only redraw changed regions)
- [ ] Batch draw calls in RenderContext
- [ ] Text rendering cache (glyph atlas)
- [ ] GPU-accelerated 2D compositing

---

## Milestones

### v0.1.0 (Done)
- Core framework, themes, basic controls, software rendering

### v0.2.0 (Current)
- Full editor layout, ECS binding, egui purge, SceneKit gap analysis

### v0.3.0 (Next)
- Keyboard input, hierarchy selection, menu bar → **first usable version**

### v0.4.0
- SceneKit math types, animation runtime → **data parity with Apple**

### v0.5.0
- WGPU viewport, real 3D rendering → **visual parity with Apple**

### v1.0.0
- Full shader pipeline, production stability, documentation
