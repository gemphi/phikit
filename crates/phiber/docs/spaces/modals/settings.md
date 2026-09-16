# Settings Modal - Full Documentation

> **Type:** `BModalType::Settings`
> **Width:** 640.0
> **Source:** `store/modals/settings.rs`, `state/modals/settings.rs`

---

## Overview

The Settings modal is Phiber's preferences editor - equivalent to Blender's
`Edit → Preferences...` (Ctrl-Comma) and Zed's `SettingsWindow`.

It displays a sidebar of categories and a content area showing setting rows
for the selected category.

---

## View Tree

```
b_modal("Settings", 640.0)
  └─ Row (gap: 8)
       ├─ Column (sidebar, gap: 2)
       │    ├─ Label("Interface")
       │    ├─ Label("Editing")
       │    ├─ Label("Navigation")
       │    ├─ Label("Add-ons")
       │    ├─ Label("Themes")
       │    └─ Label("System")
       └─ Column (content, gap: 8)
            └─ Label("Select a category from the sidebar")
```

### Planned: full view with rows

```
b_modal("Settings", 640.0)
  └─ Row (gap: 8)
       ├─ Column (sidebar, 200px)
       │    └─ for each category:
       │         Button(label, active=(category == active_category))
       └─ Column (content, flex-1)
            ├─ SectionHeader(active_category.label())
            └─ for each (name, value) in active_category.rows():
                 Row
                   ├─ Label(name)    // "Display", "Resolution", etc.
                   └─ Label(value)   // "Default", "Auto", etc.
```

---

## Store Spec

### `SettingIdx` enum

```rust
pub enum SettingType {
    Interface,      // Display, Resolution, Scale, Language
    Editing,        // Auto-Save, Undo Steps, Snap
    Navigation,     // Orbit, Zoom, Walk
    AddOns,         // Installed, Check for Updates
    Themes,         // Active Theme, Accent
    System,         // GPU, Memory, Sound
}
```

### Static row data

```rust
static INTERFACE_ROWS: &[(&str, &str)] = &[
    ("Display", "Default"),
    ("Resolution", "Auto"),
    ("Scale", "1.0x"),
    ("Language", "English"),
];

static EDITING_ROWS: &[(&str, &str)] = &[
    ("Auto-Save", "Enabled"),
    ("Undo Steps", "32"),
    ("Snap", "Grid"),
];

static NAVIGATION_ROWS: &[(&str, &str)] = &[
    ("Orbit", "Turntable"),
    ("Zoom", "Scale"),
    ("Walk", "Disabled"),
];

static ADDONS_ROWS: &[(&str, &str)] = &[
    ("Installed", "0"),
    ("Check for Updates", "Manual"),
];

static THEMES_ROWS: &[(&str, &str)] = &[
    ("Active Theme", "Dark"),
    ("Accent", "Blue"),
];

static SYSTEM_ROWS: &[(&str, &str)] = &[
    ("GPU", "Auto"),
    ("Memory", "Default"),
    ("Sound", "Enabled"),
];
```

---

## State

```rust
pub struct SettingsState {
    pub active_category: SettingType,
    pub search: String,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            active_category: SettingType::Interface,
            search: String::new(),
        }
    }
}
```

---

## View Function

```rust
// state/modals/settings.rs
fn settings_modal_view(state: &SettingsState, _cx: &App) -> Vec<BView> {
    let categories = SettingsModalSpec::categories();
    let mut sidebar: Vec<BView> = Vec::new();
    for cat in categories {
        sidebar.push(BView::label(cat.label));
    }

    vec![
        BView::Row {
            gap: Some(8.0),
            children: vec![
                BView::Column { gap: Some(2.0), children: sidebar },
                BView::Column { gap: Some(8.0), children: vec![
                    BView::label("Select a category from the sidebar"),
                ]},
            ],
        },
    ]
}
```

---

## Zed Comparison

### Zed's `SettingsWindow` (separate window, not modal)

```
SettingsWindow
  ├─ PlatformTitleBar
  ├─ Navbar (226px sidebar)
  │    ├─ Search bar (Editor entity with fuzzy search)
  │    ├─ NavBarEntry list (uniform_list with expand/collapse)
  │    │    └─ Root entries: Application, Project, Local
  │    └─ File selector (user settings, project settings, local settings)
  └─ Content area (min 400px)
       ├─ SettingsSectionHeader
       └─ SettingField controls:
            ├─ DropdownMenu (enum variants)
            ├─ NumberField (numeric settings)
            ├─ SettingsInputField (text settings)
            ├─ Switch (boolean settings)
            ├─ font_picker, theme_picker, icon_theme_picker
            └─ Sub-pages: LLM providers, MCP servers, skills, tool permissions
```

### Zed's settings store

```rust
// crates/settings/src/settings_store.rs
pub trait Settings: 'static + Send + Sync + Sized {
    const KEY: Option<&'static str>;
    fn from_settings(content: &SettingsContent) -> Self;
    fn register(cx: &mut App);
}

pub struct SettingsStore {
    // JSON file-backed, with file watchers
    // Schema validation via schemars
    // Profile support (ActiveSettingsProfileName)
    // Layered: default → user → project → local
}
```

### What to adopt from Zed

- **Search** - filter settings across all categories by name
- **File layering** - default → user → project → local overrides
- **Reset to default** - per-field reset button showing which file sets the value
- **Sub-pages** - drill into complex settings (e.g., keymap editor, theme picker)
- **Schema validation** - validate settings values against a schema

### What to adopt from Blender

Blender's Preferences has more categories than Phiber currently:
- **Viewport** - anti-aliasing, GPU subdivision, multi-sample
- **Lights** - studio lights, world lights, lookdev lights
- **Animation** - f-curve defaults, keyframe handling
- **Asset Libraries** - registered library paths
- **Input** - mouse, tablet, keyboard, ndof settings
- **Keymap** - key binding editor
- **File Paths** - data, fonts, sounds, textures, scripts
- **Save & Load** - auto-save interval, recent files count
- **Developer Tools** - debug, profiling
- **Experimental** - feature flags

---

## Blender References

- `editors/preferences/index.rst` - Preferences sections toctree
- `editors/preferences/introduction.rst` - `Edit → Preferences...` (Ctrl-Comma)
- `bpy.types.SpacePreferences` - Preferences is a space type in Blender
- `bpy.ops.screen.userpref_show` - operator to show preferences
