# Layout Space - Context (Hierarchical BContext)

> **Blender references:**
> - `BKE_context.h` - `bContext` struct, `CTX_wm_area()`, `CTX_data_scene()`
> - `DNA_windowmanager_types.h` - `wmWindowManager`, `wmWindow`
> - Blender Manual: [`interface/window_system/introduction.rst`](../../blender-manual/manual/interface/window_system/introduction.rst)

---

## What is BContext?

`BContext` is the **hierarchical active-focus path** through the UI tree.
It mirrors Blender's `bContext` - but instead of a flat bag of fields,
it's a nested structure that follows the same hierarchy as the UI itself:

```
Space → Zone → BArea → Host → Region → Pane → Part
```

Each level carries only the data that belongs to that level.
No host-level data leaks into the space level. No space-level data
leaks into the part level.

> Blender's `bContext` provides hierarchical accessors:
> - `CTX_wm_window(C)` - active window
> - `CTX_wm_screen(C)` - active screen (workspace/space)
> - `CTX_wm_area(C)` - active area
> - `CTX_wm_region(C)` - active region
> - `CTX_data_scene(C)` - active scene (space-level)
> - `CTX_data_mode(C)` - current mode (host-level)
> - `CTX_data_active_object(C)` - active object (host-level)
>
> Each accessor drills down one level. Our context mirrors this.

---

## Hierarchical Context Structure

