//! An area: a slot inside a zone that holds one host at a time. `PArea`.

/// A slot inside a zone that holds one host at a time.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PArea {
    /// The name of the host it holds now.
    pub host: String,
}

impl PArea {
    /// An area holding the host of that name.
    pub fn holding(host: &str) -> PArea {
        PArea { host: host.to_string() }
    }
}
