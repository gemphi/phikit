//! USDA text writer
//!
//! Buffered line writer that emits USDA (ASCII USD) text with prim scopes
//! and typed attribute helpers.

/// Writer for USDA (ASCII USD) format
#[derive(Debug)]
pub struct UsdaWriter {
    pub(super) lines: Vec<String>,
    indent: usize,
}

impl UsdaWriter {
    /// Create a new USDA writer with default header
    pub fn new() -> Self {
        Self::stage("Root", "Y")
    }

    /// A writer whose stage opens on `default_prim`, in metres, `up` the up axis.
    pub fn stage(default_prim: &str, up: &str) -> Self {
        let mut writer = Self {
            lines: Vec::new(),
            indent: 0,
        };
        writer.lines.push("#usda 1.0".to_string());
        writer.lines.push("(".to_string());
        writer.lines.push(format!("    defaultPrim = \"{default_prim}\""));
        writer.lines.push("    metersPerUnit = 1".to_string());
        writer.lines.push(format!("    upAxis = \"{up}\""));
        writer.lines.push(")".to_string());
        writer.lines.push(String::new());
        writer
    }

    /// Write a line with current indentation
    pub fn line(&mut self, text: &str) {
        let indent = match text.is_empty() {
            true => String::new(),
            false => "    ".repeat(self.indent),
        };
        self.lines.push(format!("{}{}", indent, text));
    }

    /// Push a new scope (Xform, Mesh, etc.)
    pub fn push_scope(&mut self, name: &str, prim_type: &str) {
        self.line(&format!("def {} \"{}\"", prim_type, name));
        self.line("{");
        self.indent += 1;
    }

    /// Push a scope whose prim carries metadata lines (`prepend apiSchemas = …`).
    pub fn push_scope_with(&mut self, name: &str, prim_type: &str, meta: &[&str]) {
        self.line(&format!("def {} \"{}\" (", prim_type, name));
        for m in meta {
            self.line(&format!("    {m}"));
        }
        self.line(")");
        self.line("{");
        self.indent += 1;
    }

    /// Pop the current scope
    pub fn pop_scope(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
        self.line("}");
        self.line("");
    }

    /// Write an attribute
    pub fn attr(&mut self, name: &str, value: &str, type_hint: Option<&str>) {
        let type_str = type_hint.unwrap_or("");
        if !type_str.is_empty() {
            self.line(&format!("{} {} = {}", type_str, name, value));
        } else {
            self.line(&format!("{} = {}", name, value));
        }
    }

    /// Write a string attribute
    pub fn attr_string(&mut self, name: &str, value: &str, type_hint: Option<&str>) {
        let type_str = type_hint.unwrap_or("string");
        self.line(&format!("{} {} = \"{}\"", type_str, name, value));
    }

    /// Write a bool attribute
    pub fn attr_bool(&mut self, name: &str, value: bool) {
        self.line(&format!(
            "bool {} = {}",
            name,
            value.to_string().to_lowercase()
        ));
    }

    /// Write an int attribute
    pub fn attr_int(&mut self, name: &str, value: i32) {
        self.line(&format!("int {} = {}", name, value));
    }

    /// Write a float attribute
    pub fn attr_float(&mut self, name: &str, value: f32) {
        self.line(&format!("float {} = {}", name, value));
    }

    /// Write a float3 attribute
    pub fn attr_float3(&mut self, name: &str, value: [f32; 3]) {
        self.line(&format!(
            "float3 {} = ({:.6}, {:.6}, {:.6})",
            name, value[0], value[1], value[2]
        ));
    }

    /// Write a float4 attribute
    pub fn attr_float4(&mut self, name: &str, value: [f32; 4]) {
        self.line(&format!(
            "float4 {} = ({:.6}, {:.6}, {:.6}, {:.6})",
            name, value[0], value[1], value[2], value[3]
        ));
    }

    /// Write a float array attribute
    pub fn attr_float_array(&mut self, name: &str, values: &[f32]) {
        if values.is_empty() {
            return;
        }
        let arr: String = values
            .iter()
            .map(|v| format!("{:.6}", v))
            .collect::<Vec<_>>()
            .join(", ");
        self.line(&format!("float[] {} = [{}]", name, arr));
    }

    /// Write an int array attribute
    pub fn attr_int_array(&mut self, name: &str, values: &[i32]) {
        if values.is_empty() {
            return;
        }
        let arr: String = values
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        self.line(&format!("int[] {} = [{}]", name, arr));
    }

    /// Write a string array attribute
    pub fn attr_string_array(&mut self, name: &str, values: &[&str]) {
        if values.is_empty() {
            return;
        }
        let arr: String = values
            .iter()
            .map(|v| format!("\"{}\"", v))
            .collect::<Vec<_>>()
            .join(", ");
        self.line(&format!("string[] {} = [{}]", name, arr));
    }

    /// Write a 4x4 matrix attribute (row-major in USD)
    pub fn matrix_attr(&mut self, name: &str, matrix: &[[f32; 4]; 4]) {
        self.line(&format!("matrix4d {} = (", name));
        self.indent += 1;
        for i in 0..4 {
            let comma = if i < 3 { "," } else { "" };
            self.line(&format!(
                "({:.6}, {:.6}, {:.6}, {:.6}){}",
                matrix[i][0], matrix[i][1], matrix[i][2], matrix[i][3], comma
            ));
        }
        self.indent -= 1;
        self.line(")");
    }

    /// Get the complete USDA text
    pub fn get_text(&self) -> String {
        self.lines.join("\n")
    }
}

impl Default for UsdaWriter {
    fn default() -> Self {
        Self::new()
    }
}