```rust
//! Hierarchical context - mirrors the UI tree.
//! Each level carries only its own data.
//! Ref: BKE_context.h - bContext

use buikit::{BHostType, BSpaceType, BZoneType, BRegionType, BPaneType, BPartType};

// ── Level 1: Space Context ──
// Data that belongs to the workspace/space level.
// Blender: CTX_wm_screen(C), CTX_data_scene(C), CTX_data_view_layer(C)

/// The current interaction mode - mirrors Blender's `bpy.context.mode`.
/// This is host-level data, not space-level.
/// Ref: Blender Manual - editors/3dview/modes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BMode {
    Object,
    Edit,
    Sculpt,
    UvEdit,
    TexturePaint,
    WeightPaint,
    VertexPaint,
    Pose,
    Particle,
    Paint,
    Mask,
    View,
    Default,
}

impl BMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Object       => "Object Mode",
            Self::Edit         => "Edit Mode",
            Self::Sculpt       => "Sculpt Mode",
            Self::UvEdit       => "UV Editing",
            Self::TexturePaint => "Texture Paint",
            Self::WeightPaint  => "Weight Paint",
            Self::VertexPaint  => "Vertex Paint",
            Self::Pose         => "Pose Mode",
            Self::Particle     => "Particle Mode",
            Self::Paint        => "Paint",
            Self::Mask         => "Mask",
            Self::View         => "View",
            Self::Default      => "Default",
        }
    }

    /// Default mode for a given host type.
    pub fn for_host(host: BHostType) -> Self {
        match host {
            BHostType::View3D              => Self::Object,
            BHostType::ImageEditor | BHostType::UvEditor => Self::View,
            BHostType::TextEditor          => Self::Edit,
            BHostType::Timeline            => Self::View,
            _ => Self::Default,
        }
    }
}

/// Space-level context data.
/// Only data that is scoped to the entire workspace/space.
/// Blender: scene, view_layer, workspace settings.
#[derive(Clone, Debug)]
pub struct SpaceCtx {
    /// Which workspace tab is active.
    /// Blender: `context.workspace` / `CTX_wm_screen(C)`
    pub space_type: BSpaceType,

    /// Active scene name.
    /// Blender: `context.scene` / `CTX_data_scene(C)`
    pub scene: String,

    /// Active view layer name.
    /// Blender: `context.view_layer` / `CTX_data_view_layer(C)`
    pub view_layer: String,

    /// Which zone within this space has focus.
    /// This is the path to the next level.
    pub active_zone: BZoneType,
}

// ── Level 2: Zone Context ──
// A zone is a grid position. It tracks which area is active at that position.

/// Zone-level context data.
#[derive(Clone, Debug)]
pub struct ZoneCtx {
    /// Position in the space grid.
    pub zone: BZoneType,

    /// Which area at this zone has focus.
    pub active_area: usize,  // index into areas at this zone
}

// ── Level 3: BArea Context ──
// An area is a slot at a zone position. It holds the active host.
// Blender: CTX_wm_area(C) - area is mainly about position + which editor

/// BArea-level context data.
#[derive(Clone, Debug)]
pub struct AreaCtx {
    /// Which host (editor) is shown in this area.
    /// Blender: `area.spacetype`
    pub host: BHostType,

    /// Which region within this host has focus.
    pub active_region: BRegionType,  // Header, Center, or Footer
}

// ── Level 4: Host Context ──
// Data that belongs to a specific editor instance.
// Blender: CTX_data_mode(C), context.active_object, context.tool

/// Host-level context data.
/// This is where most interaction state lives - mode, selection, tool.
/// Different host types use different subsets of these fields.
#[derive(Clone, Debug)]
pub struct HostCtx {
    /// Which host type this is.
    pub host_type: BHostType,

    /// Current interaction mode.
    /// Blender: `context.mode` / `CTX_data_mode(C)`
    pub mode: BMode,

    /// Currently selected object name.
    /// Blender: `context.active_object`
    pub active_object: Option<String>,

    /// Names of all selected objects.
    /// Blender: `context.selected_objects`
    pub selected_objects: Vec<String>,

    /// Active tool name.
    /// Blender: `context.tool` / `CTX_data_tool(C)`
    pub active_tool: Option<String>,

    /// Current animation frame (only relevant for animation editors).
    /// Blender: `context.scene.frame_current`
    pub frame_current: i32,

    /// Whether playback is active (only relevant for animation editors).
    pub playing: bool,
}

// ── Level 5: Region Context ──
// A region is Header, Center, or Footer. It points to the active pane.
// Blender: CTX_wm_region(C)

/// Region-level context data.
#[derive(Clone, Debug)]
pub struct RegionCtx {
    /// Which region type.
    pub region_type: BRegionType,

    /// Which pane within this region has focus.
    pub active_pane: BPaneType,  // Left, Center, or Right
}

// ── Level 6: Pane Context ──
// A pane is a layout slot. It points to the active part.

/// Pane-level context data.
#[derive(Clone, Debug)]
pub struct PaneCtx {
    /// Which pane type.
    pub pane_type: BPaneType,

    /// Which part within this pane has focus.
    pub active_part: BPartType,
}

// ── Root: BContext ──
// The root context holds the full hierarchical path.
// It's not a flat struct - it's a nested tree you drill into.

/// Hierarchical context - the active-focus path through the UI tree.
/// Each level nests the next. Access data at the level it belongs to.
/// Ref: BKE_context.h - bContext
#[derive(Clone, Debug)]
pub struct BContext {
    // Level 1: Space
    pub space: SpaceCtx,

    // Level 2: Zone (nested under space)
    pub zone: ZoneCtx,

    // Level 3: BArea (nested under zone)
    pub area: AreaCtx,

    // Level 4: Host (nested under area)
    pub host: HostCtx,

    // Level 5: Region (nested under host)
    pub region: RegionCtx,

    // Level 6: Pane (nested under region)
    pub pane: PaneCtx,
}
```

---

## Default Context (Layout Space)

```rust
impl Default for BContext {
    fn default() -> Self {
        Self {
            // Level 1: Layout space, default scene
            space: SpaceCtx {
                space_type: BSpaceType::Layout,
                scene: "Scene".to_string(),
                view_layer: "ViewLayer".to_string(),
                active_zone: BZoneType::Center,
            },

            // Level 2: Center zone
            zone: ZoneCtx {
                zone: BZoneType::Center,
                active_area: 0,
            },

            // Level 3: Center area, View3D host
            area: AreaCtx {
                host: BHostType::View3D,
                active_region: BRegionType::Center,
            },

            // Level 4: View3D host - interaction state
            host: HostCtx {
                host_type: BHostType::View3D,
                mode: BMode::Object,
                active_object: Some("Cube".to_string()),
                selected_objects: vec!["Cube".to_string()],
                active_tool: Some("Select Box".to_string()),
                frame_current: 1,
                playing: false,
            },

            // Level 5: Center region
            region: RegionCtx {
                region_type: BRegionType::Center,
                active_pane: BPaneType::Center,
            },

            // Level 6: Center pane (viewport)
            pane: PaneCtx {
                pane_type: BPaneType::Center,
                active_part: BPartType::Custom,
            },
        }
    }
}
```

