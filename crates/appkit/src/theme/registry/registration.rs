use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use crate::graphics::NSColor;
use crate::theme::{keys, ThemeData};

use super::color_palette::apply_mode;
use super::variant_loading::{apply_material, apply_variant, base_theme};
static REGISTRY: OnceLock<std::sync::Mutex<ThemeRegistry>> = OnceLock::new();

/// Global generation counter that increments on every theme change.
static CACHE_GENERATION: AtomicU64 = AtomicU64::new(0);

/// Notify that the theme has changed by incrementing the global cache generation counter.
pub fn notify_theme_changed() {
    CACHE_GENERATION.fetch_add(1, Ordering::Release);
}

/// Returns the current cache generation value.
pub fn cache_generation() -> u64 {
    CACHE_GENERATION.load(Ordering::Acquire)
}

pub fn get_registry() -> &'static std::sync::Mutex<ThemeRegistry> {
    REGISTRY.get_or_init(|| std::sync::Mutex::new(ThemeRegistry::new()))
}

pub fn get_color(key: &str) -> Option<NSColor> {
    get_registry().lock().unwrap().current().colors.get(key).cloned()
}

/// Single-call geometry lookup. Returns 0.0 if key is missing.
pub fn get_geometry(key: &str) -> f32 {
    get_registry().lock().unwrap().current().layout.geometry.get(key).copied().unwrap_or(0.0)
}

pub struct ThemeRegistry {
    themes: HashMap<String, ThemeData>,
    current: String,
    appearance_variant: String,
    appearance_mode: String,
    appearance_material: String,
}

impl ThemeRegistry {
    pub fn new() -> Self {
        let mut r = Self {
            themes: HashMap::new(),
            current: "apple.dark.glass".to_string(),
            appearance_variant: keys::variant::APPLE.to_string(),
            appearance_mode: keys::mode::DARK.to_string(),
            appearance_material: keys::material::GLASS.to_string(),
        };
        register_all_themes(&mut r);
        r
    }

    /// Compose a theme from variant + mode + material layers at runtime.
    pub fn compose(&self, variant: &str, mode: &str, material: &str) -> ThemeData {
        let mut d = base_theme();
        apply_mode(&mut d, mode);
        apply_variant(&mut d, variant);
        apply_material(&mut d, material);
        d
    }

    pub fn current_variant(&self) -> &str { &self.appearance_variant }
    pub fn current_mode(&self) -> &str { &self.appearance_mode }
    pub fn current_material(&self) -> &str { &self.appearance_material }

    /// Returns the current mode as the typed enum.
    pub fn current_mode_enum(&self) -> crate::theme::AppearanceMode {
        match self.appearance_mode.as_str() {
            "light" => crate::theme::AppearanceMode::Light,
            "ombre" => crate::theme::AppearanceMode::Ombre,
            _ => crate::theme::AppearanceMode::Dark,
        }
    }

    pub fn set_variant(&mut self, variant: &str) {
        self.appearance_variant = variant.to_string();
        self.recompose();
    }

    pub fn set_mode(&mut self, mode: &str) {
        self.appearance_mode = mode.to_string();
        self.recompose();
    }

    pub fn set_material(&mut self, material: &str) {
        self.appearance_material = material.to_string();
        self.recompose();
    }

    fn recompose(&mut self) {
        let key = format!("{}.{}.{}", self.appearance_variant, self.appearance_mode, self.appearance_material);
        let theme = {
            let mut d = base_theme();
            apply_mode(&mut d, &self.appearance_mode.clone());
            apply_variant(&mut d, &self.appearance_variant.clone());
            apply_material(&mut d, &self.appearance_material.clone());
            d
        };
        self.themes.insert(key.clone(), theme);
        self.current = key;
        notify_theme_changed();
        println!("[Appearance] Composed: {}.{}.{}", self.appearance_variant, self.appearance_mode, self.appearance_material);
    }

