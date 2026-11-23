> [JINGLE_REALITYCOMPOSERPRO](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md) > [CONFIGURE THE JINGLE PROJECT WINDOW](./--configure-the-jingle-project-window.md)

---

### 🗺 Documentation Map

- [🏠 RealityKit Home](../../../realitykit/-realitykit.md)
- [🛠 Jingle Reality Composer](../../JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [🕶 visionOS Home](../../../visionos/-visionos.md)

---

# Configuring the Jingle Reality Composer project window

Learn how to navigate the Jingle project window, customize your workspace, and manage your spatial content using the Rust implementation.

## Overview

The Jingle Reality Composer window is the primary interface for building and arranging 3D content for your Rust projects using the RealityKit API. When you launch PhiPro with a project, you're presented with a workspace designed for high-performance spatial development.

The Jingle window provides several key options:
- **New Project**: Initialize a new Jingle scene with default lighting and anchors.
- **Open Project**: Load an existing project manifest or USDZ scene.
- **Preview in visionOS**: Stream the current scene directly to a visionOS device or simulator.

![Jingle Welcome Interface](./../../assets/rcpro-welcomewindow.png)

## Window Components

The Jingle project window is composed of several integrated views that map directly to Reality Composer Pro concepts:

- **Toolbar**: Provides quick access to project-wide actions. In Jingle, this includes toggling between **Object Mode** and **Edit Mode**, adjusting viewport rendering (Wireframe, Solid, Rendered), and launching the visionOS preview.
- **3D Viewport**: Displays a high-performance visual representation of your active scene using the Phiray engine. This view supports interactive selection and manipulation of entities.
- **Hierarchy (Outliner)**: Shows the ECS entity tree. You can organize entities into groups, manage parent-child relationships, and toggle visibility.
- **Inspector (Properties)**: Allows you to view and edit components attached to the selected entity. You can modify Transforms, ModelComponents, PhysicsBodyComponents, and more.
- **Project Browser (Asset Manager)**: Manage your project's assets including meshes, textures, materials, and audio files.

![Jingle Project Workspace](./../../assets/rcpro-newproject.png)

## Navigating the Viewport

The Jingle viewport provides an interactive way to explore your scene in 3D space. Navigation matches standard spatial tool conventions:

- **Orbit**: Click and drag with the **Middle Mouse Button** (or Alt + Left Click).
- **Pan**: **Shift + Middle Mouse Button**.
- **Zoom**: **Mouse Wheel** or Ctrl + Middle Mouse Button.
- **Focus**: Press **'F'** to center the camera on the selected entity.

### Viewport Modes

Jingle supports five primary viewport modes:

1. **Selection Mode**: Select and move entities in world space.
2. **Orbit Mode**: Rotate the camera around the current focus point.
3. **Fly Mode**: Free-look navigation (WASD keys).
4. **Walk Mode**: Constrained ground navigation.
5. **Placement Mode**: Automatically snap new entities to surfaces.

![Jingle Viewport Toolbar](./../../assets/rcpro-viewporttoolbar.png)

## Configuration with Rust

You can configure the project window directly from your Rust code using the `WindowConfig` and `RenderSettings` APIs.

```rust
use jingle::reality_kit::Scene;
use phipro::blenda::{BlenderUI, Screen, Theme};

fn setup_workspace(scene: &mut Scene) {
    // Configure the UI layout
    let mut ui = BlenderUI::new();
    
    // Set theme to Dark (matching Reality Composer Pro)
    ui.theme = Theme::dark();
    
    // Configure default regions
    let screen = Screen::default_layout();
    println!("Workspace configured with {} regions", screen.regions.len());
    
    // Initialize icons
    phipro::blenda::icons::init_icons();
}
```

### Layout Customization

The Jingle window uses a flexible region-based system. You can split and join regions to create your ideal workspace:

```rust
use phipro::blenda::region::{Region, RegionType};

fn customize_layout(screen: &mut Screen) {
    // Add a new properties panel to the right
    screen.regions.push(Region {
        region_type: RegionType::UI,
        width: 0.25,
        height: 1.0,
        x: 0.75,
        y: 0.0,
    });
}
```

---

## API Comparison

| Reality Composer Pro | Jingle / PhiPro API | Description |
|---------------------|-------------------|-------------|
| Project (.rcproject) | `jingle::reality_kit::Scene` | The root container for your spatial content |
| Toolbar | `phipro::blenda::UiLayout` | Top-level UI controls |
| Viewport | `phipro::blenda::ViewportRenderer` | 3D rendering area |
| Hierarchy | `jingle::ecs::World::query` | Entity management and tree view |
| Inspector | `jingle::ecs::Component` | Property editing system |

---

## Next Steps

- [Importing and Organizing Content](../editor_projectbrowser/--editor_projectbrowser.md)
- [Designing Materials with Shader Graph](../editor_shadergraph/--editor_shadergraph.md)
- [Building Materials](../../building-materials/--building-materials.md)

---

*Generated for Jingle RealityKit Implementation v0.1.0*
