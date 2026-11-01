# Panels Architecture — Rust Design Decisions

## Overview

Each panel in `panels/` follows a Model + ViewController split.
The model holds data, the view controller builds and manages the NSView tree.
This document explains every pattern, shows memory diagrams, and lists alternatives.

---

## Pattern: Model/ViewController Split

```rust
// MODEL — pure data, no UI references.
// Derives Clone so we can snapshot for undo.
// Derives Debug so we can log state.
#[derive(Debug, Clone)]
pub struct NSViewportState {
    pub shading_mode: NSShadingMode,  // enum: Wireframe | Solid | Material | Rendered
    pub show_grid: bool,              // toggle for grid overlay
    pub show_overlays: bool,          // toggle for gizmo/label overlays
    pub selected_entity: Arc<Mutex<Option<u64>>>,  // shared selection
}

// VIEW CONTROLLER — owns the view tree + model.
// Not Clone (views are heavyweight, contain GPU resources).
pub struct NSViewportViewController {
    pub view: NSView,                 // the root view (owned)
    pub viewport: NS3DViewportGPU,    // GPU viewport (owns wgpu resources)
    pub state: NSViewportState,       // the model (owned)
}
```

**Memory diagram — model vs view controller:**

```
Stack (NSViewportViewController)
┌──────────────────────────────────────────────────────────────┐
│ view: NSView                                                 │
│   ├── id: u64                                                │
│   ├── frame: NSRect { x, y, w, h }                          │
│   ├── subviews: Vec<NSView> ──────► [Heap: child views]      │
│   └── delegate: Option<Arc<Mutex<dyn NSViewDelegate>>>       │
│                                                              │
│ viewport: NS3DViewportGPU                                    │
│   ├── camera: Arc<Mutex<ViewportCameraController>> ──────┐   │
│   ├── scene: Arc<Mutex<Scene>> ──────────────────────┐   │   │
│   └── view: NSView (GPU render target)               │   │   │
│                                                      │   │   │
│ state: NSViewportState                               │   │   │
│   ├── shading_mode: NSShadingMode (1 byte)           │   │   │
│   ├── show_grid: bool (1 byte)                       │   │   │
│   └── selected_entity: Arc<Mutex<Option<u64>>> ──┐   │   │   │
└──────────────────────────────────────────────────┼───┼───┼───┘
                                                   │   │   │
                    HEAP                           │   │   │
┌──────────────────────────────────────────────────┼───┼───┼───┐
│                                                  ▼   │   │   │
│  ┌─────────────────────────────────────────────────┐ │   │   │
│  │ ArcInner<Mutex<Option<u64>>>                    │ │   │   │
│  │ strong_count: 3 (editor + hierarchy + viewport) │ │   │   │
│  │ value: Some(42)  ← entity ID 42 is selected    │ │   │   │
│  └─────────────────────────────────────────────────┘ │   │   │
│                                                      ▼   │   │
│  ┌─────────────────────────────────────────────────────┐ │   │
│  │ ArcInner<Mutex<Scene>>                              │ │   │
│  │ strong_count: 4 (editor + viewport + hierarchy + …) │ │   │
│  │ value: Scene { world: World, root_entities: Vec }   │ │   │
│  └─────────────────────────────────────────────────────┘ │   │
│                                                          ▼   │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ ArcInner<Mutex<ViewportCameraController>>               │ │
│  │ strong_count: 2 (viewport + input handler)              │ │
│  │ value: ViewportCameraController {                       │ │
│  │   position: (2.0, 1.5, 2.0)                            │ │
│  │   target: (0.0, 0.7, 0.7)                              │ │
│  │   distance: 3.0, yaw: -45°, pitch: -30°                │ │
│  │ }                                                       │ │
│  └─────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

**Why this split:**
- Model can be cloned for undo snapshots without cloning GPU resources
- Model can be serialized to disk (project save)
- View can be rebuilt (theme change) without losing state
- Testing: model logic testable without a render context

**Alternative: Single struct**
```rust
// Alternative — rejected
// Coupling data to view means rebuilding loses state
struct ViewportPanel {
    view: NSView,
    shading_mode: NSShadingMode,  // lost if view is rebuilt
}
```

---

## Pattern: `EditorPanel` Trait (Dynamic Dispatch)

```rust
/// Trait for all editor panels. Object-safe for dynamic dispatch.
/// `Send + Sync` required because panels live in Arc<Mutex<EditorApp>>.
pub trait EditorPanel: Send + Sync {
    /// Human-readable panel name for tab labels.
    fn name(&self) -> &str;