    pub fn available_themes(&self) -> Vec<&str> { self.themes.keys().map(|s| s.as_str()).collect() }
    pub fn register(&mut self, key: &str, theme: ThemeData) { self.themes.insert(key.to_string(), theme); }
    pub fn current(&self) -> &ThemeData { self.themes.get(&self.current).expect("theme missing") }

    pub fn set_current(&mut self, key: &str) {
        if self.themes.contains_key(key) {
            self.current = key.to_string();
            notify_theme_changed();
        }
    }
}

fn register_all_themes(r: &mut ThemeRegistry) {
    use super::color_palette::{rgb, rgba};
    for variant in &keys::variant::ALL {
        for mode in &keys::mode::ALL {
            let mut d = base_theme();
            apply_mode(&mut d, mode);
            apply_variant(&mut d, variant);
            apply_material(&mut d, keys::material::STANDARD);
            r.register(&format!("{}.{}.{}", variant, mode, keys::material::STANDARD), d);
        }
    }
    for material in &keys::material::ALL {
        if *material == keys::material::STANDARD { continue; }
        for mode in &keys::mode::ALL {
            let mut d = base_theme();
            apply_mode(&mut d, mode);
            apply_variant(&mut d, keys::variant::APPLE);
            apply_material(&mut d, material);
            r.register(&format!("{}.{}.{}", keys::variant::APPLE, mode, material), d);
        }
    }
    let aliases: &[(&str, &str, &str, &str)] = &[
        ("macos_sonoma_dark", keys::variant::APPLE, keys::mode::DARK, keys::material::STANDARD),
        ("macos_sonoma_light", keys::variant::APPLE, keys::mode::LIGHT, keys::material::STANDARD),
        ("win11_dark", keys::variant::WINDOWS, keys::mode::DARK, keys::material::STANDARD),
        ("win11_light", keys::variant::WINDOWS, keys::mode::LIGHT, keys::material::STANDARD),
        ("tesla", keys::variant::TESLA, keys::mode::DARK, keys::material::STANDARD),
        ("vision_glass", keys::variant::APPLE, keys::mode::DARK, keys::material::GLASS),
        ("metal", keys::variant::APPLE, keys::mode::DARK, keys::material::METAL),
        ("onyx", keys::variant::APPLE, keys::mode::DARK, keys::material::ONYX),
        ("lace", keys::variant::APPLE, keys::mode::LIGHT, keys::material::LACE),
        ("wetland", keys::variant::APPLE, keys::mode::DARK, keys::material::WETLAND),
        ("platinum", keys::variant::APPLE, keys::mode::LIGHT, keys::material::PLATINUM),
    ];
    for &(name, v, m, mat) in aliases {
        r.register(name, r.compose(v, m, mat));
    }
    r.register("debug", {
        let mut d = base_theme();
        apply_mode(&mut d, keys::mode::DARK);
        let c = &mut d.colors;
        c.insert(keys::CLR_BG_WINDOW.into(), rgb(0.08, 0.08, 0.08));
        c.insert(keys::CLR_BG_CONTENT.into(), rgb(0.10, 0.10, 0.12));
        c.insert(keys::CLR_BG_PANEL.into(), rgb(0.12, 0.12, 0.14));
        c.insert(keys::CLR_BG_SIDEBAR.into(), rgb(0.06, 0.06, 0.08));
        c.insert(keys::CLR_BG_HEADER.into(), rgba(0.14, 0.14, 0.16, 0.95));
        c.insert(keys::CLR_BG_CANVAS.into(), rgb(0.04, 0.04, 0.05));
        c.insert(keys::CLR_BG_PANEL.into(), rgba(0.09, 0.09, 0.11, 0.95));
        c.insert(keys::CLR_BG_TAB_BAR.into(), rgb(0.07, 0.07, 0.09));
        c.insert(keys::CLR_SEPARATOR.into(), rgba(1.0, 1.0, 1.0, 0.5));
        c.insert(keys::CLR_BORDER.into(), rgba(1.0, 1.0, 1.0, 0.3));
        d
    });
}
