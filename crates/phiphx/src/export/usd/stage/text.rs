//! Numbers, strings and arrays as USDA spells them.
//!
//! Arrays go on one line each: the self check (`check.rs`) and a diff both
//! read a layer line by line, and USDA allows it.
use std::fmt::Write;

/// Text for one USDA value. A unit type: the functions are its vocabulary.
pub(super) struct UsdText;

impl UsdText {
    /// A float; USDA has no spelling for NaN or infinity, so they become 0.
    pub fn float(v: f32) -> String {
        if v.is_finite() { format!("{v}") } else { "0".into() }
    }

    pub fn vec2(v: [f32; 2]) -> String {
        format!("({}, {})", Self::float(v[0]), Self::float(v[1]))
    }

    pub fn vec3(v: [f32; 3]) -> String {
        format!("({}, {}, {})", Self::float(v[0]), Self::float(v[1]), Self::float(v[2]))
    }

    /// `[(x, y), ...]`
    pub fn vec2_list(list: &[[f32; 2]]) -> String {
        Self::join(list.iter().map(|v| Self::vec2(*v)))
    }

    /// `[(x, y, z), ...]`
    pub fn vec3_list(list: &[[f32; 3]]) -> String {
        Self::join(list.iter().map(|v| Self::vec3(*v)))
    }

    /// `[1, 2, 3]`
    pub fn int_list(list: impl IntoIterator<Item = u32>) -> String {
        Self::join(list.into_iter().map(|v| v.to_string()))
    }

    fn join(items: impl Iterator<Item = String>) -> String {
        let mut out = String::from("[");
        for (i, item) in items.enumerate() {
            if i > 0 {
                out.push_str(", ");
            }
            // Writing to a String cannot fail.
            let _ = write!(out, "{item}");
        }
        out.push(']');
        out
    }

    /// A `matrix4d` from Blender's `m[col][row]`. USD multiplies row
    /// vectors, so its row `i` is Blender's column `i`: the arrays line up
    /// as they are, translation in the last row.
    pub fn matrix(m: &[[f32; 4]; 4]) -> String {
        let row = |r: &[f32; 4]| format!("({}, {}, {}, {})", Self::float(r[0]), Self::float(r[1]), Self::float(r[2]), Self::float(r[3]));
        format!("( {}, {}, {}, {} )", row(&m[0]), row(&m[1]), row(&m[2]), row(&m[3]))
    }

    /// A quoted string with the escapes USDA reads back.
    pub fn quote(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 2);
        out.push('"');
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c => out.push(c),
            }
        }
        out.push('"');
        out
    }

    /// An asset path `@...@`. Blender's `//` (relative to the `.blend`)
    /// becomes `./`, backslashes become slashes, and an `@` in the path is
    /// percent-encoded because it would end the literal.
    pub fn asset(path: &str) -> String {
        let path = path.replace('\\', "/").replace('@', "%40");
        match path.strip_prefix("//") {
            Some(rest) => format!("@./{rest}@"),
            None => format!("@{path}@"),
        }
    }
}