---

## Hierarchical Accessors

Each level has accessors that mirror Blender's `CTX_*` macros.
You drill down level by level:

```rust
impl BContext {
    // ── Level 1: Space ──

    /// Active space type.
    /// Blender: CTX_wm_screen(C)
    pub fn space_type(&self) -> BSpaceType { self.space.space_type }

    /// Active scene.
    /// Blender: CTX_data_scene(C)
    pub fn scene(&self) -> &str { &self.space.scene }

    /// Active view layer.
    /// Blender: CTX_data_view_layer(C)
    pub fn view_layer(&self) -> &str { &self.space.view_layer }

    // ── Level 2: Zone ──

    /// Active zone position.
    /// Blender: CTX_wm_area(C) - area position
    pub fn zone_type(&self) -> BZoneType { self.zone.zone }

    // ── Level 3: BArea ──

    /// Active host in the active area.
    pub fn host_type(&self) -> BHostType { self.area.host }

    // ── Level 4: Host ──

    /// Active host context (mode, selection, tool, animation).
    /// Blender: CTX_data_mode(C), context.active_object, etc.
    pub fn host(&self) -> &HostCtx { &self.host }

    pub fn mode(&self) -> BMode { self.host.mode }
    pub fn active_object(&self) -> Option<&str> {
        self.host.active_object.as_deref()
    }
    pub fn active_tool(&self) -> Option<&str> {
        self.host.active_tool.as_deref()
    }

    // ── Level 5: Region ──

    /// Active region.
    /// Blender: CTX_wm_region(C)
    pub fn region_type(&self) -> BRegionType { self.region.region_type }

    // ── Level 6: Pane ──

    /// Active pane.
    pub fn pane_type(&self) -> BPaneType { self.pane.pane_type }

    /// Active part.
    pub fn part_type(&self) -> BPartType { self.pane.active_part }
}
```

---

## Hierarchical Mutators

Updates respect the hierarchy - changing a level invalidates
the levels below it:

