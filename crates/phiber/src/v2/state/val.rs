//! Static value constants — the strings written into `BState` for boolean
//! flags, modal slugs, and Blender's factory defaults. Enum-derived slugs
//! (`BSpaceType::slug()`, `BModalType::slug()`, `BToolSpec.label`) are used
//! directly by their callers — no duplication here.

// ── Booleans ──
pub const V_TRUE: &str = "true";
pub const V_FALSE: &str = "false";
/// `ctx.playing` while the ticker runs backwards (`screen.animation_play`
/// with `reverse`, `ANIMPLAY_FLAG_REVERSE`); `V_TRUE` runs it forwards.
pub const V_REVERSE: &str = "reverse";
pub const V_EMPTY: &str = "";

// ── Modal slugs (mirror `BModalType::slug()`) ──
pub const V_MODAL_SETTINGS: &str = "settings";
pub const V_MODAL_HELP: &str = "help";
pub const V_MODAL_PROFILE: &str = "profile";
pub const V_MODAL_COMMAND: &str = "command";
pub const V_MODAL_SPLASH: &str = "splash";
pub const V_MODAL_ABOUT: &str = "about";
pub const V_MODAL_FILE_BROWSER: &str = "file_browser";
/// Blender's render window ("Blender Render") — an Image Editor in its own window.
pub const V_MODAL_RENDER: &str = "render";
pub const V_MODAL_SPACES: &str = "spaces";
pub const V_MODAL_COLLAB: &str = "collab";
pub const V_MODAL_READER: &str = "reader";
pub const V_MODAL_RENAME: &str = "rename";
/// `wm_close_file_dialog`: "Save changes before closing?".
pub const V_MODAL_FILE_CLOSE: &str = "file_close";
pub const V_MODAL_REVERT: &str = "revert";

// ── Themes ──
pub const V_THEME_SPHERE: &str = "Sphere";
pub const V_THEME_GRAPHITE: &str = "Graphite";
pub const V_THEME_TOKYO: &str = "Tokyo";
pub const V_THEME_MOSS: &str = "Moss";
pub const V_THEME_EMBER: &str = "Ember";
pub const V_THEME_PLUM: &str = "Plum";
pub const V_THEME_ONYX: &str = "Onyx";
pub const V_THEME_GREY: &str = "Grey";
pub const V_THEME_SILVER: &str = "Silver";
pub const V_THEME_APPLE: &str = "Apple";
pub const V_THEME_APPLE_DARK: &str = "AppleDark";
pub const V_THEME_SWIFT: &str = "Swift";
pub const V_THEME_BARBIE: &str = "Barbie";
pub const V_THEME_FENTY: &str = "Fenty";
pub const V_THEME_CYBERPUNK: &str = "Cyberpunk";
pub const V_THEME_CHANEL: &str = "Chanel";
/// `ctx.theme_mode`: every theme read from black or from white.
pub const V_MODE_DARK: &str = "dark";
pub const V_MODE_LIGHT: &str = "light";

// ── Blender factory defaults (source: `startup.blend` DNA) ──
pub const V_SCENE_DEFAULT: &str = "Scene";
pub const V_VIEW_LAYER_DEFAULT: &str = "ViewLayer";
pub const V_OBJECT_DEFAULT: &str = "Cube";
pub const V_MODE_OBJECT: &str = "Object";
pub const V_TOOL_SELECT_BOX: &str = "Select Box";
pub const V_FRAME_START: &str = "1";
pub const V_FRAME_END: &str = "250";

// ── Value categories ──
//
// Every value the state holds is a string; every control that edits one
// knows which of these categories it is, and reads and writes it through
// `BValue` so a checkbox, a number field, a dropdown and a colour swatch all
// go through one encode / parse. Setting the render engine to Phiray is a
// `BValue::Enum("PHIRAY")` written to `ctx.buts.engine`, the same as any
// other enum anywhere.

/// The categories a state value can be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BValueKind {
    Bool,
    Int,
    Float,
    /// One of a declared set of options, held by its key.
    Enum,
    Text,
    /// RGBA, 0..1 each.
    Color,
    /// A file or directory path.
    Path,
}

/// A typed state value.
#[derive(Clone, Debug, PartialEq)]
pub enum BValue {
    Bool(bool),
    Int(i64),
    Float(f32),
    Enum(String),
    Text(String),
    Color([f32; 4]),
    Path(String),
}

impl BValue {
    pub const fn kind(&self) -> BValueKind {
        match self {
            Self::Bool(_) => BValueKind::Bool,
            Self::Int(_) => BValueKind::Int,
            Self::Float(_) => BValueKind::Float,
            Self::Enum(_) => BValueKind::Enum,
            Self::Text(_) => BValueKind::Text,
            Self::Color(_) => BValueKind::Color,
            Self::Path(_) => BValueKind::Path,
        }
    }

