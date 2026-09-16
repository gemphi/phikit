//! Text Editor host definitions matching Blender 5.2 `SpaceText` in `DNA_space_types.h`.

#[derive(Clone, Debug, PartialEq)]
pub struct TextEditorConfig {
    pub text_name: Option<String>,
    pub line_numbers: bool,
    pub syntax_highlight: bool,
    pub word_wrap: bool,
    pub font_size: u16,
}

impl Default for TextEditorConfig {
    fn default() -> Self {
        Self {
            text_name: None,
            line_numbers: true,
            syntax_highlight: true,
            word_wrap: false,
            font_size: 13,
        }
    }
}