    /// Construct the view tree for this panel at the given rect.
    fn build_view(&self, rect: NSRect) -> NSView;

    /// Refresh internal data (e.g., after scene change).
    fn reload_data(&mut self);

    /// Called when the user selects a different entity.
    fn on_selection_changed(&mut self, entity: Option<u64>);

    /// Downcast to concrete type. Required because trait objects
    /// erase the concrete type — this lets handlers access specific fields.
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
```

**Memory diagram — HashMap of trait objects:**

```
EditorApp.panels: HashMap<PanelKey, Box<dyn EditorPanel>>
┌─────────────────────────────────────────────────────────────────┐
│ HashMap internals (hash table with open addressing)             │
│                                                                 │
│ bucket[0]: PanelKey::Viewport → Box ──► ┌────────────────────┐ │
│                                         │ ViewportPanel      │ │
│                                         │ (concrete type)    │ │
│                                         │ vtable ──► EditorPanel vtable
│                                         └────────────────────┘ │
│                                                                 │
│ bucket[1]: PanelKey::Inspector → Box ──► ┌────────────────────┐│
│                                          │ InspectorPanel     ││
│                                          │ (concrete type)    ││
│                                          └────────────────────┘│
│                                                                 │
│ bucket[2]: PanelKey::Hierarchy → (stored on EditorApp directly) │
└─────────────────────────────────────────────────────────────────┘

Fat pointer layout of Box<dyn EditorPanel>:
┌──────────────────────────────────────┐
│ data_ptr: *mut ViewportPanel         │  8 bytes (points to heap data)
│ vtable_ptr: *const EditorPanelVtable │  8 bytes (points to static vtable)
└──────────────────────────────────────┘
Total: 16 bytes per entry (the "fat pointer")
```

**Why trait objects:**
- Open set: new panel types don't require modifying existing code
- Broadcast: iterate all panels with `for panel in panels.values_mut()`
- Downcast: `as_any_mut()` lets handlers access concrete fields when needed

**Alternative A: Enum of panels**
```rust
// Alternative — considered, rejected for extensibility
enum Panel {
    Viewport(ViewportPanel),
    Inspector(InspectorPanel),
    Timeline(TimelinePanel),
}
// Pro: No heap allocation per panel, no vtable indirection
// Con: Adding a panel = editing enum + every match site
// Con: Can't support plugin panels (closed set)
```

**Alternative B: Generics (monomorphization)**
```rust
// Alternative — rejected for compile time reasons
struct Editor<V: PanelLike, I: PanelLike, T: PanelLike> {
    viewport: V,
    inspector: I,
    timeline: T,
}
// Pro: Zero-cost dispatch (static), no heap allocation
// Con: Type explosion — every combination is a new type
// Con: Can't store in a homogeneous collection
// Con: Compile times scale with panel count
```

---

## Pattern: `Arc<Mutex<Camera>>` — Shared Mutable Resource

```rust
pub struct NS3DViewportGPU {
    // Camera is shared between:
    //   1. The render loop (reads view/projection matrices)
    //   2. The input handler (writes yaw/pitch/distance on drag)
    //   3. The focus command (writes target position)
    //
    // Arc: multiple owners (render + input + commands)
    // Mutex: exclusive access (only one writer at a time)
    pub camera: Arc<Mutex<ViewportCameraController>>,
}
```

**Memory diagram — camera access pattern:**

```
                    ┌─────────────────────────────────────┐
                    │  Heap: ViewportCameraController     │
                    │  ┌───────────────────────────────┐  │
                    │  │ position: (2.0, 1.5, 2.0)    │  │
                    │  │ target: (0.0, 0.7, 0.7)      │  │
                    │  │ yaw: -0.785 rad              │  │
                    │  │ pitch: -0.524 rad            │  │
                    │  │ distance: 3.0                │  │
                    │  │ fov_y: 0.785 rad (45°)       │  │
                    │  └───────────────────────────────┘  │
                    └────────▲──────────▲─────────────────┘
                             │          │
              ┌──────────────┘          └──────────────┐
              │ .lock().unwrap()                        │ .lock().unwrap()
              │ (blocks if other holds lock)            │ (blocks if other holds lock)
              │                                        │
┌─────────────┴──────────────┐          ┌──────────────┴─────────────┐
│ INPUT HANDLER (event loop) │          │ RENDER LOOP (frame draw)   │
│                            │          │                            │
│ // User drags mouse:       │          │ // Every frame:            │
│ cam.handle_drag(dx, dy, s) │          │ let vp = cam.view_proj()   │
│ cam.handle_zoom(scroll, s) │          │ upload_to_gpu(vp)          │
│ cam.handle_pan(dx, dy, s)  │          │                            │
└────────────────────────────┘          └────────────────────────────┘

Lock contention: ~0 in practice because:
- Input events arrive at 60-120Hz
- Render reads happen at 60Hz
- Each lock is held for <1μs (read 6 floats, compute matrix)
- Probability of collision: negligible
```

**Why `Arc<Mutex<_>>` and not alternatives:**

| Alternative | Pro | Con | Verdict |
|-------------|-----|-----|---------|
| `Rc<RefCell<_>>` | No atomic ops | `!Send` — can't use with wgpu threads | Rejected |
| `RwLock` | Multiple readers | Write-starved if render reads constantly | Overkill |
| Channel (send camera state) | No shared state | Extra copy per frame, latency | Rejected |
| `UnsafeCell` + manual sync | Maximum perf | Undefined behavior if misused | Rejected |
| `AtomicF32` per field | Lock-free | 6 separate atomics, no consistent snapshot | Rejected |

---

## Pattern: Workspace Layout Presets (Data-Driven)

```rust
/// A panel's position and size constraints.
/// This is DATA, not code. Can be serialized, interpolated, compared.
#[derive(Debug, Clone)]
pub struct PanelSlot {
    pub id: &'static str,       // "hierarchy", "viewport", etc.
    pub dock: DockPosition,     // Left | Right | Bottom | Center | Floating | Hidden
    pub order: u8,              // stacking order within a dock region
    pub visible: bool,          // whether panel is shown
    pub min_size: f32,          // minimum dimension (width or height)
    pub preferred_size: f32,    // default dimension
    pub max_size: f32,          // maximum dimension
    pub collapsible: bool,      // can user collapse this panel?
}

/// The full workspace arrangement.
/// Switching layouts = swapping this struct. No code changes.
#[derive(Debug, Clone)]
pub struct WorkspacePreset {
    pub name: &'static str,
    pub slots: Vec<PanelSlot>,
    pub activity_bar_position: DockPosition,
    pub bottom_panel_visible: bool,
    pub toolbar_visible: bool,
}
```

**Memory diagram — preset switching:**

```
BEFORE: active_preset = "default"
┌─────────────────────────────────────────────────────────────┐
│ WorkspacePreset { name: "Default" }                         │
│ slots: [                                                    │
│   PanelSlot { id: "hierarchy", dock: Left, size: 320 }     │
│   PanelSlot { id: "viewport",  dock: Center, size: 800 }   │
│   PanelSlot { id: "inspector", dock: Right, size: 320 }    │
│   PanelSlot { id: "timeline",  dock: Bottom, size: 200 }   │
│ ]                                                           │
└─────────────────────────────────────────────────────────────┘

User selects "Kero Mode" (zen mode):
  set_layout("kero")  ← writes "kero" to global Mutex<String>

AFTER: active_preset = "kero"
┌─────────────────────────────────────────────────────────────┐
│ WorkspacePreset { name: "Kero" }                            │
│ slots: [                                                    │
│   PanelSlot { id: "viewport", dock: Center, size: MAX }    │
│ ]                                                           │
│ activity_bar_position: Hidden                               │
│ toolbar_visible: false                                      │
└─────────────────────────────────────────────────────────────┘

The layout engine reads the active preset and positions panels accordingly.
No if/else chains. No special-case code paths. Just data.
```

**Alternative: Hardcoded layout functions**
```rust
// Alternative — rejected
fn build_default_layout(root: &mut NSView, w: f32, h: f32) {
    // 200 lines of manual positioning...
}
fn build_kero_layout(root: &mut NSView, w: f32, h: f32) {
    // Another 50 lines...
}
// Every new layout = new function. No interpolation. No serialization.
```

---

## Pattern: View Controller Owns Its View

```rust
pub struct NSActivityBarViewController {
    pub view: NSView,       // OWNED — not borrowed, not referenced
    pub state: NSActivityBarState,
}
```

**Why owned and not borrowed:**

```
OWNED (what we do):                    BORROWED (what we avoid):
┌────────────────────────┐            ┌────────────────────────┐
│ ActivityBarVC          │            │ ActivityBarVC<'a>      │
│ ├── view: NSView      │            │ ├── view: &'a mut NSView│
│ └── state: State      │            │ └── state: State       │
└────────────────────────┘            └────────────────────────┘
                                                │
Lifetime: 'static                     Lifetime: 'a (tied to parent)
Can store in HashMap: YES             Can store in HashMap: NO
Can move between containers: YES      Can move: only within 'a scope
Can outlive creator: YES              Must die before parent: YES
```

**Alternative: Borrowed view**
```rust
// Alternative — rejected (lifetime hell)
struct ActivityBar<'a> {
    view: &'a mut NSView,  // borrows from parent
    state: State,
}
// Now the parent can't be mutated while ActivityBar exists.
// And every container needs the lifetime parameter:
//   struct Editor<'a> { bars: Vec<ActivityBar<'a>> }
//   struct App<'a> { editor: Editor<'a> }
// Lifetime 'a infects the entire program.
```

---

## Panel Inventory

| Panel | Model Type | VC Type | Shared Resources | Key Pattern |
|-------|-----------|---------|-----------------|-------------|
| Viewport | `NSViewportState` | `NSViewportViewController` | Scene, Camera | `Arc<Mutex<_>>` for GPU sharing |
| Hierarchy | `HierarchyEntity` | `NSHierarchyViewController` | Scene, Selection | `Arc<Mutex<_>>` for sync |
| Inspector | `NSInspectorState` | `NSInspectorViewController` | None | State-driven rebuild |
| Timeline | `NSTimelineState` | `NSTimelineViewController` | None | Standalone, reads on demand |
| Asset Library | `NSAssetLibraryState` | `NSAssetLibraryViewController` | None | Static catalog data |
| Statistics | `NSStatisticsData` | `NSStatisticsViewController` | None | Computed from scene snapshot |
| Tools Sidebar | `NSToolsSidebarState` | `NSToolsSidebarViewController` | None | Enum-driven tool list |
| Activity Bar | `NSActivityBarState` | `NSActivityBarViewController` | None | Group-based layout |
| Project Browser | `NSProjectBrowserState` | `NSProjectBrowserViewController` | None | 3-pane split, folder nav |
| Top Bar | `NSTopBarState` | `NSTopBarViewController` | None | Toolbar controls |
| Material Inspector | — | `NSMaterialInspectorViewController` | None | Property grid |
| Node Editor | — | `NSNodeEditorViewController` | None | Graph canvas |
