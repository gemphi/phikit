//! Attribute parsing — decodes typed attribute assignments from USDA text
//! into `UsdAttribute` / `UsdValue` pairs.

use super::super::attribute::UsdAttribute;
use super::super::error::UsdaError;
use super::UsdaParser;

impl<'a> UsdaParser<'a> {
    /// Parse a single attribute line. Returns `Ok(None)` for lines that are
    /// skipped (relationships, connections, declarations without values).
    pub(crate) fn parse_attribute(
        &mut self,
        prim_path: &str,
    ) -> Result<Option<UsdAttribute>, UsdaError> {
        let start_pos = self.pos;
        let first_token = self.peek_identifier();

        // Skip relationship declarations
        if first_token == "rel" {
            self.skip_to_end_of_statement();
            return Ok(None);
        }

        // Skip prepend/append/delete references
        if first_token == "prepend"
            || first_token == "append"
            || first_token == "delete"
            || first_token == "add"
        {
            self.skip_to_end_of_statement();
            return Ok(None);
        }

        // Read qualifiers and type
        let token = self.read_identifier();
        let type_str;

        if token == "uniform" || token == "custom" || token == "varying" {
            self.skip_whitespace();
            type_str = self.read_type_name();
        } else {
            let mut ts = token;
            if self.peek() == Some('[') {
                self.advance();
                if self.peek() == Some(']') {
                    self.advance();
                }
                ts.push_str("[]");
            }
            type_str = ts;
        }

        if type_str.is_empty() {
            self.pos = start_pos;
            self.skip_to_end_of_statement();
            return Ok(None);
        }

        self.skip_whitespace();
        let attr_name = self.read_attribute_name();

        if attr_name.is_empty() {
            self.pos = start_pos;
            self.skip_to_end_of_statement();
            return Ok(None);
        }

        self.skip_whitespace();

        if self.peek() != Some('=') {
            self.skip_to_end_of_statement();
            return Ok(None);
        }
        self.advance(); // consume '='
        self.skip_whitespace();

        // Check for connection syntax: <path>.attr
        if self.peek() == Some('<') {
            self.skip_to_end_of_statement();
            return Ok(None);
        }

        let value = self.parse_value(&type_str, prim_path, &attr_name)?;

        // Skip optional metadata after value: ( ... )
        self.skip_whitespace();
        if self.peek() == Some('(') {
            self.advance();
            self.skip_balanced_parens();
        }

        Ok(Some(UsdAttribute::new(attr_name, value)))
    }
}
