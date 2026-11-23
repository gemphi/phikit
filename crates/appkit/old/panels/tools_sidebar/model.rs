//! Tools sidebar model.

/// Definition of a tool in the sidebar
#[derive(Debug, Clone)]
pub struct NSToolDefinition {
    /// Unique identifier for the tool
    pub id: u32,
    /// Internal name for the tool
    pub name: &'static str,
    /// Display label for the tool
    pub label: &'static str,
    /// Icon identifier (for icon lookup)
    pub icon_id: &'static str,
    /// Icon name (for icon resolver)
    pub icon_name: &'static str,
    /// Optional keyboard shortcut
    pub shortcut: Option<char>,
}
