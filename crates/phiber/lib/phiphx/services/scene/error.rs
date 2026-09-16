//! Load-error surface — `SceneLoadError` + its message clamp.
//!
//! Split out of `scene/mod.rs` so the parent stays under the 200-line cap.

/// Failure categories surfaced to the user when a `.blend` load fails.
///
/// Each variant carries a human-readable detail string. The load pipeline maps
/// open/read failures to [`Inaccessible`](SceneLoadError::Inaccessible),
/// parse-structure failures to [`Malformed`](SceneLoadError::Malformed), and
/// recognized-but-unsupported files to
/// [`Unsupported`](SceneLoadError::Unsupported).
///
/// Requirements: 3.1, 3.2, 3.3
#[derive(Clone, Debug, PartialEq)]
pub enum SceneLoadError {
    /// The file could not be opened or read from storage.
    Inaccessible(String),
    /// The file's structure could not be parsed.
    Malformed(String),
    /// The file was recognized but is not supported.
    Unsupported(String),
}

/// Upper bound (inclusive) on the length of a [`SceneLoadError::message`],
/// matching the Load_Error_State bound (Requirement 3.3).
pub const MESSAGE_MAX: usize = 256;

impl SceneLoadError {
    /// The stable discriminant string for this category.
    ///
    /// Returns `"inaccessible"`, `"malformed"`, or `"unsupported"`.
    pub fn category(&self) -> &'static str {
        match self {
            SceneLoadError::Inaccessible(_) => "inaccessible",
            SceneLoadError::Malformed(_) => "malformed",
            SceneLoadError::Unsupported(_) => "unsupported",
        }
    }

    /// A human-readable message that names the failure category.
    ///
    /// The result is guaranteed to be between 1 and 256 characters (inclusive):
    /// it always begins with the category name, so it is never empty, and it is
    /// clamped to at most 256 characters on a `char` boundary so no read of the
    /// message can overrun the Load_Error_State bound (Requirement 3.3).
    pub fn message(&self) -> String {
        let category = self.category();
        let detail = match self {
            SceneLoadError::Inaccessible(d)
            | SceneLoadError::Malformed(d)
            | SceneLoadError::Unsupported(d) => d,
        };
        // Always name the category; append the detail when present. This
        // guarantees the message is non-empty (>= 1 char) because `category`
        // is a non-empty static string.
        let raw = if detail.trim().is_empty() {
            category.to_string()
        } else {
            format!("{category}: {detail}")
        };
        clamp_message(raw)
    }
}

/// Clamp a message to at most [`MESSAGE_MAX`] characters on a `char` boundary.
///
/// Truncating by `char` count (not bytes) keeps the result valid UTF-8 and
/// guarantees `1..=MESSAGE_MAX` chars for any non-empty input.
pub(super) fn clamp_message(raw: String) -> String {
    if raw.chars().count() <= MESSAGE_MAX {
        return raw;
    }
    raw.chars().take(MESSAGE_MAX).collect()
}