```rust
impl BContext {
    // ── Space switching ──
    // Switching space resets area, BHost, region, pane.

    pub fn set_space(&mut self, space: BSpaceType) {
        self.space.space_type = space;
        // Reset to default zone for this space
        self.space.active_zone = BZoneType::Center;
        // Reset zone to center
        self.zone = ZoneCtx {
            zone: BZoneType::Center,
            active_area: 0,
        };
        // Reset host to default for center area in this space
        let default_host = space.default_host(BZoneType::Center);
        self.area.host = default_host;
        // Reset host context
        self.host = HostCtx::for_host(default_host);
        // Reset region/pane
        self.region = RegionCtx::default();
        self.pane = PaneCtx::default();
    }

    // ── Zone focus ──
    // Focusing a zone updates the area, BHost, region, pane below it.

    pub fn focus_zone(&mut self, zone: BZoneType) {
        self.space.active_zone = zone;
        self.zone = ZoneCtx {
            zone,
            active_area: 0,
        };
        // Reset area to default host for this zone
        let default_host = self.space.space_type.default_host(zone);
        self.area = AreaCtx {
            host: default_host,
            active_region: BRegionType::Center,
        };
        // Reset host context
        self.host = HostCtx::for_host(default_host);
        // Reset region/pane
        self.region = RegionCtx::default();
        self.pane = PaneCtx::default();
    }

    // ── BArea focus (within a zone) ──
    // Focusing an area updates the host, region, pane below it.

    pub fn focus_area(&mut self, BHost: BHostType) {
        self.area = AreaCtx {
            host,
            active_region: BRegionType::Center,
        };
        // Reset host context for the new host
        self.host = HostCtx::for_host(host);
        // Reset region/pane
        self.region = RegionCtx::default();
        self.pane = PaneCtx::default();
    }

    // ── Host switching (within an area) ──
    // Switching host resets region, pane, and host-level data.

    pub fn switch_host(&mut self, BHost: BHostType) {
        self.area.host = host;
        self.host = HostCtx::for_host(host);
        // Reset region/pane
        self.region = RegionCtx::default();
        self.pane = PaneCtx::default();
    }

    // ── Region focus ──
    // Focusing a region resets the pane below it.

    pub fn focus_region(&mut self, region: BRegionType) {
        self.area.active_region = region;
        self.region = RegionCtx {
            region_type: region,
            active_pane: BPaneType::Center,
        };
        self.pane = PaneCtx::default();
    }

    // ── Pane focus ──

    pub fn focus_pane(&mut self, pane: BPaneType) {
        self.region.active_pane = pane;
        self.pane.pane_type = pane;
    }

    // ── Host-level mutations ──
    // These only affect the host context, not the hierarchy above it.

    pub fn set_mode(&mut self, mode: BMode) {
        self.host.mode = mode;
    }

    pub fn select_object(&mut self, name: &str) {
        self.host.active_object = Some(name.to_string());
        if !self.host.selected_objects.iter().any(|o| o == name) {
            self.host.selected_objects.push(name.to_string());
        }
    }

    pub fn deselect_all(&mut self) {
        self.host.selected_objects.clear();
        self.host.active_object = None;
    }

    pub fn set_tool(&mut self, tool: &str) {
        self.host.active_tool = Some(tool.to_string());
    }

    pub fn set_frame(&mut self, frame: i32) {
        self.host.frame_current = frame;
    }

    pub fn toggle_playback(&mut self) {
        self.host.playing = !self.host.playing;
    }
}
```

---

## HostCtx Construction

```rust
impl HostCtx {
    /// Create host context with defaults for a given host type.
    pub fn for_host(host: BHostType) -> Self {
        Self {
            host_type: host,
            mode: BMode::for_host(host),
            active_object: if matches!(host, BHostType::View3D) {
                Some("Cube".to_string())
            } else {
                None
            },
            selected_objects: if matches!(host, BHostType::View3D) {
                vec!["Cube".to_string()]
            } else {
                vec![]
            },
            active_tool: None,
            frame_current: 1,
            playing: false,
        }
    }
}

impl Default for RegionCtx {
    fn default() -> Self {
        Self {
            region_type: BRegionType::Center,
            active_pane: BPaneType::Center,
        }
    }
}

impl Default for PaneCtx {
    fn default() -> Self {
        Self {
            pane_type: BPaneType::Center,
            active_part: BPartType::Custom,
        }
    }
}
```

---

## Context Access in Views

The hierarchical context is accessed during rendering at the appropriate level.
Each view level reads only the context level it needs:

```rust
// In BSpaceGrid::render() - reads space + zone level:
fn render(&self, ctx: &BContext, cx: &App) -> AnyElement {
    let active_zone = ctx.zone_type();
    // Walk zones, highlight the one matching active_zone
    ...

// In BHostView::render() - reads area + host level:
fn render(&self, ctx: &BContext, cx: &App) -> AnyElement {
    let is_active = ctx.host_type() == self.effect.host_type();
    let mode = ctx.mode();  // host-level data

    let mut col = vflex().size_full();

    if is_active {
        col = col.border_color(theme.accent);
    }

    // Pass relevant context level to region renderer
    for region in self.effect.regions() {
        let is_active_region = is_active && ctx.region_type() == region;
        if let Some(element) = self.render_region(region, ctx, cx) {
            col = col.child(element);
        }
    }

    col.into_any_element()
}

// In BAreaModebar::to_bview() - reads host level for mode:
fn to_bview(&self, ctx: &BContext) -> BView {
    let current_mode = ctx.mode();
    // Highlight the tab matching current_mode
    ...
}

// In BAreaChannels::to_bview() - reads host level for frame:
fn to_bview(&self, ctx: &BContext) -> BView {
    let frame = ctx.host().frame_current;
    // Show channel data for current frame
    ...
}
```

