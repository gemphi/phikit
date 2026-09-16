//! Static value constants — the strings written into `BState` for boolean
//! flags, modal slugs, and Blender's factory defaults. Enum-derived slugs
//! (`BSpaceType::slug()`, `BModalType::slug()`, `BToolSpec.label`) are used
//! directly by their callers — no duplication here.

// ── Booleans ──
pub const V_TRUE: &str = "true";
pub const V_FALSE: &str = "false";
pub const V_EMPTY: &str = "";

// ── Modal slugs (mirror `BModalType::slug()`) ──
pub const V_MODAL_SETTINGS: &str = "settings";
pub const V_MODAL_HELP: &str = "help";
pub const V_MODAL_PROFILE: &str = "profile";
pub const V_MODAL_COMMAND: &str = "command";
pub const V_MODAL_SPLASH: &str = "splash";
pub const V_MODAL_ABOUT: &str = "about";
pub const V_MODAL_FILE_BROWSER: &str = "file_browser";

// ── Themes ──
pub const V_THEME_APPLE_DARK: &str = "AppleDark";
pub const V_THEME_BLENDER_DARK: &str = "BlenderDark";
pub const V_THEME_TOKYO_NIGHT: &str = "TokyoNight";

// ── Blender factory defaults (source: `startup.blend` DNA) ──
pub const V_SCENE_DEFAULT: &str = "Scene";
pub const V_VIEW_LAYER_DEFAULT: &str = "ViewLayer";
pub const V_OBJECT_DEFAULT: &str = "Cube";
pub const V_MODE_OBJECT: &str = "Object";
pub const V_TOOL_SELECT_BOX: &str = "Select Box";
pub const V_FRAME_START: &str = "1";
pub const V_FRAME_END: &str = "250";
