//! Prim parsing — creates `UsdPrim` nodes from `def <Type> "<Name>"` blocks
//! with correct absolute path construction and recursive child handling.

use super::super::error::UsdaError;
use super::super::prim::UsdPrim;
use super::UsdaParser;

impl<'a> UsdaParser<'a> {
    /// Parse a single prim definition block and its children recursively.
    pub(crate) fn parse_prim(&mut self, parent_path: &str) -> Result<UsdPrim, UsdaError> {
        // Consume keyword (def/over/class)
        let _keyword = self.read_identifier();
        self.skip_whitespace();

        // Read type name
        let type_name = self.read_identifier();
        self.skip_whitespace();

        // Read prim name (quoted string)
        let name = self.read_string_literal();

        let path = if parent_path.is_empty() || parent_path == "/" {
            format!("/{}", name)
        } else {
            format!("{}/{}", parent_path, name)
        };

        let mut prim = UsdPrim::new(&path, &type_name);

        self.skip_whitespace_and_comments();

        // Optional metadata block on prim: ( ... )
        if self.peek() == Some('(') {
            self.advance();
            self.skip_prim_metadata();
        }

        self.skip_whitespace_and_comments();

        // Expect opening brace
        if self.peek() == Some('{') {
            self.advance();
        }

        // Parse prim body: attributes and nested prims
        loop {
            self.skip_whitespace_and_comments();
            if self.peek() == Some('}') {
                self.advance();
                break;
            }
            if self.pos >= self.input.len() {
                break;
            }

            if self.starts_with("def ")
                || self.starts_with("over ")
                || self.starts_with("class ")
            {
                let child = self.parse_prim(&path)?;
                prim.add_child(child);
            } else {
                match self.parse_attribute(&path) {
                    Ok(Some(attr)) => {
                        prim.attributes.push(attr);
                    }
                    Ok(None) => {}
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(prim)
    }

    /// Skip a prim metadata block `( ... )` — we don't interpret prim-level
    /// metadata currently.
    pub(crate) fn skip_prim_metadata(&mut self) {
        self.skip_balanced_parens();
    }
}