---

## Context vs State vs Store

```
┌──────────────────────────────────────────────────────────────────────┐
│                        LAYER COMPARISON                               │
├──────────┬────────────────────┬───────────────────────────────────────┤
│  STORE   │  STATE             │  CONTEXT                              │
│ (static) │ (per-instance)     │ (hierarchical active path)            │
├──────────┼────────────────────┼───────────────────────────────────────┤
│ What CAN │ What IS            │ What's FOCUSED                        │
│ exist    │ happening now      │ (drilled down level by level)         │
├──────────┼────────────────────┼───────────────────────────────────────┤
│SpaceSpec │ SpaceState         │ ctx.space                             │
│  .areas  │  .active_hosts{}   │  .space_type = Layout                 │
│  .grid   │  AreaState[]       │  .scene = "Scene"                    │
│          │  .focused          │  .view_layer = "ViewLayer"            │
│          │                    │  .active_zone = Center                │
├──────────┼────────────────────┼───────────────────────────────────────┤
│BArea     │ ZoneState          │ ctx.zone                              │
│  .zone   │  .visible          │  .zone = Center                       │
│  .host   │  .active_area      │  .active_area = 0                     │
│  .frac   │                    │                                       │
├──────────┼────────────────────┼───────────────────────────────────────┤
│HostSpec  │ HostState          │ ctx.area                              │
│  .regions│  .zoom             │  .host = View3D                       │
│  .panes  │  .mode             │  .active_region = Center              │
│  .parts  │  .rails            │                                       │
│          │  .shading          │                                       │
├──────────┼────────────────────┼───────────────────────────────────────┤
│PaneSpec  │ PaneState          │ ctx.host                              │
│  .type   │  .visible          │  .host_type = View3D                  │
│  .fixed  │  .width            │  .mode = Object                       │
│  .width  │  .float_type       │  .active_object = "Cube"             │
│          │  .active_part      │  .selected_objects = ["Cube"]        │
│          │                    │  .active_tool = "Select Box"          │
│          │                    │  .frame_current = 1                   │
│          │                    │  .playing = false                     │
├──────────┼────────────────────┼───────────────────────────────────────┤
│PartSpec  │ PartState          │ ctx.region                            │
│  .type   │  Modebar{open,     │  .region_type = Center                │
│  .items  │   mode}            │  .active_pane = Center                │
│          │  Menubar{dropdown} │                                       │
│          │  Toolbar{tool}     │ ctx.pane                              │
│          │  Sidebar{tab}      │  .pane_type = Center                  │
│          │  Panel{expanded}   │  .active_part = Custom                │
└──────────┴────────────────────┴───────────────────────────────────────┘
```

**Key distinction:**
- **Store** = "what structure exists" (static, never changes at runtime)
- **State** = "what's happening in each instance" (per-host zoom, per-region visibility, per-pane width, per-part open/closed, etc.)
- **Context** = "what's focused right now" - a **hierarchical path** through the tree,
  with data scoped to each level:
  - `ctx.space` - scene, view_layer, active_zone (space-level)
  - `ctx.zone` - zone, active_area (zone-level)
  - `ctx.area` - host, active_region (area-level)
  - `ctx.host` - mode, selection, tool, frame, playing (host-level)
  - `ctx.region` - region_type, active_pane (region-level)
  - `ctx.pane` - pane_type, active_part (pane-level)

The render walk reads from all three:
1. Store specs tell it WHAT to render (which regions, panes, parts)
2. State tells it HOW to render (is this region visible? what's the zoom?)
3. Context tells it what to HIGHLIGHT and what data to show (which mode? which object?)
   - accessed at the level it belongs to, not flattened into one struct
