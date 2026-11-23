# Workspace Layout Presets

Complete documentation of the 6 default workspace layout presets.

## Overview

Workspace presets define complete panel arrangements with positions, visibility, and sizing. Each preset is optimized for specific workflows, similar to IDE layout presets (Debug, Edit, Design, etc.).

All presets follow the **8pt grid system** and use the **unified layout constants** from `layout::constants` and `layout::spacing`.

---

## 1. Default Layout

**Purpose**: Balanced general-purpose arrangement for typical workflows.

**Configuration**:
- **Hierarchy Panel**: Left dock, 260pt preferred width (200-500pt range)
- **Inspector Panel**: Right dock, 300pt preferred width (260-500pt range)
- **Timeline Panel**: Bottom dock, 200pt preferred height (140-450pt range)
- **Tools Sidebar**: Left dock (order 1), 72pt preferred width (60-100pt range)
- **Activity Bar**: Left position
- **Toolbar**: Visible
- **Bottom Panel**: Visible

**Use Cases**:
- General scene editing
- Asset management
- Animation timeline work
- Standard 3D/2D workflows

**Visual Layout**:
```
┌────┬──────────────────────────────────────────┬────────┐
│Act │ Hierarchy                                │Inspec  │
│Bar │                                          │tor     │
├────┤              Viewport                    │        │
│Tool│                                          │        │
│s   │                                          │        │
└────┴──────────────────────────────────────────┴────────┘
     └──────────────────────────────────────────┘
                    Timeline
```

---

## 2. Classic Layout

**Purpose**: Traditional macOS IDE arrangement with familiar panel positions.

**Configuration**:
- **Hierarchy Panel**: Left dock, 260pt preferred width (200-500pt range)
- **Inspector Panel**: Right dock, 300pt preferred width (260-500pt range)
- **Timeline Panel**: Bottom dock, 180pt preferred height (120-400pt range)
- **Activity Bar**: Left position
- **Toolbar**: Visible
- **Bottom Panel**: Visible

**Use Cases**:
- Users familiar with Xcode/AppCode layouts
- Traditional IDE workflows
- Code-heavy projects

**Visual Layout**:
```
┌──────────────┬──────────────────────────────┬──────────────┐
│              │                              │              │
│  Hierarchy   │         Viewport             │  Inspector   │
│              │                              │              │
└──────────────┴──────────────────────────────┴──────────────┘
└──────────────────────────────────────────────────────────────┘
                        Timeline
```

---

## 3. Modern Layout

**Purpose**: Streamlined layout with floating panels and minimal chrome.

**Configuration**:
- **Hierarchy Panel**: Left dock, 240pt preferred width (200-400pt range)
- **Inspector Panel**: Right dock, 280pt preferred width (240-450pt range)
- **Tools Sidebar**: Left dock (order 1), 80pt preferred width (60-120pt range)
- **Activity Bar**: Left position
- **Toolbar**: Visible
- **Bottom Panel**: Hidden

**Use Cases**:
- Modern UI/UX workflows
- Maximized viewport space
- Touch-optimized interfaces
- Design-focused work

**Visual Layout**:
```
┌────┬──────────────────────────────────────────┬────────┐
│Act │ Hierarchy                                │Inspec  │
│Bar │                                          │tor     │
├────┤              Viewport                    │        │
│Tool│                                          │        │
│s   │                                          │        │
└────┴──────────────────────────────────────────┴────────┘
```

---

## 4. PhiPro Layout

**Purpose**: Distraction-free, viewport-only layout for focused work.

**Configuration**:
- **All Panels**: Hidden
- **Activity Bar**: Hidden
- **Toolbar**: Hidden
- **Bottom Panel**: Hidden

**Use Cases**:
- Presentation mode
- Full-screen viewport work
- Distraction-free editing
- Performance testing
- Screenshot/recording sessions

