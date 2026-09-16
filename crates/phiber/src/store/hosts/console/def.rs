//! Python Console host definitions matching Blender 5.2 `SpaceConsole` in `DNA_space_types.h`.

#[derive(Clone, Debug, PartialEq)]
pub struct ConsoleConfig {
    pub prompt: String,
    pub history: Vec<String>,
}

impl Default for ConsoleConfig {
    fn default() -> Self {
        Self {
            prompt: ">>> ".to_string(),
            history: Vec::new(),
        }
    }
}
