# Theme Architecture — Rust Design Decisions

## Overview

The `theme/` module implements a composable theming system where Brand × Mode × Material
layers combine at runtime to produce a resolved `ThemeData`. This document explains the
patterns, memory model, and alternatives.

---

## Pattern: Global Singleton Registry (`OnceLock<Mutex<ThemeRegistry>>`)

```rust
/// The theme registry is a global singleton.
/// OnceLock: initialized exactly once (first access), never re-allocated.
/// Mutex: thread-safe interior mutability for theme switching.
///
/// Why global? Every NSColor::accent(), every layout_constants::spacing_md(),
/// every view's apply_theme() needs to read the current theme. Passing a
/// &ThemeRegistry through every function would infect the entire call stack.
static REGISTRY: OnceLock<Mutex<ThemeRegistry>> = OnceLock::new();

/// Access the global registry. Cheap: just reads a pointer.
pub fn get_registry() -> &'static Mutex<ThemeRegistry> {
    REGISTRY.get_or_init(|| Mutex::new(ThemeRegistry::new()))
}
```

**Memory diagram — singleton lifecycle:**

```
Program start:
  REGISTRY = OnceLock { state: EMPTY }

First call to get_registry():
  REGISTRY = OnceLock { state: INITIALIZED }
                              │
                              ▼
  ┌─────────────────────────────────────────────────────────┐
  │ Mutex<ThemeRegistry>                                    │
  │ ├── locked: AtomicBool = false                         │
  │ └── value: ThemeRegistry                               │
  │     ├── themes: HashMap<String, ThemeData>             │
  │     │   ├── "apple.dark.standard" → ThemeData { … }   │
  │     │   ├── "apple.light.standard" → ThemeData { … }  │
  │     │   ├── "tesla.dark.standard" → ThemeData { … }   │
  │     │   ├── "cyberpunk.dark.standard" → ThemeData { … }│
  │     │   └── … (40+ pre-composed themes)               │
  │     ├── current: String = "apple.dark.standard"        │
  │     ├── appearance_brand: String = "apple"             │
  │     ├── appearance_mode: String = "dark"               │
  │     └── appearance_material: String = "standard"       │
  └─────────────────────────────────────────────────────────┘

  This lives for the entire program lifetime.
  Never reallocated. Never freed until process exit.
```

**Alternative A: Dependency injection**
```rust
// Alternative — considered for testability
struct App {
    theme: ThemeRegistry,  // owned, passed to every component
}
fn build_button(theme: &ThemeRegistry) -> NSButton { … }
// Pro: Testable (inject mock theme). No global state.
// Con: Every function needs a &ThemeRegistry parameter.
//      100+ call sites. Viral parameter pollution.
// Verdict: Rejected for ergonomics. Tests can override the global.
```

**Alternative B: Thread-local**
```rust
// Alternative — rejected
thread_local! {
    static THEME: RefCell<ThemeRegistry> = RefCell::new(ThemeRegistry::new());
}
// Pro: No Mutex (faster). Con: Each thread has its own copy.
// Theme changes on one thread don't propagate to others.
// GPU thread would see stale theme. Rejected.
```

---

## Pattern: Layered Composition (Brand × Mode × Material)

```rust
/// Compose a theme by applying layers in order:
///   1. Base (defaults for all tokens)
///   2. Mode (dark/light color overrides)
///   3. Brand (accent, fonts, identity)
///   4. Material (surface treatment, radii, opacity)
///
/// Each layer only overrides what it cares about.
/// Unset tokens fall through to the base.
pub fn compose(&self, brand: &str, mode: &str, material: &str) -> ThemeData {
    let mut d = base_theme();       // Step 1: all defaults
    apply_mode(&mut d, mode);       // Step 2: dark/light colors
    apply_brand(&mut d, brand);     // Step 3: accent + fonts
    apply_material(&mut d, material); // Step 4: surface + radii
    d
}
```

**Memory diagram — composition layers:**