**Visual Layout**:
```
┌──────────────────────────────────────────────────────────┐
│                                                          │
│                                                          │
│                      Viewport                            │
│                                                          │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

---

## 5. Minimal Layout

**Purpose**: Essential panels only, maximizing viewport space.

**Configuration**:
- **Hierarchy Panel**: Left dock, 200pt preferred width (180-350pt range)
- **Activity Bar**: Left position
- **Toolbar**: Visible
- **Inspector**: Hidden
- **Bottom Panel**: Hidden

**Use Cases**:
- Quick scene navigation
- Lightweight editing
- Small screen workflows
- Laptop/mobile development

**Visual Layout**:
```
┌────┬──────────────────────────────────────────────────┐
│Act │ Hierarchy                                        │
│Bar │                                                  │
│    │              Viewport                            │
│    │                                                  │
│    │                                                  │
└────┴──────────────────────────────────────────────────┘
```

---

## 6. Editorial Layout

**Purpose**: Optimized for content creation with expanded panels.

**Configuration**:
- **Hierarchy Panel**: Left dock, 280pt preferred width (220-500pt range)
- **Inspector Panel**: Right dock, 340pt preferred width (280-600pt range)
- **Timeline Panel**: Bottom dock, 220pt preferred height (160-500pt range)
- **Activity Bar**: Left position
- **Toolbar**: Visible
- **Bottom Panel**: Visible

**Use Cases**:
- Content authoring
- Documentation work
- Asset-heavy projects
- Detailed property editing
- Animation/timeline-focused workflows

**Visual Layout**:
```
┌──────────────┬──────────────────────────────┬──────────────┐
│              │                              │              │
│  Hierarchy   │         Viewport             │  Inspector   │
│  (wider)     │                              │  (wider)     │
└──────────────┴──────────────────────────────┴──────────────┘
└──────────────────────────────────────────────────────────────┘
                    Timeline (taller)
```

---

## Implementation Details

### Panel Sizing

All panel sizes follow the **8pt grid system**:
- Minimum sizes: multiples of 8pt (e.g., 180pt, 200pt, 220pt)
- Preferred sizes: multiples of 8pt (e.g., 240pt, 260pt, 280pt)
- Maximum sizes: multiples of 8pt (e.g., 400pt, 450pt, 500pt)

### Dock Positions

```rust
pub enum DockPosition {
    Left,       // Left sidebar area
    Right,      // Right sidebar area
    Bottom,     // Bottom panel area
    Top,        // Top toolbar area
    Center,     // Main viewport area
    Floating,   // Detached window
    Hidden,     // Not visible
}
```

### Panel Slots

Each panel slot defines:
- `id`: Panel identifier (e.g., "hierarchy", "inspector")
- `dock`: Where the panel is docked
- `order`: Stacking order within the same dock position
- `visible`: Initial visibility state
- `min_size`: Minimum width/height (8pt grid)
- `preferred_size`: Default width/height (8pt grid)
- `max_size`: Maximum width/height (8pt grid)
- `collapsible`: Whether the panel can be collapsed

### Switching Presets

```rust
use rust_appkit::workspace::WorkspacePreset;

// Get a preset
let preset = WorkspacePreset::default();
let preset = WorkspacePreset::classic();
let preset = WorkspacePreset::modern();
let preset = WorkspacePreset::phipro();
let preset = WorkspacePreset::minimal();
let preset = WorkspacePreset::editorial();

// Apply preset to workspace
workspace.apply_preset(preset);
```

---

## Design Principles

1. **8pt Grid Alignment**: All sizes are multiples of 8pt
2. **Collapsible Panels**: Most panels can be collapsed to save space
3. **Responsive Ranges**: Min/max sizes allow flexible resizing
4. **Consistent Ordering**: Panels maintain logical stacking order
5. **Theme Integration**: All colors use `hierarchy::bg_color()` system
6. **Spacing Consistency**: All padding/margins use `spacing::*` constants

---

## Related Documentation

- `layout/constants.rs` — All size constants
- `layout/spacing.rs` — Padding, margin, gap, inset values
- `layout/hierarchy.rs` — Theme-based color system
- `workspace/mod.rs` — Workspace types and API
- `workspace/presets.rs` — Preset implementations
