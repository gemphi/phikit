//! The layer under construction: indented lines, prim blocks that must close.

/// Text of a layer being written, with the open-block depth it tracks so a
/// block left open is caught where it is written, not when USD refuses it.
#[derive(Debug, Default)]
pub(crate) struct UsdOut {
    text: String,
    depth: usize,
}

impl UsdOut {
    /// One line at the current depth.
    pub fn line(&mut self, s: &str) {
        for _ in 0..self.depth {
            self.text.push_str("    ");
        }
        self.text.push_str(s);
        self.text.push('\n');
    }

    /// Raw lines with no indent: the layer header, which sits at depth 0.
    pub fn raw(&mut self, s: &str) {
        self.text.push_str(s);
        self.text.push('\n');
    }

    /// `def kind "name" ( meta... ) {`; each `meta` entry is one metadata line.
    pub fn open(&mut self, kind: &str, name: &str, meta: &[String]) {
        self.line(&format!("def {kind} \"{name}\""));
        if !meta.is_empty() {
            self.line("(");
            self.depth += 1;
            for m in meta {
                self.line(m);
            }
            self.depth -= 1;
            self.line(")");
        }
        self.line("{");
        self.depth += 1;
    }

    /// Close the innermost block.
    pub fn close(&mut self) {
        debug_assert!(self.depth > 0, "close without open");
        self.depth = self.depth.saturating_sub(1);
        self.line("}");
    }

    /// The finished text; every block is closed.
    pub fn finish(self) -> String {
        debug_assert_eq!(self.depth, 0, "a prim block was left open");
        self.text
    }
}