```
base_theme()          apply_mode("dark")      apply_brand("cyberpunk")   apply_material("glass")
┌──────────────┐     ┌──────────────┐        ┌──────────────┐           ┌──────────────┐
│ bg_window:   │     │ bg_window:   │        │ bg_window:   │           │ bg_window:   │
│  (0.1,0.1,0.1)│ ──► │  (0.08,…)    │ ──────► │  (0.04,0.02, │ ─────────► │  (0.05,…)    │
│              │     │              │        │   0.08)      │           │              │
│ accent:      │     │ accent:      │        │ accent:      │           │ accent:      │
│  (0,0.48,1)  │     │  (unchanged) │        │  (0,1.0,0.9) │           │  (unchanged) │
│              │     │              │        │              │           │              │
│ radius_panel:│     │ radius_panel:│        │ radius_panel:│           │ radius_panel:│
│  10.0        │     │  (unchanged) │        │  (unchanged) │           │  24.0        │
│              │     │              │        │              │           │              │
│ glass_opacity│     │ glass_opacity│        │ glass_opacity│           │ glass_opacity│
│  1.0         │     │  (unchanged) │        │  (unchanged) │           │  0.65        │
└──────────────┘     └──────────────┘        └──────────────┘           └──────────────┘

Final ThemeData = cyberpunk + dark + glass:
  bg_window: deep purple-black
  accent: neon cyan
  radius_panel: 24px (glass rounds corners)
  glass_opacity: 0.65 (translucent panels)
  font: Orbitron
```

**Why layered composition:**
- 8 brands × 3 modes × 8 materials = 192 possible themes
- Only 19 layer functions needed (8 + 3 + 8) instead of 192 full definitions
- Adding a new brand = one function, automatically works with all modes and materials

**Alternative: Flat theme definitions**
```rust
// Alternative — rejected (combinatorial explosion)
fn cyberpunk_dark_glass() -> ThemeData { /* 50 lines */ }
fn cyberpunk_dark_metal() -> ThemeData { /* 50 lines */ }
fn cyberpunk_light_glass() -> ThemeData { /* 50 lines */ }
// 192 functions × 50 lines = 9,600 lines of theme code. Unmaintainable.
```

---

## Pattern: Token-Based Color Resolution

```rust
/// Get a themed color by token key.
/// The token is a string constant (e.g., "bg_panel", "text_label").
/// Resolution: token → HashMap lookup → NSColor value.
///
/// Why string keys and not an enum?
/// - Themes are data-driven (loaded from files in the future)
/// - New tokens can be added without recompiling the enum
/// - HashMap lookup is O(1) amortized
pub fn color(key: &str) -> NSColor {
    registry::get_color(key).unwrap_or_else(|| {
        // Unknown key: fall back to label color (always visible)
        eprintln!("[theme] WARNING: unknown color key '{}'", key);
        NSColor::system_label()
    })
}

/// Get a themed f32 value (spacing, radius, font size).
/// Falls back to the provided default if the token isn't in the theme.
pub fn f32_or(key: &str, fallback: f32) -> f32 {
    get_registry().lock().unwrap()
        .current()
        .layout.geometry
        .get(key)
        .copied()       // Copy the f32 out of the HashMap
        .unwrap_or(fallback)
}
```

**Alternative: Enum keys (compile-time checked)**
```rust
// Alternative — viable, more type-safe
enum ColorToken { BgPanel, BgSidebar, TextLabel, Accent, /* 60+ variants */ }
fn color(token: ColorToken) -> NSColor { COLORS[token as usize] }
// Pro: No string hashing, array index is O(1), typos caught at compile time.
// Con: Adding a token = editing the enum + every theme definition.
//      Can't load custom tokens from a file. Closed set.
// Verdict: May migrate to this for core tokens. Keep strings for extensibility.
```

---

## Pattern: `ThemeProvider` Trait (Inheritance via Composition)