    /// The string the state stores. Round-trips through `parse`.
    pub fn encode(&self) -> String {
        match self {
            Self::Bool(true) => V_TRUE.to_string(),
            Self::Bool(false) => V_FALSE.to_string(),
            Self::Int(v) => v.to_string(),
            Self::Float(v) => format!("{v}"),
            Self::Enum(v) | Self::Text(v) | Self::Path(v) => v.clone(),
            Self::Color([r, g, b, a]) => format!("{r} {g} {b} {a}"),
        }
    }

    /// The value a stored string holds, read as `kind`. A string that does
    /// not parse as its category is `None`, so a control shows its default
    /// rather than garbage.
    pub fn parse(kind: BValueKind, raw: &str) -> Option<Self> {
        Some(match kind {
            BValueKind::Bool => Self::Bool(match raw {
                V_TRUE => true,
                V_FALSE => false,
                _ => return None,
            }),
            BValueKind::Int => Self::Int(raw.parse().ok()?),
            BValueKind::Float => Self::Float(raw.parse().ok()?),
            BValueKind::Enum => Self::Enum(raw.to_string()),
            BValueKind::Text => Self::Text(raw.to_string()),
            BValueKind::Path => Self::Path(raw.to_string()),
            BValueKind::Color => {
                let mut c = [0.0f32; 4];
                let mut n = 0;
                for (slot, part) in c.iter_mut().zip(raw.split_whitespace()) {
                    *slot = part.parse().ok()?;
                    n += 1;
                }
                if n < 3 {
                    return None;
                }
                if n == 3 {
                    c[3] = 1.0;
                }
                Self::Color(c)
            }
        })
    }

    /// What a control shows for the value.
    pub fn display(&self) -> String {
        match self {
            Self::Bool(b) => if *b { "On" } else { "Off" }.to_string(),
            Self::Int(v) => v.to_string(),
            Self::Float(v) => format!("{v:.3}"),
            Self::Enum(v) | Self::Text(v) | Self::Path(v) => v.clone(),
            Self::Color([r, g, b, _]) => format!(
                "#{:02X}{:02X}{:02X}",
                (r * 255.0).round() as u8,
                (g * 255.0).round() as u8,
                (b * 255.0).round() as u8
            ),
        }
    }
}

impl super::BState {
    /// The value at `path` read as `kind`, if it is there and parses.
    pub fn value(&self, key: impl super::BKey, kind: BValueKind) -> Option<BValue> {
        self.get(key).and_then(|raw| BValue::parse(kind, raw))
    }
}

#[cfg(test)]
mod value_tests {
    use super::*;

    #[test]
    fn every_category_round_trips_through_the_state_string() {
        for v in [
            BValue::Bool(true),
            BValue::Int(-42),
            BValue::Float(1.5),
            BValue::Enum("PHIRAY".into()),
            BValue::Text("Cube".into()),
            BValue::Color([1.0, 0.5, 0.0, 1.0]),
            BValue::Path("/tmp/a.blend".into()),
        ] {
            assert_eq!(BValue::parse(v.kind(), &v.encode()), Some(v.clone()), "{v:?}");
        }
        assert_eq!(BValue::parse(BValueKind::Int, "abc"), None);
        assert_eq!(BValue::parse(BValueKind::Color, "1 0 0"), Some(BValue::Color([1.0, 0.0, 0.0, 1.0])));
    }
}

/// The Animation workspace's bottom zone: the Dope Sheet's header, its
/// window, and the status bar, in pixels (Blender: 433 of 1054).
pub const V_ANIMATION_BOTTOM_HEIGHT: &str = "370";
/// The Animation workspace's camera viewport (Blender: 559 of 1920).
pub const V_CAMERA_VIEW_WIDTH: &str = "440";
/// The Compositing workspace's Dope Sheet strip (Blender: 189 of 1054).
pub const V_COMPOSITING_BOTTOM_HEIGHT: &str = "190";
/// The Geometry Nodes node editor with the Timeline under it (Blender:
/// 477 + 74 of 1054).
pub const V_GEOMETRY_NODES_BOTTOM_HEIGHT: &str = "460";
/// The Geometry Nodes Spreadsheet column (Blender: 654 of 1920).
pub const V_SPREADSHEET_WIDTH: &str = "460";
/// Scripting's viewport / console / Info column (Blender: 558 of 1920).
pub const V_SCRIPTING_LEFT_WIDTH: &str = "420";
/// The UV Editing and Texture Paint image editors (Blender: 786 of 1920).
pub const V_IMAGE_COLUMN_WIDTH: &str = "560";
