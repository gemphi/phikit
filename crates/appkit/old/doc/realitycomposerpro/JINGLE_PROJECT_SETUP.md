# Setting Up a Jingle Reality Composer Project

> Step-by-step guide to configuring your Reality Composer Pro-style project in Rust with Jingle

---

## 🗺 Documentation Map

- [← Back to Reality Composer Pro Guide](./JINGLE_REALITY_COMPOSER_PRO_GUIDE.md)
- [📦 Importing Assets](STEP_BY_STEP_ASSET_IMPORT_GUIDE.md)
- [🎨 Shader Graph](STEP_BY_STEP_SHADER_GRAPH_GUIDE.md)

---

## Overview

This guide walks you through setting up a new Jingle project with the same workflow as Reality Composer Pro. You'll learn how to configure your project window, organize assets, and prepare for spatial content creation.

![Project Window](./assets/project-window.png)

---

## Table of Contents

1. [Creating a New Project](#creating-a-new-project)
2. [Configuring the Project Window](#configuring-the-project-window)
3. [Setting Up the Asset Pipeline](#setting-up-the-asset-pipeline)
4. [Configuring Render Settings](#configuring-render-settings)
5. [Project Templates](#project-templates)
6. [Best Practices](#best-practices)

---

## Creating a New Project

### Using Cargo

```bash
# Create new Rust project
cargo new my_jingle_project
cd my_jingle_project

# Add Jingle dependency
cargo add --path ../../phi/ext/jingle
cargo add glam
```

### Project Structure

Create the following directory structure:

```
my_jingle_project/
├── Cargo.toml
├── assets/
│   ├── meshes/
│   ├── textures/
│   ├── materials/
│   └── audio/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── scene.rs
│   └── materials.rs
└── .cargo/
    └── config.toml
```

### Cargo.toml Configuration

```toml
[package]
name = "my_jingle_project"
version = "0.1.0"
edition = "2021"

[dependencies]
jingle = { path = "../../phi/ext/jingle" }
glam = "0.25"
phiwin = { path = "../../app/phipro/src/phiwin" }

[features]
default = ["desktop"]
desktop = ["jingle/desktop"]
visionos = ["jingle/visionos"]
```

---

## Configuring the Project Window

### Main Entry Point

```rust
// src/main.rs
use jingle::{Scene, RenderConfig, WindowConfig};
use jingle::reality_kit::{Camera, Lighting};

fn main() {
    println!("╔════════════════════════════════════════════════════════╗");
    println!("║  Jingle Reality Composer - Project Setup               ║");
    println!("╚════════════════════════════════════════════════════════╝\n");
    
    // Initialize Jingle
    let config = ProjectConfig {
        name: "My Reality Project".to_string(),
        window: WindowConfig {
            width: 1600,
            height: 900,
            title: "Reality Composer - Jingle".to_string(),
        },
        render: RenderConfig {
            vsync: true,
            msaa: 4,
            hdr: true,
        },
    };
    
    // Create application
    let mut app = RealityComposerApp::new(config);
    
    // Run main loop
    app.run();
}
```

### Project Configuration

```rust
// src/lib.rs
use jingle::asset::AssetManager;
use jingle::reality_kit::Scene;

pub struct RealityComposerApp {
    pub config: ProjectConfig,
    pub asset_manager: AssetManager,
    pub scene: Scene,
    pub render_settings: RenderSettings,
}

impl RealityComposerApp {
    pub fn new(config: ProjectConfig) -> Self {
        println!("🚀 Initializing Reality Composer Project...");
        
        // Setup asset manager
        let mut asset_manager = AssetManager::new();
        asset_manager.set_base_path(std::path::PathBuf::from("assets"));
        
        // Create empty scene
        let scene = Scene::new();
        
        println!("   ✅ Project initialized: {}", config.name);
        
        Self {
            config,
            asset_manager,
            scene,
            render_settings: RenderSettings::default(),
        }
    }
    
    pub fn run(&mut self) {
        println!("▶️  Starting main loop...");
        
        // Main application loop would go here
        // - Process input
        // - Update scene
        // - Render frame
        
        loop {
            // Application logic
            break; // For demo purposes
        }
        
        println!("✅ Project closed successfully");
    }
}

pub struct ProjectConfig {
    pub name: String,
    pub window: WindowConfig,
    pub render: RenderConfig,
}

pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

pub struct RenderConfig {
    pub vsync: bool,
    pub msaa: u32,
    pub hdr: bool,
}

pub struct RenderSettings {
    pub show_grid: bool,
    pub show_gizmos: bool,
    pub wireframe: bool,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            show_grid: true,
            show_gizmos: true,
            wireframe: false,
        }
    }
}
```

---

## Setting Up the Asset Pipeline

### Asset Manager Configuration

```rust
// src/asset_setup.rs
use jingle::asset::{AssetManager, AssetType, AssetManifest, AssetEntry};
use std::path::PathBuf;

pub fn setup_asset_pipeline() -> AssetManager {
    println!("📦 Setting up asset pipeline...");
    
    let mut manager = AssetManager::new();
    
    // Configure paths
    manager.set_base_path(PathBuf::from("assets"));
    
    // Set up default directories
    let directories = vec![
        ("meshes", AssetType::Mesh),
        ("textures", AssetType::Texture),
        ("materials", AssetType::Material),
        ("audio", AssetType::Sound),
        ("shaders", AssetType::Shader),
    ];
    
    for (dir, asset_type) in directories {
        let path = PathBuf::from(format!("assets/{}", dir));
        if !path.exists() {
            std::fs::create_dir_all(&path).expect("Failed to create directory");
            println!("   📁 Created: {}", dir);
        }
    }
    
    println!("   ✅ Asset pipeline ready");
    
    manager
}

pub fn create_default_manifest() -> AssetManifest {
    AssetManifest {
        meshes: vec![
            AssetEntry {
                name: "default_cube".to_string(),
                path: "meshes/cube.obj".to_string(),
                asset_type: AssetType::Mesh,
            },
            AssetEntry {
                name: "default_sphere".to_string(),
                path: "meshes/sphere.obj".to_string(),
                asset_type: AssetType::Mesh,
            },
        ],
        textures: vec![],
        materials: vec![],
        sounds: vec![],
    }
}
```

---

## Configuring Render Settings

### Render Configuration

```rust
// src/render_config.rs
use jingle::render::{Renderer, RenderSettings, ViewportConfig};

pub struct RealityComposerRenderer {
    settings: RenderSettings,
    viewport: ViewportConfig,
}

impl RealityComposerRenderer {
    pub fn new() -> Self {
        Self {
            settings: RenderSettings {
                // Reality Composer Pro defaults
                exposure: 1.0,
                gamma: 2.2,
                shadow_quality: ShadowQuality::High,
                ambient_occlusion: true,
                bloom: true,
                motion_blur: false,
            },
            viewport: ViewportConfig {
                width: 1600,
                height: 900,
                camera: CameraConfig::default(),
            },
        }
    }
    
    pub fn configure_for_quality(&mut self, quality: RenderQuality) {
        match quality {
            RenderQuality::Draft => {
                self.settings.shadow_quality = ShadowQuality::Low;
                self.settings.ambient_occlusion = false;
                self.settings.bloom = false;
            }
            RenderQuality::Preview => {
                self.settings.shadow_quality = ShadowQuality::Medium;
                self.settings.ambient_occlusion = true;
                self.settings.bloom = false;
            }
            RenderQuality::Final => {
                self.settings.shadow_quality = ShadowQuality::Ultra;
                self.settings.ambient_occlusion = true;
                self.settings.bloom = true;
                self.settings.motion_blur = true;
            }
        }
    }
}

pub enum RenderQuality {
    Draft,   // Fast preview
    Preview, // Balanced
    Final,   // Production quality
}

pub enum ShadowQuality {
    Low,    // 1024x1024
    Medium, // 2048x2048
    High,   // 4096x4096
    Ultra,  // 8192x8192
}

pub struct CameraConfig {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub position: glam::Vec3,
    pub rotation: glam::Quat,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            fov: 60.0,
            near: 0.01,
            far: 1000.0,
            position: glam::Vec3::new(0.0, 1.6, 3.0),
            rotation: glam::Quat::IDENTITY,
        }
    }
}
```

---

## Project Templates

### Spatial Experience Template

```rust
// templates/spatial_experience.rs
use jingle::reality_kit::{Scene, Entity, Transform, ModelComponent, AnchorComponent};
use jingle::material::Material;
use glam::{Vec3, Quat};

pub struct SpatialExperienceTemplate;

impl SpatialExperienceTemplate {
    pub fn create() -> Scene {
        let mut scene = Scene::new();
        
        // Create floor anchor
        let floor_anchor = scene.spawn(Transform {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        });
        scene.add_component(floor_anchor, AnchorComponent::floor());
        
        // Add default lighting
        let sun = scene.spawn(Transform {
            position: Vec3::new(10.0, 20.0, 5.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        });
        scene.add_component(sun, jingle::reality_kit::DirectionalLight {
            color: Vec3::new(1.0, 0.95, 0.9),
            intensity: 1.0,
        });
        
        // Add ambient light
        scene.set_ambient_light(Vec3::new(0.3, 0.3, 0.4), 0.5);
        
        scene
    }
}
```

### Game Template

```rust
// templates/game_template.rs
use jingle::{Scene, PhysicsWorld, AssetManager};
use jingle::reality_kit::{Transform, Entity, ModelComponent, RigidBody};

pub struct GameTemplate;

impl GameTemplate {
    pub fn create() -> (Scene, PhysicsWorld) {
        let mut scene = Scene::new();
        let mut physics = PhysicsWorld::new();
        
        // Create player spawn point
        let player_spawn = scene.spawn(Transform::from_position(
            glam::Vec3::new(0.0, 1.0, 0.0)
        ));
        
        // Create ground
        let ground = scene.spawn(Transform {
            position: glam::Vec3::new(0.0, -0.5, 0.0),
            rotation: glam::Quat::IDENTITY,
            scale: glam::Vec3::new(50.0, 1.0, 50.0),
        });
        
        // Add physics collider to ground
        let ground_body = RigidBody::static_body()
            .with_position(glam::Vec3::new(0.0, -0.5, 0.0));
        let ground_handle = physics.add_rigid_body(ground_body);
        physics.add_collider(
            jingle::physics::Collider::cuboid(25.0, 0.5, 25.0),
            ground_handle
        );
        
        (scene, physics)
    }
}
```

---

## Best Practices

### 1. Organize Assets by Type

```
assets/
├── meshes/
│   ├── characters/
│   ├── props/
│   └── environment/
├── textures/
│   ├── characters/
│   ├── props/
│   └── environment/
└── materials/
    ├── characters/
    ├── props/
    └── environment/
```

### 2. Use Consistent Naming

```rust
// Good naming
let robot_head_mesh = "character_robot_head_01.obj";
let ground_texture = "env_ground_grass_01.png";

// Bad naming
let mesh1 = "mesh.obj";
let tex = "texture.png";
```

### 3. Version Control

```bash
# Add to .gitignore
echo "target/" >> .gitignore
echo "*.rlib" >> .gitignore
echo "*.rmeta" >> .gitignore

# Track only source and assets
git add src/ assets/ Cargo.toml
```

### 4. Build Configuration

```toml
# .cargo/config.toml
[build]
target-dir = "target"
rustflags = ["-C", "opt-level=3"]

[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]
```

---

## Next Steps

- [Importing Assets](STEP_BY_STEP_ASSET_IMPORT_GUIDE.md) - Learn how to import USDZ, OBJ, and other formats
- [Shader Graph](STEP_BY_STEP_SHADER_GRAPH_GUIDE.md) - Create materials with node-based editor
- [Physics Setup](../realitykit/physics.md) - Configure physics for your scene

---

*Generated for Jingle RealityKit Implementation v0.1.0*