```rust
/// Trait for defining theme variations with inheritance.
/// A theme can override just the colors it cares about.
/// Everything else falls through to the base.
pub trait ThemeProvider {
    /// Parent theme to inherit from (None = absolute base).
    fn base(&self) -> Option<&dyn ThemeProvider> { None }

    /// Override geometry tokens (spacing, radii, sizes).
    fn override_geometry(&self, _g: &mut HashMap<String, f32>) {}

    /// Override color tokens.
    fn override_colors(&self, _c: &mut HashMap<String, NSColor>) {}

    /// Build the full theme by resolving the inheritance chain.
    fn build(&self) -> ThemeData {
        // Start from parent (or absolute base if no parent)
        let mut d = if let Some(parent) = self.base() {
            parent.build()  // recursive — walks up the chain
        } else {
            BaseTheme.build_base()
        };
        // Apply our overrides on top
        self.override_geometry(&mut d.layout.geometry);
        self.override_colors(&mut d.colors);
        d
    }
}

/// Tuple composition: combine two providers into one.
/// The second provider's overrides win (applied last).
impl<A: ThemeProvider, B: ThemeProvider> ThemeProvider for (A, B) {
    fn override_geometry(&self, g: &mut HashMap<String, f32>) {
        self.0.override_geometry(g);  // A first
        self.1.override_geometry(g);  // B overwrites A's values
    }
    fn override_colors(&self, c: &mut HashMap<String, NSColor>) {
        self.0.override_colors(c);
        self.1.override_colors(c);
    }
}
```

**Memory diagram — inheritance chain:**

```
build() call on CyberpunkGlass:

  CyberpunkGlass.build()
       │
       │ self.base() = Some(&CyberpunkDark)
       ▼
  CyberpunkDark.build()
       │
       │ self.base() = Some(&BaseDark)
       ▼
  BaseDark.build()
       │
       │ self.base() = None
       ▼
  BaseTheme.build_base()  ← returns ThemeData with all defaults
       │
       │ BaseDark.override_colors() ← dark mode colors
       ▼
  ThemeData (dark base)
       │
       │ CyberpunkDark.override_colors() ← neon cyan accent, purple bg
       ▼
  ThemeData (cyberpunk dark)
       │
       │ CyberpunkGlass.override_geometry() ← radius=24, glass_opacity=0.65
       ▼
  ThemeData (cyberpunk dark glass) ← FINAL RESULT
```

**Alternative: CSS-like cascading**
```rust
// Alternative — considered for future
// Themes as ordered list of partial overrides
let theme = cascade(&[base, dark_mode, cyberpunk_brand, glass_material]);
// Pro: Familiar to web devs. Con: Order-dependent, harder to reason about.
```

---

## NSAppearance — Apple Parity

```rust
/// Mirrors Apple's NSAppearance — a lightweight name tag.
/// Views inherit appearance from parent unless overridden.
///
/// Apple's actual class is ~20 lines of meaningful API.
/// Everything else (color resolution, vibrancy) happens in the
/// color system, not in NSAppearance itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NSAppearance {
    name: AppearanceName,  // 1 byte (enum discriminant)
}

impl NSAppearance {
    /// Create from a name. This is the primary constructor.
    pub fn named(name: AppearanceName) -> Self { Self { name } }

    /// Read the current system appearance from the theme registry.
    /// Equivalent to [NSAppearance currentDrawingAppearance] in ObjC.
    pub fn current_drawing() -> Self {
        let mode = get_registry().lock().unwrap().current_mode_enum();
        Self { name: AppearanceName::from_mode(mode) }
    }
}
```

**How Apple uses it vs how we use it:**

```
APPLE (Objective-C):                    US (Rust):
─────────────────────                   ──────────────────
window.appearance =                     view.set_appearance(Some(
  [NSAppearance                           NSAppearance::named(
    appearanceNamed:                        AppearanceName::DarkAqua
    NSAppearanceNameDarkAqua];            )
                                        ));

// Color resolution:                    // Color resolution:
[NSColor labelColor]                    NSColor::text_label()
  → queries effectiveAppearance           → queries theme registry
  → returns dark or light variant         → returns themed color

// Inheritance:                         // Inheritance:
childView.effectiveAppearance           view.effective_appearance()
  → walks superview chain                 → returns override or system default
  → returns first non-nil appearance      (parent traversal: future work)
```
