# Modals - Store Structure

> **Source files:** `crates/phiber/src/store/modals/`
> **Enum file:** `crates/buikit/src/modal.rs`

---

## Store File Layout

```
crates/phiber/src/store/modals/
  mod.rs          - re-exports, module declarations
  settings.rs     - SettingIdx enum, SettingSpec, SettingsModal
  help.rs         - HelpIdx enum, HelpSpec, BHelpModal
  profile.rs      - ProfileIdx enum, ProfileSpec, ProfileModal
```

> **Note:** Host switching is not a modal. The `Switcher` is a
> `BPartType::Switcher` menu part in the header region.

---

## BModalType Enum

```rust
// crates/buikit/src/modal.rs

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BModalType {
    Settings,
    Help,
    Profile,
}

impl BModalType {
    pub fn title(self) -> &'static str {
        match self {
            Self::Settings => "Settings",
            Self::Help => "Help",
            Self::Profile => "Profile",
        }
    }

    pub fn width(self) -> f32 {
        match self {
            Self::Settings => 640.0,
            Self::Help => 480.0,
            Self::Profile => 400.0,
        }
    }

    pub fn all() -> &'static [BModalType] {
        &[Self::Settings, Self::Help, Self::Profile]
    }
}
```

---

## Modal View Pattern

Modals use store specs + state + view functions - no handler trait, no registry.
The app's view layer reads `ModalState`, calls the appropriate view function,
wraps the result in `b_modal()`, and renders it.

```rust
// Example: settings modal view function
fn settings_modal_view(state: &SettingsState, cx: &App) -> Vec<BView> {
    let categories = SettingsModal::categories();
    // ... build BView tree from store specs + state
}
```

### Comparison with Zed

| Phiber | Zed |
|--------|-----|
| Store specs + state → view function → BView | `Render::render() → impl IntoElement` |
| Pure data - app wraps in `b_modal()` | Direct GPUI element - `ModalLayer` wraps |
| `BModalType` enum identifies modal | Type identity via `AnyView::downcast::<V>()` |
| App manages `ModalState` | `ModalLayer` stores `Box<dyn ModalViewHandle>` |

---

## Settings Modal Store: `store/modals/settings.rs`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SettingType {
    Interface,
    Editing,
    Navigation,
    AddOns,
    Themes,
    System,
}

impl SettingType {
    pub fn label(self) -> &'static str { ... }
    pub fn rows(self) -> &'static [(&'static str, &'static str)] { ... }
}

pub struct SettingSpec {
    pub idx: SettingType,
    pub label: &'static str,
    pub rows: &'static [(&'static str, &'static str)],
}

pub struct SettingsModal;

impl SettingsModal {
    pub fn categories() -> Vec<SettingSpec> { ... }
}
```

### Static row data

| Category | Rows |
|----------|------|
| Interface | Display, Resolution, Scale, Language |
| Editing | Auto-Save, Undo Steps, Snap |
| Navigation | Orbit, Zoom, Walk |
| Add-ons | Installed, Check for Updates |
| Themes | Active Theme, Accent |
| System | GPU, Memory, Sound |

### Zed comparison

Zed's settings store (`settings_store.rs`) is a full JSON-backed system:
- `SettingsStore` global entity with file watchers
- `SettingsContent` struct with all settings as JSON fields
- `Settings` trait - `from_settings(&SettingsContent) → Self`
- `SettingsKey` trait - `const KEY: Option<&'static str>`
- Profile support (`ActiveSettingsProfileName`)
- Schema validation via `schemars`

Phiber's settings store is simpler - static enum-indexed rows. To match Zed's
approach, we would need:
- A `SettingsStore` entity with JSON file I/O
- `SettingsContent` struct mirroring Blender's preferences
- Per-category `Settings` trait implementations
- File watchers for live-reload

### Blender comparison

Blender's Preferences (`SpacePreferences`) has these sections:
- Interface, Viewport, Lights, Editing, Animation
- Extensions (Add-ons), Themes
- Asset Libraries
- Input, Navigation, Keymap
- System, Save & Load, File Paths
- Developer Tools, Experimental

Our `SettingIdx` maps closely but is missing: Viewport, Lights, Animation,
Asset Libraries, File Paths, Keymap, Developer Tools, Experimental.

---

## Help Modal Store: `store/modals/help.rs`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HelpType {
    Documentation,
    Tutorials,
    KeyboardShortcuts,
    ApiReference,
    CommunityForum,
    ReportBug,
}

impl HelpType {
    pub fn label(self) -> &'static str { ... }
    pub fn icon(self) -> &'static str { ... }  // "book", "play", "keyboard", "code", "users", "bug"
}

pub struct HelpSpec {
    pub idx: HelpType,
    pub label: &'static str,
    pub icon: &'static str,
}

pub struct BHelpModal;

impl BHelpModal {
    pub fn links() -> Vec<HelpSpec> { ... }
}
```

---

## Profile Modal Store: `store/modals/profile.rs`

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProfileType {
    SignIn,
    Preferences,
    ThemeSettings,
    KeyboardShortcuts,
    SignOut,
}

impl ProfileType {
    pub fn label(self) -> &'static str { ... }
    pub fn icon(self) -> &'static str { ... }  // "login", "settings", "palette", "keyboard", "logout"
}

pub struct ProfileSpec {
    pub idx: ProfileType,
    pub label: &'static str,
    pub icon: &'static str,
}

pub struct ProfileModal;

impl ProfileModal {
    pub fn actions() -> Vec<ProfileSpec> { ... }
}
```

---

## Modal State Management

The app manages `ModalState` directly - no registry needed:

```rust
// App opens a modal:
modal_state.set_open(BModalType::Settings);

// App closes:
modal_state.set_close();

// App toggles:
modal_state.set_toggle(BModalType::Help);
```

### Zed comparison

Zed doesn't have a modal registry either. Each modal is toggled individually:

```rust
// Zed: workspace.rs
workspace.toggle_modal(window, cx, SettingsModal::new);
workspace.toggle_modal(window, cx, BHelpModal::new);
```

The `ModalLayer` handles show/hide/stash. There's no central registry -
each call site knows which modal type to construct.
